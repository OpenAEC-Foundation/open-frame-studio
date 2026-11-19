//! Blind joints at surviving arch spring lines and circular frame seams. Elevation
//! points are in frame coordinates; depths are measured from the inside face.
use super::*;
use crate::curved_timber::{dot, line, norm, sub, JointFace};
use crate::joint::{Joint, JointType, ThroughMember};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameMemberRef {
    pub role: TimberRole,
    pub index: usize,
}

impl From<&CurvedTimberMember> for FrameMemberRef {
    fn from(m: &CurvedTimberMember) -> Self {
        Self {
            role: m.role,
            index: m.index,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchJointKind {
    #[default]
    Springing,
    RingSeam,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchJoint {
    #[serde(default)]
    pub kind: ArchJointKind,
    pub corner: usize,
    pub source: FrameMemberRef,
    pub receiver: FrameMemberRef,
    /// The entire shared cut, from the outside edge to the clear opening.
    pub shoulder_face: JointFace,
    /// Acute angles between the shared cut and each member's own end tangent.
    pub source_angle_deg: f64,
    pub receiver_angle_deg: f64,
    pub requested_length_mm: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenon: Option<ArchTenon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<TenonIssue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchTenon {
    /// Unit insertion axis in the elevation, from tenon into mortise.
    pub direction: Point,
    /// Corresponding sides of the tongue at the shoulder, flat tip and blind bottom.
    pub shoulder: JointFace,
    pub shoulder_center: Point,
    pub tip: JointFace,
    pub bottom: JointFace,
    pub z_from_inside_mm: Vec<[f64; 2]>,
    /// Along the insertion axis from the centre of the tongue's shoulder.
    pub tenon_length_mm: f64,
    pub mortise_depth_mm: f64,
    pub bottom_clearance_mm: f64,
    /// Additional straight / developed outside blank beyond the net long point.
    pub blank_extension_mm: f64,
}

#[derive(Clone, Copy)]
struct MemberEnd<'a> {
    piece: &'a CurvedTimberMember,
    at_end: bool,
}

impl<'a> MemberEnd<'a> {
    fn at_corner(member: &'a CurvedTimberMember, corner: usize) -> Option<Self> {
        let at_end = member.end_corner == Some(corner);
        (at_end || member.start_corner == Some(corner)).then_some(Self {
            piece: member,
            at_end,
        })
    }
    fn cut(self) -> JointFace {
        if self.at_end {
            self.piece.end_cut
        } else {
            self.piece.start_cut
        }
    }
    fn angle(self) -> f64 {
        if self.at_end {
            self.piece.end_angle_deg
        } else {
            self.piece.start_angle_deg
        }
    }
    fn direction(self) -> Point {
        let tangent = match &self.piece.outer_edge {
            Segment::Line { from, to } => sub(*to, *from),
            Segment::Arc {
                center,
                sweep_angle,
                ..
            } => {
                // Use the actual end radius. On either side of a round frame
                // it is exactly horizontal, giving exactly parallel axes.
                let radius = sub(self.cut()[0], *center);
                let turn = sweep_angle.signum();
                [-radius[1] * turn, radius[0] * turn]
            }
        };
        let length = norm(tangent);
        let sign = if self.at_end { 1. } else { -1. };
        tangent.map(|v| sign * v / length)
    }
}

/// A proper rotation; the positive local Y axis points along insertion.
struct Axes {
    origin: Point,
    along: Point,
    across: Point,
}

impl Axes {
    fn new(origin: Point, along: Point) -> Self {
        Self {
            origin,
            along,
            across: [along[1], -along[0]],
        }
    }
    fn local(&self, point: Point) -> Point {
        let d = sub(point, self.origin);
        [dot(d, self.across), dot(d, self.along)]
    }
    fn world(&self, p: Point) -> Point {
        [
            self.origin[0] + self.across[0] * p[0] + self.along[0] * p[1],
            self.origin[1] + self.across[1] * p[0] + self.along[1] * p[1],
        ]
    }
    fn contour(&self, path: &Contour) -> Contour {
        let segments = path
            .segments
            .iter()
            .map(|s| {
                let from = self.local(s.from());
                let to = self.local(s.to());
                match s {
                    Segment::Line { .. } => line(from, to),
                    Segment::Arc {
                        center,
                        radius,
                        sweep_angle,
                        ..
                    } => {
                        let center = self.local(*center);
                        let v = sub(from, center);
                        Segment::Arc {
                            from,
                            to,
                            center,
                            radius: *radius,
                            start_angle: v[1].atan2(v[0]),
                            sweep_angle: *sweep_angle,
                        }
                    }
                }
            })
            .collect();
        Contour { segments }
    }
}

fn quad(a: JointFace, b: JointFace) -> Contour {
    Contour {
        segments: vec![
            line(a[0], a[1]),
            line(a[1], b[1]),
            line(b[1], b[0]),
            line(b[0], a[0]),
        ],
    }
}

fn cut_by_wall(path: &Contour, walls: &[Rect2D]) -> bool {
    walls
        .iter()
        .any(|wall| path.clone().clipped(wall).area() > TOL)
}

/// A straight tongue sawn out of a bent blank must remain within its radial
/// stock. The least radius can fall halfway along a side of the tongue.
fn blank_extension(source: MemberEnd, axes: &Axes, tongue: &Contour) -> Option<f64> {
    let Segment::Arc {
        center,
        radius,
        sweep_angle,
        ..
    } = source.piece.outer_edge
    else {
        let long = source
            .piece
            .outline
            .segments
            .iter()
            .map(|s| axes.local(s.from())[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let top = tongue
            .segments
            .iter()
            .map(|s| axes.local(s.from())[1])
            .fold(long, f64::max);
        return Some(top - long);
    };
    let inner = 2. * source.piece.arc.as_ref()?.centerline_radius_mm - radius;
    let reference = sub(axes.origin, center);
    let sign = sweep_angle.signum() * if source.at_end { 1. } else { -1. };
    let mut extra: f64 = 0.;
    for e in &tongue.segments {
        let a = sub(e.from(), center);
        let side = sub(e.to(), e.from());
        let t = (-dot(a, side) / dot(side, side)).clamp(0., 1.);
        let nearest = [a[0] + t * side[0], a[1] + t * side[1]];
        if norm(a) > radius + TOL || norm(nearest) < inner - TOL {
            return None;
        }
        let angle = (reference[0] * a[1] - reference[1] * a[0]).atan2(dot(reference, a));
        extra = extra.max(sign * radius * angle);
    }
    Some(extra)
}

fn fit(
    source: MemberEnd,
    receiver: MemberEnd,
    source_stock: &Stock,
    receiver_stock: &Stock,
    length: f64,
    pockets: &[Pocket],
    occupied: &mut Vec<Occupied>,
) -> Option<ArchTenon> {
    if !length.is_finite() || length <= EPS {
        return None;
    }
    let face = source.cut();
    let axes = Axes::new(face[0], source.direction());
    let local_face = face.map(|p| axes.local(p));
    let shoulder = [line(local_face[0], local_face[1])];
    let dx = local_face[1][0];
    let reach = TONGUE_WIDTH / 2. + WEB;
    let shift = dx.abs() / 2. - reach;
    if shift < -EPS {
        return None;
    }
    let source_path = axes.contour(&source.piece.outline);
    let receiver_path = axes.contour(&receiver.piece.outline);
    // Start at mid-face; prefer the same flank (outside first) on mirrored
    // ends. Include the extreme admissible centre even for fractional widths.
    let mut offsets = vec![0.];
    for delta in 1..=shift.max(0.).min(128.).ceil() as usize {
        let offset = (delta as f64).min(shift);
        offsets.extend([-offset, offset]);
    }
    if shift > 128. {
        offsets.extend([-shift, shift]);
    }
    for offset in offsets {
        let center_u = dx / 2. + dx.signum() * offset;
        let axial = |u: f64| local_face[1][1] * u / dx;
        let middle = axial(center_u);
        let range = [center_u - reach, center_u + reach];
        let exposed = range.map(axial);
        let low = exposed[0].min(exposed[1]);
        let high = exposed[0].max(exposed[1]);
        let tip_v = middle + length;
        let bottom_v = tip_v + CLEARANCE;
        if tip_v <= high + EPS {
            continue;
        }
        // The cheek/side web must fit throughout, including circular extrema.
        if host_exit(&receiver_path, &[], &shoulder, 1, 1., range)
            .map_or(true, |limit| limit < bottom_v + WEB - TOL)
            || host_exit(&source_path, &[], &shoulder, 1, -1., range)
                .map_or(true, |limit| limit < -low + WEB - TOL)
        {
            continue;
        }
        let face_at =
            |us: [f64; 2], level: &dyn Fn(f64) -> f64| us.map(|u| axes.world([u, level(u)]));
        let outer_shoulder = face_at(range, &axial);
        let anchor = face_at(range, &|u| axial(u) - WEB);
        let extended_bottom = face_at(range, &|_| bottom_v + WEB);
        if cut_by_wall(&quad(anchor, outer_shoulder), &source.piece.masonry_notches)
            || cut_by_wall(
                &quad(outer_shoulder, extended_bottom),
                &receiver.piece.masonry_notches,
            )
        {
            continue;
        }
        let us = [center_u - TONGUE_WIDTH / 2., center_u + TONGUE_WIDTH / 2.];
        let mouth = face_at(us, &axial);
        let tip = face_at(us, &|_| tip_v);
        let bottom = face_at(us, &|_| bottom_v);
        let Some(blank_extension_mm) = blank_extension(source, &axes, &quad(mouth, tip)) else {
            continue;
        };
        let volume = quad(face_at(us, &|u| axial(u) - WEB), bottom).bounds();
        let Some(bands) = z_bands(source_stock, receiver_stock, &volume, pockets, occupied) else {
            continue;
        };
        occupied.push(Occupied {
            rect: volume.inset(-WEB / 2.),
            bands: bands.clone(),
        });
        return Some(ArchTenon {
            direction: axes.along,
            shoulder: mouth,
            shoulder_center: axes.world([center_u, middle]),
            tip,
            bottom,
            z_from_inside_mm: bands,
            tenon_length_mm: length,
            mortise_depth_mm: length + CLEARANCE,
            bottom_clearance_mm: CLEARANCE,
            blank_extension_mm,
        });
    }
    None
}

pub(super) fn arch_joints(
    pieces: &[CurvedTimberMember],
    hosts: &[Host],
    settings: &[Joint; 4],
    pockets: &[Pocket],
    occupied: &mut Vec<Occupied>,
) -> Vec<ArchJoint> {
    let mut result = Vec::new();
    for (corner, setting) in settings.iter().take(2).enumerate() {
        if setting.joint_type != JointType::PenSlis {
            continue;
        }
        let end = |role, side| {
            pieces
                .iter()
                .filter(|m| m.role == role && m.profile_side == side)
                .find_map(|m| MemberEnd::at_corner(m, corner))
        };
        let Some(head) = end(TimberRole::Arc, TimberProfileSide::Top) else {
            continue;
        };
        let jamb = if corner == 0 {
            (TimberRole::Left, TimberProfileSide::Left)
        } else {
            (TimberRole::Right, TimberProfileSide::Right)
        };
        let (kind, mating_end) =
            if let Some(lower) = end(TimberRole::Arc, TimberProfileSide::Bottom) {
                (ArchJointKind::RingSeam, lower)
            } else if let Some(stile) = end(jamb.0, jamb.1) {
                (ArchJointKind::Springing, stile)
            } else {
                continue;
            };
        if head
            .cut()
            .iter()
            .zip(mating_end.cut())
            .any(|(a, b)| norm(sub(*a, b)) > TOL)
        {
            continue;
        }
        let (source, receiver) = if setting.through_member == ThroughMember::Dorpel {
            (mating_end, head)
        } else {
            (head, mating_end)
        };
        let stock_of = |end: MemberEnd| {
            let key = TenonReceiver::Frame {
                role: end.piece.role,
                index: end.piece.index,
            };
            hosts
                .iter()
                .find(|h| h.key == key)
                .ok_or(TenonIssue::NoReceiver)?
                .stock
                .as_ref()
                .map_err(|e| *e)
        };
        let a = stock_of(source);
        let b = stock_of(receiver);
        if matches!(a, Err(TenonIssue::NoReceiver)) || matches!(b, Err(TenonIssue::NoReceiver)) {
            continue;
        }
        let machining = a.and_then(|s| {
            b.and_then(|r| {
                fit(
                    source,
                    receiver,
                    s,
                    r,
                    setting.pen_length,
                    pockets,
                    occupied,
                )
                .ok_or(TenonIssue::InsufficientStock)
            })
        });
        let (tenon, issue) = match machining {
            Ok(t) => (Some(t), None),
            Err(e) => (None, Some(e)),
        };
        result.push(ArchJoint {
            kind,
            corner,
            source: source.piece.into(),
            receiver: receiver.piece.into(),
            shoulder_face: source.cut(),
            source_angle_deg: source.angle(),
            receiver_angle_deg: receiver.angle(),
            requested_length_mm: setting.pen_length,
            tenon,
            issue,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curved_timber::edge_direction;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::ShapeType;

    fn frame(shape: ShapeType, rise: f64) -> Kozijn {
        let mut k = Kozijn::new("Boog", "K1", 1200., 1450.);
        k.frame.shape.shape_type = shape;
        k.frame.shape.arch_height = Some(rise);
        k
    }
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-5, "{a} ≠ {b}");
    }
    fn inside(path: &Contour, p: Point) {
        let rect = Rect2D {
            x: p[0] - 0.05,
            y: p[1] - 0.05,
            width: 0.1,
            height: 0.1,
        };
        near(path.clone().clipped(&rect).area(), 0.01);
    }

    #[test]
    fn vertical_springs_reference_each_mouth_and_each_real_saw_tangent() {
        let g = compute_2d_geometry(&frame(ShapeType::Arched, 300.));
        assert_eq!(g.arch_joints.len(), 2);
        let head = &g.curved_timber[0];
        for j in &g.arch_joints {
            let jamb = &g.curved_timber[1 + j.corner];
            let t = j.tenon.as_ref().unwrap();
            assert_eq!(j.source, FrameMemberRef::from(jamb));
            assert_eq!(j.receiver, FrameMemberRef::from(head));
            assert_eq!(j.shoulder_face, jamb.start_cut);
            near(j.source_angle_deg, jamb.start_angle_deg);
            near(
                j.receiver_angle_deg,
                if j.corner == 0 {
                    head.start_angle_deg
                } else {
                    head.end_angle_deg
                },
            );
            assert!((j.source_angle_deg - j.receiver_angle_deg).abs() > 0.1);
            near(t.direction[0], 0.);
            near(t.direction[1], -1.);
            near((t.shoulder[0][0] - t.shoulder[1][0]).abs(), 25.);
            near(t.shoulder_center[1] - t.tip[0][1], 20.);
            near(t.shoulder_center[1] - t.bottom[1][1], 22.);
            near(
                t.blank_extension_mm,
                (j.shoulder_face[0][1] - t.tip[0][1]).max(0.),
            );
            near(t.tip[0][1], t.tip[1][1]);
            for p in t.bottom {
                inside(&head.outline, p);
            }
        }
        let left = g.arch_joints[0].tenon.as_ref().unwrap();
        let right = g.arch_joints[1].tenon.as_ref().unwrap();
        near(left.shoulder_center[0] + right.shoulder_center[0], 1200.);
        near(left.shoulder_center[1], right.shoulder_center[1]);
        assert_eq!(left.z_from_inside_mm, right.z_from_inside_mm);
    }

    #[test]
    fn sloped_jamb_and_reversed_arch_follow_their_own_insertion_axes() {
        let mut k = frame(ShapeType::ArchedTrapezoid, 230.);
        k.frame.shape.left_angle = Some(82.);
        k.frame.shape.right_angle = Some(83.);
        k.frame.joints_configured = true;
        k.frame.corner_joints[0].through_member = ThroughMember::Dorpel;
        k.frame.corner_joints[0].pen_length = 28.;
        k.frame.corner_joints[1].through_member = ThroughMember::Stijl;
        k.frame.corner_joints[1].pen_length = 19.;
        let g = compute_2d_geometry(&k);
        let [left, right] = g.arch_joints.as_slice() else {
            panic!("two spring lines")
        };
        let a = left.tenon.as_ref().unwrap();
        let b = right.tenon.as_ref().unwrap();
        near(a.direction[0], 82_f64.to_radians().cos());
        near(a.direction[1], -82_f64.to_radians().sin());
        let head = &g.curved_timber[0];
        let tangent = edge_direction(&head.outer_edge, 1.);
        near(dot(tangent, b.direction), 1.);
        assert_eq!(right.source.role, TimberRole::Arc);
        assert_eq!(right.receiver.role, TimberRole::Right);
        let Segment::Arc { center, radius, .. } = head.outer_edge else {
            unreachable!()
        };
        let v = sub(head.end_cut[0], center);
        let projected = |p| {
            let u = sub(p, center);
            radius * (v[0] * u[1] - v[1] * u[0]).atan2(dot(v, u))
        };
        near(
            b.blank_extension_mm,
            b.tip.iter().map(|p| projected(*p)).fold(0., f64::max),
        );
        for (t, depth, receiver) in [(a, 28., head), (b, 19., &g.curved_timber[2])] {
            near(dot(sub(t.tip[0], t.shoulder_center), t.direction), depth);
            near(
                dot(sub(t.bottom[1], t.shoulder_center), t.direction),
                depth + 2.,
            );
            near(dot(sub(t.tip[1], t.tip[0]), t.direction), 0.);
            near(norm(sub(t.tip[1], t.tip[0])), 25.);
            for p in t.bottom {
                inside(&receiver.outline, p);
            }
        }
    }

    #[test]
    fn only_intact_matching_wooden_pen_slis_corners_are_machined() {
        let mut k = frame(ShapeType::Round, 0.);
        assert!(compute_2d_geometry(&k)
            .arch_joints
            .iter()
            .all(|j| j.kind == ArchJointKind::RingSeam));
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.frame.left_material = Some(Material::Pvc);
        let joints = compute_2d_geometry(&k).arch_joints;
        assert_eq!(joints.len(), 1);
        assert_eq!(joints[0].corner, 1);
        k.frame.left_material = None;
        for kind in [JointType::Verstek, JointType::Contramal, JointType::Stomp] {
            k.frame.corner_joints[0].joint_type = kind;
            assert!(compute_2d_geometry(&k)
                .arch_joints
                .iter()
                .all(|j| j.corner == 1));
        }
        k.frame.corner_joints[0].joint_type = JointType::PenSlis;
        k.frame.left_profile = Some(ProfileRef {
            id: "unknown".into(),
            name: "?".into(),
        });
        assert_eq!(
            compute_2d_geometry(&k).arch_joints[0].issue,
            Some(TenonIssue::UnknownProfile)
        );
        k.frame.left_profile = None;
        k.frame.corner_joints[0].through_member = ThroughMember::Dorpel;
        k.frame.corner_joints[0].pen_length = 500.;
        let j = compute_2d_geometry(&k).arch_joints.remove(0);
        assert!(j.tenon.is_none());
        assert_eq!(j.issue, Some(TenonIssue::InsufficientStock));
    }

    #[test]
    fn reserved_divider_holes_and_wall_cuts_leave_no_drilling_in_empty_stock() {
        let k = frame(ShapeType::Arched, 600.);
        let g = compute_2d_geometry(&k);
        let pockets = pockets(&g.cell_rects, k.frame.frame_width * 0.48);
        let settings = g.corner_settings;
        let mut pieces = g.curved_timber;
        let blocked = Rect2D {
            x: -1.,
            y: 555.,
            width: 70.,
            height: 60.,
        };
        let occupied = Occupied {
            rect: blocked.clone(),
            bands: vec![[0., 114.]],
        };
        let joints = arch_joints(
            &pieces,
            &hosts(&k, &pieces, &[]),
            &settings,
            &pockets,
            &mut vec![occupied],
        );
        assert_eq!(joints[0].issue, Some(TenonIssue::InsufficientStock));
        assert!(joints[1].tenon.is_some());
        pieces[0].masonry_notches.push(blocked);
        let joints = arch_joints(
            &pieces,
            &hosts(&k, &pieces, &[]),
            &settings,
            &pockets,
            &mut Vec::new(),
        );
        assert_eq!(joints[0].issue, Some(TenonIssue::InsufficientStock));
        pieces.retain(|p| p.role != TimberRole::Left);
        let joints = arch_joints(
            &pieces,
            &hosts(&k, &pieces, &[]),
            &settings,
            &pockets,
            &mut Vec::new(),
        );
        assert_eq!(joints.len(), 1);
        assert_eq!(joints[0].corner, 1);
    }

    #[test]
    fn spring_joint_and_adjacent_transom_mortises_do_not_intersect() {
        let mut conflicts_exercised = 0;
        let mut fitted_pairs = 0;
        for through in [ThroughMember::Dorpel, ThroughMember::Stijl] {
            let mut base = frame(ShapeType::Arched, 300.);
            base.frame.joints_configured = true;
            base.frame.corner_joints[0].through_member = through;
            let ordinary = compute_2d_geometry(&base)
                .arch_joints
                .remove(0)
                .tenon
                .unwrap();
            for position in (265..=335).step_by(5) {
                let mut k = base.clone();
                k.add_divider_in_elevation(true, f64::from(position));
                let g = compute_2d_geometry(&k);
                let end = &g.curved_divider_timber[0].start;
                let Some(d) = &end.tenon else {
                    assert!(end.tenon_issue.is_some());
                    continue;
                };
                let shoulder = Contour {
                    segments: d.shoulder.clone(),
                }
                .bounds();
                let d_rect = box_xy(
                    0,
                    d.cross_range_mm,
                    [d.bottom_axis_mm, shoulder.x + shoulder.width],
                );
                let collides = |joint: &ArchTenon| {
                    let a_rect = quad(joint.shoulder, joint.bottom).bounds();
                    overlaps(&a_rect, &d_rect)
                        && joint.z_from_inside_mm.iter().any(|a| {
                            d.z_from_inside_mm
                                .iter()
                                .any(|b| a[0] < b[1] + WEB - TOL && b[0] < a[1] + WEB - TOL)
                        })
                };
                if collides(&ordinary) {
                    conflicts_exercised += 1;
                }
                if let Some(joint) = &g.arch_joints[0].tenon {
                    assert!(!collides(joint), "transom centre {position}");
                    fitted_pairs += 1;
                }
            }
        }
        assert!(conflicts_exercised > 0);
        assert!(fitted_pairs > 0);
    }

    #[test]
    fn radial_seams_have_two_square_shoulders_and_a_developed_tongue_allowance() {
        let g = compute_2d_geometry(&frame(ShapeType::Round, 0.));
        let [upper, lower] = g.curved_timber.as_slice() else {
            panic!("two half-rings")
        };
        assert_eq!((upper.start_corner, upper.end_corner), (Some(0), Some(1)));
        assert_eq!((lower.start_corner, lower.end_corner), (Some(1), Some(0)));
        assert_eq!(g.arch_joints.len(), 2);
        for j in &g.arch_joints {
            assert_eq!(j.kind, ArchJointKind::RingSeam);
            assert_eq!(j.source, FrameMemberRef::from(lower));
            assert_eq!(j.receiver, FrameMemberRef::from(upper));
            let radial = if j.corner == 0 {
                upper.start_cut
            } else {
                upper.end_cut
            };
            assert_eq!(j.shoulder_face, radial);
            near(j.source_angle_deg, 90.);
            near(j.receiver_angle_deg, 90.);
            let t = j.tenon.as_ref().unwrap();
            assert_eq!(t.direction, [0., -1.]);
            near(t.shoulder_center[1], 725.); // circle centred in a taller opening
            near(t.tip[0][1], 705.);
            near(t.bottom[1][1], 703.);
            near(t.tenon_length_mm, 20.);
            near(t.mortise_depth_mm, 22.);
            assert_eq!(t.z_from_inside_mm.len(), 2);
            for b in &t.z_from_inside_mm {
                near(b[1] - b[0], 12.);
            }
            near(norm(sub(t.shoulder[0], t.shoulder[1])), 25.);
            let to_centre = t
                .tip
                .into_iter()
                .map(|p| (p[0] - 600.).abs())
                .fold(f64::INFINITY, f64::min);
            near(t.blank_extension_mm, 600. * 20_f64.atan2(to_centre));
            assert!(t.blank_extension_mm > t.tenon_length_mm);
            for p in t.bottom {
                inside(&upper.outline, p);
            }
            // Re-open documents exported before the joint discriminator existed.
            let mut old = serde_json::to_value(j).unwrap();
            old.as_object_mut().unwrap().remove("kind");
            assert_eq!(
                serde_json::from_value::<ArchJoint>(old).unwrap().kind,
                ArchJointKind::Springing
            );
        }
    }

    #[test]
    fn ring_mates_by_one_translation_with_reversed_or_mixed_donors() {
        let mut k = frame(ShapeType::Round, 0.);
        k.frame.joints_configured = true;
        k.frame.corner_joints[0].pen_length = 14.;
        k.frame.corner_joints[1].pen_length = 32.;
        for through in [ThroughMember::Dorpel, ThroughMember::Stijl] {
            k.frame.corner_joints[0].through_member = through;
            k.frame.corner_joints[1].through_member = ThroughMember::Stijl;
            let g = compute_2d_geometry(&k);
            for j in &g.arch_joints {
                let t = j.tenon.as_ref().unwrap();
                let donor_is_upper = j.source.index == 0;
                assert_eq!(
                    donor_is_upper,
                    j.corner == 1 || through == ThroughMember::Stijl
                );
                let upper_to_lower = if donor_is_upper {
                    t.direction
                } else {
                    t.direction.map(|v| -v)
                };
                assert_eq!(upper_to_lower, [0., 1.]);
                near(t.tenon_length_mm, if j.corner == 0 { 14. } else { 32. });
                near(
                    dot(sub(t.tip[1], t.shoulder_center), t.direction),
                    t.tenon_length_mm,
                );
                near(dot(sub(t.bottom[0], t.tip[0]), t.direction), 2.);
            }
        }
        for kind in [JointType::Stomp, JointType::Verstek, JointType::Contramal] {
            k.frame.corner_joints[0].joint_type = kind;
            let joints = compute_2d_geometry(&k).arch_joints;
            assert_eq!(joints.len(), 1);
            assert_eq!(joints[0].corner, 1);
        }
        k.frame.bottom_material = Some(Material::Aluminum);
        assert!(compute_2d_geometry(&k).arch_joints.is_empty());
        k.frame.bottom_material = None;
        k.frame.bottom_profile = Some(ProfileRef {
            id: "missing-section".into(),
            name: "Maatwerk".into(),
        });
        assert_eq!(
            compute_2d_geometry(&k).arch_joints[0].issue,
            Some(TenonIssue::UnknownProfile)
        );
        k.frame.bottom_profile = None;
        k.frame.corner_joints[1].pen_length = 500.;
        let row = compute_2d_geometry(&k).arch_joints.remove(0);
        assert!(row.tenon.is_none());
        assert_eq!(row.issue, Some(TenonIssue::InsufficientStock));
    }

    #[test]
    fn ring_seams_use_both_selected_sections_and_the_actual_glazing_face() {
        let mut k = frame(ShapeType::Round, 0.);
        k.frame.frame_width = 40.;
        k.set_grid_sizes_fitted(&[], &[]);
        k.frame.bottom_profile = Some(ProfileRef::default_sill());
        for side in [GlaslatPosition::Binnen, GlaslatPosition::Buiten] {
            for cell in &mut k.cells {
                cell.glaslat = Some(crate::glaslat::Glaslat {
                    position: side,
                    ..Default::default()
                });
            }
            let g = compute_2d_geometry(&k);
            assert_eq!(g.arch_joints.len(), 2);
            for j in g.arch_joints {
                let t = j.tenon.unwrap();
                assert_eq!(t.z_from_inside_mm.len(), 2);
                for [lo, hi] in t.z_from_inside_mm {
                    if side == GlaslatPosition::Binnen {
                        assert!(lo >= 56. - TOL, "tongue in sloped inside section at {lo}");
                    } else {
                        assert!(hi <= 58. + TOL, "tongue in sloped outside section at {hi}");
                    }
                }
            }
        }
    }

    #[test]
    fn cut_ring_seams_reference_surviving_segments_and_exclude_open_ends() {
        use crate::layout::{buiten, glas, split_row, VakChild};
        let mut k = frame(ShapeType::Round, 0.);
        let bay = |node| VakChild { size: 1., node };
        k.layout = Some(split_row(vec![bay(buiten()), bay(glas())]));
        let g = compute_2d_geometry(&k);
        assert_eq!(g.arch_joints.len(), 1);
        assert_eq!(g.arch_joints[0].corner, 1);
        assert!(g.arch_joints[0].tenon.is_some());
        assert!(g
            .curved_timber
            .iter()
            .all(|p| p.start_corner != Some(0) && p.end_corner != Some(0)));

        k.layout = Some(split_row(vec![bay(glas()), bay(buiten()), bay(glas())]));
        let g = compute_2d_geometry(&k);
        assert_eq!(g.curved_timber.len(), 4);
        assert_eq!(g.arch_joints.len(), 2);
        assert_ne!(g.arch_joints[0].source, g.arch_joints[1].source);
        assert_ne!(g.arch_joints[0].receiver, g.arch_joints[1].receiver);
        for j in g.arch_joints {
            let src = g
                .curved_timber
                .iter()
                .find(|p| FrameMemberRef::from(*p) == j.source)
                .unwrap();
            let dst = g
                .curved_timber
                .iter()
                .find(|p| FrameMemberRef::from(*p) == j.receiver)
                .unwrap();
            assert_eq!(src.profile_side, TimberProfileSide::Bottom);
            assert_eq!(dst.profile_side, TimberProfileSide::Top);
            for member in [src, dst] {
                let original = MemberEnd::at_corner(member, j.corner).unwrap();
                assert_eq!(original.cut(), j.shoulder_face);
                assert!(member.start_corner.is_none() || member.end_corner.is_none());
            }
        }
    }

    #[test]
    fn ring_seam_shifts_off_a_partial_notch_and_disappears_without_shared_contact() {
        let k = frame(ShapeType::Round, 0.);
        let g = compute_2d_geometry(&k);
        let mut pieces = g.curved_timber;
        pieces[0].masonry_notches.push(Rect2D {
            x: 0.,
            width: 29.,
            y: 680.,
            height: 45.,
        });
        let rows = arch_joints(
            &pieces,
            &hosts(&k, &pieces, &[]),
            &g.corner_settings,
            &[],
            &mut Vec::new(),
        );
        let t = rows[0].tenon.as_ref().unwrap();
        assert!(t.shoulder.iter().all(|p| p[0] >= 32. - EPS));
        // Each member still reaches the nominal seam, but their remaining
        // strips of shoulder are mutually disjoint on this side.
        pieces[1].masonry_notches.push(Rect2D {
            x: 29.,
            width: 38.,
            y: 725.,
            height: 40.,
        });
        crate::curved_timber::clear_unpaired_ring_ends(&mut pieces);
        let rows = arch_joints(
            &pieces,
            &hosts(&k, &pieces, &[]),
            &g.corner_settings,
            &[],
            &mut Vec::new(),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].corner, 1);
    }

    #[test]
    fn diameter_transom_mortises_and_both_ring_joints_fit_together() {
        let mut k = frame(ShapeType::Round, 0.);
        let ordinary = compute_2d_geometry(&k).arch_joints;
        k.add_divider_in_elevation(true, 725.);
        let g = compute_2d_geometry(&k);
        let divider = &g.curved_divider_timber[0];
        let mut overlaps_original = false;
        for (corner, end) in [&divider.start, &divider.end].into_iter().enumerate() {
            let d = end.tenon.as_ref().unwrap();
            let shoulder = Contour {
                segments: d.shoulder.clone(),
            }
            .bounds();
            let outer_x = if corner == 0 {
                shoulder.x + shoulder.width
            } else {
                shoulder.x
            };
            let rect = box_xy(0, d.cross_range_mm, [d.bottom_axis_mm, outer_x]);
            let conflicts = |t: &ArchTenon| {
                overlaps(&rect, &quad(t.shoulder, t.bottom).bounds())
                    && t.z_from_inside_mm.iter().any(|a| {
                        d.z_from_inside_mm
                            .iter()
                            .any(|b| a[0] < b[1] + WEB - TOL && b[0] < a[1] + WEB - TOL)
                    })
            };
            overlaps_original |= conflicts(ordinary[corner].tenon.as_ref().unwrap());
            assert!(!conflicts(g.arch_joints[corner].tenon.as_ref().unwrap()));
        }
        assert!(overlaps_original);
    }
}
