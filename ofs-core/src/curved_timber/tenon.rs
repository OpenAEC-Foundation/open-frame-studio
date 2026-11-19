//! Blind mortises and square-ended tongues on shaped divider shoulders.
use super::{
    Contour, CurvedDividerMember, CurvedTimberMember, DividerEnd, Point, Segment,
    TimberProfileSide, TimberRole, EPS,
};
use crate::geometry::{CellRect, Rect2D};
use crate::glaslat::GlaslatPosition;
use crate::kozijn::{Kozijn, Material};
use crate::profile::ProfileRef;
use serde::{Deserialize, Serialize};

mod arch;
pub use arch::{ArchJoint, ArchJointKind, ArchTenon, FrameMemberRef};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum TenonReceiver {
    Frame {
        role: TimberRole,
        index: usize,
    },
    Divider {
        vertical: bool,
        source_index: usize,
        piece_index: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenonIssue {
    NoReceiver,
    InsufficientStock,
    UnknownProfile,
}

impl std::fmt::Display for TenonIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoReceiver => "Geen ontvangend hout",
            Self::InsufficientStock => "Onvoldoende hout voor pen en gat",
            Self::UnknownProfile => "Profielgegevens ontbreken voor pen en gat",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DividerTenon {
    pub receiver: TenonReceiver,
    pub shoulder_center: Point,
    pub shoulder: Vec<Segment>,
    pub cross_range_mm: [f64; 2],
    /// Distance into the frame from its interior face; one range per tongue.
    pub z_from_inside_mm: Vec<[f64; 2]>,
    /// Tip and bottom are flat, perpendicular to the axis of insertion.
    pub tip_axis_mm: f64,
    pub bottom_axis_mm: f64,
    pub tenon_length_mm: f64,
    pub mortise_depth_mm: f64,
    pub blank_extension_mm: f64,
    pub bottom_clearance_mm: f64,
}

// Workspace dimensions for shaped timber T joints (docs/verbindingsleer.md).
// The rear and side webs and the axial bottom clearance are model allowances.
const TONGUE_WIDTH: f64 = 25.;
const TONGUE_THICKNESS: f64 = 12.;
const NOMINAL_INSERTION: f64 = 25.;
const CROSSING_MIN: f64 = 12.5;
const CLEARANCE: f64 = 2.;
const WEB: f64 = 3.;
const TOL: f64 = 1e-5;

#[derive(Clone)]
struct Stock {
    depth: f64,
    glass_depth: f64,
    blocked: Vec<(f64, f64)>,
    outside: bool,
}

// Keep a full-width layer of wood behind nonstandard grooves and sloping sills.
// The ordinary glass rebate is already swept around the actual pane outline.
fn section_stock(
    points: &[Point],
    depth: f64,
    glass_depth: f64,
    h: f64,
    outside: bool,
) -> Option<Stock> {
    let mut stock = Stock {
        depth,
        glass_depth,
        blocked: Vec::new(),
        outside,
    };
    if !(depth.is_finite() && depth > 0.) {
        return None;
    }
    if points.is_empty() {
        return Some(stock);
    }
    if points.len() < 3 || points.iter().flatten().any(|x| !x.is_finite()) {
        return None;
    }
    let limits = |axis: usize| {
        points
            .iter()
            .fold([f64::INFINITY, f64::NEG_INFINITY], |[a, b], p| {
                [a.min(p[axis]), b.max(p[axis])]
            })
    };
    let [left, right] = limits(0);
    let [back, face] = limits(1);
    stock.depth = face - back;
    if stock.depth <= EPS || right - left < EPS {
        return None;
    }
    let mut levels: Vec<_> = points.iter().map(|p| p[1]).collect();
    levels.extend([
        (back + glass_depth).clamp(back, face),
        (face - glass_depth).clamp(back, face),
    ]);
    levels.sort_by(f64::total_cmp);
    levels.dedup_by(|a, b| (*a - *b).abs() < EPS);
    let mut irregular = false;
    for pair in levels.windows(2) {
        let [low, high] = [pair[0], pair[1]];
        let in_rebate = if outside {
            high <= back + glass_depth + EPS
        } else {
            low >= face - glass_depth - EPS
        };
        let allowance = if in_rebate { h } else { 0. };
        let mut full = true;
        for y in [low + (high - low) * 1e-7, high - (high - low) * 1e-7] {
            let mut xs = Vec::new();
            for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
                if (a[1] > y) != (b[1] > y) {
                    xs.push(a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]));
                }
            }
            xs.sort_by(f64::total_cmp);
            let spans = |margin: f64| {
                xs.chunks_exact(2)
                    .any(|i| i[0] <= left + margin + TOL && i[1] >= right - margin - TOL)
            };
            full &= spans(0.);
            irregular |= !spans(allowance);
        }
        if !full {
            stock.blocked.push((face - high - WEB, face - low + WEB));
        }
    }
    if !irregular {
        stock.blocked.clear();
    }
    Some(stock)
}

