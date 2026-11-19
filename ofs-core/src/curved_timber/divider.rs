//! Finished straight stock fitted to the frame day and to crossing dividers.
use super::{Contour, Rect2D, Segment, EPS};
use crate::layout::{intersect_interval_unions, subtract_intervals, LayoutGeometry, LayoutRect};
use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DividerEnd {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenon: Option<super::DividerTenon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenon_issue: Option<super::TenonIssue>,
    /// Acute angle between the end-to-end chord and the length of the blank.
    pub angle_deg: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius_mm: Option<f64>,
    /// Axial setbacks from the longest point at this end. For a mullion these
    /// are read left/right and for a transom top/bottom in the elevation.
    pub edge_offsets_mm: [f64; 2],
    /// Open machining path in that same order. A shoulder crossing the arch
    /// spring can include both part of the arc and part of the jamb.
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvedDividerMember {
    pub vertical: bool,
    /// Zero-based source position within the vertical or horizontal dividers.
    pub source_index: usize,
    pub piece_index: usize,
    pub outline: Contour,
    pub net_length_mm: f64,
    pub start: DividerEnd,
    pub end: DividerEnd,
}

/// A single arc can form both ends at the tip of a narrow circular segment.
/// Split exactly at transverse extrema before assigning edges to the two ends.
fn end_edges(contour: &Contour, vertical: bool) -> [Vec<Segment>; 2] {
    let cross = usize::from(!vertical);
    let mut ends = [Vec::new(), Vec::new()];
    for edge in &contour.segments {
        let mut cuts = vec![0., 1.];
        if let Segment::Arc {
            start_angle,
            sweep_angle,
            ..
        } = edge
        {
            let first = if vertical { 0. } else { FRAC_PI_2 };
            for angle in [first, first + PI] {
                let t = ((angle - start_angle) * sweep_angle.signum()).rem_euclid(TAU)
                    / sweep_angle.abs();
                if t > 0. && t < 1. {
                    cuts.push(t);
                }
            }
            cuts.sort_by(f64::total_cmp);
        }
        for pair in cuts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let delta = edge.point(b)[cross] - edge.point(a)[cross];
            if delta.abs() <= EPS {
                continue; // parallel sides of the blank are not its end faces
            }
            // Clockwise border: the top runs right, the left side runs up.
            let at_end = if vertical { delta < 0. } else { delta > 0. };
            ends[usize::from(at_end)].push(if delta > 0. {
                edge.section(a, b)
            } else {
                edge.section(b, a)
            });
        }
    }
    for parts in &mut ends {
        parts.sort_by(|a, b| a.from()[cross].total_cmp(&b.from()[cross]));
        // A full circle is stored as two SVG arcs. Its horizontal diameter is
        // not a break in a transom's machined left or right end.
        let mut joined = Vec::with_capacity(parts.len());
        for part in parts.drain(..) {
            if let (
                Some(Segment::Arc {
                    center,
                    radius,
                    sweep_angle,
                    to,
                    ..
                }),
                Segment::Arc {
                    center: c,
                    radius: r,
                    sweep_angle: s,
                    from,
                    to: b,
                    ..
                },
            ) = (joined.last_mut(), &part)
            {
                if center == c
                    && radius == r
                    && sweep_angle.signum() == s.signum()
                    && (to[0] - from[0]).hypot(to[1] - from[1]) < EPS
                {
                    *sweep_angle += s;
                    *to = *b;
                    continue;
                }
            }
            joined.push(part);
        }
        *parts = joined;
    }
    ends
}

fn finish_end(segments: Vec<Segment>, axis: usize, long_point: f64) -> Option<DividerEnd> {
    let a = segments.first()?.from();
    let b = segments.last()?.to();
    let radius_mm = segments.iter().find_map(|s| match s {
        Segment::Arc { radius, .. } => Some(*radius),
        Segment::Line { .. } => None,
    });
    Some(DividerEnd {
        tenon: None,
        tenon_issue: None,
        angle_deg: (b[1 - axis] - a[1 - axis])
            .abs()
            .atan2((b[axis] - a[axis]).abs())
            .to_degrees(),
        radius_mm,
        edge_offsets_mm: [(a[axis] - long_point).abs(), (b[axis] - long_point).abs()],
        segments,
    })
}

