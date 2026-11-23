use super::{Contour, CurvedDividerMember, HeadMachiningStatus, Rect2D, Segment, EPS};
use crate::kozijn::{Frame, Kozijn, Material};

// VEKA SOFTLINE 82 MD, technical information §§4.5, 4.6 and 5.12–5.15.
// For the standard mechanical connector the PVC is cut to the glazing bead
// groove dimension (GLM). At a square joint the groove lies 73−48 = 47−22
// = 25 mm behind the day face of frame 101.290 or the face of T 102.310.
const VEKA_MD_GROOVE_OFFSET: f64 = 25.0;

struct StraightFace {
    axis: usize,
    at: f64,
    lo: f64,
    hi: f64,
}

impl StraightFace {
    fn from_segment(edge: &Segment) -> Option<Self> {
        let Segment::Line { from, to } = edge else {
            return None;
        };
        let axis = if (from[0] - to[0]).abs() < EPS {
            0
        } else if (from[1] - to[1]).abs() < EPS {
            1
        } else {
            return None;
        };
        Some(Self {
            axis,
            at: from[axis],
            lo: from[1 - axis].min(to[1 - axis]),
            hi: from[1 - axis].max(to[1 - axis]),
        })
    }

    fn contains(&self, end: &Self) -> bool {
        self.axis == end.axis
            && (self.at - end.at).abs() < EPS
            && self.lo <= end.lo + EPS
            && self.hi >= end.hi - EPS
    }

    fn is_md_frame(&self, frame: &Frame) -> bool {
        let fw = frame.frame_width;
        if frame.profile.id != "veka-softline82"
            || (fw - 73.0).abs() > EPS
            || (frame.frame_depth - 82.0).abs() > EPS
        {
            return false;
        }
        let (article, material) = match self.axis {
            0 if (self.at - fw).abs() < EPS => (&frame.left_profile, &frame.left_material),
            0 if (self.at - (frame.outer_width - fw)).abs() < EPS => {
                (&frame.right_profile, &frame.right_material)
            }
            1 if (self.at - (frame.outer_height - fw)).abs() < EPS
                && frame.sill.is_none()
                && frame
                    .sill_profile
                    .as_ref()
                    .is_none_or(|s| s.id == frame.profile.id) =>
            {
                (&frame.bottom_profile, &frame.bottom_material)
            }
            _ => return false,
        };
        article.as_ref().is_none_or(|p| p.id == frame.profile.id)
            && material.as_ref().unwrap_or(&frame.material) == &Material::Pvc
    }
}

pub(crate) fn resolve_system_heads(
    kozijn: &Kozijn,
    day: &Contour,
    verticals: &[Rect2D],
    horizontals: &[Rect2D],
    parts: &mut [CurvedDividerMember],
) {
    if !kozijn.frame.material.is_profile_system() {
        return;
    }
    let mut receivers: Vec<_> = day
        .segments
        .iter()
        .filter_map(StraightFace::from_segment)
        .filter(|face| face.is_md_frame(&kozijn.frame))
        .map(|face| (face, HeadMachiningStatus::VekaMdGlm))
        .collect();
    for part in parts.iter_mut() {
        let i = part.source_index;
        let (divisions, width) = if part.vertical {
            (&kozijn.grid.columns, verticals[i].width)
        } else {
            (&kozijn.grid.rows, horizontals[i].height)
        };
        let division = kozijn
            .layout
            .is_none()
            .then(|| divisions.get(i + 1))
            .flatten();
        let Some(article) = kozijn.system_divider_article(division, width) else {
            continue;
        };
        part.profile = Some(article.reference());
        if article.id == "veka-102310" {
            // Only the two long faces of a T receive another member. Its
            // own square end must never qualify itself as a receiving face.
            receivers.extend(
                part.outline
                    .segments
                    .iter()
                    .filter_map(StraightFace::from_segment)
                    .filter(|face| face.axis == usize::from(!part.vertical))
                    .map(|face| (face, HeadMachiningStatus::VekaMdGlmAtT)),
            );
        }
    }
    for part in parts {
        let standard_t = part.profile.as_ref().is_some_and(|p| p.id == "veka-102310");
        for end in [&mut part.start, &mut part.end] {
            let status = if end.radius_mm.is_some() {
                HeadMachiningStatus::DrawingRequired
            } else {
                match end.segments.as_slice() {
                    [line] if standard_t => StraightFace::from_segment(line)
                        .filter(|face| face.axis == usize::from(part.vertical))
                        .and_then(|face| receivers.iter().find(|(r, _)| r.contains(&face)))
                        .map_or(
                            HeadMachiningStatus::SectionDrawingRequired,
                            |(_, status)| *status,
                        ),
                    _ => HeadMachiningStatus::SectionDrawingRequired,
                }
            };
            end.machining_status = Some(status);
            end.saw_allowance_mm = (!status.requires_drawing()).then_some(VEKA_MD_GROOVE_OFFSET);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::ShapeType;
    use crate::profile::ProfileRef;

    fn frame() -> Kozijn {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let mut k = Kozijn::new_with_sjabloon("Standaardaansluiting", "K", 1200., 1500., &sj);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.add_divider_in_elevation(false, 600.);
        k.add_divider_in_elevation(true, 800.);
        k
    }

    fn rail(k: &Kozijn) -> CurvedDividerMember {
        compute_2d_geometry(k)
            .curved_divider_timber
            .into_iter()
            .find(|p| !p.vertical && p.piece_index == 0)
            .unwrap()
    }

    #[test]
    fn standard_cut_dimension_requires_known_donor_and_contact_over_the_whole_end() {
        let mut k = frame();
        let confirmed = rail(&k);
        assert_eq!(confirmed.profile.as_ref().unwrap().id, "veka-102310");
        assert_eq!(
            confirmed.start.machining_status,
            Some(HeadMachiningStatus::VekaMdGlm)
        );
        assert_eq!(
            confirmed.end.machining_status,
            Some(HeadMachiningStatus::VekaMdGlmAtT)
        );
        k.frame.left_profile = Some(ProfileRef {
            id: "eigen-kader".into(),
            name: "Eigen kader".into(),
        });
        let bespoke_jamb = rail(&k);
        assert_eq!(bespoke_jamb.start.saw_allowance_mm, None);
        assert_eq!(bespoke_jamb.end.saw_allowance_mm, Some(25.0));
        k.grid.columns[1].divider_profile = Some(ProfileRef {
            id: "eigen-stijl".into(),
            name: "Eigen T".into(),
        });
        assert_eq!(rail(&k).end.saw_allowance_mm, None);
        let mut k = frame();
        k.grid.rows[1].divider_depth = Some(79.);
        let altered = rail(&k);
        assert!(altered.profile.is_none());
        assert!(altered.start.saw_allowance_mm.is_none() && altered.end.saw_allowance_mm.is_none());
    }

    #[test]
    fn square_chord_against_a_round_head_has_no_glm_allowance() {
        let mut k = frame();
        k.frame.shape.shape_type = ShapeType::Round;
        k.frame.outer_height = 1200.;
        let g = compute_2d_geometry(&k);
        let stile = g.curved_divider_timber.iter().find(|p| p.vertical).unwrap();
        assert!((stile.start.angle_deg - 90.).abs() < EPS);
        assert_eq!(
            stile.start.machining_status,
            Some(HeadMachiningStatus::DrawingRequired)
        );
        assert_eq!(stile.start.saw_allowance_mm, None);
    }
}