fn custom_stock(p: &crate::profile::ProfileDefinition) -> Option<Stock> {
    let glass = p.sponning.as_ref().map_or(p.glazing_rebate, |s| s.width);
    let height = p.sponning.as_ref().map_or(0., |s| s.depth);
    let outside = p
        .sponning
        .as_ref()
        .is_some_and(|s| s.position.contains("buiten"));
    let mut s = section_stock(&p.cross_section, p.depth, glass, height, outside)?;
    let face = p
        .cross_section
        .iter()
        .map(|p| p[1])
        .max_by(f64::total_cmp)
        .unwrap_or(p.depth);
    if let Some(walls) = p.details.get("innerWalls").and_then(|v| v.as_array()) {
        for wall in walls {
            let ring = wall.as_array()?;
            if ring.is_empty() {
                continue;
            }
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for pt in ring {
                let y = pt.get(1)?.as_f64()?;
                lo = lo.min(y);
                hi = hi.max(y);
            }
            s.blocked.push((face - hi - WEB, face - lo + WEB));
        }
    }
    Some(s)
}

fn wooden(material: &Material) -> bool {
    matches!(material, Material::Wood(_) | Material::WoodAluminum)
}

fn stock(k: &Kozijn, r: &ProfileRef) -> Result<Stock, TenonIssue> {
    static TABLE: &[(&str, &str, f64, &[Point])] =
        include!(concat!(env!("OUT_DIR"), "/joinery_stock.rs"));
    let frame = &k.frame;
    if let Some(p) = frame.member_profiles.get(&r.id) {
        if !matches!(p.material.as_str(), "wood" | "wood_aluminum") {
            return Err(TenonIssue::NoReceiver);
        }
        return custom_stock(p).ok_or(TenonIssue::UnknownProfile);
    }
    let item = TABLE
        .binary_search_by(|row| row.0.cmp(&r.id))
        .ok()
        .map(|i| TABLE[i]);
    let depth = match item {
        Some((_, "wood" | "wood_aluminum", d, _)) => d,
        None if r.id == frame.profile.id && !r.id.is_empty() => frame.frame_depth,
        Some(_) => return Err(TenonIssue::NoReceiver),
        None => return Err(TenonIssue::UnknownProfile),
    };
    let snapshot = (r.id == frame.profile.id)
        .then_some(frame.profile_snapshot.as_ref())
        .flatten()
        .or_else(|| crate::glazing_fit::catalog_snapshot(&r.id, &frame.material));
    let depth = if r.id == frame.profile.id {
        depth.min(frame.frame_depth)
    } else {
        depth
    };
    if !depth.is_finite() || depth <= 0. {
        return Err(TenonIssue::UnknownProfile);
    }
    let points = item.map_or([].as_slice(), |row| row.3);
    let glass = snapshot.and_then(|s| s.sponning_diepte).unwrap_or(51.);
    let h = snapshot.and_then(|s| s.sponning_hoogte).unwrap_or(17.);
    let outside = matches!(
        snapshot.and_then(|s| s.bead_position),
        Some(GlaslatPosition::Buiten)
    );
    let mut section =
        section_stock(points, depth, glass, h, outside).ok_or(TenonIssue::UnknownProfile)?;
    section.depth = section.depth.min(depth);
    Ok(section)
}

fn divider_stock(k: &Kozijn, d: &CurvedDividerMember) -> Result<Stock, TenonIssue> {
    let divisions = if d.vertical {
        &k.grid.columns
    } else {
        &k.grid.rows
    };
    let r = if k.layout.is_none() {
        divisions
            .get(d.source_index + 1)
            .and_then(|s| s.divider_profile.as_ref())
    } else {
        None
    };
    stock(k, r.unwrap_or(&k.frame.profile))
}

#[derive(Clone)]
struct Host<'a> {
    key: TenonReceiver,
    outline: &'a Contour,
    bounds: Rect2D,
    notches: &'a [Rect2D],
    stock: Result<Stock, TenonIssue>,
}