fn member(
    outline: Contour,
    vertical: bool,
    source_index: usize,
    piece_index: usize,
) -> Option<CurvedDividerMember> {
    let axis = usize::from(vertical);
    let b = outline.bounds();
    let (lo, length) = if vertical {
        (b.y, b.height)
    } else {
        (b.x, b.width)
    };
    let [start, end] = end_edges(&outline, vertical);
    Some(CurvedDividerMember {
        vertical,
        source_index,
        piece_index,
        start: finish_end(start, axis, lo)?,
        end: finish_end(end, axis, lo + length)?,
        net_length_mm: length,
        outline,
    })
}

/// Side of the divider and the axial interval occupied by an adjacent room.
fn contact(rect: &Rect2D, vertical: bool, room: &LayoutRect) -> Option<(usize, (f64, f64))> {
    let side = |first: f64, last: f64, room_first: f64, room_last: f64| {
        if (first - room_last).abs() < EPS {
            Some(0)
        } else if (last - room_first).abs() < EPS {
            Some(1)
        } else {
            None
        }
    };
    if vertical {
        side(rect.x, rect.x + rect.width, room.x, room.x + room.width)
            .map(|i| (i, (room.y, room.y + room.height)))
    } else {
        side(rect.y, rect.y + rect.height, room.y, room.y + room.height)
            .map(|i| (i, (room.x, room.x + room.width)))
    }
}

fn walled_spans(rect: &Rect2D, vertical: bool, walls: &[LayoutRect]) -> Vec<(f64, f64)> {
    let mut sides = [Vec::new(), Vec::new()];
    for (side, span) in walls.iter().filter_map(|w| contact(rect, vertical, w)) {
        sides[side].push(span);
    }
    intersect_interval_unions(&sides[0], &sides[1], EPS)
}

