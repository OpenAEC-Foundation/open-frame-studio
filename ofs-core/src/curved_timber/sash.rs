//! Split a movable leaf into stock between the outside of the sash and its own
//! glazing day. Both boundaries have already been clipped to this particular
//! opening, including the meeting edge of a double leaf.
use super::*;

fn line_direction(edge: &Segment) -> Point {
    sub(edge.to(), edge.from())
}

fn parallel_forward(a: Point, b: Point) -> bool {
    let scale = norm(a) * norm(b);
    scale > 0. && dot(a, b) > 0. && (a[0] * b[1] - a[1] * b[0]).abs() <= 1e-8 * scale
}

fn merge_edges(first: &mut Segment, next: &Segment) -> bool {
    match (first, next) {
        (
            Segment::Arc {
                center,
                radius,
                to,
                sweep_angle,
                ..
            },
            Segment::Arc {
                center: c,
                radius: r,
                to: end,
                sweep_angle: sweep,
                ..
            },
        ) if norm(sub(*center, *c)) < EPS && (*radius - r).abs() < EPS => {
            *to = *end;
            *sweep_angle += sweep;
            true
        }
        (Segment::Line { from, to }, Segment::Line { from: a, to: b })
            if parallel_forward(sub(*to, *from), sub(*b, *a)) =>
        {
            *to = *b;
            true
        }
        _ => false,
    }
}

fn stock_edges(path: &Contour) -> Vec<Segment> {
    let mut edges = path.segments.clone();
    // The two semicircles in a complete ring each have radial end cuts. Once
    // clipped, the SVG seam at the horizontal diameter is no longer a joint.
    if let Some(i) = edges.iter().position(|e| matches!(e, Segment::Line { .. })) {
        edges.rotate_left(i);
        let mut joined = Vec::<Segment>::new();
        for edge in edges {
            if !joined
                .last_mut()
                .is_some_and(|last| merge_edges(last, &edge))
            {
                joined.push(edge);
            }
        }
        if joined.len() > 1 {
            let first = joined[0].clone();
            if merge_edges(joined.last_mut().unwrap(), &first) {
                joined.remove(0);
            }
        }
        joined
    } else {
        edges.sort_by(|a, b| a.point(0.5)[1].total_cmp(&b.point(0.5)[1]));
        edges
    }
}

/// The angular interval of an inset arc is contained in that of its parent:
/// all clipping planes are also offset inward by the sash width. For straight
/// stock the directed normal identifies the matching support line.
fn supports(outer: &Segment, inner: &Segment) -> bool {
    match (outer, inner) {
        (
            Segment::Arc {
                center,
                start_angle,
                sweep_angle,
                ..
            },
            Segment::Arc { .. },
        ) => {
            let p = sub(inner.point(0.5), *center);
            (p[1].atan2(p[0]) - start_angle).rem_euclid(TAU) < sweep_angle + 1e-8
        }
        (Segment::Line { .. }, Segment::Line { .. }) => {
            parallel_forward(line_direction(outer), line_direction(inner))
        }
        _ => false,
    }
}

fn role(edge: &Segment) -> TimberRole {
    if matches!(edge, Segment::Arc { .. }) {
        return TimberRole::Arc;
    }
    let [dx, dy] = line_direction(edge);
    if dy.abs() <= 1e-8 * dx.abs() {
        if dx > 0. {
            TimberRole::Top
        } else {
            TimberRole::Bottom
        }
    } else if dy > 0. {
        TimberRole::Right
    } else {
        TimberRole::Left
    }
}

fn stile(edge: &Segment) -> bool {
    matches!(role(edge), TimberRole::Left | TimberRole::Right)
}

#[derive(Clone, Copy)]
struct Joint {
    face: JointFace,
    // Wood rails terminate on a stile. The stile continues to the far edge of
    // the rail, so its end saw face differs from the mating shoulder.
    through_stile: bool,
}

fn make_joint(previous: &Segment, next: &Segment, inside: Point, timber: bool) -> Joint {
    let outside = next.from();
    let mitre = Joint {
        face: [outside, inside],
        through_stile: false,
    };
    if !timber
        || matches!(previous, Segment::Arc { .. })
        || matches!(next, Segment::Arc { .. })
        || stile(previous) == stile(next)
    {
        return mitre;
    }
    let (jamb, rail) = if stile(previous) {
        (previous, next)
    } else {
        (next, previous)
    };
    let dir = line_direction(jamb);
    let q = [
        inside[0] + (outside[1] - inside[1]) * dir[0] / dir[1],
        outside[1],
    ];
    if (q[0] - rail.from()[0]) * (q[0] - rail.to()[0]) > EPS {
        return mitre;
    }
    Joint {
        face: [q, inside],
        through_stile: true,
    }
}