fn hosts<'a>(
    k: &Kozijn,
    frame: &'a [CurvedTimberMember],
    pieces: &'a [CurvedDividerMember],
) -> Vec<Host<'a>> {
    let f = &k.frame;
    let mut hosts: Vec<_> = frame
        .iter()
        .map(|p| {
            let (r, m) = match p.profile_side {
                TimberProfileSide::Top => (&f.top_profile, &f.top_material),
                TimberProfileSide::Bottom => (&f.bottom_profile, &f.bottom_material),
                TimberProfileSide::Left => (&f.left_profile, &f.left_material),
                TimberProfileSide::Right => (&f.right_profile, &f.right_material),
            };
            let fallback = (p.role == TimberRole::Bottom)
                .then_some(f.sill_profile.as_ref())
                .flatten();
            let section = r.as_ref().or(fallback).unwrap_or(&f.profile);
            Host {
                key: TenonReceiver::Frame {
                    role: p.role,
                    index: p.index,
                },
                outline: &p.outline,
                bounds: p.outline.bounds(),
                notches: &p.masonry_notches,
                stock: if wooden(m.as_ref().unwrap_or(&f.material)) {
                    stock(k, section)
                } else {
                    Err(TenonIssue::NoReceiver)
                },
            }
        })
        .collect();
    hosts.extend(pieces.iter().map(|p| Host {
        key: TenonReceiver::Divider {
            vertical: p.vertical,
            source_index: p.source_index,
            piece_index: p.piece_index,
        },
        outline: &p.outline,
        bounds: p.outline.bounds(),
        notches: &[],
        stock: divider_stock(k, p),
    }));
    hosts
}

fn interval(rect: &Rect2D, axis: usize) -> [f64; 2] {
    if axis == 0 {
        [rect.x, rect.x + rect.width]
    } else {
        [rect.y, rect.y + rect.height]
    }
}

fn box_xy(axis: usize, cross: [f64; 2], ends: [f64; 2]) -> Rect2D {
    let along = [ends[0].min(ends[1]), ends[0].max(ends[1])];
    let (x, y) = if axis == 0 {
        (along, cross)
    } else {
        (cross, along)
    };
    Rect2D {
        x: x[0],
        y: y[0],
        width: x[1] - x[0],
        height: y[1] - y[0],
    }
}

fn overlaps(a: &Rect2D, b: &Rect2D) -> bool {
    a.x < b.x + b.width - EPS
        && b.x < a.x + a.width - EPS
        && a.y < b.y + b.height - EPS
        && b.y < a.y + a.height - EPS
}

fn hits(edge: &Segment, cross: usize, value: f64) -> Vec<(f64, Point)> {
    let mut ts = Vec::new();
    match edge {
        Segment::Line { from, to } => {
            let span = to[cross] - from[cross];
            if span.abs() > EPS {
                let t = (value - from[cross]) / span;
                if t > 0. && t < 1. {
                    ts.push(t);
                }
            }
        }
        Segment::Arc {
            center,
            radius,
            start_angle,
            sweep_angle,
            ..
        } => {
            let ratio = (value - center[cross]) / radius;
            if ratio.abs() <= 1. {
                let phase = if cross == 0 {
                    0.
                } else {
                    std::f64::consts::FRAC_PI_2
                };
                for a in [phase - ratio.acos(), phase + ratio.acos()] {
                    let t = ((a - start_angle) * sweep_angle.signum())
                        .rem_euclid(std::f64::consts::TAU)
                        / sweep_angle.abs();
                    if t > 0. && t < 1. {
                        ts.push(t);
                    }
                }
            }
        }
    }
    if (edge.from()[cross] - value).abs() < EPS {
        ts.push(0.);
    }
    if (edge.to()[cross] - value).abs() < EPS {
        ts.push(1.);
    }
    ts.into_iter().map(|t| (t, edge.point(t))).collect()
}

fn shoulder_at(end: &DividerEnd, cross: usize, u: f64) -> Option<Point> {
    point_on_shoulder(&end.segments, cross, u)
}

fn point_on_shoulder(segments: &[Segment], cross: usize, u: f64) -> Option<Point> {
    segments
        .iter()
        .find_map(|s| hits(s, cross, u).into_iter().next().map(|(_, p)| p))
}

fn shoulder_slice(end: &DividerEnd, cross: usize, range: [f64; 2]) -> Vec<Segment> {
    end.segments
        .iter()
        .filter_map(|s| {
            let lo = range[0].max(s.from()[cross]);
            let hi = range[1].min(s.to()[cross]);
            if hi <= lo + EPS {
                return None;
            }
            Some(s.section(hits(s, cross, lo).first()?.0, hits(s, cross, hi).first()?.0))
        })
        .collect()
}

// Over each interval between segment ends and circular extrema, each wall of
// a ray interval is monotone. Its closest possible exit is at an endpoint.
fn breakpoints(path: &[Segment], cross: usize, range: [f64; 2], out: &mut Vec<f64>) {
    for s in path {
        out.extend(
            [s.from()[cross], s.to()[cross]]
                .into_iter()
                .filter(|x| *x > range[0] && *x < range[1]),
        );
        if let Segment::Arc { center, radius, .. } = s {
            out.extend(
                [
                    center[cross] - radius,
                    center[cross],
                    center[cross] + radius,
                ]
                .into_iter()
                .filter(|x| *x > range[0] && *x < range[1]),
            );
        }
    }
}

