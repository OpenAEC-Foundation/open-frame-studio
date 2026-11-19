//! Finished outside-frame stock. The glass geometry defines both shoulders;
//! their connecting plane is shared by the head and the adjoining jamb.
use crate::curved_contour::{Contour, CurvedFrame, Segment};
use crate::geometry::Rect2D;
use crate::joint::{frame_corner_model, CornerModel, ThroughMember};
use crate::kozijn::{Frame, ShapeType};
use crate::layout::LayoutGeometry;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

mod divider;
mod tenon;
mod sash;
pub use divider::{CurvedDividerMember, DividerEnd};
pub use tenon::{ArchJoint, ArchJointKind, ArchTenon, DividerTenon, FrameMemberRef, TenonIssue, TenonReceiver};
pub(crate) use tenon::fit_timber_tenons;
pub(crate) use divider::divider_members;
pub(crate) use sash::sash_members;

type Point = [f64; 2];
type JointFace = [Point; 2]; // outer boundary to day boundary, mm
const EPS: f64 = 1e-7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimberRole {
    Arc,
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimberProfileSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimberArc {
    pub outer_radius_mm: f64,
    pub outer_length_mm: f64,
    pub centerline_radius_mm: f64,
    pub centerline_length_mm: f64,
    pub sweep_deg: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvedTimberMember {
    pub role: TimberRole,
    pub profile_side: TimberProfileSide,
    pub index: usize,
    /// For a through-member these stop lines are on its mating side.
    pub start_cut: JointFace,
    pub end_cut: JointFace,
    pub start_corner: Option<usize>,
    pub end_corner: Option<usize>,
    pub outer_edge: Segment,
    pub outline: Contour,
    /// Subtract these rectangles in global elevation coordinates after cutting the blank.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub masonry_notches: Vec<Rect2D>,
    /// Developed outside edge for a bend; projected long point for straight stock.
    pub net_length_mm: f64,
    /// Acute angle of the end cut to the outer tangent / straight member axis.
    pub start_angle_deg: f64,
    pub end_angle_deg: f64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub start_tenon: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub end_tenon: bool,
    pub arc: Option<TimberArc>,
}

fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm(p: Point) -> f64 {
    p[0].hypot(p[1])
}
fn lerp(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
fn line(from: Point, to: Point) -> Segment {
    Segment::Line { from, to }
}
fn cut_angle(axis: Point, face: Point) -> f64 {
    (axis[0] * face[1] - axis[1] * face[0])
        .abs()
        .atan2(dot(axis, face).abs())
        .to_degrees()
}
fn face_axis(f: JointFace) -> Point {
    sub(f[1], f[0])
}

/// Locate a concentric reference circle on the actual flat cut. Using the
/// outside sweep on every radius would miss the intersection on a segment arch.
fn circle_on_face(face: JointFace, center: Point, r: f64) -> Point {
    let q = sub(face[0], center);
    let d = face_axis(face);
    let b = dot(q, d);
    let c = (norm(q) - r) * (norm(q) + r);
    let root = (-b + (b * b - dot(d, d) * c).max(0.).sqrt()).max(EPS);
    lerp(face[0], face[1], c / root)
}

fn arc_member(
    outer: &Segment,
    inner: &Segment,
    index: usize,
    corners: [Option<usize>; 2],
) -> Option<CurvedTimberMember> {
    let Segment::Arc {
        center,
        radius,
        start_angle,
        sweep_angle,
        ..
    } = outer
    else {
        return None;
    };
    let Segment::Arc {
        radius: inner_r, ..
    } = inner
    else {
        return None;
    };
    if !(radius > inner_r && *inner_r > EPS) {
        return None;
    }
    let start = [outer.from(), inner.from()];
    let end = [outer.to(), inner.to()];
    let mid_radius = (radius + inner_r) / 2.;
    let angle = |p: Point| (p[1] - center[1]).atan2(p[0] - center[0]);
    let mid_start = circle_on_face(start, *center, mid_radius);
    let mid_end = circle_on_face(end, *center, mid_radius);
    let mid_sweep = (angle(mid_end) - angle(mid_start)).rem_euclid(TAU);
    let tangent = |a: f64| [-a.sin(), a.cos()];
    Some(CurvedTimberMember {
        role: TimberRole::Arc,
        start_tenon: false,
        end_tenon: false,
        profile_side: TimberProfileSide::Top,
        masonry_notches: Vec::new(),
        index,
        start_cut: start,
        end_cut: end,
        start_corner: corners[0],
        end_corner: corners[1],
        outer_edge: outer.clone(),
        outline: Contour {
            segments: vec![
                outer.clone(),
                line(end[0], end[1]),
                inner.section(1., 0.),
                line(start[1], start[0]),
            ],
        },
        net_length_mm: outer.length(),
        start_angle_deg: cut_angle(tangent(*start_angle), face_axis(start)),
        end_angle_deg: cut_angle(tangent(start_angle + sweep_angle), face_axis(end)),
        arc: Some(TimberArc {
            outer_radius_mm: *radius,
            outer_length_mm: outer.length(),
            centerline_radius_mm: mid_radius,
            centerline_length_mm: mid_radius * mid_sweep,
            sweep_deg: sweep_angle.to_degrees(),
        }),
    })
}

struct Foot {
    inner: Point,
    joint: JointFace,
    jamb_end: Point,
    sill_end: Point,
    jamb_cut: Point,
    sill_cut: Point,
}

fn at_y(from: Point, to: Point, y: f64) -> Point {
    let t = (y - from[1]) / (to[1] - from[1]);
    [from[0] + t * (to[0] - from[0]), y]
}

fn foot(outer: Point, inner: Point, spring: Point, corner: CornerModel) -> Foot {
    let jamb_axis = sub(outer, spring);
    let sill_axis = [1., 0.];
    let (jamb_end, sill_end, joint_outer) = if corner.verstek {
        (outer, outer, outer)
    } else if corner.through == ThroughMember::Dorpel {
        let shoulder = at_y(spring, outer, inner[1]);
        (shoulder, outer, shoulder)
    } else {
        let y = outer[1] - inner[1];
        let shoulder = [inner[0] + y * jamb_axis[0] / jamb_axis[1], outer[1]];
        (outer, shoulder, shoulder)
    };
    let joint = [joint_outer, inner];
    Foot {
        inner,
        joint,
        jamb_end,
        sill_end,
        jamb_cut: if corner.verstek {
            face_axis(joint)
        } else {
            sill_axis
        },
        sill_cut: if corner.verstek {
            face_axis(joint)
        } else {
            jamb_axis
        },
    }
}

fn straight(
    role: TimberRole,
    corners: [Option<usize>; 2],
    cuts: [JointFace; 2],
    edge: [Point; 2],
    end_axes: [Point; 2],
    points: &[Point],
) -> Option<CurvedTimberMember> {
    let direction = sub(edge[1], edge[0]);
    let length = norm(direction);
    if length < EPS {
        return None;
    }
    let axis = [direction[0] / length, direction[1] / length];
    let projection = |p: &Point| dot(sub(*p, edge[0]), axis);
    let min = points.iter().map(projection).fold(f64::INFINITY, f64::min);
    let max = points
        .iter()
        .map(projection)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut segments = vec![];
    for (&a, &b) in points.iter().zip(points.iter().cycle().skip(1)) {
        if norm(sub(a, b)) > EPS {
            segments.push(line(a, b));
        }
    }
    if matches!(role, TimberRole::Left | TimberRole::Bottom) {
        segments = segments.iter().rev().map(|s| s.section(1., 0.)).collect();
    }
    let outline = Contour { segments };
    if outline.area() < EPS {
        return None;
    }
    let profile_side = match role {
        TimberRole::Top => TimberProfileSide::Top,
        TimberRole::Left => TimberProfileSide::Left,
        TimberRole::Right => TimberProfileSide::Right,
        _ => TimberProfileSide::Bottom,
    };
    Some(CurvedTimberMember {
        role,
        start_tenon: false,
        end_tenon: false,
        profile_side,
        masonry_notches: Vec::new(),
        index: 0,
        start_cut: cuts[0],
        end_cut: cuts[1],
        start_corner: corners[0],
        end_corner: corners[1],
        outer_edge: line(edge[0], edge[1]),
        outline,
        net_length_mm: max - min,
        start_angle_deg: cut_angle(axis, end_axes[0]),
        end_angle_deg: cut_angle(axis, end_axes[1]),
        arc: None,
    })
}

fn whole_members(frame: &Frame, ring: &CurvedFrame) -> Vec<CurvedTimberMember> {
    if frame.shape.shape_type == ShapeType::Round {
        // Concentric circle outlines are bottom half (right→left), then top half.
        return [1, 0]
            .into_iter()
            .enumerate()
            .filter_map(|(i, n)| {
                let mut member = arc_member(
                    ring.outer.segments.get(n)?,
                    ring.day.segments.get(n)?,
                    i,
                    // The radial cuts share the two seam settings. The upper
                    // arc runs left to right, the lower one right to left.
                    [Some(i), Some(1 - i)],
                )?;
                if i == 1 {
                    member.profile_side = TimberProfileSide::Bottom;
                }
                Some(member)
            })
            .collect();
    }
    let Some(outer) = ring
        .outer
        .segments
        .iter()
        .find(|s| matches!(s, Segment::Arc { .. }))
    else {
        return vec![];
    };
    let Some(inner) = ring
        .day
        .segments
        .iter()
        .find(|s| matches!(s, Segment::Arc { .. }))
    else {
        return vec![];
    };
    // A very shallow frame can have a clipped shoulder meeting the inside sill.
    let Some(Segment::Line { from: ir, to: il }) = ring.day.segments.iter().find(
        |s| matches!(s,Segment::Line{from,to} if (from[1]-to[1]).abs()<EPS && from[0]>to[0]+EPS),
    ) else {
        return vec![];
    };
    let Some(head) = arc_member(outer, inner, 0, [Some(0), Some(1)]) else {
        return vec![];
    };
    let upper = [head.start_cut, head.end_cut];
    let corners = frame_corner_model(
        &frame.material,
        &frame.corner_joints,
        frame.joints_configured,
    );
    let left = foot([0., frame.outer_height], *il, upper[0][0], corners[2]);
    let right = foot(
        [frame.outer_width, frame.outer_height],
        *ir,
        upper[1][0],
        corners[3],
    );
    let mut pieces = vec![head];
    for (i, f) in [&left, &right].into_iter().enumerate() {
        let top = upper[i];
        pieces.extend(straight(
            if i == 0 {
                TimberRole::Left
            } else {
                TimberRole::Right
            },
            [Some(i), Some(i + 2)],
            [top, f.joint],
            [top[0], f.jamb_end],
            [face_axis(top), f.jamb_cut],
            &[top[0], f.jamb_end, f.joint[0], f.inner, top[1]],
        ));
    }
    pieces.extend(straight(
        TimberRole::Bottom,
        [Some(2), Some(3)],
        [left.joint, right.joint],
        [left.sill_end, right.sill_end],
        [left.sill_cut, right.sill_cut],
        &[
            left.joint[0],
            left.sill_end,
            right.sill_end,
            right.joint[0],
            right.inner,
            left.inner,
        ],
    ));
    pieces
}

#[derive(Clone, Copy)]
struct Crosscut {
    outer_t: f64,
    inner_t: f64,
    face: JointFace,
    corner: Option<usize>,
}

fn edge_direction(edge: &Segment, t: f64) -> Point {
    match edge {
        Segment::Line { from, to } => sub(*to, *from),
        Segment::Arc {
            start_angle,
            sweep_angle,
            ..
        } => {
            let a = start_angle + t * sweep_angle;
            [
                -a.sin() * sweep_angle.signum(),
                a.cos() * sweep_angle.signum(),
            ]
        }
    }
}

fn inside_rect(rect: &Rect2D, p: Point) -> bool {
    p[0] >= rect.x - EPS
        && p[0] <= rect.x + rect.width + EPS
        && p[1] >= rect.y - EPS
        && p[1] <= rect.y + rect.height + EPS
}

fn timber_interval(
    member: &CurvedTimberMember,
    inner: &Segment,
    a: Crosscut,
    b: Crosscut,
) -> Option<CurvedTimberMember> {
    let outer = member.outer_edge.section(a.outer_t, b.outer_t);
    let corners = [a.corner, b.corner];
    let mut piece = if member.arc.is_some() {
        arc_member(&outer, &inner.section(a.inner_t, b.inner_t), 0, corners)?
    } else {
        let mut straight = straight(
            member.role,
            corners,
            [a.face, b.face],
            [outer.from(), outer.to()],
            [face_axis(a.face), face_axis(b.face)],
            &[
                a.face[0],
                outer.from(),
                outer.to(),
                b.face[0],
                b.face[1],
                a.face[1],
            ],
        )?;
        // For an uncut original end of a through-member the free end face
        // measures the saw angle, while the shared joint lies on the side.
        if a.outer_t == 0. {
            straight.start_angle_deg = member.start_angle_deg;
        }
        if b.outer_t == 1. {
            straight.end_angle_deg = member.end_angle_deg;
        }
        straight
    };
    piece.profile_side = member.profile_side;
    Some(piece)
}

/// A masonry edge produces a new saw face only where it passes through both
/// long edges of the stock. A local notch still requires a continuous blank.
fn interrupt_at_wall(member: CurvedTimberMember, wall: &Rect2D) -> Vec<CurvedTimberMember> {
    let bounds = member.outline.bounds();
    let min = [bounds.x, bounds.y];
    let max = [bounds.x + bounds.width, bounds.y + bounds.height];
    if inside_rect(wall, min) && inside_rect(wall, max) {
        return vec![];
    }
    if min[0] >= wall.x + wall.width - EPS
        || max[0] <= wall.x + EPS
        || min[1] >= wall.y + wall.height - EPS
        || max[1] <= wall.y + EPS
    {
        return vec![member];
    }
    let inner = if member.arc.is_some() {
        member.outline.segments[2].section(1., 0.)
    } else {
        line(member.start_cut[1], member.end_cut[1])
    };
    let outer = &member.outer_edge;
    let mut splits = vec![
        Crosscut {
            outer_t: 0.,
            inner_t: 0.,
            face: member.start_cut,
            corner: member.start_corner,
        },
        Crosscut {
            outer_t: 1.,
            inner_t: 1.,
            face: member.end_cut,
            corner: member.end_corner,
        },
    ];
    for (axis, plane) in [
        (0, wall.x),
        (0, wall.x + wall.width),
        (1, wall.y),
        (1, wall.y + wall.height),
    ] {
        let inner_hits = inner.crossings(axis, plane);
        for t in outer.crossings(axis, plane) {
            let p = outer.point(t);
            if !inside_rect(wall, p) {
                continue;
            }
            if let Some(&u) = inner_hits.iter().find(|&&u| {
                inside_rect(wall, inner.point(u))
                    && edge_direction(outer, t)[axis] * edge_direction(&inner, u)[axis] > 0.
            }) {
                splits.push(Crosscut {
                    outer_t: t,
                    inner_t: u,
                    face: [p, inner.point(u)],
                    corner: None,
                });
            }
        }
    }
    if splits.len() == 2 {
        return vec![member];
    }
    splits.sort_by(|a, b| a.outer_t.total_cmp(&b.outer_t));
    splits.dedup_by(|a, b| (a.outer_t - b.outer_t).abs() < 1e-10);
    let mut kept = vec![];
    let mut run = None;
    for pair in splits.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if b.outer_t - a.outer_t < EPS * 1e-3 || b.inner_t <= a.inner_t {
            continue;
        }
        let Some(part) = timber_interval(&member, &inner, a, b) else {
            continue;
        };
        let bb = part.outline.bounds();
        let gone = inside_rect(wall, [bb.x, bb.y])
            && inside_rect(wall, [bb.x + bb.width, bb.y + bb.height]);
        if gone {
            if let Some(start) = run.take() {
                kept.extend(timber_interval(&member, &inner, start, a));
            }
        } else {
            run.get_or_insert(a);
        }
    }
    if let Some(start) = run {
        kept.extend(timber_interval(
            &member,
            &inner,
            start,
            *splits.last().unwrap(),
        ));
    }
    kept
}

/// Between successive vertex heights, horizontal tangencies and crossings of
/// the two vertical wall edges, overlap cannot start or stop. A horizontal
/// slice in each such interval distinguishes an actual notch from a rectangle
/// that merely touches the wood or fits entirely inside its bounding box.
fn notch_in_blank(blank: &Contour, wall: &Rect2D) -> Option<Rect2D> {
    let bounds = blank.bounds();
    let x = bounds.x.max(wall.x);
    let y = bounds.y.max(wall.y);
    let right = (bounds.x + bounds.width).min(wall.x + wall.width);
    let bottom = (bounds.y + bounds.height).min(wall.y + wall.height);
    if right - x <= EPS || bottom - y <= EPS {
        return None;
    }
    let mut heights = vec![y, bottom];
    for edge in &blank.segments {
        heights.push(edge.from()[1]);
        if let Segment::Arc { center, radius, .. } = edge {
            heights.extend([center[1] - radius, center[1] + radius]);
        }
        for boundary in [x, right] {
            heights.extend(
                edge.crossings(0, boundary)
                    .into_iter()
                    .map(|t| edge.point(t)[1]),
            );
        }
    }
    heights.retain(|h| *h >= y && *h <= bottom);
    heights.sort_by(f64::total_cmp);
    for levels in heights.windows(2) {
        if levels[1] - levels[0] <= EPS {
            continue;
        }
        let height = (levels[0] + levels[1]) / 2.;
        let mut hits: Vec<_> = blank
            .segments
            .iter()
            .flat_map(|edge| {
                edge.crossings(1, height)
                    .into_iter()
                    .map(|t| edge.point(t)[0])
            })
            .collect();
        hits.sort_by(f64::total_cmp);
        if hits
            .chunks_exact(2)
            .any(|interval| right.min(interval[1]) - x.max(interval[0]) > EPS)
        {
            return Some(Rect2D {
                x,
                y,
                width: right - x,
                height: bottom - y,
            });
        }
    }
    None
}

/// A wall can remove either mating end or the last strip of common shoulder.
/// An exposed radial cut still belongs on the cut list as an end of its blank.
fn clear_unpaired_ring_ends(pieces: &mut [CurvedTimberMember]) {
    for corner in 0..2 {
        let mut ends = pieces.iter().filter_map(|p| {
            if p.start_corner == Some(corner) {
                Some((p, p.start_cut))
            } else if p.end_corner == Some(corner) {
                Some((p, p.end_cut))
            } else {
                None
            }
        });
        let paired = if let (Some((first, a)), Some((second, b))) = (ends.next(), ends.next()) {
            let y = a[0][1];
            let occupied: Vec<_> = first.masonry_notches.iter().chain(&second.masonry_notches)
                .filter(|r| y >= r.y - EPS && y <= r.y + r.height + EPS)
                .map(|r| (r.x, r.x + r.width))
                .collect();
            a.iter().zip(b).all(|(p, q)| norm(sub(*p, q)) < EPS)
                && !crate::layout::subtract_intervals(
                    a[0][0].min(a[1][0]), a[0][0].max(a[1][0]), &occupied, EPS,
                ).is_empty()
        } else {
            false
        };
        if !paired {
            for p in pieces.iter_mut() {
                if p.start_corner == Some(corner) { p.start_corner = None; }
                if p.end_corner == Some(corner) { p.end_corner = None; }
            }
        }
    }
}

pub(crate) fn frame_members(
    frame: &Frame,
    ring: &CurvedFrame,
    layout: Option<&LayoutGeometry>,
) -> Vec<CurvedTimberMember> {
    let mut members = whole_members(frame, ring);
    let Some(layout) = layout else { return members };
    let fw = frame.frame_width;
    let mut walls = Vec::new();
    for leaf in layout.leaves.iter().filter(|l| l.vulling.is_buiten()) {
        let r = leaf.rect;
        // Same extension to the frame exterior as the masonry volume in the elevation.
        let left = if (r.x - fw).abs() < 1. { r.x - fw } else { r.x };
        let top = if (r.y - fw).abs() < 1. { r.y - fw } else { r.y };
        let mut right = r.x + r.width;
        let mut bottom = r.y + r.height;
        if (right - (frame.outer_width - fw)).abs() < 1. {
            right += fw;
        }
        if (bottom - (frame.outer_height - fw)).abs() < 1. {
            bottom += fw;
        }
        let wall = Rect2D {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        };
        members = members
            .into_iter()
            .flat_map(|m| interrupt_at_wall(m, &wall))
            .collect();
        walls.push(wall);
    }
    for member in &mut members {
        member.masonry_notches = walls
            .iter()
            .filter_map(|wall| notch_in_blank(&member.outline, wall))
            .collect();
    }
    if frame.shape.shape_type == ShapeType::Round {
        clear_unpaired_ring_ends(&mut members);
    }
    for role in [
        TimberRole::Arc,
        TimberRole::Bottom,
        TimberRole::Left,
        TimberRole::Right,
    ] {
        for (i, m) in members.iter_mut().filter(|m| m.role == role).enumerate() {
            m.index = i;
        }
    }
    members
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::joint::{Joint, JointType};
    use crate::kozijn::Kozijn;

    fn frame(shape: ShapeType, rise: f64) -> Kozijn {
        let mut k = Kozijn::new("Boog", "B01", 1200., 1500.);
        k.frame.shape.shape_type = shape;
        k.frame.shape.arch_height = Some(rise);
        k
    }
    fn close(p: f64, q: f64) {
        assert!((p - q).abs() < 1e-6, "{p} vs {q}");
    }
    fn assert_partition(k: &Kozijn) {
        let g = compute_2d_geometry(k);
        let ring = g.curved_frame.unwrap();
        close(
            g.curved_timber.iter().map(|p| p.outline.area()).sum(),
            ring.outer.area() - ring.day.area(),
        );
        for p in g.curved_timber {
            for (a, b) in p
                .outline
                .segments
                .iter()
                .zip(p.outline.segments.iter().cycle().skip(1))
            {
                close(norm(sub(a.to(), b.from())), 0.);
            }
        }
    }

    #[test]
    fn segmental_head_has_shared_faces_and_measures_each_tangent() {
        let k = frame(ShapeType::Arched, 300.);
        let g = compute_2d_geometry(&k);
        assert_partition(&k);
        let pieces = &g.curved_timber;
        let [arc, left, right, sill] = pieces.as_slice() else {
            panic!("expected four pieces")
        };
        let measure = arc.arc.as_ref().unwrap();
        let sweep = 2. * (600f64 / 750.).asin();
        close(measure.outer_radius_mm, 750.);
        close(measure.outer_length_mm, 750. * sweep);
        close(measure.sweep_deg, sweep.to_degrees());
        let day_y = 750. - (683f64.powi(2) - 533f64.powi(2)).sqrt();
        let slope = (day_y - 300.) / 67.;
        close(left.start_angle_deg, 90. - slope.atan().to_degrees());
        close(
            arc.start_angle_deg,
            (4f64 / 3.).atan().to_degrees() + slope.atan().to_degrees(),
        );
        close(arc.end_angle_deg, arc.start_angle_deg);
        close(right.start_angle_deg, left.start_angle_deg);
        assert_eq!(arc.start_cut, left.start_cut);
        assert_eq!(arc.end_cut, right.start_cut);
        close(left.net_length_mm, 1500. - 67. - 300.);
        close(sill.net_length_mm, 1200.);
        close(sill.start_angle_deg, 90.);
        close(measure.centerline_radius_mm, 716.5);
        // Intersect the shared left cut y=300+slope*x with the centreline
        // circle, then use both symmetric endpoints for its true sweep.
        let b = -1200. - 900. * slope;
        let c = 600f64.powi(2) + 450f64.powi(2) - 716.5f64.powi(2);
        let x = 2. * c / (-b + (b * b - 4. * (1. + slope * slope) * c).sqrt());
        close(
            measure.centerline_length_mm,
            2. * 716.5 * ((600. - x) / 716.5).asin(),
        );
        assert!((measure.centerline_length_mm - 716.5 * sweep).abs() > 1.);
    }

    #[test]
    fn tangent_arch_and_round_half_rings_have_square_faces() {
        let semi = frame(ShapeType::Arched, 600.);
        assert_partition(&semi);
        let s = compute_2d_geometry(&semi).curved_timber;
        close(s[0].net_length_mm, 600. * std::f64::consts::PI);
        close(s[0].start_angle_deg, 90.);
        close(s[1].start_angle_deg, 90.);
        let circle = frame(ShapeType::Round, 0.);
        let parts = compute_2d_geometry(&circle).curved_timber;
        assert_partition(&circle);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].profile_side, TimberProfileSide::Top);
        assert_eq!(parts[1].profile_side, TimberProfileSide::Bottom);
        assert_eq!(parts[0].end_cut, parts[1].start_cut);
        for p in parts {
            close(p.net_length_mm, 600. * std::f64::consts::PI);
            close(p.start_angle_deg, 90.);
            close(p.end_angle_deg, 90.);
        }
    }

    #[test]
    fn sloping_stiles_and_mixed_feet_share_full_outline() {
        let mut k = frame(ShapeType::ArchedTrapezoid, 180.);
        k.frame.shape.left_angle = Some(86.);
        k.frame.shape.right_angle = Some(81.);
        for jt in [JointType::Stomp, JointType::Verstek] {
            for through in [ThroughMember::Dorpel, ThroughMember::Stijl] {
                k.frame.corner_joints[2] = Joint {
                    joint_type: jt,
                    through_member: through,
                    ..Default::default()
                };
                assert_partition(&k);
                let p = compute_2d_geometry(&k).curved_timber;
                assert_ne!(p[1].start_angle_deg, p[2].start_angle_deg);
                assert_eq!(p[1].end_cut, p[3].start_cut);
                if jt == JointType::Stomp {
                    close(p[1].end_angle_deg, 86.);
                    close(p[3].start_angle_deg, 86.);
                } else {
                    close(p[1].end_angle_deg, 43.);
                    close(p[3].start_angle_deg, 43.);
                }
            }
        }
    }

    #[test]
    fn wall_shortens_arch_sill_and_stile_and_interrupts_round_blank() {
        let mut k = frame(ShapeType::Arched, 300.);
        k.layout = Some(crate::layout::melkmeisje1());
        let g = compute_2d_geometry(&k);
        let p = &g.curved_timber;
        let left = p.iter().find(|p| p.role == TimberRole::Left).unwrap();
        let right = p.iter().find(|p| p.role == TimberRole::Right).unwrap();
        let sill = p.iter().find(|p| p.role == TimberRole::Bottom).unwrap();
        assert!(right.net_length_mm < left.net_length_mm && right.end_corner.is_none());
        assert!(sill.net_length_mm < 1000. && sill.end_corner.is_none());
        k.frame.shape.shape_type = ShapeType::Round;
        let p = compute_2d_geometry(&k).curved_timber;
        assert!(
            p.iter()
                .filter(|p| p.profile_side == TimberProfileSide::Bottom)
                .count()
                >= 2
        );
        assert!(p.iter().map(|p| p.net_length_mm).sum::<f64>() < TAU * 600.);
        for (i, part) in p.iter().enumerate() {
            assert_eq!(part.index, i);
        }
    }

    #[test]
    fn partial_masonry_notch_keeps_blank_length_and_excludes_touching_faces() {
        let k = frame(ShapeType::Round, 0.);
        let top = compute_2d_geometry(&k).curved_timber.remove(0);
        let notch = Rect2D {
            x: 1090.,
            y: 440.,
            width: 20.,
            height: 20.,
        };
        let pieces = interrupt_at_wall(top.clone(), &notch);
        assert_eq!(pieces.len(), 1);
        close(top.net_length_mm, pieces[0].net_length_mm);
        let cut = notch_in_blank(&pieces[0].outline, &notch).unwrap();
        close(cut.x, 1090.);
        close(cut.y, 440.);
        close(cut.width, 20.);
        close(cut.height, 20.);
        let inside_opening = Rect2D {
            x: 550.,
            y: 440.,
            width: 20.,
            height: 20.,
        };
        assert!(notch_in_blank(&top.outline, &inside_opening).is_none());
        let meets_rim = Rect2D {
            x: 0.,
            y: 750.,
            width: 1200.,
            height: 2.,
        };
        assert!(notch_in_blank(&top.outline, &meets_rim).is_none());
    }

    #[test]
    fn empty_and_very_low_arch_emit_no_phantom_stiles() {
        let mut k = frame(ShapeType::Arched, 100.);
        k.frame.outer_height = 167.;
        assert_partition(&k);
        assert!(compute_2d_geometry(&k)
            .curved_timber
            .iter()
            .all(|p| p.net_length_mm > EPS));
        k.frame.frame_width = 300.;
        assert!(compute_2d_geometry(&k).curved_timber.is_empty());
    }
}