fn arc_side(edge: &Segment, in_round_frame: bool) -> TimberProfileSide {
    if let Segment::Arc { center, .. } = edge {
        let [x, y] = sub(edge.point(0.5), *center);
        if in_round_frame && x.abs() > y.abs() + EPS {
            return if x < 0. {
                TimberProfileSide::Left
            } else {
                TimberProfileSide::Right
            };
        }
        if in_round_frame && y > 0. {
            return TimberProfileSide::Bottom;
        }
    }
    TimberProfileSide::Top
}

fn make_straight(
    edge: &Segment,
    previous: &Segment,
    next: &Segment,
    start: Joint,
    end: Joint,
) -> Option<CurvedTimberMember> {
    let role = role(edge);
    let is_stile = stile(edge);
    let long = |joint: Joint, corner| {
        if joint.through_stile && is_stile {
            corner
        } else {
            joint.face[0]
        }
    };
    let sa = long(start, edge.from());
    let sb = long(end, edge.to());
    let mut points = vec![
        start.face[0],
        sa,
        sb,
        end.face[0],
        end.face[1],
        start.face[1],
    ];
    let axis = |joint: Joint, adjacent| {
        if joint.through_stile && is_stile {
            line_direction(adjacent)
        } else {
            face_axis(joint.face)
        }
    };
    let mut edges = [sa, sb];
    let mut cuts = [start.face, end.face];
    let mut axes = [axis(start, previous), axis(end, next)];
    let mut tenons = [
        start.through_stile && !is_stile,
        end.through_stile && !is_stile,
    ];
    // Start/end conventions are left→right for rails and top→bottom for stiles.
    if matches!(role, TimberRole::Bottom | TimberRole::Left) {
        edges.reverse();
        cuts.reverse();
        axes.reverse();
        points.reverse();
        tenons.reverse();
    }
    let mut part = straight(role, [None, None], cuts, edges, axes, &points)?;
    part.start_tenon = tenons[0];
    part.end_tenon = tenons[1];
    Some(part)
}