fn host_exit(
    outline: &Contour,
    notches: &[Rect2D],
    shoulder: &[Segment],
    axis: usize,
    sign: f64,
    range: [f64; 2],
) -> Option<f64> {
    let cross = 1 - axis;
    let mut breaks = range.to_vec();
    breakpoints(&outline.segments, cross, range, &mut breaks);
    breakpoints(shoulder, cross, range, &mut breaks);
    for r in notches {
        breaks.extend(
            interval(r, cross)
                .into_iter()
                .filter(|v| *v > range[0] && *v < range[1]),
        );
    }
    breaks.sort_by(f64::total_cmp);
    breaks.dedup_by(|a, b| (*a - *b).abs() < EPS);
    let mut limit = f64::INFINITY;
    for pair in breaks.windows(2) {
        let step = ((pair[1] - pair[0]) / 4.).min(1e-6);
        for u in [pair[0] + step, (pair[0] + pair[1]) / 2., pair[1] - step] {
            let mouth = sign * point_on_shoulder(shoulder, cross, u)?[axis];
            let mut stops: Vec<_> = outline
                .segments
                .iter()
                .flat_map(|s| hits(s, cross, u))
                .map(|(_, p)| sign * p[axis])
                .collect();
            stops.sort_by(f64::total_cmp);
            stops.dedup_by(|a, b| (*a - *b).abs() < EPS);
            let pair = stops
                .chunks_exact(2)
                .find(|r| (r[0] - mouth).abs() < TOL && r[1] > mouth + EPS)?;
            limit = limit.min(pair[1]);
            for notch in notches {
                let w = interval(notch, cross);
                if u >= w[0] && u <= w[1] {
                    let limits = interval(notch, axis).map(|v| sign * v);
                    if limits[0].max(limits[1]) > mouth + EPS {
                        limit = limit.min(limits[0].min(limits[1]));
                    }
                }
            }
        }
    }
    limit.is_finite().then_some(limit)
}

#[derive(Clone)]
struct Occupied {
    rect: Rect2D,
    bands: Vec<[f64; 2]>,
}

struct Pocket<'a> {
    path: &'a Contour,
    opening: &'a Contour,
    band: f64,
    bounds: Rect2D,
    outside: bool,
    depth: Option<f64>,
}

fn pockets(cells: &[CellRect], band: f64) -> Vec<Pocket<'_>> {
    cells
        .iter()
        .filter_map(|cell| {
            let f = cell.fit.as_ref()?;
            let c = f.contours.as_ref()?;
            let (path, outside, depth) = if let Some(seat) = &c.sash_seat {
                (
                    seat,
                    c.sash_opens_outward.unwrap_or(false),
                    c.sash_seat_depth,
                )
            } else {
                (
                    &c.rebate,
                    matches!(f.bead_position, GlaslatPosition::Buiten),
                    None,
                )
            };
            Some(Pocket {
                opening: cell.opening_contour.as_ref()?,
                band,
                path,
                bounds: path.bounds(),
                outside,
                depth,
            })
        })
        .collect()
}

