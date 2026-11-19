//! Separate exterior stop against the reveal. All contours use the frame datum
//! (outer frame without the lip). Glass and sash sizes are independent of it.
use crate::curved_contour::{Contour, FrameCurve, Segment};
use crate::curved_timber::{arc_member, cut_angle};
use crate::kozijn::{Frame, ShapeType};
use crate::production::{CutListItem, MemberType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopProfile {
    pub profile_name: String,
    pub projection_mm: f64,
    pub thickness_mm: f64,
}

impl StopProfile {
    fn valid(&self, frame: &Frame) -> bool {
        !self.profile_name.trim().is_empty()
            && self.projection_mm.is_finite()
            && self.projection_mm > 0.
            && self.thickness_mm.is_finite()
            && self.thickness_mm > 0.
            && self.thickness_mm <= frame.frame_depth
    }
}

impl Frame {
    pub fn set_perimeter_stop(&mut self, json: &str) -> Result<(), String> {
        let mut profile: Option<StopProfile> =
            serde_json::from_str(json).map_err(|e| e.to_string())?;
        if let Some(p) = &mut profile {
            if !self.material.is_profile_system() || !p.valid(self) {
                return Err("Geef een profielnaam, positieve uitsteek en een lipdikte binnen de profieldiepte op.".into());
            }
            p.profile_name = p.profile_name.trim().to_owned();
        }
        self.perimeter_stop = profile;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopGeometry {
    #[serde(flatten)]
    pub profile: StopProfile,
    pub inner: Contour,
    pub outer: Contour,
    pub center: Contour,
    pub outer_bounds: crate::geometry::Rect2D,
}

type Point = [f64; 2];
fn delta(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

// Parallel offset of each side of a convex, clockwise polygon in elevation.
// Neighboring offsets intersect at the long point of each miter.
fn polygon_offset(points: &[Point], distance: f64) -> Option<Contour> {
    let mut points = points.to_vec();
    let n = points.len();
    if n < 3 {
        return None;
    }
    let area: f64 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if area.abs() < 1e-7 {
        return None;
    }
    if area < 0. {
        points.reverse();
    }
    let normals: Vec<_> = (0..n)
        .map(|i| {
            let d = delta(points[(i + 1) % n], points[i]);
            let length = d[0].hypot(d[1]);
            (length > 1e-7).then(|| [d[1] / length, -d[0] / length])
        })
        .collect::<Option<_>>()?;
    let corners = (0..n)
        .map(|i| {
            let (a, b) = (normals[(i + n - 1) % n], normals[i]);
            let denom = 1. + a[0] * b[0] + a[1] * b[1];
            (denom > 1e-7).then(|| {
                [
                    points[i][0] + distance * (a[0] + b[0]) / denom,
                    points[i][1] + distance * (a[1] + b[1]) / denom,
                ]
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Contour {
        segments: (0..n)
            .map(|i| Segment::Line {
                from: corners[i],
                to: corners[(i + 1) % n],
            })
            .collect(),
    })
}

pub(crate) fn outline(
    frame: &Frame,
    curve: Option<&FrameCurve>,
    polygon: &[Point],
    stepped: bool,
) -> Option<StopGeometry> {
    let profile = frame.perimeter_stop.as_ref()?;
    if !frame.material.is_profile_system() || !profile.valid(frame) || stepped {
        return None;
    }
    let corners = [
        [0., 0.],
        [frame.outer_width, 0.],
        [frame.outer_width, frame.outer_height],
        [0., frame.outer_height],
    ];
    let at = |projection: f64| {
        if let Some(curve) = curve {
            let contour = curve.outline(-projection);
            return (!contour.segments.is_empty()).then_some(contour);
        }
        match frame.shape.shape_type {
            ShapeType::Rectangular => polygon_offset(&corners, projection),
            ShapeType::Trapezoid => polygon_offset(polygon, projection),
            _ => None,
        }
    };
    let outer = at(profile.projection_mm)?;
    let stop = StopGeometry {
        profile: profile.clone(),
        inner: at(0.)?,
        outer_bounds: outer.bounds(),
        outer,
        center: at(profile.projection_mm / 2.)?,
    };
    (stop.inner.segments.len() == stop.outer.segments.len()).then_some(stop)
}

impl StopGeometry {
    pub(crate) fn cuts(&self, mark: &str, material: &str, kerf: f64) -> Vec<CutListItem> {
        self.outer
            .segments
            .iter()
            .zip(&self.inner.segments)
            .enumerate()
            .map(|(i, (outer, inner))| {
                let bend = arc_member(outer, inner, i, [None, None]);
                let axis = delta(outer.to(), outer.from());
                let (start, end) = bend.as_ref().map_or_else(
                    || {
                        (
                            cut_angle(axis, delta(inner.from(), outer.from())),
                            cut_angle(axis, delta(inner.to(), outer.to())),
                        )
                    },
                    |b| (b.start_angle_deg, b.end_angle_deg),
                );
                CutListItem {
                    piece_id: format!("{mark}-AP{}", i + 1),
                    member_type: MemberType::PerimeterStop,
                    profile_name: format!(
                        "{} ({} × {} mm)",
                        self.profile.profile_name,
                        self.profile.projection_mm,
                        self.profile.thickness_mm
                    ),
                    material: material.to_string(),
                    net_length_mm: outer.length(),
                    gross_length_mm: outer.length() + kerf,
                    miter_left_deg: start,
                    miter_right_deg: end,
                    quantity: 1,
                    arc: bend.and_then(|b| b.arc),
                    profile_side: None,
                    start_cope_radius_mm: None,
                    end_cope_radius_mm: None,
                    divider_start: None,
                    divider_end: None,
                    masonry_notches: vec![],
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{Kozijn, Material};
    use crate::production::compute_production_data;
    use std::f64::consts::PI;

    const PROFILE: &str = r#"{"profileName":"AP20","projectionMm":20,"thicknessMm":3}"#;
    fn kozijn(shape: ShapeType) -> Kozijn {
        let mut k = Kozijn::new("Aanslag", "K1", 1000., 1100.);
        k.frame.material = Material::Aluminum;
        k.frame.shape.shape_type = shape;
        k.frame.set_perimeter_stop(PROFILE).unwrap();
        k
    }

    #[test]
    fn rectangular_stop_miters_at_the_long_points_and_leaves_glass_unchanged() {
        let mut k = kozijn(ShapeType::Rectangular);
        let with = compute_production_data(&k);
        let cuts: Vec<_> = with
            .cut_list
            .iter()
            .filter(|p| p.member_type == MemberType::PerimeterStop)
            .collect();
        assert_eq!(cuts.len(), 4);
        assert_eq!(
            (cuts[0].net_length_mm, cuts[1].net_length_mm),
            (1040., 1140.)
        );
        for cut in cuts {
            assert_eq!((cut.miter_left_deg, cut.miter_right_deg), (45., 45.));
            assert_eq!(cut.gross_length_mm - cut.net_length_mm, 4.);
        }
        let reopened: Kozijn = serde_json::from_str(&serde_json::to_string(&k).unwrap()).unwrap();
        assert_eq!(
            compute_2d_geometry(&reopened)
                .perimeter_stop
                .unwrap()
                .outer_bounds
                .width,
            1040.
        );
        k.frame.set_perimeter_stop("null").unwrap();
        let without = compute_production_data(&k);
        assert_eq!(with.glass_list[0].width_mm, without.glass_list[0].width_mm);
        assert_eq!(without.cut_list.len() + 4, with.cut_list.len());
    }

    #[test]
    fn round_stop_has_two_half_rings_and_radial_ends() {
        let k = kozijn(ShapeType::Round);
        let data = compute_production_data(&k);
        let cuts: Vec<_> = data
            .cut_list
            .iter()
            .filter(|p| p.member_type == MemberType::PerimeterStop)
            .collect();
        assert_eq!(cuts.len(), 2);
        for cut in cuts {
            let arc = cut.arc.as_ref().unwrap();
            assert_eq!(
                (arc.outer_radius_mm, arc.centerline_radius_mm),
                (520., 510.)
            );
            assert!((cut.net_length_mm - 520. * PI).abs() < 1e-7);
            assert!((arc.centerline_length_mm - 510. * PI).abs() < 1e-7);
            assert!(
                (cut.miter_left_deg - 90.).abs() < 1e-7 && (cut.miter_right_deg - 90.).abs() < 1e-7
            );
        }
    }

    #[test]
    fn arched_stop_meets_both_jambs_and_invalid_edits_preserve_it() {
        let mut k = kozijn(ShapeType::Arched);
        k.frame.shape.arch_height = Some(250.);
        assert!(k
            .frame
            .set_perimeter_stop(r#"{"profileName":"x","projectionMm":0,"thicknessMm":3}"#)
            .is_err());
        assert!(k
            .frame
            .set_perimeter_stop(r#"{"profileName":"x","projectionMm":20,"thicknessMm":200}"#)
            .is_err());
        let stop = compute_2d_geometry(&k).perimeter_stop.unwrap();
        assert_eq!(stop.profile.profile_name, "AP20");
        assert_eq!(stop.outer.segments.len(), 4);
        for (i, edge) in stop.outer.segments.iter().enumerate() {
            assert_eq!(edge.to(), stop.outer.segments[(i + 1) % 4].from());
        }
        let cuts = stop.cuts("K1", "Aluminium", 4.);
        let arc = cuts.iter().find_map(|c| c.arc.as_ref()).unwrap();
        assert_eq!(arc.outer_radius_mm, 645.);
        assert_eq!(arc.centerline_radius_mm, 635.);
        assert!(cuts[0].miter_left_deg > 45. && cuts[0].miter_left_deg < 90.);
    }
}