pub(crate) fn divider_members(
    contours: &[Contour],
    mullions: &[Rect2D],
    transoms: &[Rect2D],
    layout: Option<&LayoutGeometry>,
) -> Vec<CurvedDividerMember> {
    let mut pieces = Vec::new();
    let walls: Vec<_> = layout
        .into_iter()
        .flat_map(|l| &l.leaves)
        .filter(|leaf| leaf.vulling.is_buiten())
        .map(|leaf| leaf.rect)
        .collect();
    if layout.is_some_and(|l| l.leaves.len() == walls.len()) {
        return pieces;
    }
    for (index, outline) in contours.iter().enumerate() {
        if outline.segments.is_empty() {
            continue;
        }
        let vertical = index < mullions.len();
        let source_index = if vertical {
            index
        } else {
            index - mullions.len()
        };
        // Grid transoms run through. Free-tree members already terminate on
        // either side of the parent splitter, irrespective of its orientation.
        let rect = if vertical {
            &mullions[source_index]
        } else {
            &transoms[source_index]
        };
        let blocked = if vertical && layout.is_none() {
            transoms
                .iter()
                .filter(|h| h.x <= rect.x + EPS && h.x + h.width >= rect.x + rect.width - EPS)
                .map(|h| (h.y, h.y + h.height))
                .collect()
        } else {
            walled_spans(rect, vertical, &walls)
        };
        if blocked.is_empty() {
            pieces.extend(member(outline.clone(), vertical, source_index, 0));
            continue;
        }
        let b = outline.bounds();
        let (lo, hi) = if vertical {
            (b.y, b.y + b.height)
        } else {
            (b.x, b.x + b.width)
        };
        let mut piece_index = 0;
        for (a, z) in subtract_intervals(lo, hi, &blocked, EPS) {
            // At a T junction the child splitter leaves a gap between the
            // two wall rectangles. That gap alone is not a stub to machine.
            if !walls.is_empty()
                && layout.is_some_and(|l| {
                    !l.leaves
                        .iter()
                        .filter(|leaf| !leaf.vulling.is_buiten())
                        .filter_map(|leaf| contact(rect, vertical, &leaf.rect))
                        .any(|(_, (start, end))| end.min(z) > start.max(a) + EPS)
                })
            {
                continue;
            }
            let window = if vertical {
                Rect2D {
                    y: a,
                    height: z - a,
                    ..b
                }
            } else {
                Rect2D {
                    x: a,
                    width: z - a,
                    ..b
                }
            };
            let segment = outline.clone().clipped(&window);
            if let Some(piece) = member(segment, vertical, source_index, piece_index) {
                pieces.push(piece);
                piece_index += 1;
            }
        }
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curved_contour::FrameCurve;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{Kozijn, ShapeType};

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }

    fn day(shape: ShapeType) -> FrameCurve {
        let mut k = Kozijn::new("Test", "R1", 1200., 1200.);
        k.frame.shape.shape_type = shape;
        k.frame.shape.arch_height = Some(300.);
        FrameCurve::new(&k.frame).unwrap()
    }

    fn fitted(shape: &FrameCurve, rect: Rect2D, vertical: bool) -> CurvedDividerMember {
        member(shape.window(67., &rect), vertical, 0, 0).unwrap()
    }

    #[test]
    fn off_centre_round_mullion_has_two_handed_copes_and_real_chord_angle() {
        let d = fitted(
            &day(ShapeType::Round),
            Rect2D {
                x: 300.,
                y: 0.,
                width: 67.,
                height: 1200.,
            },
            true,
        );
        let reach = |x: f64| (533_f64.powi(2) - (x - 600.).powi(2)).sqrt();
        let dx = reach(367.) - reach(300.);
        near(d.net_length_mm, 2. * reach(367.));
        for end in [&d.start, &d.end] {
            assert_eq!(end.radius_mm, Some(533.));
            near(end.angle_deg, 67_f64.atan2(dx).to_degrees());
            near(end.edge_offsets_mm[0], dx);
            near(end.edge_offsets_mm[1], 0.);
            assert_eq!(end.segments.len(), 1);
            near(end.segments[0].from()[0], 300.);
            near(end.segments[0].to()[0], 367.);
        }
        near(d.start.segments[0].from()[1], 600. - reach(300.));
        near(d.end.segments[0].from()[1], 600. + reach(300.));
    }

    #[test]
    fn diameter_transom_uses_crown_for_length_and_ignores_svg_semicircle_seam() {
        let d = fitted(
            &day(ShapeType::Round),
            Rect2D {
                x: 0.,
                y: 566.5,
                width: 1200.,
                height: 67.,
            },
            false,
        );
        let inset = 533. - (533_f64.powi(2) - 33.5_f64.powi(2)).sqrt();
        near(d.net_length_mm, 1066.);
        for end in [&d.start, &d.end] {
            near(end.angle_deg, 90.);
            assert_eq!(end.radius_mm, Some(533.));
            near(end.edge_offsets_mm[0], inset);
            near(end.edge_offsets_mm[1], inset);
            assert_eq!(end.segments.len(), 1);
            near(end.segments[0].from()[1], 566.5);
            near(end.segments[0].to()[1], 633.5);
        }
    }

    #[test]
    fn circle_tip_splits_the_same_arc_between_both_ends_without_a_phantom_side() {
        let r = day(ShapeType::Round);
        let rect = Rect2D {
            x: 35.,
            y: 0.,
            width: 46.,
            height: 1200.,
        };
        let d = fitted(&r, rect, true);
        let reach = (533_f64.powi(2) - 519_f64.powi(2)).sqrt();
        near(d.net_length_mm, 2. * reach);
        for end in [&d.start, &d.end] {
            assert_eq!(end.radius_mm, Some(533.));
            near(end.segments[0].from()[0], 67.);
            near(end.segments[0].from()[1], 600.);
            near(end.edge_offsets_mm[0], reach);
            near(end.angle_deg, 14_f64.atan2(reach).to_degrees());
        }
        let tangent_only = r.window(
            67.,
            &Rect2D {
                x: 0.,
                y: 0.,
                width: 67.,
                height: 1200.,
            },
        );
        assert!(member(tangent_only, true, 0, 0).is_none());
    }

    #[test]
    fn transom_at_spring_records_arc_then_straight_jamb_at_both_ends() {
        let y = 750. - (683_f64.powi(2) - 533_f64.powi(2)).sqrt();
        let rect = Rect2D {
            x: 0.,
            y: y - 20.,
            width: 1200.,
            height: 67.,
        };
        let d = fitted(&day(ShapeType::Arched), rect, false);
        let upper = 600. - (683_f64.powi(2) - (750. - y + 20.).powi(2)).sqrt();
        near(d.net_length_mm, 1066.);
        for end in [&d.start, &d.end] {
            assert_eq!(end.radius_mm, Some(683.));
            assert!(matches!(
                &end.segments[..],
                [Segment::Arc { .. }, Segment::Line { .. }]
            ));
            near(end.segments[0].to()[1], y);
            near(end.segments[1].from()[1], y);
            near(end.edge_offsets_mm[0], upper - 67.);
            near(end.edge_offsets_mm[1], 0.);
            near(end.angle_deg, 67_f64.atan2(upper - 67.).to_degrees());
        }
    }

    #[test]
    fn sloping_straight_jamb_uses_saw_angle_without_cope() {
        let mut k = Kozijn::new("Test", "B1", 1500., 1200.);
        k.frame.shape.shape_type = ShapeType::ArchedTrapezoid;
        k.frame.shape.arch_height = Some(170.);
        k.frame.shape.left_angle = Some(78.);
        k.frame.shape.right_angle = Some(82.);
        let d = fitted(
            &FrameCurve::new(&k.frame).unwrap(),
            Rect2D {
                x: 0.,
                y: 765.,
                width: 1500.,
                height: 67.,
            },
            false,
        );
        for (end, angle) in [(&d.start, 78_f64), (&d.end, 82_f64)] {
            assert!(end.radius_mm.is_none());
            assert_eq!(end.segments.len(), 1);
            near(end.angle_deg, angle);
            near(
                end.edge_offsets_mm[0],
                67. * (90. - angle).to_radians().tan(),
            );
            near(end.edge_offsets_mm[1], 0.);
        }
    }

    #[test]
    fn through_transom_and_both_mullion_pieces_partition_a_grid_crossing() {
        let mut k = Kozijn::new("Test", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        k.add_column(533.);
        k.add_row(533.);
        let geo = compute_2d_geometry(&k);
        let parts = &geo.curved_divider_timber;
        assert_eq!(parts.len(), 3);
        let (top, bottom, rail) = (&parts[0], &parts[1], &parts[2]);
        assert_eq!(
            (
                top.source_index,
                top.piece_index,
                bottom.source_index,
                bottom.piece_index
            ),
            (0, 0, 0, 1)
        );
        assert!(top.vertical && bottom.vertical && !rail.vertical);
        for (formed, square) in [(&top.start, &top.end), (&bottom.end, &bottom.start)] {
            assert_eq!(formed.radius_mm, Some(533.));
            assert!(square.radius_mm.is_none());
            near(square.angle_deg, 90.);
            near(square.edge_offsets_mm[0], 0.);
            near(square.edge_offsets_mm[1], 0.);
        }
        near(rail.net_length_mm, 1066.);
        near(top.net_length_mm + bottom.net_length_mm + 67., 1066.);
        near(
            parts.iter().map(|d| d.outline.area()).sum::<f64>(),
            geo.curved_dividers.iter().map(Contour::area).sum::<f64>() - 67. * 67.,
        );
    }

    #[test]
    fn day_referenced_arch_grid_can_place_a_transom_in_the_cap() {
        let mut k = Kozijn::new("Test", "B1", 1200., 1500.);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.add_row(100.);
        let old = compute_2d_geometry(&k);
        assert!(old.h_dividers[0].y >= 300.);
        k.grid.rows_from_day = true;
        let g = compute_2d_geometry(&k);
        let rail = &g.curved_divider_timber[0];
        let y = 67. + k.grid.rows[0].size;
        near(g.h_dividers[0].y, y);
        near(
            rail.net_length_mm,
            2. * (683_f64.powi(2) - (750. - y - 67.).powi(2)).sqrt(),
        );
        for end in [&rail.start, &rail.end] {
            assert_eq!(end.radius_mm, Some(683.));
            near(end.segments[0].from()[1], y);
        }
    }

    #[test]
    fn side_light_on_masonry_retains_real_sill_and_full_mullion() {
        let mut k = Kozijn::new("Test", "B1", 1400., 1900.);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(250.);
        k.layout = Some(crate::layout::melkmeisje1());
        let g = compute_2d_geometry(&k);
        assert_eq!(
            (
                g.v_dividers.len(),
                g.h_dividers.len(),
                g.curved_divider_timber.len()
            ),
            (1, 1, 2)
        );
        let (post, rail) = (&g.curved_divider_timber[0], &g.curved_divider_timber[1]);
        assert!(post.vertical && !rail.vertical);
        assert!(post.start.radius_mm.is_some());
        near(post.outline.bounds().y + post.net_length_mm, 1833.);
        near(rail.net_length_mm, g.h_dividers[0].width);
        assert_eq!(rail.start.radius_mm, None);
        assert_eq!(rail.end.radius_mm, None);
    }

    #[test]
    fn no_phantom_stock_or_display_contour_inside_masonry() {
        let mut k = Kozijn::new("Test", "R", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        let inside = serde_json::json!({"kind":"leaf","vulling":{"type":"glas"}});
        let wall = serde_json::json!({"kind":"leaf","vulling":{"type":"buiten"}});
        let right = serde_json::json!({"kind":"split","direction":"column","children":[{"size":1,"node":wall},{"size":1,"node":wall}]});
        let left = serde_json::json!({"kind":"split","direction":"column","children":[{"size":1,"node":inside},{"size":1,"node":wall}]});
        k.layout = Some(serde_json::from_value(serde_json::json!({"kind":"split","direction":"row","children":[{"size":1,"node":left},{"size":1,"node":right}]})).unwrap());
        let g = compute_2d_geometry(&k);
        assert_eq!(g.curved_divider_timber.len(), 2);
        assert_eq!(
            g.curved_dividers
                .iter()
                .filter(|c| c.segments.is_empty())
                .count(),
            1
        );
        let post = g.curved_divider_timber.iter().find(|p| p.vertical).unwrap();
        near(post.net_length_mm, 566.5);
    }

    #[test]
    fn free_tree_keeps_through_mullion_and_terminates_transom_at_it() {
        let mut k = Kozijn::new("Test", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        let leaf = serde_json::json!({"kind":"leaf", "vulling":{"type":"glas"}});
        let row = serde_json::json!({"kind":"split","direction":"column","children":[{"size":1,"node":leaf},{"size":1,"node":leaf}]});
        k.layout = Some(serde_json::from_value(serde_json::json!({"kind":"split","direction":"row","children":[{"size":1,"node":leaf},{"size":1,"node":row}]})).unwrap());
        let g = compute_2d_geometry(&k);
        let (stile, rail) = (&g.curved_divider_timber[0], &g.curved_divider_timber[1]);
        assert_eq!(g.curved_divider_timber.len(), 2);
        near(stile.net_length_mm, 1066.);
        assert_eq!(stile.start.radius_mm, Some(533.));
        assert_eq!(stile.end.radius_mm, Some(533.));
        near(rail.net_length_mm, 499.5);
        near(rail.start.angle_deg, 90.);
        assert_eq!(rail.start.radius_mm, None);
        assert_eq!(rail.end.radius_mm, Some(533.));
    }
}