fn z_bands(
    source: &Stock,
    receiver: &Stock,
    volume: &Rect2D,
    pockets: &[Pocket],
    used: &[Occupied],
) -> Option<Vec<[f64; 2]>> {
    let depth = source.depth.min(receiver.depth);
    let count = if depth >= 114. - EPS { 2 } else { 1 };
    let mut blocked = Vec::new();
    if !source.blocked.is_empty() || !receiver.blocked.is_empty() {
        let mut faces = [false; 2];
        for p in pockets {
            let band = volume.inset(-p.band);
            if overlaps(&p.bounds, &band) && p.opening.clone().clipped(&band).area() > EPS {
                faces[usize::from(p.outside)] = true;
            }
        }
        for s in [source, receiver] {
            if faces[usize::from(s.outside)] || !faces.contains(&true) {
                blocked.extend_from_slice(&s.blocked);
            }
            if faces[usize::from(!s.outside)] {
                blocked.extend(s.blocked.iter().map(|&(a, b)| (s.depth - b, s.depth - a)));
            }
        }
    }
    for p in pockets {
        if overlaps(&p.bounds, volume) && p.path.clone().clipped(volume).area() > EPS {
            for s in [source, receiver] {
                let cut = p.depth.unwrap_or(s.glass_depth).clamp(0., s.depth);
                blocked.push(if p.outside {
                    (s.depth - cut - WEB, s.depth)
                } else {
                    (0., cut + WEB)
                });
            }
        }
    }
    // The stored box includes half the required web. Inflate the candidate by
    // the other half so adjacent cuts in one depth band retain the whole web.
    let clearance_volume = volume.inset(-WEB / 2.);
    for other in used.iter().filter(|o| overlaps(&o.rect, &clearance_volume)) {
        blocked.extend(other.bands.iter().map(|v| (v[0] - WEB, v[1] + WEB)));
    }
    let free = crate::layout::subtract_intervals(WEB, depth - WEB, &blocked, EPS);
    let mut placed = Vec::new();
    // First try equal centres in the available depth. Shift only when a
    // glazing rebate, sash stop, or another hole occupies that position.
    for i in 0..count {
        let desired = (i + 1) as f64 * depth / (count + 1) as f64 - TONGUE_THICKNESS / 2.;
        let prev = placed.last().map_or(WEB, |b: &[f64; 2]| b[1] + WEB);
        let mut options: Vec<_> = free
            .iter()
            .filter_map(|&(lo, hi)| {
                let lo = lo.max(prev);
                let hi = hi - TONGUE_THICKNESS;
                (hi >= lo).then(|| desired.clamp(lo, hi))
            })
            .collect();
        options.sort_by(|a, b| (a - desired).abs().total_cmp(&(b - desired).abs()));
        if let Some(&lo) = options.first() {
            placed.push([lo, lo + TONGUE_THICKNESS]);
        } else {
            // Earliest-fit is complete for equal-size intervals.
            placed.clear();
            for (lo, hi) in free {
                let mut at = lo;
                while at + TONGUE_THICKNESS <= hi + EPS && placed.len() < count {
                    placed.push([at, at + TONGUE_THICKNESS]);
                    at += TONGUE_THICKNESS + WEB;
                }
            }
            break;
        }
    }
    (placed.len() == count).then_some(placed)
}

fn opposing(
    bounds: &Rect2D,
    source: &CurvedDividerMember,
    pieces: &[CurvedDividerMember],
    axis: usize,
    at_end: bool,
    u: f64,
) -> bool {
    let high = interval(bounds, axis)[usize::from(at_end)];
    pieces.iter().any(|p| {
        p.vertical == source.vertical
            && shoulder_at(if at_end { &p.start } else { &p.end }, 1 - axis, u)
                .is_some_and(|s| (s[axis] - high).abs() < TOL)
    })
}