pub(crate) fn sash_members(
    outside: &Contour,
    day: &Contour,
    width: f64,
    timber: bool,
    round: bool,
) -> Vec<CurvedTimberMember> {
    let outer = stock_edges(outside);
    let inner = stock_edges(day);
    if inner.is_empty() || outer.is_empty() {
        return Vec::new();
    }
    let n = outer.len();
    let prev = |i: usize| (i + n - 1) % n;
    let next = |i: usize| (i + 1) % n;
    let matched: Vec<_> = outer
        .iter()
        .map(|e| inner.iter().find(|s| supports(e, s)))
        .collect();
    let Some(anchor) = matched.iter().position(Option::is_some) else {
        return Vec::new();
    };
    // A short chamfer, arc or jamb can disappear at the day. Both adjoining
    // saw planes then reach the same inner corner; there is still real stock.
    let mut inside = vec![[0., 0.]; n];
    let mut corner = matched[anchor].unwrap().from();
    for offset in 0..n {
        let i = (anchor + offset) % n;
        inside[i] = corner;
        if let Some(edge) = matched[i] {
            corner = edge.to();
        }
    }
    let mut joints: Vec<_> = (0..n)
        .map(|i| make_joint(&outer[prev(i)], &outer[i], inside[i], timber))
        .collect();
    for i in 0..n {
        if !matches!(role(&outer[i]), TimberRole::Top | TimberRole::Bottom) {
            continue;
        }
        let end = next(i);
        if dot(
            sub(joints[end].face[0], joints[i].face[0]),
            line_direction(&outer[i]),
        ) <= EPS
        {
            // The rail is too short for two through-stile shoulders. Retain
            // the full outer edge and join to the adjacent pieces on a bevel.
            for k in [i, end] {
                joints[k] = Joint {
                    face: [outer[k].from(), inside[k]],
                    through_stile: false,
                };
            }
        }
    }
    let mut pieces = Vec::<CurvedTimberMember>::with_capacity(n);
    for (i, edge) in outer.iter().enumerate() {
        let j = next(i);
        let start = joints[i];
        let end = joints[j];
        let member = if let Segment::Arc { center, radius, .. } = edge {
            let apex = Segment::Arc {
                from: start.face[1],
                to: end.face[1],
                center: *center,
                radius: radius - width,
                start_angle: 0.,
                sweep_angle: 0.,
            };
            arc_member(edge, matched[i].unwrap_or(&apex), 0, [None, None]).map(|mut p| {
                p.outline.segments.retain(|s| s.length() > EPS);
                p.profile_side = arc_side(edge, round);
                p.start_tenon = timber && stile(&outer[prev(i)]);
                p.end_tenon = timber && stile(&outer[j]);
                p
            })
        } else {
            make_straight(edge, &outer[prev(i)], &outer[j], start, end)
        };
        if let Some(mut member) = member {
            member.index = pieces.iter().filter(|p| p.role == member.role).count();
            pieces.push(member);
        }
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curved_contour::{FitContours, FrameCurve, SashLeaf};
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{Kozijn, OpeningDirection, PanelType};
    use std::f64::consts::PI;

    fn leaf(shape: ShapeType) -> Kozijn {
        let mut k = Kozijn::new("Draairaam", "R01", 1200., 1500.);
        k.frame.shape.shape_type = shape;
        k.frame.shape.arch_height = Some(300.);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k
    }
    fn fit(k: &Kozijn) -> FitContours {
        compute_2d_geometry(k)
            .cell_rects
            .remove(0)
            .fit
            .unwrap()
            .contours
            .unwrap()
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 2e-6, "{a} vs {b}");
    }
    fn arc_radius(contour: &Contour) -> f64 {
        contour
            .segments
            .iter()
            .find_map(|e| {
                if let Segment::Arc { radius, .. } = e {
                    Some(*radius)
                } else {
                    None
                }
            })
            .unwrap()
    }
    fn exact_ring(outer: &Contour, inner: &Contour, parts: &[CurvedTimberMember]) {
        assert!(!parts.is_empty());
        close(
            parts.iter().map(|p| p.outline.area()).sum(),
            outer.area() - inner.area(),
        );
        for p in parts {
            assert!(
                p.net_length_mm > 0.
                    && p.start_angle_deg.is_finite()
                    && p.end_angle_deg.is_finite()
            );
            for (i, s) in p.outline.segments.iter().enumerate() {
                let next = &p.outline.segments[(i + 1) % p.outline.segments.len()];
                close(norm(sub(s.to(), next.from())), 0.);
            }
        }
    }

    #[test]
    fn full_round_leaf_sits_in_the_catalogue_rebate_in_both_directions() {
        let mut k = leaf(ShapeType::Round);
        for (direction, overslag) in [
            (OpeningDirection::Inward, 14.),
            (OpeningDirection::Outward, 17.),
        ] {
            k.cells[0].opening_direction = Some(direction);
            let c = fit(&k);
            let outside = c.sash_outer.as_ref().unwrap();
            close(arc_radius(outside), 600. - 67. + overslag);
            assert_eq!(c.sash_clearance, Some(6.));
            assert_eq!(c.sash_seat_depth, Some(29.));
            assert_eq!(
                c.sash_opens_outward,
                Some(direction == OpeningDirection::Outward)
            );
            close(
                arc_radius(c.sash_seat.as_ref().unwrap()) - arc_radius(outside),
                6.,
            );
            assert_eq!(c.sash_timber.len(), 2);
            assert_eq!(c.sash_timber[0].profile_side, TimberProfileSide::Top);
            assert_eq!(c.sash_timber[1].profile_side, TimberProfileSide::Bottom);
            for piece in &c.sash_timber {
                let a = piece.arc.as_ref().unwrap();
                close(a.outer_length_mm, PI * arc_radius(outside));
                close(a.centerline_length_mm, PI * (arc_radius(outside) - 34.5));
                close(piece.start_angle_deg, 90.);
                close(piece.end_angle_deg, 90.);
                assert!(!piece.start_tenon && !piece.end_tenon);
            }
            exact_ring(outside, &c.day, &c.sash_timber);
        }
    }

    #[test]
    fn segment_head_and_stiles_share_the_exact_sash_shoulders() {
        let c = fit(&leaf(ShapeType::Arched));
        let outside = c.sash_outer.as_ref().unwrap();
        let stock = &c.sash_timber;
        let head = stock.iter().find(|p| p.role == TimberRole::Arc).unwrap();
        let jamb = stock.iter().find(|p| p.role == TimberRole::Left).unwrap();
        let rail = stock.iter().find(|p| p.role == TimberRole::Bottom).unwrap();
        let outside_r: f64 = 750. - 67. + 14.;
        let inside_r = outside_r - 69.;
        let x0 = 600. - 53.;
        let x1 = x0 - 69.;
        let y0 = 750. - (outside_r * outside_r - x0 * x0).sqrt();
        let y1 = 750. - (inside_r * inside_r - x1 * x1).sqrt();
        let sweep = 2. * (x0 / outside_r).asin();
        close(
            head.arc.as_ref().unwrap().outer_length_mm,
            outside_r * sweep,
        );
        let slope = ((y1 - y0) / 69.).atan();
        close(head.start_angle_deg, (sweep / 2. + slope).to_degrees());
        close(jamb.start_angle_deg, 90. - slope.to_degrees());
        close(norm(sub(head.start_cut[1], jamb.start_cut[1])), 0.);
        assert!(head.start_tenon && head.end_tenon && rail.start_tenon && rail.end_tenon);
        close(rail.net_length_mm, 2. * x1);
        exact_ring(outside, &c.day, stock);
    }

    #[test]
    fn paired_leaves_share_one_seat_and_keep_the_meeting_stiles_apart() {
        let mut k = leaf(ShapeType::Round);
        k.layout = Some(crate::layout::deur("dubbel"));
        let c = compute_2d_geometry(&k).cell_rects.remove(0);
        let left = c.fit.unwrap().contours.unwrap();
        let right = c.second_fit.unwrap().contours.unwrap();
        assert_eq!(left.sash_leaf, Some(SashLeaf::Left));
        assert_eq!(right.sash_leaf, Some(SashLeaf::Right));
        close(
            arc_radius(left.sash_seat.as_ref().unwrap()),
            arc_radius(right.sash_seat.as_ref().unwrap()),
        );
        let l = left.sash_outer.as_ref().unwrap().bounds();
        let r = right.sash_outer.as_ref().unwrap().bounds();
        close(r.x - (l.x + l.width), 6.);
        for (c, side) in [
            (&left, TimberProfileSide::Left),
            (&right, TimberProfileSide::Right),
        ] {
            let bow = c.sash_timber.iter().find(|p| p.arc.is_some()).unwrap();
            assert_eq!(bow.profile_side, side);
            assert_eq!(c.sash_timber.len(), 2);
            exact_ring(c.sash_outer.as_ref().unwrap(), &c.day, &c.sash_timber);
        }
    }

    #[test]
    fn clipped_leaves_tile_the_ring_even_when_inner_short_edges_disappear() {
        let mut count = 0;
        let mut changed = 0;
        for shape in [
            ShapeType::Round,
            ShapeType::Arched,
            ShapeType::ArchedTrapezoid,
        ] {
            let mut k = leaf(shape);
            k.frame.shape.left_angle = Some(75.);
            k.frame.shape.right_angle = Some(84.);
            let boundary = FrameCurve::new(&k.frame).unwrap();
            for x in [53., 107., 310., 587., 736.] {
                for y in [53., 233., 499., 740.] {
                    for width in [249., 497., 928.] {
                        for height in [299., 528., 901.] {
                            let clip = Rect2D {
                                x,
                                y,
                                width,
                                height,
                            };
                            let o = boundary.window(53., &clip);
                            let d = boundary.window(122., &clip.inset(69.));
                            if d.segments.is_empty() {
                                continue;
                            }
                            for wood in [true, false] {
                                let pieces =
                                    sash_members(&o, &d, 69., wood, shape == ShapeType::Round);
                                exact_ring(&o, &d, &pieces);
                            }
                            if stock_edges(&o).len() != stock_edges(&d).len() {
                                changed += 1;
                            }
                            count += 1;
                        }
                    }
                }
            }
        }
        assert!(count > 250 && changed > 10);
    }

    #[test]
    fn paired_door_in_the_top_left_sector_keeps_its_surviving_right_leaf_id() {
        use crate::layout::{deur, glas, split_col, split_row, VakChild};
        let mut k = leaf(ShapeType::Round);
        k.frame.outer_height = 1200.;
        let child = |size, node| VakChild { size, node };
        k.layout = Some(split_row(vec![
            child(
                39.,
                split_col(vec![child(24., deur("dubbel")), child(76., glas())]),
            ),
            child(61., glas()),
        ]));
        let cr = compute_2d_geometry(&k).cell_rects.remove(0);
        assert_eq!(cr.missing_glass_panes, 1);
        assert!(cr.second_fit.is_none());
        let f = cr.fit.unwrap().contours.unwrap();
        assert_eq!(f.sash_leaf, Some(SashLeaf::Right));
        exact_ring(f.sash_outer.as_ref().unwrap(), &f.day, &f.sash_timber);
    }

    #[test]
    fn known_and_custom_frame_shoulders_resolve_per_saved_dimension() {
        let mut k = leaf(ShapeType::Round);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot::default());
        let c = fit(&k);
        assert_eq!(c.sash_clearance, Some(6.));
        assert_eq!(c.sash_seat_depth, Some(29.));
        k.frame.profile.id = "custom".into();
        let c = fit(&k);
        assert_eq!(c.sash_clearance, Some(4.));
        assert_eq!(c.sash_seat_depth, None);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sash_clearance: Some(3.5),
            sash_seat_depth: Some(26.),
            ..Default::default()
        });
        let c = fit(&k);
        assert_eq!(c.sash_clearance, Some(3.5));
        assert_eq!(c.sash_seat_depth, Some(26.));
        close(
            arc_radius(c.sash_seat.as_ref().unwrap()) - arc_radius(c.sash_outer.as_ref().unwrap()),
            3.5,
        );
    }
}
