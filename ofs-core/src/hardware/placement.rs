//! Generic fitting locations in elevation mm. Equipment is resolved from the
//! assembled leaf so changing a divider, glazing or handing updates all views.
use super::*;
use crate::curved_contour::{Contour, SashLeaf, Segment};
use crate::geometry::{CellRect, Rect2D};
use crate::glazing_fit::GlazingFit;
use crate::kozijn::{Cell, Frame, Glazing, ShapeType};
use crate::layout::Vakvulling;

type Point = [f64; 2];
const EPS: f64 = 1e-6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FittingKind {
    Hinge,
    Pivot,
    Handle,
    Lock,
    LockingPoint,
    Keeper,
    Stay,
    Roller,
    FlushBolt,
    Closer,
    Vent,
}

impl FittingKind {
    pub fn label_nl(self) -> &'static str {
        match self {
            Self::Hinge => "Scharnier",
            Self::Pivot => "Draaipunt",
            Self::Handle => "Greep",
            Self::Lock => "Sluitmechanisme",
            Self::LockingPoint => "Sluitnok",
            Self::Keeper => "Sluitplaat",
            Self::Stay => "Schaar",
            Self::Roller => "Loopwagen",
            Self::FlushBolt => "Kantschuif",
            Self::Closer => "Deurdranger",
            Self::Vent => "Ventilatierooster",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareMount {
    Sash,
    Frame,
    MeetingLeaf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareFitIssue {
    UnsupportedShape,
    InsufficientMountingEdge,
    UnsupportedOpening,
    ProfileLoadExceeded,
    IncompatibleSet,
}

impl HardwareFitIssue {
    pub fn label_nl(self) -> &'static str {
        match self {
            Self::UnsupportedShape => {
                "Beslag voor deze vleugelvorm vraagt een systeemspecifieke uitwerking"
            }
            Self::InsufficientMountingEdge => "Onvoldoende montageruimte voor het gekozen beslag",
            Self::ProfileLoadExceeded => "Vleugelgewicht overschrijdt de profielgrens",
            Self::IncompatibleSet => "De gekozen beslagset past niet bij deze openingswijze",
            Self::UnsupportedOpening => {
                "Beslag voor deze opening vraagt een systeemspecifieke uitwerking"
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareComponent {
    pub kind: FittingKind,
    pub mount: HardwareMount,
    pub side: HingeSide,
    pub position: Point,
    pub tangent: Point,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareLeaf {
    pub leaf: Option<SashLeaf>,
    pub panel_type: PanelType,
    pub opens_outward: bool,
    pub set: Option<HardwareSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<HardwareFitIssue>,
    pub components: Vec<HardwareComponent>,
}

struct Opening {
    panel: PanelType,
    right: bool,
    outward: bool,
    pair: bool,
    supported: bool,
}

impl Opening {
    fn resolve(cell: Option<&Cell>, v: Option<&Vakvulling>) -> Option<Self> {
        // A layout leaf owns its handing, including its default. An aligned
        // grid cell may still store the opening direction of its old vulling.
        let dir = cell
            .filter(|_| v.is_none())
            .and_then(|c| c.opening_direction);
        let mut supported = true;
        let (panel, side, outward, pair) = match v {
            Some(Vakvulling::Raam {
                open_type,
                hinge_side,
                opens_outward,
                hardware,
                ..
            }) => {
                if *hardware == Some(false) {
                    return None;
                }
                let p = match open_type.as_str() {
                    "draai" | "" => PanelType::Turn,
                    "draaikiep" => PanelType::TurnTilt,
                    "valraam" => PanelType::Tilt,
                    "uitzet" => PanelType::TopHung,
                    "tuimel" => PanelType::BottomHung,
                    "taats" => PanelType::Pivot,
                    "schuif" => PanelType::Sliding,
                    "hefschuif" => PanelType::LiftSlide,
                    _ => {
                        supported = false;
                        PanelType::Turn
                    }
                };
                (p, hinge_side.as_deref(), *opens_outward, false)
            }
            Some(Vakvulling::Deur {
                door_kind,
                hinge_side,
                opens_outward,
                hardware,
                ..
            }) => {
                if *hardware == Some(false) {
                    return None;
                }
                let p = match door_kind.as_str() {
                    "enkel" | "dubbel" | "" => PanelType::Door,
                    "taats" => PanelType::Pivot,
                    "schuif" => PanelType::Sliding,
                    "hefschuif" => PanelType::LiftSlide,
                    _ => {
                        supported = false;
                        PanelType::Door
                    }
                };
                (
                    p,
                    hinge_side.as_deref(),
                    *opens_outward,
                    door_kind == "dubbel",
                )
            }
            Some(_) => return None,
            None => (cell?.panel_type, None, None, false),
        };
        panel.is_operable().then_some(Self {
            panel,
            right: side.map_or(dir == Some(OpeningDirection::Right), |s| {
                s == "rechts" || s == "right"
            }),
            outward: outward.unwrap_or(match dir {
                Some(OpeningDirection::Outward) => true,
                Some(OpeningDirection::Inward) => false,
                _ => panel == PanelType::TopHung,
            }),
            pair,
            supported,
        })
    }
}

fn rectangle(r: &Rect2D) -> Contour {
    let [x, y] = [r.x, r.y];
    let corners = [
        [x, y],
        [x + r.width, y],
        [x + r.width, y + r.height],
        [x, y + r.height],
    ];
    Contour {
        segments: (0..4)
            .map(|i| Segment::Line {
                from: corners[i],
                to: corners[(i + 1) % 4],
            })
            .collect(),
    }
}

#[derive(Clone, Copy)]
struct Site {
    point: Point,
    tangent: Point,
    side: HingeSide,
}

impl Site {
    fn at(edge: &Segment, t: f64) -> Self {
        let point = edge.point(t);
        let d = match edge {
            Segment::Line { from, to } => [to[0] - from[0], to[1] - from[1]],
            Segment::Arc {
                center,
                sweep_angle,
                ..
            } => [
                -(point[1] - center[1]) * sweep_angle.signum(),
                (point[0] - center[0]) * sweep_angle.signum(),
            ],
        };
        let len = d[0].hypot(d[1]);
        let tangent = [d[0] / len, d[1] / len];
        let side = if d[0].abs() > d[1].abs() {
            if d[0] > 0. {
                HingeSide::Top
            } else {
                HingeSide::Bottom
            }
        } else if d[1] > 0. {
            HingeSide::Right
        } else {
            HingeSide::Left
        };
        Self {
            point,
            tangent,
            side,
        }
    }
    fn part(self, kind: FittingKind, mount: HardwareMount, inset: f64) -> HardwareComponent {
        HardwareComponent {
            kind,
            mount,
            side: self.side,
            tangent: self.tangent,
            position: [
                self.point[0] - inset * self.tangent[1],
                self.point[1] + inset * self.tangent[0],
            ],
        }
    }
}

struct Rim<'a> {
    path: &'a Contour,
    bounds: Rect2D,
    width: f64,
}

fn opposite(side: HingeSide) -> HingeSide {
    match side {
        HingeSide::Left => HingeSide::Right,
        HingeSide::Right => HingeSide::Left,
        HingeSide::Top => HingeSide::Bottom,
        HingeSide::Bottom => HingeSide::Top,
    }
}

impl Rim<'_> {
    fn line(&self, side: HingeSide) -> Option<&Segment> {
        self.path
            .segments
            .iter()
            .filter(|e| {
                matches!(e, Segment::Line { .. })
                    && e.length() > EPS
                    && Site::at(e, 0.5).side == side
            })
            .max_by(|a, b| a.length().total_cmp(&b.length()))
    }
    /// Intersect the real rim with a horizontal or vertical line. Taking the
    /// outermost hit also works where an arc ends at an inclined jamb.
    fn hit(&self, side: HingeSide, coordinate: f64) -> Option<Site> {
        let axis = usize::from(matches!(side, HingeSide::Left | HingeSide::Right));
        let sign = if matches!(side, HingeSide::Left | HingeSide::Top) {
            -1.
        } else {
            1.
        };
        let mut best: Option<Site> = None;
        for edge in self.path.segments.iter().filter(|e| e.length() > EPS) {
            let mut ts = edge.crossings(axis, coordinate);
            if (edge.from()[axis] - coordinate).abs() < EPS {
                ts.push(0.);
            }
            if (edge.to()[axis] - coordinate).abs() < EPS {
                ts.push(1.);
            }
            for t in ts {
                let site = Site::at(edge, t);
                if best.as_ref().map_or(true, |other| {
                    sign * (site.point[1 - axis] - other.point[1 - axis]) > EPS
                }) {
                    best = Some(site);
                }
            }
        }
        best
    }
    fn mid(&self, side: HingeSide) -> Option<Site> {
        let b = &self.bounds;
        self.hit(
            side,
            if matches!(side, HingeSide::Left | HingeSide::Right) {
                b.y + b.height / 2.
            } else {
                b.x + b.width / 2.
            },
        )
    }
}

fn glazing_kg_m2(glass: &Glazing) -> f64 {
    let panes: f64 = glass
        .panes
        .iter()
        .filter(|p| p.thickness_mm.is_finite() && p.thickness_mm > 0.)
        .map(|p| p.thickness_mm)
        .sum();
    if panes > 0. {
        return panes * 2.5;
    }
    if let Some(p) = crate::glass_library::builtin_glass_library()
        .into_iter()
        .find(|p| {
            p.id.eq_ignore_ascii_case(&glass.glass_type)
                || p.name.eq_ignore_ascii_case(&glass.glass_type)
        })
    {
        return p.weight_kg_m2;
    }
    // Older cells can specify an insulated unit without the pane build-up.
    // Use 4 mm sheets for the generic double/triple glazing estimate.
    let solid =
        if glass.glass_type.to_lowercase().contains("triple") || glass.glass_type.contains("+++") {
            12.
        } else if glass.thickness_mm <= 12. {
            glass.thickness_mm.max(0.)
        } else {
            8.
        };
    2.5 * solid
}

fn leaf_mass_kg(frame: &Frame, fit: &GlazingFit, path: &Contour, cell: Option<&Cell>) -> f64 {
    let area = fit
        .contours
        .as_ref()
        .map_or(fit.glass.width * fit.glass.height, |c| c.glass.area());
    let glass_kg_m2 = cell.map_or_else(
        || glazing_kg_m2(&Glazing::default()),
        |c| glazing_kg_m2(&c.glazing),
    );
    let profile_kg_m = if frame.material == Material::Pvc {
        2.5
    } else {
        2.
    };
    glass_kg_m2 * area / 1e6 + path.perimeter() / 1000. * profile_kg_m
}

fn auto_set(
    o: &Opening,
    right: bool,
    frame: &Frame,
    fit: &GlazingFit,
    path: &Contour,
    security: SecurityClass,
    mass: f64,
    passive: bool,
) -> Option<HardwareSet> {
    let b = path.bounds();
    let mut set = default_hardware_set(
        o.panel,
        Some(if right {
            OpeningDirection::Right
        } else {
            OpeningDirection::Left
        }),
        b.width,
        b.height,
        fit.glass_thickness,
        &frame.material,
        security,
    )?;
    if let Some(hinges) = &mut set.hinges {
        hinges.count = match o.panel {
            PanelType::Pivot | PanelType::BottomHung => 2,
            PanelType::Door => calculate_hinge_count(b.height, mass).max(3),
            PanelType::TopHung | PanelType::Tilt => calculate_hinge_count(b.width, mass),
            _ => calculate_hinge_count(b.height, mass),
        };
        hinges.load_capacity_kg = (mass * 1.5).ceil();
    }
    if let Some(h) = &mut set.handle {
        if matches!(
            o.panel,
            PanelType::Sliding | PanelType::LiftSlide | PanelType::Pivot
        ) {
            h.side = if right {
                HandleSide::Left
            } else {
                HandleSide::Right
            };
            if o.panel == PanelType::LiftSlide {
                h.handle_type = HandleType::TGreep;
            }
        }
        h.height_mm = if matches!(
            o.panel,
            PanelType::Door | PanelType::Sliding | PanelType::LiftSlide
        ) {
            1050_f64.min(b.height / 2. + 100.)
        } else {
            b.height / 2.
        };
    }
    if let Some(l) = &mut set.locking {
        let count = match o.panel {
            PanelType::Door => 3,
            PanelType::Sliding | PanelType::LiftSlide => {
                if b.height > 1800. {
                    3
                } else {
                    2
                }
            }
            PanelType::Tilt | PanelType::TopHung => (b.width / 700.).ceil() as u8,
            _ => (path.perimeter() / 900.).ceil() as u8,
        };
        l.locking_points = count.max(2).max(security.min_locking_points());
    }
    if passive {
        set.handle = None;
        set.locking = None;
        set.closer = None;
    }
    Some(set)
}

fn spaced(edge: &Segment, count: u8) -> Option<Vec<Site>> {
    if count == 0 {
        return Some(vec![]);
    }
    // End clearances are along the actual straight mounting face.
    let len = edge.length();
    let margin = (len * 0.20).min(150.);
    if margin < 35. || (count > 1 && len - 2. * margin < 70. * f64::from(count - 1)) {
        return None;
    }
    Some(
        (0..count)
            .map(|i| {
                let f = if count == 1 {
                    0.5
                } else {
                    f64::from(i) / f64::from(count - 1)
                };
                Site::at(edge, (margin + f * (len - 2. * margin)) / len)
            })
            .collect(),
    )
}

fn fit_components(
    rim: &Rim,
    o: &Opening,
    set: &HardwareSet,
    passive: bool,
    meeting: Option<HingeSide>,
) -> Option<Vec<HardwareComponent>> {
    use FittingKind::*;
    use HardwareMount::{Frame as Fixed, Sash};
    let b = &rim.bounds;
    if b.width < 100. || b.height < 100. || rim.width < 18. {
        return None;
    }
    let mut parts = Vec::new();
    let face = (0.5 * rim.width).min(32.);
    let side = set.hinges.as_ref().map_or(
        if o.right {
            HingeSide::Right
        } else {
            HingeSide::Left
        },
        |h| h.side,
    );
    if let Some(h) = set.hinges.as_ref().filter(|h| h.count > 0) {
        if matches!(o.panel, PanelType::Pivot | PanelType::BottomHung) {
            if h.count != 2 {
                return None;
            }
            let axis = if o.panel == PanelType::Pivot {
                HingeSide::Top
            } else {
                HingeSide::Left
            };
            // Both pins must share one axis even when the local tangent at a
            // curved head differs from the tangent at the sill.
            for edge in [axis, opposite(axis)] {
                parts.push(rim.mid(edge)?.part(Pivot, Sash, 0.));
            }
        } else {
            let edge = rim.line(side)?;
            for at in spaced(edge, h.count)? {
                parts.push(at.part(Hinge, Sash, 4.));
            }
        }
    }
    // These operating supports have no overrides in HardwareSet. They follow
    // the motion even when a stored handle, hinge or lock has been edited.
    match o.panel {
        PanelType::TurnTilt => {
            // The upper stay anchors to the straight jamb below the arch spring.
            let edge = rim.line(side)?;
            let end = if edge.from()[1] < edge.to()[1] {
                0.06
            } else {
                0.94
            };
            parts.push(Site::at(edge, end).part(Stay, Sash, face));
        }
        PanelType::TopHung | PanelType::Tilt => {
            for flank in [HingeSide::Left, HingeSide::Right] {
                parts.push(rim.mid(flank)?.part(Stay, Sash, face));
            }
        }
        PanelType::Sliding | PanelType::LiftSlide => {
            let rail = rim.line(HingeSide::Bottom)?;
            let guide = rim.line(HingeSide::Top)?;
            if (rail.from()[1] - rail.to()[1]).abs() > EPS
                || (guide.from()[1] - guide.to()[1]).abs() > EPS
            {
                return None;
            }
            for at in spaced(rail, 2)? {
                parts.push(at.part(Roller, Sash, 3.));
            }
        }
        _ => {}
    }
    let closing = match set.handle.as_ref().map(|h| h.side) {
        Some(HandleSide::Right) => HingeSide::Right,
        Some(HandleSide::Left) => HingeSide::Left,
        _ => opposite(side),
    };
    let handle_site = if let Some(h) = &set.handle {
        let at = if matches!(closing, HingeSide::Left | HingeSide::Right) {
            if !h.height_mm.is_finite() || h.height_mm < 25. || h.height_mm > b.height - 25. {
                return None;
            }
            rim.hit(closing, b.y + b.height - h.height_mm)?
        } else {
            rim.mid(closing)?
        };
        parts.push(at.part(Handle, Sash, face));
        Some(at)
    } else {
        None
    };
    if let Some(l) = &set.locking {
        let at = match handle_site {
            Some(site) => site,
            None => rim.mid(closing)?,
        };
        parts.push(at.part(Lock, Sash, 5.));
        let mut sites = Vec::new();
        let sides: &[HingeSide] = if matches!(o.panel, PanelType::Turn | PanelType::TurnTilt) {
            &[closing, HingeSide::Top, HingeSide::Bottom]
        } else {
            std::slice::from_ref(&closing)
        };
        for (n, edge) in sides.iter().enumerate() {
            let count = (usize::from(l.locking_points) + sides.len() - 1 - n) / sides.len();
            if count == 0 {
                continue;
            }
            let (lo, span) = if matches!(edge, HingeSide::Left | HingeSide::Right) {
                (b.y, b.height)
            } else {
                (b.x, b.width)
            };
            if span / ((count + 1) as f64) < 36. {
                return None;
            }
            for i in 0..count {
                let coordinate = lo + span * (i + 1) as f64 / (count + 1) as f64;
                let mut at = rim.hit(*edge, coordinate)?;
                // Keep a separate roller clear of the central gearbox.
                if let Some(handle) = handle_site {
                    if at.side == handle.side
                        && (at.point[0] - handle.point[0]).hypot(at.point[1] - handle.point[1])
                            < 35.
                    {
                        at = rim.hit(*edge, (coordinate + 40.).min(lo + span - 18.))?;
                    }
                }
                if sites.iter().any(|s: &Site| {
                    (s.point[0] - at.point[0]).hypot(s.point[1] - at.point[1]) < 24.
                }) {
                    return None;
                }
                sites.push(at);
                parts.push(at.part(LockingPoint, Sash, 2.));
                let receiver = if meeting == Some(*edge) {
                    HardwareMount::MeetingLeaf
                } else {
                    Fixed
                };
                parts.push(at.part(
                    Keeper,
                    receiver,
                    if receiver == HardwareMount::MeetingLeaf {
                        -10.
                    } else {
                        -5.
                    },
                ));
            }
        }
    }
    if passive {
        let x = if meeting == Some(HingeSide::Left) {
            b.x + face
        } else {
            b.x + b.width - face
        };
        for edge in [HingeSide::Top, HingeSide::Bottom] {
            let at = rim.hit(edge, x)?;
            parts.push(at.part(FlushBolt, Sash, 10.));
            parts.push(at.part(Keeper, Fixed, -5.));
        }
    }
    if set.closer.is_some() {
        parts.push(rim.mid(HingeSide::Top)?.part(Closer, Sash, face));
    }
    if set.ventilation.is_some() {
        parts.push(rim.mid(HingeSide::Top)?.part(Vent, Sash, face));
    }
    Some(parts)
}

#[derive(Default)]
struct ProfileLimits {
    sliding_only: bool,
    max_kg: Option<f64>,
}

fn profile_limits(frame: &Frame, id: &str) -> ProfileLimits {
    use crate::profile::ProfileApplication::*;
    const SYSTEMS: &[(&str, bool, Option<f64>)] =
        include!(concat!(env!("OUT_DIR"), "/fitting_limits.rs"));
    if let Some(p) = frame.member_profiles.get(id) {
        let sliding_only = p.material_subtype.as_deref() == Some("sliding")
            || (p.applicable_as.contains(&SlidingSash)
                && !p.applicable_as.iter().any(|a| {
                    matches!(
                        a,
                        Sash | DoorPanel | RaamStijl | RaamDorpel | DeurStijl | DeurDorpel
                    )
                }));
        return ProfileLimits {
            sliding_only,
            max_kg: p
                .details
                .get("maxWeight")
                .and_then(|v| v.as_f64())
                .filter(|n| n.is_finite() && *n > 0.),
        };
    }
    SYSTEMS
        .binary_search_by_key(&id, |v| v.0)
        .ok()
        .map(|i| ProfileLimits {
            sliding_only: SYSTEMS[i].1,
            max_kg: SYSTEMS[i].2,
        })
        .unwrap_or_default()
}

pub(crate) fn hardware_leaves(
    frame: &Frame,
    cell: Option<&Cell>,
    cr: &CellRect,
) -> Vec<HardwareLeaf> {
    if !frame.material.is_profile_system() {
        return vec![];
    }
    let Some(o) = Opening::resolve(cell, cr.vulling.as_ref()) else {
        return vec![];
    };
    let manual = cell
        .and_then(|c| c.hardware_set.as_ref())
        .filter(|s| !s.auto_selected);
    let legacy = cell.is_some_and(|c| !c.hardware.is_empty() && c.hardware_set.is_none());
    let both = cr.second_fit.is_some();
    let frame_limits = profile_limits(frame, &frame.profile.id);
    let sash_limits = cell
        .and_then(|c| c.sash_profile.as_ref())
        .map(|p| profile_limits(frame, &p.id))
        .unwrap_or_default();
    let incompatible = !matches!(o.panel, PanelType::Sliding | PanelType::LiftSlide)
        && (frame_limits.sliding_only || sash_limits.sliding_only);
    let max_kg = frame_limits
        .max_kg
        .into_iter()
        .chain(sash_limits.max_kg)
        .reduce(f64::min);
    let security = cell
        .and_then(|c| c.hardware_set.as_ref())
        .map_or(SecurityClass::None, |s| s.security_class);
    [&cr.fit, &cr.second_fit]
        .into_iter()
        .enumerate()
        .filter_map(|(index, f)| {
            let fit = f.as_ref()?;
            let rect = fit.sash_outer.as_ref()?;
            let contours = fit.contours.as_ref();
            let leaf = contours.and_then(|c| c.sash_leaf).or_else(|| {
                o.pair.then_some(if index == 0 {
                    SashLeaf::Left
                } else {
                    SashLeaf::Right
                })
            });
            let right = leaf.map_or(o.right, |l| l == SashLeaf::Right);
            let passive = o.pair && right != o.right;
            let path = contours
                .and_then(|c| c.sash_outer.clone())
                .unwrap_or_else(|| rectangle(rect));
            let width = contours.map_or(fit.day.x - rect.x, |c| c.sash_width);
            let mass = leaf_mass_kg(frame, fit, &path, cell);
            let set = if legacy {
                None
            } else if let Some(s) = manual.filter(|_| !passive) {
                Some(s.clone())
            } else {
                auto_set(&o, right, frame, fit, &path, security, mass, passive)
            };
            let mut row = HardwareLeaf {
                leaf,
                panel_type: o.panel,
                opens_outward: o.outward,
                set,
                issue: None,
                components: vec![],
            };
            if !o.supported || incompatible {
                row.issue = Some(HardwareFitIssue::UnsupportedOpening);
            } else if row
                .set
                .as_ref()
                .is_some_and(|s| !s.supports_motion(o.panel))
            {
                row.issue = Some(HardwareFitIssue::IncompatibleSet);
            } else if contours.is_none() && frame.shape.shape_type != ShapeType::Rectangular {
                row.issue = Some(HardwareFitIssue::UnsupportedShape);
            } else if max_kg.is_some_and(|max| mass > max) {
                row.issue = Some(HardwareFitIssue::ProfileLoadExceeded);
            } else if o.pair
                && (!both
                    || row
                        .set
                        .as_ref()
                        .and_then(|s| s.hinges.as_ref())
                        .is_some_and(|h| {
                            h.side
                                != if right {
                                    HingeSide::Right
                                } else {
                                    HingeSide::Left
                                }
                        }))
            {
                row.issue = Some(HardwareFitIssue::InsufficientMountingEdge);
            } else if let Some(set) = &row.set {
                let rim = Rim {
                    path: &path,
                    bounds: path.bounds(),
                    width,
                };
                match fit_components(
                    &rim,
                    &o,
                    set,
                    passive,
                    leaf.map(|l| {
                        if l == SashLeaf::Left {
                            HingeSide::Right
                        } else {
                            HingeSide::Left
                        }
                    }),
                ) {
                    Some(parts) => {
                        if let Some(handle) = row
                            .set
                            .as_mut()
                            .filter(|s| s.auto_selected)
                            .and_then(|s| s.handle.as_mut())
                        {
                            if let Some(part) = parts.iter().find(|p| p.kind == FittingKind::Handle)
                            {
                                handle.height_mm =
                                    rim.bounds.y + rim.bounds.height - part.position[1];
                            }
                        }
                        row.components = parts;
                    }
                    None => row.issue = Some(HardwareFitIssue::InsufficientMountingEdge),
                }
            }
            Some(row)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{Hardware, HardwareType, Kozijn};
    use crate::layout::{self, VakChild, VakNode};

    fn model(material: Material, panel: PanelType, width: f64, height: f64) -> Kozijn {
        let template = crate::template::builtin_sjablonen()
            .into_iter()
            .find(|s| s.material == material)
            .unwrap();
        let mut k = Kozijn::new_with_sjabloon("Proef", "K", width, height, &template);
        k.change_cell_opening(0, panel, Some(OpeningDirection::Left))
            .unwrap();
        k
    }

    fn leaves(k: &Kozijn) -> Vec<HardwareLeaf> {
        compute_2d_geometry(k)
            .cell_rects
            .into_iter()
            .flat_map(|cr| cr.hardware_leaves)
            .collect()
    }

    fn parts(row: &HardwareLeaf, kind: FittingKind) -> Vec<&HardwareComponent> {
        row.components.iter().filter(|c| c.kind == kind).collect()
    }

    #[test]
    fn turn_tilt_mirrors_support_and_handle_and_counts_matching_keepers() {
        for material in [Material::Aluminum, Material::Pvc] {
            let mut k = model(material, PanelType::TurnTilt, 1100., 1350.);
            let row = leaves(&k).pop().unwrap();
            assert_eq!(row.issue, None);
            let first = parts(&row, FittingKind::Handle)[0];
            assert_eq!(first.side, HingeSide::Right);
            assert_eq!(parts(&row, FittingKind::Stay).len(), 1);
            assert!(parts(&row, FittingKind::Hinge).len() >= 2);
            let locks = row
                .set
                .as_ref()
                .unwrap()
                .locking
                .as_ref()
                .unwrap()
                .locking_points as usize;
            assert_eq!(parts(&row, FittingKind::LockingPoint).len(), locks);
            assert_eq!(parts(&row, FittingKind::Keeper).len(), locks);
            assert!(parts(&row, FittingKind::Keeper)
                .iter()
                .all(|c| c.mount == HardwareMount::Frame));
            for p in &row.components {
                assert!((p.tangent[0].hypot(p.tangent[1]) - 1.).abs() < EPS);
            }
            k.change_cell_opening(0, PanelType::TurnTilt, Some(OpeningDirection::Right))
                .unwrap();
            let mirrored = leaves(&k).pop().unwrap();
            let second = parts(&mirrored, FittingKind::Handle)[0];
            assert_eq!(second.side, HingeSide::Left);
            assert!((first.position[0] + second.position[0] - 1100.).abs() < EPS);
            assert!(parts(&mirrored, FittingKind::Hinge)
                .iter()
                .all(|p| p.side == HingeSide::Right));
        }
    }

    #[test]
    fn actual_glass_build_up_and_leaf_size_determine_bearing_set() {
        let mut k = model(Material::Pvc, PanelType::Turn, 1450., 1400.);
        k.cells[0].glazing = Glazing::default();
        assert_eq!(glazing_kg_m2(&k.cells[0].glazing), 20.);
        let before = leaves(&k)[0].set.as_ref().unwrap().hinges.clone().unwrap();
        k.cells[0].glazing.thickness_mm += 8.; // different spacer, same panes
        let unchanged = leaves(&k)[0].set.as_ref().unwrap().hinges.clone().unwrap();
        assert_eq!(before.load_capacity_kg, unchanged.load_capacity_kg);
        k.cells[0]
            .glazing
            .panes
            .iter_mut()
            .for_each(|p| p.thickness_mm = 12.);
        let heavy = leaves(&k)[0].set.as_ref().unwrap().hinges.clone().unwrap();
        assert!(
            heavy.load_capacity_kg > before.load_capacity_kg && heavy.count > before.count,
            "light {before:?}, heavy {heavy:?}"
        );
    }

    #[test]
    fn circle_requires_coaxial_pivots_and_arch_can_use_its_straight_jamb() {
        let mut k = model(Material::Aluminum, PanelType::Turn, 1250., 1250.);
        k.frame.shape.shape_type = ShapeType::Round;
        let row = leaves(&k).pop().unwrap();
        assert_eq!(row.issue, Some(HardwareFitIssue::InsufficientMountingEdge));
        assert!(row.components.is_empty());
        for (panel, constant_axis) in [(PanelType::Pivot, 0), (PanelType::BottomHung, 1)] {
            k.change_cell_opening(0, panel, None).unwrap();
            let row = leaves(&k).pop().unwrap();
            assert_eq!(row.issue, None);
            let pins = parts(&row, FittingKind::Pivot);
            assert_eq!(pins.len(), 2);
            assert!(
                (pins[0].position[constant_axis] - pins[1].position[constant_axis]).abs() < EPS
            );
            assert!(
                (pins[0].position[1 - constant_axis] - pins[1].position[1 - constant_axis]).abs()
                    > 800.
            );
            let handle = parts(&row, FittingKind::Handle)[0];
            assert!(pins.iter().all(|p| (p.position[0] - handle.position[0])
                .hypot(p.position[1] - handle.position[1])
                > 300.));
        }
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.change_cell_opening(0, PanelType::TurnTilt, None).unwrap();
        let row = leaves(&k).pop().unwrap();
        assert_eq!(row.issue, None);
        assert!(parts(&row, FittingKind::Hinge)
            .iter()
            .all(|p| p.side == HingeSide::Left && p.position[1] > 300.));
        assert!(parts(&row, FittingKind::LockingPoint)
            .iter()
            .any(|p| p.side == HingeSide::Top && p.tangent[1].abs() > 0.05));
        k.frame.shape.shape_type = ShapeType::Triangle;
        let row = leaves(&k).pop().unwrap();
        assert_eq!(row.issue, Some(HardwareFitIssue::UnsupportedShape));
        assert!(row.components.is_empty());
    }

    #[test]
    fn paired_doors_use_meeting_leaf_and_passive_bolts_and_mirror_together() {
        let mut k = model(Material::Pvc, PanelType::Door, 2000., 2250.);
        k.layout = Some(layout::deur("dubbel"));
        for right in [false, true] {
            let Some(VakNode::Leaf {
                vulling:
                    Vakvulling::Deur {
                        hinge_side,
                        opens_outward,
                        ..
                    },
                ..
            }) = k.layout.as_mut()
            else {
                panic!()
            };
            *hinge_side = Some(if right { "rechts" } else { "links" }.into());
            *opens_outward = Some(true);
            let rows = leaves(&k);
            assert_eq!(rows.len(), 2);
            assert!(rows.iter().all(|l| l.issue.is_none() && l.opens_outward));
            let active = &rows[usize::from(right)];
            let passive = &rows[usize::from(!right)];
            assert_eq!(parts(active, FittingKind::Handle).len(), 1);
            assert!(parts(active, FittingKind::Keeper)
                .iter()
                .all(|p| p.mount == HardwareMount::MeetingLeaf));
            assert!(parts(passive, FittingKind::Handle).is_empty());
            assert_eq!(parts(passive, FittingKind::FlushBolt).len(), 2);
            assert_eq!(parts(passive, FittingKind::Keeper).len(), 2);
            assert!(parts(passive, FittingKind::Keeper)
                .iter()
                .all(|p| p.mount == HardwareMount::Frame));
            assert!(parts(passive, FittingKind::Hinge).iter().all(|p| p.side
                == if right {
                    HingeSide::Left
                } else {
                    HingeSide::Right
                }));
        }
        k.layout = Some(layout::deur("vouw"));
        let row = leaves(&k).pop().unwrap();
        assert_eq!(row.issue, Some(HardwareFitIssue::UnsupportedOpening));
        assert!(row.components.is_empty());
    }

    #[test]
    fn tilt_and_top_hung_require_their_real_hinge_rail_and_follow_layout_defaults() {
        let mut k = model(Material::Aluminum, PanelType::TopHung, 1100., 1250.);
        k.cells[0].opening_direction = Some(OpeningDirection::Outward);
        for (mode, hinge, closing, outward) in [
            ("valraam", HingeSide::Bottom, HingeSide::Top, false),
            ("uitzet", HingeSide::Top, HingeSide::Bottom, true),
        ] {
            k.layout = Some(layout::raam(mode));
            let row = leaves(&k).pop().unwrap();
            assert_eq!(row.opens_outward, outward);
            assert_eq!(row.issue, None);
            assert_eq!(parts(&row, FittingKind::Stay).len(), 2);
            assert!(parts(&row, FittingKind::Hinge)
                .iter()
                .all(|p| p.side == hinge));
            let handle = parts(&row, FittingKind::Handle)[0];
            assert_eq!(handle.side, closing);
            let sash = compute_2d_geometry(&k)
                .cell_rects
                .remove(0)
                .fit
                .unwrap()
                .sash_outer
                .unwrap();
            assert!(
                (sash.y + sash.height
                    - handle.position[1]
                    - row.set.as_ref().unwrap().handle.as_ref().unwrap().height_mm)
                    .abs()
                    < EPS
            );
        }
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        for mode in ["uitzet", "schuif"] {
            k.layout = Some(layout::raam(mode));
            let row = leaves(&k).pop().unwrap();
            assert_eq!(row.issue, Some(HardwareFitIssue::InsufficientMountingEdge));
            assert!(row.components.is_empty());
        }
        k.layout = Some(layout::paneel());
        assert!(leaves(&k).is_empty());
        k.layout = Some(layout::glas());
        assert!(leaves(&k).is_empty());
    }

    #[test]
    fn hand_selected_set_persists_through_security_and_operation_changes() {
        let mut k = model(Material::Aluminum, PanelType::Turn, 1250., 1400.);
        let mut custom = k.cells[0].hardware_set.clone().unwrap();
        custom.hinges.as_mut().unwrap().count = 4;
        custom.handle.as_mut().unwrap().height_mm = 720.;
        custom.locking = None;
        k.set_manual_hardware(0, custom).unwrap();
        k.set_hardware_security(0, SecurityClass::RC3).unwrap();
        k.change_cell_opening(0, PanelType::FixedGlass, None)
            .unwrap();
        assert!(leaves(&k).is_empty());
        k.change_cell_opening(0, PanelType::TurnTilt, None).unwrap();
        let row = leaves(&k).pop().unwrap();
        assert_eq!(parts(&row, FittingKind::Hinge).len(), 4);
        assert_eq!(parts(&row, FittingKind::Stay).len(), 1);
        assert!(parts(&row, FittingKind::Lock).is_empty());
        let s = row.set.unwrap();
        assert_eq!(s.handle.unwrap().height_mm, 720.);
        assert_eq!(s.security_class, SecurityClass::RC3);
        assert!(!s.auto_selected);
        k.layout = Some(layout::raam("draai"));
        let Some(VakNode::Leaf {
            vulling: Vakvulling::Raam { hardware, .. },
            ..
        }) = k.layout.as_mut()
        else {
            panic!()
        };
        *hardware = Some(false);
        assert!(leaves(&k).is_empty());
        k.select_hardware(0, None).unwrap();
        assert!(!k.cells[0].hardware_set.as_ref().unwrap().auto_selected);
        k.layout = Some(layout::split_row(vec![
            VakChild {
                size: 1.,
                node: layout::buiten(),
            },
            VakChild {
                size: 1.,
                node: layout::raam("draai"),
            },
        ]));
        assert!(k.set_hardware_security(0, SecurityClass::RC1).is_err());
        assert_eq!(leaves(&k).len(), 1);
        assert!(leaves(&k)[0].set.as_ref().unwrap().auto_selected);
    }

    #[test]
    fn changing_handle_height_preserves_motion_supports() {
        for (panel, kind, quantity) in [
            (PanelType::TurnTilt, FittingKind::Stay, 1),
            (PanelType::TopHung, FittingKind::Stay, 2),
            (PanelType::Tilt, FittingKind::Stay, 2),
            (PanelType::Sliding, FittingKind::Roller, 2),
            (PanelType::LiftSlide, FittingKind::Roller, 2),
        ] {
            let mut k = model(Material::Aluminum, panel, 1300., 1500.);
            let auto = leaves(&k).pop().unwrap();
            let positions = |leaf: &HardwareLeaf| {
                parts(leaf, kind)
                    .into_iter()
                    .map(|p| p.position)
                    .collect::<Vec<_>>()
            };
            let original = positions(&auto);
            assert_eq!(original.len(), quantity);
            let mut set = auto.set.unwrap();
            set.handle.as_mut().unwrap().height_mm = 650.;
            k.set_manual_hardware(0, set).unwrap();
            let manual = leaves(&k).pop().unwrap();
            assert_eq!(manual.issue, None);
            assert_eq!(positions(&manual), original);
            assert_eq!(manual.set.unwrap().handle.unwrap().height_mm, 650.);
        }
    }

    #[test]
    fn hand_selected_set_mirrors_with_handing_and_is_retained_on_incompatible_motion() {
        let mut k = model(Material::Pvc, PanelType::TurnTilt, 1260., 1390.);
        let mut set = leaves(&k).pop().unwrap().set.unwrap();
        set.handle.as_mut().unwrap().height_mm = 620.;
        set.hinges.as_mut().unwrap().count = 4;
        k.set_manual_hardware(0, set).unwrap();
        let left = leaves(&k).pop().unwrap();
        let first = parts(&left, FittingKind::Handle)[0].position;
        k.change_cell_opening(0, PanelType::TurnTilt, Some(OpeningDirection::Right))
            .unwrap();
        let right = leaves(&k).pop().unwrap();
        assert_eq!(right.issue, None);
        assert_eq!(parts(&right, FittingKind::Hinge).len(), 4);
        assert!(parts(&right, FittingKind::Hinge)
            .iter()
            .all(|p| p.side == HingeSide::Right));
        assert_eq!(parts(&right, FittingKind::Stay)[0].side, HingeSide::Right);
        let handle = parts(&right, FittingKind::Handle)[0];
        assert_eq!(handle.side, HingeSide::Left);
        assert!(
            (first[0] + handle.position[0] - 1260.).abs() < EPS
                && (first[1] - handle.position[1]).abs() < EPS
        );
        let stored = serde_json::to_value(&k.cells[0].hardware_set).unwrap();
        k.change_cell_opening(0, PanelType::Sliding, Some(OpeningDirection::Left))
            .unwrap();
        let incompatible = leaves(&k).pop().unwrap();
        assert_eq!(incompatible.issue, Some(HardwareFitIssue::IncompatibleSet));
        assert!(incompatible.components.is_empty());
        assert_eq!(
            serde_json::to_value(&k.cells[0].hardware_set).unwrap(),
            stored
        );
        k.select_hardware(0, None).unwrap();
        let rolling = leaves(&k).pop().unwrap();
        assert_eq!(rolling.issue, None);
        assert_eq!(parts(&rolling, FittingKind::Roller).len(), 2);
        assert!(parts(&rolling, FittingKind::Hinge).is_empty());
    }

    #[test]
    fn sliding_rollers_explicit_legacy_parts_and_security_round_trip() {
        let mut k = model(Material::Pvc, PanelType::LiftSlide, 1750., 2100.);
        let row = leaves(&k).pop().unwrap();
        assert!(parts(&row, FittingKind::Hinge).is_empty());
        assert_eq!(parts(&row, FittingKind::Roller).len(), 2);
        assert_eq!(
            row.set.unwrap().handle.unwrap().handle_type,
            HandleType::TGreep
        );
        k.cells[0].hardware_set = None;
        k.cells[0].hardware.push(Hardware {
            hardware_type: HardwareType::Handle,
            position: "rechts".into(),
            brand: None,
            model: None,
        });
        k.change_cell_opening(0, PanelType::Turn, None).unwrap();
        let row = leaves(&k).pop().unwrap();
        assert!(row.set.is_none() && row.components.is_empty());
        assert!(k.set_hardware_security(0, SecurityClass::RC2).is_err());
        k.select_hardware(0, Some(SecurityClass::RC2)).unwrap();
        assert!(leaves(&k)[0].set.as_ref().unwrap().auto_selected);
        for i in 1..=6 {
            let level = format!("rc{i}");
            for spelling in [level.clone(), format!("r_c{i}"), format!("RC{i}")] {
                let decoded: SecurityClass =
                    serde_json::from_value(serde_json::Value::String(spelling)).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), level);
            }
        }
    }

    #[test]
    fn dedicated_sliding_series_only_select_rolling_gear() {
        for (material, series) in [
            (Material::Pvc, "veka-ekipazz-sliding"),
            (Material::Aluminum, "reynaers-cp155-sliding"),
            (Material::Aluminum, "schuco-ass77pd-sliding"),
        ] {
            let mut k = model(material, PanelType::Turn, 1350., 1650.);
            k.frame.profile.id = series.into();
            let rejected = leaves(&k).pop().unwrap();
            assert_eq!(rejected.issue, Some(HardwareFitIssue::UnsupportedOpening));
            assert!(rejected.components.is_empty());
            k.change_cell_opening(0, PanelType::Sliding, None).unwrap();
            let accepted = leaves(&k).pop().unwrap();
            assert_eq!(accepted.issue, None);
            assert_eq!(parts(&accepted, FittingKind::Roller).len(), 2);
            let mut oversized = model(material, PanelType::LiftSlide, 6200., 4200.);
            oversized.frame.profile.id = series.into();
            let rejected = leaves(&oversized).pop().unwrap();
            assert_eq!(rejected.issue, Some(HardwareFitIssue::ProfileLoadExceeded));
            assert!(rejected.components.is_empty());
        }
    }
}