fn joint_at(
    source: &CurvedDividerMember,
    at_end: bool,
    section: &Stock,
    hosts: &[Host],
    pieces: &[CurvedDividerMember],
    pockets: &[Pocket],
    occupied: &[Occupied],
) -> Result<(DividerTenon, Occupied), TenonIssue> {
    let axis = usize::from(source.vertical);
    let cross = 1 - axis;
    let sign = if at_end { 1. } else { -1. };
    let end = if at_end { &source.end } else { &source.start };
    let end_box = Contour {
        segments: end.segments.clone(),
    }
    .bounds();
    let lo = end.segments.first().unwrap().from()[cross];
    let hi = end.segments.last().unwrap().to()[cross];
    let middle = (lo + hi) / 2.;
    let reach = TONGUE_WIDTH / 2. + WEB;
    if hi - lo < 2. * reach {
        return Err(TenonIssue::InsufficientStock);
    }
    let mut centers = vec![middle, lo + reach, hi - reach];
    // A tongue on the diameter may have to move to one side of a frame
    // joint. Include the seam positions and their exact web clearances.
    let nearby: Vec<_> = hosts
        .iter()
        .filter(|h| {
            !matches!(h.key,TenonReceiver::Divider {vertical,..} if vertical==source.vertical)
                && overlaps(&h.bounds, &end_box.inset(-TOL))
        })
        .collect();
    for h in &nearby {
        for edge in &h.outline.segments {
            let u = edge.from()[cross];
            centers.extend(
                [u - reach, u + reach]
                    .into_iter()
                    .filter(|u| *u >= lo + reach && *u <= hi - reach),
            );
        }
    }
    // Test alternate shoulder positions to clear rebates, when a rebate
    // interrupts just one side of the nominal mouth.
    for offset in 1..=((hi - lo - 2. * reach) / 2.).min(64.) as i32 {
        centers.extend([middle - f64::from(offset), middle + f64::from(offset)]);
    }
    centers.sort_by(|a, b| {
        (a - middle)
            .abs()
            .total_cmp(&(b - middle).abs())
            .then(a.total_cmp(b))
    });
    centers.dedup_by(|a, b| (*a - *b).abs() < EPS);
    let mut failure = TenonIssue::NoReceiver;
    let mut best = None;
    let mut best_length = 0.;
    for u in centers {
        let range = [u - TONGUE_WIDTH / 2., u + TONGUE_WIDTH / 2.];
        let Some(center) = shoulder_at(end, cross, u) else {
            continue;
        };
        let shoulder = shoulder_slice(end, cross, range);
        if shoulder.is_empty() {
            continue;
        }
        let b = Contour {
            segments: shoulder.clone(),
        }
        .bounds();
        let ext = interval(&b, axis).map(|v| sign * v);
        let front = ext[0].max(ext[1]);
        let rear = ext[0].min(ext[1]);
        for h in &nearby {
            if host_exit(
                h.outline,
                h.notches,
                &end.segments,
                axis,
                sign,
                [u - 0.01, u + 0.01],
            )
            .is_none()
            {
                continue;
            }
            let stock = match &h.stock {
                Ok(s) => s,
                Err(issue) => {
                    if failure == TenonIssue::NoReceiver {
                        failure = *issue;
                    }
                    continue;
                }
            };
            failure = TenonIssue::InsufficientStock;
            let Some(exit) = host_exit(
                h.outline,
                h.notches,
                &end.segments,
                axis,
                sign,
                [u - reach, u + reach],
            ) else {
                continue;
            };
            let crossing = matches!(h.key, TenonReceiver::Divider { .. });
            let mut max_tip = exit - WEB - CLEARANCE;
            if crossing && opposing(&h.bounds, source, pieces, axis, at_end, u) {
                max_tip = max_tip.min((exit + rear - WEB) / 2. - CLEARANCE);
            }
            let tip = (front + NOMINAL_INSERTION).min(max_tip);
            let engage = tip - front;
            let min = if crossing {
                CROSSING_MIN
            } else {
                NOMINAL_INSERTION
            };
            if engage + TOL < min || engage <= best_length + TOL {
                continue;
            }
            let bottom = tip + CLEARANCE;
            let volume = box_xy(axis, range, [(rear - WEB) * sign, bottom * sign]);
            let Some(bands) = z_bands(section, stock, &volume, pockets, occupied) else {
                continue;
            };
            let blank_limit = interval(&end_box, axis)[usize::from(at_end)] * sign;
            let joint = DividerTenon {
                receiver: h.key.clone(),
                shoulder_center: center,
                shoulder: shoulder.clone(),
                cross_range_mm: range,
                z_from_inside_mm: bands.clone(),
                tip_axis_mm: sign * tip,
                bottom_axis_mm: sign * bottom,
                tenon_length_mm: tip - center[axis] * sign,
                mortise_depth_mm: bottom - center[axis] * sign,
                blank_extension_mm: (tip - blank_limit).max(0.),
                bottom_clearance_mm: CLEARANCE,
            };
            let used = Occupied {
                rect: volume.inset(-WEB / 2.),
                bands,
            };
            if engage >= NOMINAL_INSERTION - TOL {
                return Ok((joint, used));
            }
            best_length = engage;
            best = Some((joint, used));
        }
    }
    best.ok_or(failure)
}

