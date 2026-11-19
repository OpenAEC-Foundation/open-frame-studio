//! Exact circular and straight boundaries, in elevation coordinates (mm, y down).
use crate::geometry::Rect2D;
use crate::glazing_fit::GlazingFit;
use crate::kozijn::{Frame, ShapeType};
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

type Point = [f64; 2];
const EPS: f64 = 1e-7;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Segment {
    Line {
        from: Point,
        to: Point,
    },
    Arc {
        from: Point,
        to: Point,
        center: Point,
        radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    },
}

fn distance(a: Point, b: Point) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

impl Segment {
    pub fn from(&self) -> Point {
        match self {
            Self::Line { from, .. } | Self::Arc { from, .. } => *from,
        }
    }
    pub fn to(&self) -> Point {
        match self {
            Self::Line { to, .. } | Self::Arc { to, .. } => *to,
        }
    }
    pub fn length(&self) -> f64 {
        match self {
            Self::Line { from, to } => distance(*from, *to),
            Self::Arc {
                radius,
                sweep_angle,
                ..
            } => radius * sweep_angle.abs(),
        }
    }
    pub(crate) fn point(&self, t: f64) -> Point {
        if t == 0.0 {
            return self.from();
        }
        if t == 1.0 {
            return self.to();
        }
        match self {
            Self::Line { from, to } => [
                from[0] + t * (to[0] - from[0]),
                from[1] + t * (to[1] - from[1]),
            ],
            Self::Arc {
                center,
                radius,
                start_angle,
                sweep_angle,
                ..
            } => {
                let a = start_angle + t * sweep_angle;
                [center[0] + radius * a.cos(), center[1] + radius * a.sin()]
            }
        }
    }
    pub(crate) fn section(&self, a: f64, b: f64) -> Self {
        let from = self.point(a);
        let to = self.point(b);
        match self {
            Self::Line { .. } => Self::Line { from, to },
            Self::Arc {
                center,
                radius,
                start_angle,
                sweep_angle,
                ..
            } => Self::Arc {
                from,
                to,
                center: *center,
                radius: *radius,
                start_angle: start_angle + a * sweep_angle,
                sweep_angle: (b - a) * sweep_angle,
            },
        }
    }
    /// Strictly internal crossings of an axis-aligned plane. Tangencies do not split stock.
    pub(crate) fn crossings(&self, axis: usize, limit: f64) -> Vec<f64> {
        let mut cuts = vec![];
        match self {
            Self::Line { from, to } => {
                let delta = to[axis] - from[axis];
                if delta.abs() > EPS {
                    let t = (limit - from[axis]) / delta;
                    if t > 0.0 && t < 1.0 {
                        cuts.push(t);
                    }
                }
            }
            Self::Arc {
                center,
                radius,
                start_angle,
                sweep_angle,
                ..
            } => {
                let ratio = (limit - center[axis]) / radius;
                if ratio.abs() < 1.0 {
                    let a = ratio.acos();
                    let phase = if axis == 0 { 0.0 } else { FRAC_PI_2 };
                    for angle in [phase - a, phase + a] {
                        let t = ((angle - start_angle)*sweep_angle.signum()).rem_euclid(TAU) / sweep_angle.abs();
                        if t > 0.0 && t < 1.0 {
                            cuts.push(t);
                        }
                    }
                }
            }
        }
        cuts
    }
    /// Clip against one side of an axis-aligned cell. Intersections lie on the
    /// actual circle, independent of any sampling resolution used by a viewer.
    fn inside_parts(&self, axis: usize, limit: f64, keep_less: bool, result: &mut Vec<Self>) {
        let mut cuts = self.crossings(axis,limit);
        cuts.extend([0.,1.]);
        cuts.sort_by(f64::total_cmp);
        for pair in cuts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let delta = self.point((a + b) / 2.0)[axis] - limit;
            if (if keep_less {
                delta <= EPS
            } else {
                delta >= -EPS
            }) && (b - a) * self.length() > EPS
            {
                result.push(self.section(a, b));
            }
        }
    }
}