pub(crate) fn fit_timber_tenons(
    k: &Kozijn,
    frame: &[CurvedTimberMember],
    pieces: &mut [CurvedDividerMember],
    cells: &[CellRect],
    settings: &[crate::joint::Joint; 4],
) -> Vec<ArchJoint> {
    if frame.is_empty() && pieces.is_empty() {
        return Vec::new();
    }
    let pockets = pockets(cells, k.frame.frame_width * 0.48);
    let hosts = hosts(k, frame, pieces);
    let mut occupied = Vec::new();
    let mut joints = Vec::new();
    for p in pieces.iter().filter(|_| wooden(&k.frame.material)) {
        let section = divider_stock(k, p);
        for at_end in [false, true] {
            if matches!(section, Err(TenonIssue::NoReceiver)) {
                joints.push(None);
                continue;
            }
            let joint = section
                .as_ref()
                .map_err(|e| *e)
                .and_then(|s| joint_at(p, at_end, s, &hosts, pieces, &pockets, &occupied))
                .map(|(joint, vol)| {
                    occupied.push(vol);
                    joint
                });
            joints.push(Some(joint));
        }
    }
    let arch_joints = arch::arch_joints(frame, &hosts, settings, &pockets, &mut occupied);
    for (end, joint) in pieces
        .iter_mut()
        .flat_map(|p| [&mut p.start, &mut p.end])
        .zip(joints)
    {
        if let Some(joint) = joint {
            match joint {
                Ok(t) => end.tenon = Some(t),
                Err(reason) => end.tenon_issue = Some(reason),
            }
        }
    }
    arch_joints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::ShapeType;

    fn round() -> Kozijn {
        let mut k = Kozijn::new("Rond", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        k
    }

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-5, "{a} ≠ {b}");
    }

    #[test]
    fn off_axis_pen_has_a_flat_tip_and_a_blind_pocket_inside_the_ring() {
        let mut k = round();
        k.add_divider_in_elevation(false, 373.5);
        let g = compute_2d_geometry(&k);
        let d = &g.curved_divider_timber[0];
        let j = d.start.tenon.as_ref().unwrap();
        let [l, r] = j.cross_range_mm;
        near(r - l, 25.);
        assert_eq!(j.z_from_inside_mm.len(), 2);
        for z in &j.z_from_inside_mm {
            near(z[1] - z[0], 12.);
        }
        let day_y = |x: f64| 600. - (533_f64.powi(2) - (x - 600.).powi(2)).sqrt();
        near(j.shoulder_center[1], day_y((l + r) / 2.));
        near(j.tip_axis_mm, day_y(r) - 25.);
        near(j.mortise_depth_mm, j.tenon_length_mm + 2.);
        near(j.bottom_axis_mm, j.tip_axis_mm - 2.);
        near(
            j.blank_extension_mm,
            (d.outline.bounds().y - j.tip_axis_mm).max(0.),
        );
        for x in [l, r] {
            assert!((x - 600.).hypot(j.bottom_axis_mm - 600.) <= 600. - WEB);
        }
        assert!(matches!(
            j.receiver,
            TenonReceiver::Frame {
                role: TimberRole::Arc,
                index: 0
            }
        ));
    }

    #[test]
    fn centred_transom_never_drills_through_the_joint_between_half_rings() {
        let mut k = round();
        k.add_row(533.);
        let g = compute_2d_geometry(&k);
        for e in [
            &g.curved_divider_timber[0].start,
            &g.curved_divider_timber[0].end,
        ] {
            let j = e.tenon.as_ref().unwrap();
            assert!(
                j.cross_range_mm[1] <= 600. - WEB + TOL || j.cross_range_mm[0] >= 600. + WEB - TOL
            );
            let arc = if j.cross_range_mm[1] < 600. { 0 } else { 1 };
            assert_eq!(
                j.receiver,
                TenonReceiver::Frame {
                    role: TimberRole::Arc,
                    index: arc
                }
            );
        }
    }

    #[test]
    fn two_opposing_mortises_leave_a_web_in_a_narrow_transom() {
        let mut k = round();
        k.frame.frame_width = 40.;
        k.set_grid_sizes_fitted(&[], &[]);
        k.add_column(560.);
        k.add_row(560.);
        let g = compute_2d_geometry(&k);
        let a = g.curved_divider_timber[0].end.tenon.as_ref().unwrap();
        let b = g.curved_divider_timber[1].start.tenon.as_ref().unwrap();
        assert_eq!(a.receiver, b.receiver);
        assert!(matches!(
            a.receiver,
            TenonReceiver::Divider {
                vertical: false,
                ..
            }
        ));
        near(a.tenon_length_mm, 16.5);
        near(b.tenon_length_mm, 16.5);
        near(b.bottom_axis_mm - a.bottom_axis_mm, 3.);
    }

    #[test]
    fn insufficient_material_unknown_profiles_and_nonwood_are_distinct() {
        let mut k = round();
        k.add_column(533.);
        k.grid.columns[1].divider_profile = Some(ProfileRef {
            id: "unknown".into(),
            name: "Custom".into(),
        });
        let g = compute_2d_geometry(&k);
        assert_eq!(
            g.curved_divider_timber[0].start.tenon_issue,
            Some(TenonIssue::UnknownProfile)
        );
        k.grid.columns[1].divider_profile = None;
        k.frame.frame_depth = 16.;
        let g = compute_2d_geometry(&k);
        assert_eq!(
            g.curved_divider_timber[0].start.tenon_issue,
            Some(TenonIssue::InsufficientStock)
        );
        k.frame.material = Material::Pvc;
        let g = compute_2d_geometry(&k);
        assert!(g.curved_divider_timber[0].start.tenon.is_none());
        assert!(g.curved_divider_timber[0].start.tenon_issue.is_none());
    }

    #[test]
    fn mixed_arch_and_jamb_shoulders_receive_each_mortise_in_one_piece() {
        let mut k = round();
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.add_divider_in_elevation(true, 333.5);
        let g = compute_2d_geometry(&k);
        let beam = &g.curved_divider_timber[0];
        for (end, sign) in [(&beam.start, -1.), (&beam.end, 1.)] {
            assert!(end
                .segments
                .iter()
                .any(|s| matches!(s, Segment::Arc { .. })));
            assert!(end
                .segments
                .iter()
                .any(|s| matches!(s, Segment::Line { .. })));
            let j = end.tenon.as_ref().unwrap();
            let TenonReceiver::Frame { role, index } = j.receiver else {
                panic!("expected frame stock")
            };
            let host = g
                .curved_timber
                .iter()
                .find(|p| p.role == role && p.index == index)
                .unwrap();
            for u in [
                j.cross_range_mm[0],
                j.shoulder_center[1],
                j.cross_range_mm[1],
            ] {
                let probe = Rect2D {
                    x: j.bottom_axis_mm - 0.01,
                    y: u - 0.01,
                    width: 0.02,
                    height: 0.02,
                };
                near(host.outline.clone().clipped(&probe).area(), 0.02 * 0.02);
            }
            near(j.bottom_axis_mm - j.tip_axis_mm, sign * 2.);
        }
    }

    #[test]
    fn adjacent_mortises_in_one_depth_band_keep_the_full_side_web() {
        let section = Stock {
            depth: 18.,
            glass_depth: 0.,
            outside: false,
            blocked: vec![],
        };
        let old = Rect2D {
            x: 0.,
            y: 0.,
            width: 25.,
            height: 27.,
        };
        let occupied = [Occupied {
            rect: old.inset(-WEB / 2.),
            bands: vec![[3., 15.]],
        }];
        let mut next = Rect2D { x: 27.9, ..old };
        assert!(z_bands(&section, &section, &next, &[], &occupied).is_none());
        next.x = 28.;
        assert_eq!(
            z_bands(&section, &section, &next, &[], &occupied),
            Some(vec![[3., 15.]])
        );
    }

    #[test]
    fn opposed_glazing_pockets_reduce_the_available_depth_of_the_same_joint() {
        let rect = Rect2D {
            x: 0.,
            y: 0.,
            width: 24.,
            height: 25.,
        };
        let pts = [[0., 0.], [30., 0.], [30., 30.], [0., 30.]];
        let contour = Contour {
            segments: (0..4)
                .map(|i| Segment::Line {
                    from: pts[i],
                    to: pts[(i + 1) % 4],
                })
                .collect(),
        };
        let stock = Stock {
            depth: 114.,
            glass_depth: 51.,
            blocked: vec![],
            outside: false,
        };
        let inner = Pocket {
            path: &contour,
            opening: &contour,
            band: 32.,
            bounds: contour.bounds(),
            outside: false,
            depth: None,
        };
        let outer = Pocket {
            path: &contour,
            opening: &contour,
            band: 32.,
            bounds: contour.bounds(),
            outside: true,
            depth: None,
        };
        let bands = z_bands(&stock, &stock, &rect, std::slice::from_ref(&inner), &[]).unwrap();
        assert!(bands.iter().all(|b| b[0] >= 54.));
        assert!(z_bands(&stock, &stock, &rect, &[inner, outer], &[]).is_none());
    }

    #[test]
    fn a_custom_middle_groove_and_an_internal_void_cannot_contain_a_tongue() {
        let raw = include_str!("../../../profiles/wood/meranti.json");
        let mut p: crate::profile::ProfileDefinition =
            serde_json::from_str::<Vec<_>>(raw).unwrap().remove(0);
        p.id = "custom".into();
        p.cross_section = vec![
            [0., 0.],
            [67., 0.],
            [67., 47.],
            [30., 47.],
            [30., 67.],
            [67., 67.],
            [67., 114.],
            [0., 114.],
        ];
        p.details.insert(
            "innerWalls".into(),
            serde_json::json!([[[16, 30], [49, 30], [49, 40], [16, 40]]]),
        );
        let s = custom_stock(&p).unwrap();
        let bands = z_bands(
            &s,
            &s,
            &Rect2D {
                x: 0.,
                y: 0.,
                width: 20.,
                height: 25.,
            },
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(bands.len(), 2);
        for b in bands {
            assert!(b[1] <= 71. + TOL || b[0] >= 87. - TOL);
            assert!(b[1] <= 44. + TOL || b[0] >= 70. - TOL);
        }
    }

    #[test]
    fn catalogue_sloping_sill_keeps_both_mortises_in_its_full_width_stock() {
        let mut k = round();
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.add_column(380.);
        for side in [GlaslatPosition::Binnen, GlaslatPosition::Buiten] {
            for c in &mut k.cells {
                c.glaslat = Some(crate::glaslat::Glaslat {
                    position: side,
                    ..Default::default()
                });
            }
            let g = compute_2d_geometry(&k);
            let j = g.curved_divider_timber[0].end.tenon.as_ref().unwrap();
            assert_eq!(j.z_from_inside_mm.len(), 2);
            for [a, b] in &j.z_from_inside_mm {
                if side == GlaslatPosition::Binnen {
                    assert!(*a >= 56. - TOL, "{a}");
                } else {
                    assert!(*b <= 58. + TOL, "{b}");
                }
            }
        }
    }
}