/// Clockwise closed path. An empty path represents a cell outside the frame.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Contour {
    pub segments: Vec<Segment>,
}

impl Contour {
    pub fn bounds(&self) -> Rect2D {
        if self.segments.is_empty() {
            return Rect2D {
                x: 0.,
                y: 0.,
                width: 0.,
                height: 0.,
            };
        }
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        let mut include = |p: Point| {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        };
        for s in &self.segments {
            include(s.from());
            include(s.to());
            if let Segment::Arc {
                center,
                radius,
                start_angle,
                sweep_angle,
                ..
            } = s
            {
                for (i, d) in [[1., 0.], [0., 1.], [-1., 0.], [0., -1.]]
                    .iter()
                    .enumerate()
                {
                    let delta = i as f64 * FRAC_PI_2 - start_angle;
                    if (sweep_angle.signum() * delta).rem_euclid(TAU) <= sweep_angle.abs() {
                        include([center[0] + radius * d[0], center[1] + radius * d[1]]);
                    }
                }
            }
        }
        Rect2D {
            x: lo[0],
            y: lo[1],
            width: hi[0] - lo[0],
            height: hi[1] - lo[1],
        }
    }
    pub fn perimeter(&self) -> f64 {
        self.segments.iter().map(Segment::length).sum()
    }
    pub fn area(&self) -> f64 {
        let Some(first) = self.segments.first() else {
            return 0.;
        };
        let origin = first.from();
        self.segments
            .iter()
            .map(|s| {
                let a = s.from();
                let b = s.to();
                let triangle = (a[0] - origin[0]) * (b[1] - origin[1])
                    - (a[1] - origin[1]) * (b[0] - origin[0]);
                let cap = if let Segment::Arc {
                    radius,
                    sweep_angle,
                    ..
                } = s
                {
                    radius * radius * (sweep_angle - sweep_angle.sin())
                } else {
                    0.
                };
                (triangle + cap) / 2.
            })
            .sum::<f64>()
            .abs()
    }
    pub(crate) fn clipped(mut self, rect: &Rect2D) -> Self {
        if rect.width <= EPS || rect.height <= EPS {
            return Self::default();
        }
        for (axis, limit, keep_less) in [
            (0, rect.x, false),
            (0, rect.x + rect.width, true),
            (1, rect.y, false),
            (1, rect.y + rect.height, true),
        ] {
            let bounds = self.bounds();
            let (lo, hi) = if axis == 0 {
                (bounds.x, bounds.x + bounds.width)
            } else {
                (bounds.y, bounds.y + bounds.height)
            };
            if (keep_less && lo >= limit - EPS) || (!keep_less && hi <= limit + EPS) {
                return Self::default();
            }
            if (keep_less && hi <= limit + EPS) || (!keep_less && lo >= limit - EPS) {
                continue;
            }
            let mut retained = Vec::new();
            for s in &self.segments {
                s.inside_parts(axis, limit, keep_less, &mut retained);
            }
            if retained.is_empty() {
                return Self::default();
            }
            let mut prev = retained.last().unwrap().to();
            self.segments.clear();
            for s in retained {
                if distance(prev, s.from()) > EPS {
                    self.segments.push(Segment::Line {
                        from: prev,
                        to: s.from(),
                    });
                }
                prev = s.to();
                self.segments.push(s);
            }
        }
        if self.area() < EPS {
            Self::default()
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurvedFrame {
    pub outer: Contour,
    pub day: Contour,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FitContours {
    pub day: Contour,
    pub rebate: Contour,
    pub glass: Contour,
    pub bead_inner: Contour,
    pub sash_outer: Option<Contour>,
    /// Recess of the fixed frame surrounding the whole opening (also for paired leaves).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_seat: Option<Contour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_clearance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_seat_depth: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_opens_outward: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_leaf: Option<SashLeaf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sash_timber: Vec<crate::curved_timber::CurvedTimberMember>,
    pub rebate_height: f64,
    pub edge_clearance: f64,
    pub sash_width: f64,
    pub sash_overlap: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SashLeaf { Left, Right }

pub(crate) enum FrameCurve {
    Round {
        center: Point,
        radius: f64,
    },
    Arch {
        center: Point,
        radius: f64,
        rise: f64,
        width: f64,
        bottom: f64,
        slopes: [f64; 2],
    },
}

impl FrameCurve {
    pub(crate) fn max_rise(frame: &Frame) -> f64 {
        (frame.outer_width / 2.)
            .min(frame.outer_height - frame.frame_width)
            .max(0.)
    }

    pub(crate) fn rise(frame: &Frame) -> f64 {
        frame
            .shape
            .arch_height
            .filter(|v| v.is_finite() && *v > 0.)
            .unwrap_or(frame.outer_width / 4.)
            .min(Self::max_rise(frame))
    }

    pub(crate) fn new(frame: &Frame) -> Option<Self> {
        let (w, h) = (frame.outer_width, frame.outer_height);
        match frame.shape.shape_type {
            ShapeType::Round => Some(Self::Round {
                center: [w / 2., h / 2.],
                radius: w.min(h) / 2.,
            }),
            ShapeType::Arched | ShapeType::ArchedTrapezoid => {
                let slope = |a: Option<f64>| {
                    if frame.shape.shape_type == ShapeType::ArchedTrapezoid {
                        (90. - a.unwrap_or(90.).clamp(45., 90.)).to_radians().tan()
                    } else {
                        0.
                    }
                };
                let slopes = [
                    slope(frame.shape.left_angle),
                    slope(frame.shape.right_angle),
                ];
                // Segmental/semicircular head within the outer envelope.
                let rise = Self::rise(frame);
                let left = slopes[0] * (h - rise);
                let right = w - slopes[1] * (h - rise);
                let half = (right - left) / 2.;
                let radius = if rise > EPS && half > EPS && rise <= half + EPS {
                    (half * half / rise + rise) / 2.
                } else {
                    0.
                };
                Some(Self::Arch {
                    center: [(left + right) / 2., radius],
                    radius,
                    rise,
                    width: w,
                    bottom: h,
                    slopes,
                })
            }
            _ => None,
        }
    }
    pub(crate) fn outline(&self, inset: f64) -> Contour {
        match self {
            Self::Round { center, radius } => {
                let r = radius - inset;
                if r <= EPS {
                    return Contour::default();
                }
                let right = [center[0] + r, center[1]];
                let left = [center[0] - r, center[1]];
                Contour {
                    segments: vec![
                        Segment::Arc {
                            from: right,
                            to: left,
                            center: *center,
                            radius: r,
                            start_angle: 0.,
                            sweep_angle: PI,
                        },
                        Segment::Arc {
                            from: left,
                            to: right,
                            center: *center,
                            radius: r,
                            start_angle: PI,
                            sweep_angle: PI,
                        },
                    ],
                }
            }
            Self::Arch {
                center,
                radius,
                rise,
                width,
                bottom,
                slopes,
            } => {
                let r = radius - inset;
                if r <= EPS || bottom - inset <= inset {
                    return Contour::default();
                }
                // Intersect the concentric arc with each inward-offset jamb.
                let mut springs = [[0.; 2]; 2];
                let mut feet = [[0.; 2]; 2];
                for (i, slope) in slopes.iter().copied().enumerate() {
                    let sign = if i == 0 { 1. } else { -1. };
                    let outer_x = if i == 0 { 0. } else { *width };
                    let dx = -sign * slope;
                    let norm = slope.hypot(1.);
                    let x_at = |y: f64| outer_x + sign * (slope * (bottom - y) + inset * norm);
                    let foot_y = bottom - inset;
                    feet[i] = [x_at(foot_y), foot_y];
                    // Foot of the perpendicular from the circle centre to the jamb.
                    let y0 = center[1] - dx * (x_at(center[1]) - center[0]) / (norm * norm);
                    let x0 = x_at(y0);
                    let dist = distance(*center, [x0, y0]);
                    if dist > r + EPS {
                        return Contour::default();
                    }
                    let along = ((r - dist).max(0.) * (r + dist)).sqrt() / norm;
                    let outer_y0 = y0 + dx * sign * inset / norm;
                    let y = if *rise <= outer_y0 {
                        y0 - along
                    } else {
                        y0 + along
                    };
                    springs[i] = [x_at(y), y];
                }
                if springs[1][0] <= springs[0][0] + EPS {
                    return Contour::default();
                }
                // A shallow frame can put the shoulder below the inside sill.
                // Close the jambs below both springs, then intersect the sill.
                let foot_y = bottom - inset;
                let extend = springs[0][1].max(springs[1][1]) - foot_y;
                if extend > 0. {
                    feet[0] = [feet[0][0] - slopes[0] * extend, foot_y + extend];
                    feet[1] = [feet[1][0] + slopes[1] * extend, foot_y + extend];
                }
                let angle = |p: Point| (p[1] - center[1]).atan2(p[0] - center[0]);
                let start = angle(springs[0]);
                let sweep = (angle(springs[1]) - start).rem_euclid(TAU);
                let segments = vec![
                    Segment::Arc {
                        from: springs[0],
                        to: springs[1],
                        center: *center,
                        radius: r,
                        start_angle: start,
                        sweep_angle: sweep,
                    },
                    Segment::Line {
                        from: springs[1],
                        to: feet[1],
                    },
                    Segment::Line {
                        from: feet[1],
                        to: feet[0],
                    },
                    Segment::Line {
                        from: feet[0],
                        to: springs[0],
                    },
                ];
                let contour = Contour {
                    segments: segments.into_iter().filter(|s| s.length() > EPS).collect(),
                };
                if extend > EPS {
                    let bounds = contour.bounds();
                    contour.clipped(&Rect2D {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: foot_y - bounds.y,
                    })
                } else {
                    contour
                }
            }
        }
    }
    pub(crate) fn window(&self, inset: f64, rect: &Rect2D) -> Contour {
        self.outline(inset).clipped(rect)
    }

    pub(crate) fn shape_fit(&self, rect: &Rect2D, fw: f64, fit: &mut GlazingFit,
        mount: Option<&crate::glazing_fit::SashMount>, index: usize) -> bool {
        let day_offset = fit.day.y - rect.y;
        let rebate_height = fit.day.y - fit.rebate.y;
        let edge_clearance = fit.glass.y - fit.rebate.y;
        let profile = |r: &Rect2D| self.window(fw + r.y - rect.y, r);
        let day = profile(&fit.day);
        let glass = profile(&fit.glass);
        if day.segments.is_empty() || glass.segments.is_empty() {
            return false;
        }
        let rebate = profile(&fit.rebate);
        let bead_inner = self.window(
            fw + day_offset - rebate_height + fit.bead_height,
            &fit.rebate.inset(fit.bead_height),
        );
        let sash_outer = fit.sash_outer.as_ref().map(profile);
        let sash_width = fit.sash_outer.as_ref().map_or(0., |r| fit.day.y - r.y);
        let sash_overlap = fit.sash_outer.as_ref().map_or(0., |r| rect.y - r.y);
        let sash_timber = match (&sash_outer, mount) {
            (Some(outer), Some(m)) => crate::curved_timber::sash_members(
                outer, &day, sash_width, m.timber, matches!(self, Self::Round { .. }),
            ),
            _ => Vec::new(),
        };
        let sash_seat = mount.map(|m| {
            let reach = sash_overlap + m.clearance;
            self.window(fw - reach, &rect.inset(-reach))
        });
        fit.day = day.bounds();
        fit.glass = glass.bounds();
        fit.rebate = rebate.bounds();
        fit.sash_outer = sash_outer.as_ref().map(Contour::bounds);
        fit.contours = Some(FitContours {
            day,
            glass,
            rebate,
            bead_inner,
            sash_outer,
            sash_seat,
            sash_clearance: mount.map(|m| m.clearance),
            sash_seat_depth: mount.and_then(|m| m.seat_depth),
            sash_opens_outward: mount.map(|m| m.outward),
            sash_leaf: mount.filter(|m| m.pair).map(|_| if index == 0 { SashLeaf::Left } else { SashLeaf::Right }),
            sash_timber,
            rebate_height,
            edge_clearance,
            sash_width,
            sash_overlap,
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{FrameShape, Kozijn};

    fn near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-6, "{actual} vs {expected}");
    }

    fn pane(w: f64, h: f64, shape: ShapeType, rise: Option<f64>) -> Kozijn {
        let mut k = Kozijn::new("Gebogen", "B01", w, h);
        k.frame.shape = FrameShape {
            shape_type: shape,
            arch_height: rise,
            ..Default::default()
        };
        k
    }

    #[test]
    fn round_glass_and_bead_offsets_are_radial_and_centred() {
        let mut k = pane(1200., 900., ShapeType::Round, None);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat {
            height_mm: 28.,
            ..Default::default()
        });
        let geom = compute_2d_geometry(&k);
        let fit = geom.cell_rects[0].fit.as_ref().unwrap();
        let curves = fit.contours.as_ref().unwrap();
        for (p, r) in [
            (&curves.day, 383.),
            (&curves.rebate, 400.),
            (&curves.glass, 395.),
            (&curves.bead_inner, 372.),
        ] {
            near(p.area(), PI * r * r);
            near(p.perimeter(), TAU * r);
            near(p.bounds().x, 600. - r);
            near(p.bounds().y, 450. - r);
            near(p.bounds().width, 2. * r);
            near(p.bounds().height, 2. * r);
            assert_eq!(p.segments.len(), 2);
        }
    }

    #[test]
    fn clipping_on_the_circle_uses_analytic_arc_area_and_extrema() {
        let circle = FrameCurve::Round {
            center: [50., 50.],
            radius: 50.,
        };
        let half = circle.window(
            0.,
            &Rect2D {
                x: 50.,
                y: 0.,
                width: 50.,
                height: 100.,
            },
        );
        near(half.area(), 0.5 * PI * 50. * 50.);
        near(half.perimeter(), PI * 50. + 100.);
        let cap = circle.window(
            0.,
            &Rect2D {
                x: 0.,
                y: 80.,
                width: 100.,
                height: 20.,
            },
        );
        near(cap.bounds().x, 10.);
        near(cap.bounds().width, 80.);
        near(cap.area(), 50_f64.powi(2) * (0.6_f64).acos() - 30. * 40.);
        // Tangent contact has no orderable area.
        let tangent = circle.window(
            0.,
            &Rect2D {
                x: 0.,
                y: 100.,
                width: 100.,
                height: 25.,
            },
        );
        assert!(tangent.segments.is_empty());
    }

    #[test]
    fn segmental_head_day_glass_and_bead_share_a_centre_and_exact_spring() {
        let k = pane(1200., 1500., ShapeType::Arched, Some(250.));
        let geom = compute_2d_geometry(&k);
        let fit = geom.cell_rects[0].fit.as_ref().unwrap();
        let curves = fit.contours.as_ref().unwrap();
        let Segment::Arc {
            from,
            center,
            radius,
            ..
        } = curves.glass.segments[0]
        else {
            panic!("top arc missing")
        };
        near(center[0], 600.);
        near(center[1], 845.);
        near(radius, 790.);
        near(from[0], 55.);
        near(from[1], 845. - (790_f64.powi(2) - 545_f64.powi(2)).sqrt());
        near(fit.glass.y, 55.);
        near(fit.glass.height, 1390.);
        near(fit.glass.width, 1090.);
        assert!(curves.glass.area() < 1390. * 1090.);
        near(curves.rebate_height, 17.);
        near(curves.edge_clearance, 5.);
        assert_eq!(curves.bead_inner.segments.len(), 4);
        near(curves.day.area(), curves.bead_inner.area());
    }

    #[test]
    fn top_row_reaches_the_crown_and_an_off_centre_pane_ends_on_the_arc() {
        let mut k = pane(1200., 1500., ShapeType::Arched, Some(250.));
        k.add_row(800.);
        k.add_column(300.);
        let g = compute_2d_geometry(&k);
        let left = g.cell_rects[0].fit.as_ref().unwrap();
        let right = g.cell_rects[1].fit.as_ref().unwrap();
        assert!(left.glass.y > 55.);
        near(right.glass.y, 55.);
        let x = left.glass.x + left.glass.width;
        near(
            left.glass.y,
            845. - (790_f64.powi(2) - (x - 600.).powi(2)).sqrt(),
        );
        assert!(g.curved_dividers[0].bounds().y < 250.);
        for cr in &g.cell_rects[2..] {
            assert!(cr
                .fit
                .as_ref()
                .unwrap()
                .contours
                .as_ref()
                .unwrap()
                .glass
                .segments
                .iter()
                .all(|s| matches!(s, Segment::Line { .. })));
        }
    }

    #[test]
    fn tilted_jamb_is_offset_normal_to_its_actual_face() {
        let mut k = pane(1400., 1200., ShapeType::ArchedTrapezoid, Some(200.));
        k.frame.shape.left_angle = Some(75.);
        let g = compute_2d_geometry(&k);
        let glass = &g.cell_rects[0]
            .fit
            .as_ref()
            .unwrap()
            .contours
            .as_ref()
            .unwrap()
            .glass;
        let slope = 15_f64.to_radians().tan();
        let left_jamb = glass
            .segments
            .iter()
            .find_map(|s| match s {
                Segment::Line { from, to } if to[1] < from[1] => Some((*from, *to)),
                _ => None,
            })
            .unwrap();
        for p in [left_jamb.0, left_jamb.1] {
            near((p[0] - slope * (1200. - p[1])) / slope.hypot(1.), 55.);
        }
    }

    #[test]
    fn layout_leaves_outside_circle_are_empty_and_a_hinged_sash_follows_the_circle() {
        use crate::layout::{glas, raam, split_col, split_row, VakChild};
        let child = |size, node| VakChild { size, node };
        let mut k = pane(1000., 1000., ShapeType::Round, None);
        k.layout = Some(split_row(vec![
            child(1., split_col(vec![child(1., glas()), child(12., glas())])),
            child(12., raam("draaikiep")),
        ]));
        let g = compute_2d_geometry(&k);
        let corner = &g.cell_rects[0];
        assert!(corner.opening_contour.as_ref().unwrap().segments.is_empty());
        assert!(corner.fit.is_none());
        let leaf = g.cell_rects[2]
            .fit
            .as_ref()
            .unwrap()
            .contours
            .as_ref()
            .unwrap();
        let arc_radius = |c: &Contour| {
            c.segments
                .iter()
                .find_map(|s| match s {
                    Segment::Arc { radius, .. } => Some(*radius),
                    _ => None,
                })
                .unwrap()
        };
        let outer_radius = arc_radius(leaf.sash_outer.as_ref().unwrap());
        near(outer_radius, 500. - 67. + leaf.sash_overlap);
        near(arc_radius(&leaf.day), outer_radius - leaf.sash_width);
        near(arc_radius(&leaf.glass), arc_radius(&leaf.day) + 12.);
    }
}
