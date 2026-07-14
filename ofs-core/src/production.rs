use serde::{Deserialize, Serialize};

use crate::joint::{effective_corner_joints, Joint, JointType, ThroughMember};
use crate::kozijn::{Kozijn, Material, PanelType};
use crate::layout::Vakvulling;

// ── Cut list constants ─────────────────────────────────────────────

const WOOD_PEN_ALLOWANCE_MM: f64 = 20.0;   // tenon allowance per side
const SAW_KERF_MM: f64 = 4.0;              // saw blade width
const PVC_WELD_OVERMEASURE_MM: f64 = 4.0;  // weld overmeasure per side
const GLASS_CLEARANCE_MM: f64 = 4.0;       // legacy glass/panel clearance per side (kozijnen zonder profielsnapshot; panelen altijd)
const GLASS_EDGE_CLEARANCE_MM: f64 = 5.0;  // omtrekspeling per zijde in de sponning (NPR 3577-minimum, steldikte; R2)
const GASKET_OVERLAP_MM: f64 = 20.0;       // gasket overlap at corners

// ── Data structures ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionData {
    pub kozijn_mark: String,
    pub kozijn_name: String,
    pub cut_list: Vec<CutListItem>,
    pub glass_list: Vec<GlassListItem>,
    pub hardware_list: Vec<HardwareListItem>,
    pub gasket_list: Vec<GasketListItem>,
    pub panel_list: Vec<PanelListItem>,
    pub glaslat_list: Vec<GlaslatListItem>,
    pub bom: Vec<BomItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberType {
    FrameTop,
    FrameBottom,
    FrameLeft,
    FrameRight,
    DividerH,
    DividerV,
    SashTop,
    SashBottom,
    SashLeft,
    SashRight,
}

impl MemberType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::FrameTop => "Bovendorpel",
            Self::FrameBottom => "Onderdorpel",
            Self::FrameLeft => "Stijl links",
            Self::FrameRight => "Stijl rechts",
            Self::DividerH => "Tussendorpel",
            Self::DividerV => "Tussenstijl",
            Self::SashTop => "Raamhout boven",
            Self::SashBottom => "Raamhout onder",
            Self::SashLeft => "Raamhout links",
            Self::SashRight => "Raamhout rechts",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CutListItem {
    pub piece_id: String,
    pub member_type: MemberType,
    pub profile_name: String,
    pub material: String,
    pub net_length_mm: f64,
    pub gross_length_mm: f64,
    pub miter_left_deg: f64,
    pub miter_right_deg: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlassListItem {
    pub piece_id: String,
    pub cell_index: usize,
    pub glass_type: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub thickness_mm: f64,
    pub ug_value: f64,
    pub area_m2: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareListItem {
    pub cell_index: usize,
    pub component: String,
    pub description: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GasketType {
    GlazingInner,
    GlazingOuter,
    SashSeal,
    FrameSeal,
}

impl GasketType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::GlazingInner => "Binnenrubber beglazing",
            Self::GlazingOuter => "Buitenrubber beglazing",
            Self::SashSeal => "Vleugelafdichting",
            Self::FrameSeal => "Kozijnafdichting",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasketListItem {
    pub gasket_type: GasketType,
    pub length_mm: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelListItem {
    pub piece_id: String,
    pub cell_index: usize,
    pub width_mm: f64,
    pub height_mm: f64,
    pub panel_type: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlaslatListItem {
    pub piece_id: String,
    pub cell_index: usize,
    pub position: String, // Binnen / Buiten
    pub material: String,
    pub width_mm: f64,
    pub height_mm: f64,
    /// Combined cut length of the four beads around the glazing rebate (mm).
    pub total_length_mm: f64,
    pub mitered: bool,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BomItem {
    pub category: String,
    pub description: String,
    pub unit: String,
    pub quantity: f64,
}

// ── Length calculation helpers ──────────────────────────────────────

fn gross_length(net: f64, material: &Material, miter: bool) -> f64 {
    match material {
        Material::Wood(_) => {
            if miter {
                net + 2.0 * SAW_KERF_MM
            } else {
                net + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM
            }
        }
        Material::Aluminum => {
            // Miter at 45 degrees: gross = net (longest edge stays the same)
            // but add saw kerf
            net + 2.0 * SAW_KERF_MM
        }
        Material::Pvc => {
            net + 2.0 * PVC_WELD_OVERMEASURE_MM + SAW_KERF_MM
        }
        Material::WoodAluminum => {
            // Wood rules for the wood part
            net + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM
        }
    }
}

fn miter_angle(material: &Material) -> f64 {
    match material {
        Material::Aluminum | Material::Pvc => 45.0,
        _ => 90.0,
    }
}

/// Gross length of a mitered (verstek) alu/PVC member: the net length is the
/// short (inner) edge, but the physical piece runs to the LONG point — `fw`
/// (the face width) per end — plus the saw kerf; PVC adds the weld overmeasure
/// per side. Same systematics as the configured-joint path
/// (`joint_end_allowance`), so identical kozijnen get identical gross lengths
/// whether or not the default joints were ever touched. Wood paths never call
/// this — their historic `gross_length` numbers stay byte-identical.
fn miter_gross_length(net: f64, material: &Material, fw: f64) -> f64 {
    let weld = match material {
        Material::Pvc => PVC_WELD_OVERMEASURE_MM,
        _ => 0.0,
    };
    net + SAW_KERF_MM + 2.0 * (fw.max(0.0) + weld)
}

/// Glasmaat voor één dagmaat, per R2 (KVT 12.2 / NPR 3577-uitleg, zie
/// docs/profielmaten-onderzoek.md §5): met profielsnapshot valt het glas in de
/// sponning — glasmaat = sponningmaat − 2 × omtrekspeling (5 mm per zijde,
/// NPR-minimum). Kozijnen zonder snapshot houden de historische dagmaat −
/// 2 × GLASS_CLEARANCE_MM, zodat oude projecten identiek blijven. Gedeeld
/// door het grid-pad en het layout-pad zodat beide dezelfde glasmaat leveren.
fn glass_size_for_day(
    day_w: f64,
    day_h: f64,
    snapshot: Option<&crate::profile::ProfileSnapshot>,
    material: &Material,
) -> (f64, f64) {
    match snapshot {
        Some(snap) => {
            let sh = snap.resolved_sponning_hoogte(material);
            let oversize = 2.0 * sh - 2.0 * GLASS_EDGE_CLEARANCE_MM;
            ((day_w + oversize).max(0.0), (day_h + oversize).max(0.0))
        }
        None => (
            day_w - 2.0 * GLASS_CLEARANCE_MM,
            day_h - 2.0 * GLASS_CLEARANCE_MM,
        ),
    }
}

/// Glaslat-zaaglengte (som van de vier latten) voor één dagmaat, per KVT
/// 12.3.2: met profielsnapshot is de sponningmaat = dagmaat + 2 ×
/// sponninghoogte; horizontale latten lopen door (sponningmaat − 1),
/// verticale latten passen ertussen (sponningmaat − 2 × lathoogte − 1,
/// binnenbeglazing); in verstek (mitered) lopen alle vier tot de hoek
/// (sponningmaat − 1). Kozijnen zonder snapshot houden de historische
/// dagmaat-omtrek, zodat oude projecten identiek blijven. Gedeeld door het
/// grid-pad en het layout-pad zodat beide dezelfde latlengtes leveren.
fn glaslat_total_length(
    day_w: f64,
    day_h: f64,
    snapshot: Option<&crate::profile::ProfileSnapshot>,
    material: &Material,
    gl: &crate::glaslat::Glaslat,
) -> f64 {
    match snapshot {
        Some(snap) => {
            let sh = snap.resolved_sponning_hoogte(material);
            let sponningmaat_w = day_w.max(0.0) + 2.0 * sh;
            let sponningmaat_h = day_h.max(0.0) + 2.0 * sh;
            let horizontaal = (sponningmaat_w - 1.0).max(0.0);
            let verticaal = if gl.mitered {
                (sponningmaat_h - 1.0).max(0.0)
            } else {
                (sponningmaat_h - 2.0 * gl.height_mm - 1.0).max(0.0)
            };
            2.0 * horizontaal + 2.0 * verticaal
        }
        None => 2.0 * (day_w.max(0.0) + day_h.max(0.0)),
    }
}

/// Vleugelleden (raamhout/deurhout) voor één vak op het layout-pad — dezelfde
/// systematiek als het grid-pad: bij hout lopen de VLEUGELstijlen door en
/// passen de dorpels ertussen (omgekeerd aan het kozijn, zie `crate::joint`),
/// bij alu/PVC zijn alle vier de leden in verstek. `id_prefix` is "R"
/// (raamhout) of "D" (deurhout), zodat de stuk-ids `-RSL`/`-DSL` etc. worden.
#[allow(clippy::too_many_arguments)]
fn push_layout_sash_members(
    cut_list: &mut Vec<CutListItem>,
    cell_id: &str,
    id_prefix: &str,
    sash_profile_name: &str,
    material: &Material,
    material_name: &str,
    is_miter: bool,
    angle: f64,
    vak_w: f64,
    vak_h: f64,
    sash_w: f64,
) {
    let (stile_net, rail_net) = if is_miter {
        (vak_h - 2.0 * sash_w, vak_w - 2.0 * sash_w)
    } else {
        (vak_h, vak_w - 2.0 * sash_w)
    };
    let members = [
        ("SL", MemberType::SashLeft, stile_net),
        ("SR", MemberType::SashRight, stile_net),
        ("DB", MemberType::SashTop, rail_net),
        ("DO", MemberType::SashBottom, rail_net),
    ];
    for (suffix, member_type, net) in members {
        // Alu/PVC verstek-vleugels: gross runs to the long point (sash face
        // width per end) — same systematics as the frame members.
        let gross = if is_miter {
            miter_gross_length(net, material, sash_w)
        } else {
            gross_length(net, material, is_miter)
        };
        cut_list.push(CutListItem {
            piece_id: format!("{}-{}{}", cell_id, id_prefix, suffix),
            member_type,
            profile_name: sash_profile_name.to_string(),
            material: material_name.to_string(),
            net_length_mm: net,
            gross_length_mm: gross,
            miter_left_deg: angle,
            miter_right_deg: angle,
            quantity: 1,
        });
    }
}

// ── Corner-joint driven frame member cuts ───────────────────────────
//
// `kozijn.frame.corner_joints` holds per-corner joint configurations in the
// order [top-left, top-right, bottom-left, bottom-right] (see kozijn.rs).
// When the user has configured them, the saw list is derived from those
// joints. When they are absent, incomplete, or still the untouched
// auto-populated default set, we fall back to the historic material-based
// behaviour so existing projects keep producing identical saw lists.

/// Which way a frame member runs; decides who is the "through" member at a corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameOrientation {
    /// Vertical member (stijl)
    Stijl,
    /// Horizontal member (dorpel)
    Dorpel,
}

/// Resolved saw-list numbers for one frame member.
struct MemberCut {
    net: f64,
    gross: f64,
    /// Cut angle at the first end (top end for stijlen, left end for dorpels).
    angle_start: f64,
    /// Cut angle at the second end (bottom end for stijlen, right end for dorpels).
    angle_end: f64,
}

/// How much a member's net length is reduced at one end by the joint there.
fn joint_end_reduction(orientation: FrameOrientation, joint: &Joint, fw: f64) -> f64 {
    match joint.joint_type {
        // Miter: both members stop at the diagonal; net = short (inner) edge.
        JointType::Verstek => fw,
        // Pen/slis, contramal, stomp: the through member runs to the outer
        // edge, the other member fits against/into it.
        _ => {
            let is_through = matches!(
                (orientation, joint.through_member),
                (FrameOrientation::Stijl, ThroughMember::Stijl)
                    | (FrameOrientation::Dorpel, ThroughMember::Dorpel)
            );
            if is_through {
                0.0
            } else {
                fw
            }
        }
    }
}

/// Extra gross-length allowance at one end (tenon length, miter long point,
/// weld overmeasure). `fw` is the frame member face width at this corner.
fn joint_end_allowance(
    orientation: FrameOrientation,
    joint: &Joint,
    material: &Material,
    fw: f64,
) -> f64 {
    match joint.joint_type {
        JointType::PenSlis => {
            let is_through = matches!(
                (orientation, joint.through_member),
                (FrameOrientation::Stijl, ThroughMember::Stijl)
                    | (FrameOrientation::Dorpel, ThroughMember::Dorpel)
            );
            // The pen (tenon) sits on the non-through member; the through
            // member only receives the slis (mortise) and needs no extra length.
            if is_through {
                0.0
            } else {
                joint.pen_length.max(0.0)
            }
        }
        // Verstek: netmaat = korte zijde (binnenmaat), maar het fysieke stuk
        // loopt door tot de LANGE punt (buitenmaat) — werkplaats-conventie is
        // meten over de lange punt, dus per uiteinde fw erbij zodat
        // bruto ≥ lange punt + zaagsnede. PVC krijgt daarbovenop de
        // lasovermaat per zijde.
        JointType::Verstek => {
            fw.max(0.0)
                + match material {
                    Material::Pvc => PVC_WELD_OVERMEASURE_MM,
                    _ => 0.0,
                }
        }
        JointType::Contramal | JointType::Stomp => 0.0,
    }
}

/// Saw angle at one end. Verstek uses the configured angle (45 deg when the
/// stored angle is missing/implausible); all other joints are square cuts.
fn joint_end_angle(joint: &Joint) -> f64 {
    match joint.joint_type {
        JointType::Verstek => {
            if joint.angle > 0.0 && joint.angle < 90.0 {
                joint.angle
            } else {
                45.0
            }
        }
        _ => 90.0,
    }
}

/// Compute net/gross length and end angles for one outer-frame member.
///
/// `ends` is `Some((start, end))` with the corner joints at both ends of the
/// member (top/bottom for stijlen, left/right for dorpels). `None` selects the
/// historic material-based fallback, which must stay byte-identical to the old
/// behaviour for regression safety.
fn frame_member_cut(
    orientation: FrameOrientation,
    outer_span: f64,
    ends: Option<(&Joint, &Joint)>,
    material: &Material,
    fw: f64,
    default_is_miter: bool,
    default_angle: f64,
) -> MemberCut {
    match ends {
        Some((start, end)) => {
            let net = outer_span
                - joint_end_reduction(orientation, start, fw)
                - joint_end_reduction(orientation, end, fw);
            let gross = net
                + SAW_KERF_MM
                + joint_end_allowance(orientation, start, material, fw)
                + joint_end_allowance(orientation, end, material, fw);
            MemberCut {
                net,
                gross,
                angle_start: joint_end_angle(start),
                angle_end: joint_end_angle(end),
            }
        }
        None => {
            // Material defaults: mitered members (alu/PVC) lose fw at both
            // ends; for wood the dorpels run the full width and the stijlen
            // fit between them (NL-timmerpraktijk, KVT katern 15: dorpels
            // doorlopend, stijlen ertussen gepend — mirrors
            // `joint::frame_corner_model` so the drawing and saw list agree).
            let net = match (orientation, default_is_miter) {
                (FrameOrientation::Dorpel, false) => outer_span,
                _ => outer_span - 2.0 * fw,
            };
            // Verstek gross runs to the long point (`miter_gross_length`) —
            // the same systematics as the configured-joint path, so identical
            // kozijnen get identical gross lengths either way. Wood keeps the
            // historic `gross_length` numbers byte-identical.
            let gross = if default_is_miter {
                miter_gross_length(net, material, fw)
            } else {
                gross_length(net, material, default_is_miter)
            };
            MemberCut {
                net,
                gross,
                angle_start: default_angle,
                angle_end: default_angle,
            }
        }
    }
}

/// Stepped outer-frame spans for a melkmeisje (free-subdivision layout with
/// `Buiten` leaves): the onderdorpel only spans the vakken that actually reach
/// the bottom of the opening, and an outer stijl only spans the vakken that
/// reach that side — so a full-width bottom sill (or full-height side stijl)
/// isn't cut where the outline steps into the wall (issue 10).
///
/// Returns `(left_span, right_span, top_span, bottom_span)` as *outer* member
/// spans, so `frame_member_cut` still subtracts the corner reductions exactly
/// as before. Without a layout — or without any `Buiten` leaf — it returns the
/// full outer dimensions, so every non-melkmeisje kozijn stays byte-identical.
fn stepped_frame_spans(kozijn: &Kozijn, fw: f64) -> (f64, f64, f64, f64) {
    let ow = kozijn.frame.outer_width;
    let oh = kozijn.frame.outer_height;
    let full = (oh, oh, ow, ow);
    let node = match &kozijn.layout {
        Some(n) => n,
        None => return full,
    };
    let lg = crate::layout::compute_layout_geometry(
        node,
        crate::layout::LayoutRect { x: fw, y: fw, width: ow - 2.0 * fw, height: oh - 2.0 * fw },
        fw,
    );
    if !lg.leaves.iter().any(|l| l.vulling.is_buiten()) {
        return full;
    }

    const EPS: f64 = 1.0;
    let inner_left = fw;
    let inner_right = ow - fw;
    let inner_top = fw;
    let inner_bottom = oh - fw;

    // Covered extent along each edge from the real (non-`Buiten`) leaves that
    // touch it. `None` = nothing reaches this edge → keep the full span.
    let mut bottom: Option<(f64, f64)> = None;
    let mut top: Option<(f64, f64)> = None;
    let mut left: Option<(f64, f64)> = None;
    let mut right: Option<(f64, f64)> = None;
    let extend = |acc: &mut Option<(f64, f64)>, a: f64, b: f64| {
        *acc = Some(match *acc {
            Some((lo, hi)) => (lo.min(a), hi.max(b)),
            None => (a, b),
        });
    };
    for l in &lg.leaves {
        if l.vulling.is_buiten() {
            continue;
        }
        let r = l.rect;
        if (r.y + r.height - inner_bottom).abs() < EPS {
            extend(&mut bottom, r.x, r.x + r.width);
        }
        if (r.y - inner_top).abs() < EPS {
            extend(&mut top, r.x, r.x + r.width);
        }
        if (r.x - inner_left).abs() < EPS {
            extend(&mut left, r.y, r.y + r.height);
        }
        if (r.x + r.width - inner_right).abs() < EPS {
            extend(&mut right, r.y, r.y + r.height);
        }
    }
    // Covered clear extent + the two corner members = outer member span.
    let span = |acc: Option<(f64, f64)>, full: f64| acc.map_or(full, |(lo, hi)| (hi - lo) + 2.0 * fw);
    (
        span(left, oh),
        span(right, oh),
        span(top, ow),
        span(bottom, ow),
    )
}

/// Intersection of two interval unions — the stretches covered by BOTH sets.
/// Used to find divider stretches that border the wall (`Buiten`) on both
/// sides; only those fall out of the saw list (one-sided stretches keep
/// running — the member is longer than the drawn, clipped divider).
fn intersect_interval_unions(a: &[(f64, f64)], b: &[(f64, f64)], eps: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for &(a0, a1) in a {
        for &(b0, b1) in b {
            let lo = a0.max(b0);
            let hi = a1.min(b1);
            if hi > lo + eps {
                out.push((lo, hi));
            }
        }
    }
    out
}

fn material_name(material: &Material) -> &'static str {
    match material {
        Material::Wood(w) => match w {
            crate::kozijn::WoodType::Meranti => "Meranti",
            crate::kozijn::WoodType::Accoya => "Accoya",
            crate::kozijn::WoodType::Vuren => "Vuren",
            crate::kozijn::WoodType::Eiken => "Eiken",
        },
        Material::Aluminum => "Aluminium",
        Material::Pvc => "Kunststof",
        Material::WoodAluminum => "Hout-aluminium",
    }
}

// ── Main computation ───────────────────────────────────────────────

pub fn compute_production_data(kozijn: &Kozijn) -> ProductionData {
    let mark = &kozijn.mark;
    let mat = &kozijn.frame.material;
    let fw = kozijn.frame.frame_width;
    let is_miter = matches!(mat, Material::Aluminum | Material::Pvc);
    let angle = miter_angle(mat);
    let mat_name = material_name(mat).to_string();
    let profile_name = format!("{}x{}", kozijn.frame.frame_width, kozijn.frame.frame_depth);

    let inner_w = kozijn.inner_width();
    let inner_h = kozijn.inner_height();

    let mut cut_list = Vec::new();
    let mut glass_list = Vec::new();
    let mut hardware_list = Vec::new();
    let mut gasket_list = Vec::new();
    let mut panel_list = Vec::new();
    let mut glaslat_list = Vec::new();

    // ── Frame members ──────────────────────────────────────────

    // Corner joints in [top-left, top-right, bottom-left, bottom-right] order,
    // or None → historic material-based fallback (wood: stiles full height,
    // rails between; alu/PVC: all mitered at 45°). Shared with the front-view
    // drawing via `crate::joint` so the picture and this cut list agree.
    let corner_joints =
        effective_corner_joints(&kozijn.frame.corner_joints, kozijn.frame.joints_configured);

    // Melkmeisje (layout with `Buiten` leaves): the onderdorpel / outer stijl
    // spans step to follow the layout; full outer dims for every other kozijn.
    let (left_span, right_span, top_span, bottom_span) = stepped_frame_spans(kozijn, fw);

    let left_stile = frame_member_cut(
        FrameOrientation::Stijl,
        left_span,
        corner_joints.map(|j| (j[0], j[2])), // top-left, bottom-left
        mat, fw, is_miter, angle,
    );
    let right_stile = frame_member_cut(
        FrameOrientation::Stijl,
        right_span,
        corner_joints.map(|j| (j[1], j[3])), // top-right, bottom-right
        mat, fw, is_miter, angle,
    );
    let top_rail = frame_member_cut(
        FrameOrientation::Dorpel,
        top_span,
        corner_joints.map(|j| (j[0], j[1])), // top-left, top-right
        mat, fw, is_miter, angle,
    );
    let bottom_rail = frame_member_cut(
        FrameOrientation::Dorpel,
        bottom_span,
        corner_joints.map(|j| (j[2], j[3])), // bottom-left, bottom-right
        mat, fw, is_miter, angle,
    );

    // Left stile (angles: left = top end, right = bottom end)
    cut_list.push(CutListItem {
        piece_id: format!("{}-SL", mark),
        member_type: MemberType::FrameLeft,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: left_stile.net,
        gross_length_mm: left_stile.gross,
        miter_left_deg: left_stile.angle_start,
        miter_right_deg: left_stile.angle_end,
        quantity: 1,
    });

    // Right stile (angles: left = top end, right = bottom end)
    cut_list.push(CutListItem {
        piece_id: format!("{}-SR", mark),
        member_type: MemberType::FrameRight,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: right_stile.net,
        gross_length_mm: right_stile.gross,
        miter_left_deg: right_stile.angle_start,
        miter_right_deg: right_stile.angle_end,
        quantity: 1,
    });

    // Top rail
    cut_list.push(CutListItem {
        piece_id: format!("{}-DB", mark),
        member_type: MemberType::FrameTop,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: top_rail.net,
        gross_length_mm: top_rail.gross,
        miter_left_deg: top_rail.angle_start,
        miter_right_deg: top_rail.angle_end,
        quantity: 1,
    });

    // Bottom rail / sill
    let sill_profile = if kozijn.frame.sill_profile.is_some() {
        format!("Dorpel {}", profile_name)
    } else {
        profile_name.clone()
    };
    cut_list.push(CutListItem {
        piece_id: format!("{}-DO", mark),
        member_type: MemberType::FrameBottom,
        profile_name: sill_profile,
        material: mat_name.clone(),
        net_length_mm: bottom_rail.net,
        gross_length_mm: bottom_rail.gross,
        miter_left_deg: bottom_rail.angle_start,
        miter_right_deg: bottom_rail.angle_end,
        quantity: 1,
    });

    // ── Dividers + vakken ──────────────────────────────────────
    //
    // With a free-subdivision layout (`kozijn.layout`) the tussenstijlen/
    // tussendorpels and the vak contents come from the SAME layout geometry
    // the drawing uses (`compute_layout_geometry`), so the saw list matches
    // the picture. Without a layout the historic grid path in the `else`
    // branch runs byte-identical (regression-tested).
    if let Some(node) = kozijn.layout.as_ref() {
        // Raw (unclipped) layout geometry: the drawing clips its dividers
        // against `Buiten` (a mullion visually stops at the melkmeisje step),
        // but that clip is for the PICTURE — production resolves its own
        // member lengths below. Leaves are identical in both variants, so the
        // vak contents still match the drawing.
        let lg = crate::layout::compute_layout_geometry_unclipped(
            node,
            crate::layout::LayoutRect { x: fw, y: fw, width: inner_w, height: inner_h },
            fw,
        );

        // Dividers: verticaal → tussenstijl (net = segmenthoogte), horizontaal
        // → tussendorpel (net = segmentbreedte); bruto volgens dezelfde
        // toeslagsystematiek als het grid-pad (altijd stomp ingekort), met het
        // kozijnprofiel als fallback-profielnaam. Rond `Buiten` (melkmeisje):
        // een stuk vervalt alleen waar de muur aan BEIDE zijden grenst; grenst
        // hij aan één zijde dan loopt het lid gewoon door tot de dorpel/stijl.
        // Een tussendorpel-stuk óp de muur (de zijlicht-onderdorpel op de
        // borstwering) is dus een echt lid: dorpel-doorlopend, verlengd met fw
        // aan elke laterale kant waar de buitenstijl door de sprong is
        // weggevallen — de stijl erboven stopt óp deze borstweringdorpel.
        // Zonder `Buiten` leaves is dit byte-identiek aan de volle dividers.
        const EPS: f64 = 1.0;
        let buiten_rects: Vec<crate::layout::LayoutRect> = lg
            .leaves
            .iter()
            .filter(|l| l.vulling.is_buiten())
            .map(|l| l.rect)
            .collect();
        let (mut ts, mut td) = (0usize, 0usize);
        for d in &lg.dividers {
            let r = d.rect;
            if d.direction == "v" {
                // Tussenstijl: only stretches walled in on BOTH sides drop.
                let (lo, hi) = (r.y, r.y + r.height);
                let mut left_blocked: Vec<(f64, f64)> = Vec::new();
                let mut right_blocked: Vec<(f64, f64)> = Vec::new();
                for b in &buiten_rects {
                    let iv = (b.y.max(lo), (b.y + b.height).min(hi));
                    if (b.x + b.width - r.x).abs() < EPS {
                        left_blocked.push(iv);
                    }
                    if (b.x - (r.x + r.width)).abs() < EPS {
                        right_blocked.push(iv);
                    }
                }
                let walled = intersect_interval_unions(&left_blocked, &right_blocked, EPS);
                for (a, c) in crate::layout::subtract_intervals(lo, hi, &walled, EPS) {
                    let piece_id = format!("{}-TS{}", mark, ts);
                    ts += 1;
                    let div_net = c - a;
                    cut_list.push(CutListItem {
                        piece_id,
                        member_type: MemberType::DividerV,
                        profile_name: profile_name.clone(),
                        material: mat_name.clone(),
                        net_length_mm: div_net,
                        gross_length_mm: gross_length(div_net, mat, false), // dividers always butt-joined
                        miter_left_deg: 90.0,
                        miter_right_deg: 90.0,
                        quantity: 1,
                    });
                }
            } else {
                // Tussendorpel: stretches walled in on both sides drop; a
                // stretch on the wall (one side) becomes a borstweringdorpel.
                let (lo, hi) = (r.x, r.x + r.width);
                let mut top_blocked: Vec<(f64, f64)> = Vec::new();
                let mut bottom_blocked: Vec<(f64, f64)> = Vec::new();
                for b in &buiten_rects {
                    let iv = (b.x.max(lo), (b.x + b.width).min(hi));
                    if (b.y + b.height - r.y).abs() < EPS {
                        top_blocked.push(iv);
                    }
                    if (b.y - (r.y + r.height)).abs() < EPS {
                        bottom_blocked.push(iv);
                    }
                }
                let walled = intersect_interval_unions(&top_blocked, &bottom_blocked, EPS);
                for (a, c) in crate::layout::subtract_intervals(lo, hi, &walled, EPS) {
                    // Borstweringdorpel-verlenging: aan een uiteinde op de
                    // binnenrand waar de muur tot diezelfde rand doorloopt is
                    // de buitenstijl weggevallen → de dorpel loopt fw door
                    // tot het buitenvlak (de stijl erboven stopt óp hem).
                    let extend_left = (a - fw).abs() < EPS
                        && top_blocked
                            .iter()
                            .chain(bottom_blocked.iter())
                            .any(|&(b0, _)| (b0 - a).abs() < EPS);
                    let extend_right = (c - (fw + inner_w)).abs() < EPS
                        && top_blocked
                            .iter()
                            .chain(bottom_blocked.iter())
                            .any(|&(_, b1)| (b1 - c).abs() < EPS);
                    let div_net = (c - a)
                        + if extend_left { fw } else { 0.0 }
                        + if extend_right { fw } else { 0.0 };
                    let piece_id = format!("{}-TD{}", mark, td);
                    td += 1;
                    cut_list.push(CutListItem {
                        piece_id,
                        member_type: MemberType::DividerH,
                        profile_name: profile_name.clone(),
                        material: mat_name.clone(),
                        net_length_mm: div_net,
                        gross_length_mm: gross_length(div_net, mat, false), // dividers always butt-joined
                        miter_left_deg: 90.0,
                        miter_right_deg: 90.0,
                        quantity: 1,
                    });
                }
            }
        }

        // Vakken: per leaf (behalve `Buiten`) de glas-/vleugel-/paneel-items.
        // Cell data (glazing, sash profile, hardware) is aligned by the
        // stable depth-first leaf index, but ONLY when the leaf count (all
        // leaves, `Buiten` included) equals the cell count — a tree converted
        // from the grid (gridToLayout) lists its leaves in row-major order =
        // cell order and satisfies this. Template/edited trees without
        // matching cells fall back to the defaults everywhere instead of
        // picking up an unrelated cell's data.
        let default_glazing = crate::kozijn::Glazing::default();
        let cells_aligned = lg.leaves.len() == kozijn.cells.len();
        for l in &lg.leaves {
            if l.vulling.is_buiten() {
                continue;
            }
            let cell = if cells_aligned { kozijn.cells.get(l.leaf_index) } else { None };
            let glazing = cell.map(|c| &c.glazing).unwrap_or(&default_glazing);
            let cell_id = format!("{}-V{}", mark, l.leaf_index + 1);
            let vak_w = l.rect.width;
            let vak_h = l.rect.height;

            // Vleugelbreedte: raamhout-default 69 mm (KVT-draaikiep 69x90,
            // zelfde default als het grid-pad); deurhout = kozijnhoutbreedte.
            let sash_w = match &l.vulling {
                Vakvulling::Raam { .. } => Some(cell.and_then(|c| c.sash_width).unwrap_or(69.0)),
                Vakvulling::Deur { .. } => Some(cell.and_then(|c| c.sash_width).unwrap_or(fw)),
                _ => None,
            };

            match &l.vulling {
                Vakvulling::Glas { .. } | Vakvulling::Raam { .. } | Vakvulling::Deur { .. } => {
                    // Vleugelleden (raamhout/deurhout) voor draaiende
                    // vullingen — zelfde systematiek als het grid-pad.
                    if let Some(sw) = sash_w {
                        let (id_prefix, fallback_profile) =
                            if matches!(&l.vulling, Vakvulling::Deur { .. }) {
                                ("D", format!("Deurhout {}", profile_name))
                            } else {
                                ("R", format!("Raamhout {}", profile_name))
                            };
                        let sash_profile_name = cell
                            .and_then(|c| c.sash_profile.as_ref())
                            .map(|p| p.name.clone())
                            .unwrap_or(fallback_profile);
                        push_layout_sash_members(
                            &mut cut_list,
                            &cell_id,
                            id_prefix,
                            &sash_profile_name,
                            mat,
                            &mat_name,
                            is_miter,
                            angle,
                            vak_w,
                            vak_h,
                            sw,
                        );
                        // Sash seal gasket rond de vleugel.
                        gasket_list.push(GasketListItem {
                            gasket_type: GasketType::SashSeal,
                            length_mm: 2.0 * (vak_w + vak_h) + GASKET_OVERLAP_MM,
                            quantity: 1,
                        });
                    }

                    // Glasmaat: dagmaat = vakmaat, bij draaiende delen minus
                    // de vleugel; daarna dezelfde R2-snapshotregels als het
                    // grid-pad via de gedeelde helper `glass_size_for_day`.
                    let (day_w, day_h) = match sash_w {
                        Some(sw) => (vak_w - 2.0 * sw, vak_h - 2.0 * sw),
                        None => (vak_w, vak_h),
                    };
                    let (glass_w, glass_h) = glass_size_for_day(
                        day_w,
                        day_h,
                        kozijn.frame.profile_snapshot.as_ref(),
                        mat,
                    );
                    let area = (glass_w / 1000.0) * (glass_h / 1000.0);
                    glass_list.push(GlassListItem {
                        piece_id: cell_id.clone(),
                        cell_index: l.leaf_index,
                        glass_type: glazing.glass_type.clone(),
                        width_mm: glass_w,
                        height_mm: glass_h,
                        thickness_mm: glazing.thickness_mm,
                        ug_value: glazing.ug_value,
                        area_m2: area,
                        quantity: 1,
                    });

                    // Glazing gaskets (inner + outer per pane)
                    let gasket_perimeter = 2.0 * (glass_w + glass_h) + GASKET_OVERLAP_MM;
                    gasket_list.push(GasketListItem {
                        gasket_type: GasketType::GlazingInner,
                        length_mm: gasket_perimeter,
                        quantity: 1,
                    });
                    gasket_list.push(GasketListItem {
                        gasket_type: GasketType::GlazingOuter,
                        length_mm: gasket_perimeter,
                        quantity: 1,
                    });

                    // Glaslatten: primaire bron is de uitgelijnde cel; zonder
                    // cel-spec valt de canvas-vulling ("binnen"/"buiten")
                    // terug op de default latspec (15×17, verstek). Lengte
                    // via dezelfde KVT 12.3.2-helper als het grid-pad
                    // (`glaslat_total_length`), op de dagmaat die hierboven
                    // al voor het glas is bepaald.
                    let vulling_glaslat = match &l.vulling {
                        Vakvulling::Glas { glaslat }
                        | Vakvulling::Raam { glaslat, .. }
                        | Vakvulling::Deur { glaslat, .. } => glaslat.as_deref(),
                        _ => None,
                    };
                    let glaslat_spec = cell
                        .and_then(|c| c.glaslat.clone())
                        .or_else(|| match vulling_glaslat {
                            Some("binnen") => Some(crate::glaslat::Glaslat::default()),
                            Some("buiten") => Some(crate::glaslat::Glaslat {
                                position: crate::glaslat::GlaslatPosition::Buiten,
                                ..Default::default()
                            }),
                            _ => None,
                        });
                    if let Some(gl) = glaslat_spec {
                        glaslat_list.push(GlaslatListItem {
                            piece_id: cell_id.clone(),
                            cell_index: l.leaf_index,
                            position: gl.position.label_nl().to_string(),
                            material: gl.material.clone(),
                            width_mm: gl.width_mm,
                            height_mm: gl.height_mm,
                            total_length_mm: glaslat_total_length(
                                day_w,
                                day_h,
                                kozijn.frame.profile_snapshot.as_ref(),
                                mat,
                                &gl,
                            ),
                            mitered: gl.mitered,
                            quantity: 4,
                        });
                    }

                    // Hardware van de uitgelijnde cel (raam/deur-vullingen).
                    if !matches!(&l.vulling, Vakvulling::Glas { .. }) {
                        if let Some(hw) = cell.and_then(|c| c.hardware_set.as_ref()) {
                            if let Some(ref hinges) = hw.hinges {
                                hardware_list.push(HardwareListItem {
                                    cell_index: l.leaf_index,
                                    component: "Scharnier".into(),
                                    description: format!(
                                        "{:?} - draagkracht {:.0} kg",
                                        hinges.hinge_type, hinges.load_capacity_kg
                                    ),
                                    quantity: hinges.count as u32,
                                });
                            }
                            if let Some(ref handle) = hw.handle {
                                hardware_list.push(HardwareListItem {
                                    cell_index: l.leaf_index,
                                    component: "Greep".into(),
                                    description: format!(
                                        "{:?} - hoogte {} mm",
                                        handle.handle_type, handle.height_mm
                                    ),
                                    quantity: 1,
                                });
                            }
                            if let Some(ref locking) = hw.locking {
                                hardware_list.push(HardwareListItem {
                                    cell_index: l.leaf_index,
                                    component: "Sluiting".into(),
                                    description: format!(
                                        "{:?} - {} sluitpunten",
                                        locking.lock_type, locking.locking_points
                                    ),
                                    quantity: 1,
                                });
                            }
                        }
                    }
                }
                Vakvulling::Paneel { filling } => {
                    // Vulling-spec op de leaf wint; anders de uitgelijnde cel;
                    // anders de sandwich-default (zelfde label als grid-pad).
                    let panel_label = filling
                        .as_ref()
                        .or(cell.and_then(|c| c.panel_filling.as_ref()))
                        .map(|f| f.filling_type.label_nl().to_string())
                        .unwrap_or_else(|| "Sandwichpaneel".to_string());
                    panel_list.push(PanelListItem {
                        piece_id: cell_id.clone(),
                        cell_index: l.leaf_index,
                        width_mm: vak_w - 2.0 * GLASS_CLEARANCE_MM,
                        height_mm: vak_h - 2.0 * GLASS_CLEARANCE_MM,
                        panel_type: panel_label,
                        quantity: 1,
                    });
                }
                Vakvulling::Rooster => {
                    // Zoals het grid-pad Ventilation: alleen een paneel-item
                    // wanneer er een roosterspec op de (uitgelijnde) cel staat.
                    if let Some(filling) = cell.and_then(|c| c.panel_filling.as_ref()) {
                        panel_list.push(PanelListItem {
                            piece_id: cell_id.clone(),
                            cell_index: l.leaf_index,
                            width_mm: vak_w - 2.0 * GLASS_CLEARANCE_MM,
                            height_mm: vak_h - 2.0 * GLASS_CLEARANCE_MM,
                            panel_type: filling.filling_type.label_nl().to_string(),
                            quantity: 1,
                        });
                    }
                }
                Vakvulling::Buiten => {}
            }
        }
    } else {
        // ── Dividers (grid) ────────────────────────────────────

        // Vertical dividers (tussenstijlen)
        for (i, col) in kozijn.grid.columns.iter().enumerate() {
            if col.divider_profile.is_some() {
                let div_net = inner_h;
                cut_list.push(CutListItem {
                    piece_id: format!("{}-TS{}", mark, i),
                    member_type: MemberType::DividerV,
                    profile_name: profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: div_net,
                    gross_length_mm: gross_length(div_net, mat, false), // dividers always butt-joined
                    miter_left_deg: 90.0,
                    miter_right_deg: 90.0,
                    quantity: 1,
                });
            }
        }

        // Horizontal dividers (tussendorpels)
        for (i, row) in kozijn.grid.rows.iter().enumerate() {
            if row.divider_profile.is_some() {
                let div_net = inner_w;
                cut_list.push(CutListItem {
                    piece_id: format!("{}-TD{}", mark, i),
                    member_type: MemberType::DividerH,
                    profile_name: profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: div_net,
                    gross_length_mm: gross_length(div_net, mat, false),
                    miter_left_deg: 90.0,
                    miter_right_deg: 90.0,
                    quantity: 1,
                });
            }
        }

        // ── Cells: glass, panels, hardware, gaskets ────────────

        let num_cols = kozijn.grid.columns.len();
        for (i, cell) in kozijn.cells.iter().enumerate() {
            let col_idx = i % num_cols;
            let row_idx = i / num_cols;
            let cell_w = kozijn.grid.columns[col_idx].size;
            let cell_h = kozijn.grid.rows[row_idx].size;

            let cell_id = format!("{}-V{}", mark, i + 1);

            // Determine sash dimensions for operable cells (affects glass size)
            let sash_fw = cell.sash_width.unwrap_or(0.0); // 0 for fixed glass (no sash frame)
            let has_sash = cell.panel_type.is_operable();

            match cell.panel_type {
                PanelType::FixedGlass | PanelType::TurnTilt | PanelType::Turn
                | PanelType::Tilt | PanelType::Sliding | PanelType::Door
                | PanelType::TopHung | PanelType::BottomHung | PanelType::LiftSlide | PanelType::Pivot => {
                    // Glasmaat per R2 (KVT 12.2 / NPR 3577-uitleg /
                    // glasdiscount-inmeetregel, zie docs/profielmaten-onderzoek.md
                    // §5): het glas valt in de sponning — sponningmaat = dagmaat +
                    // 2 × sponninghoogte; glasmaat = sponningmaat − 2 × omtrek-
                    // speling met 5 mm speling per zijde (NPR-minimum, steldikte;
                    // = dagmaat + 24 bij de 17 mm houtsponning, en altijd binnen
                    // de praktijkcheck "sponningmaat − 6 à 10"). De dagmaat is de
                    // celmaat, bij draaiende delen minus het raamhout; de vleugel-
                    // sponning benaderen we met de kozijnsnapshot-waarde (zelfde
                    // gedocumenteerde benadering als de glaslatlengtes hieronder).
                    // Kozijnen zonder snapshot houden de historische dagmaat −
                    // 2 × GLASS_CLEARANCE_MM, zodat oude projecten identiek
                    // blijven. Berekening gedeeld met het layout-pad via
                    // `glass_size_for_day`.
                    let (day_w, day_h) = if has_sash && sash_fw > 0.0 {
                        (cell_w - 2.0 * sash_fw, cell_h - 2.0 * sash_fw)
                    } else {
                        (cell_w, cell_h)
                    };
                    let (glass_w, glass_h) = glass_size_for_day(
                        day_w,
                        day_h,
                        kozijn.frame.profile_snapshot.as_ref(),
                        mat,
                    );
                    let area = (glass_w / 1000.0) * (glass_h / 1000.0);
                    glass_list.push(GlassListItem {
                        piece_id: cell_id.clone(),
                        cell_index: i,
                        glass_type: cell.glazing.glass_type.clone(),
                        width_mm: glass_w,
                        height_mm: glass_h,
                        thickness_mm: cell.glazing.thickness_mm,
                        ug_value: cell.glazing.ug_value,
                        area_m2: area,
                        quantity: 1,
                    });

                    // Glazing gaskets (inner + outer per pane)
                    let gasket_perimeter = 2.0 * (glass_w + glass_h) + GASKET_OVERLAP_MM;
                    gasket_list.push(GasketListItem {
                        gasket_type: GasketType::GlazingInner,
                        length_mm: gasket_perimeter,
                        quantity: 1,
                    });
                    gasket_list.push(GasketListItem {
                        gasket_type: GasketType::GlazingOuter,
                        length_mm: gasket_perimeter,
                        quantity: 1,
                    });
                }
                PanelType::Panel => {
                    let panel_w = cell_w - 2.0 * GLASS_CLEARANCE_MM;
                    let panel_h = cell_h - 2.0 * GLASS_CLEARANCE_MM;
                    let panel_label = cell
                        .panel_filling
                        .as_ref()
                        .map(|f| f.filling_type.label_nl().to_string())
                        .unwrap_or_else(|| "Sandwichpaneel".to_string());
                    panel_list.push(PanelListItem {
                        piece_id: cell_id.clone(),
                        cell_index: i,
                        width_mm: panel_w,
                        height_mm: panel_h,
                        panel_type: panel_label,
                        quantity: 1,
                    });
                }
                PanelType::Ventilation => {
                    // Ventilation grille — emit a panel entry when a grille filling is set.
                    if let Some(filling) = cell.panel_filling.as_ref() {
                        panel_list.push(PanelListItem {
                            piece_id: cell_id.clone(),
                            cell_index: i,
                            width_mm: cell_w - 2.0 * GLASS_CLEARANCE_MM,
                            height_mm: cell_h - 2.0 * GLASS_CLEARANCE_MM,
                            panel_type: filling.filling_type.label_nl().to_string(),
                            quantity: 1,
                        });
                    }
                }
            }

            // Glaslatten (glazing beads) — emit a bead cut entry per glazed cell
            // that has one. Cut lengths per KVT 12.3.2 via the shared helper
            // `glaslat_total_length` (also used by the layout path). Voor
            // draaiende delen benaderen we de vleugelsponning met de
            // kozijnsnapshot-waarde (een eigen vleugelprofiel-snapshot is een
            // gedocumenteerde follow-up). Kozijnen zonder snapshot houden de
            // historische dagmaat-omtrek, zodat oude projecten identiek blijven.
            if let Some(gl) = cell.glaslat.as_ref() {
                let (gw, gh) = if cell.panel_type.is_operable() {
                    // Sash-default 69mm (KVT-draaikiep 69x90) — één consistente default,
                    // was inconsistent 67 hier vs 54 in de sash-cut hieronder.
                    let sw = cell.sash_width.unwrap_or(69.0);
                    (cell_w - 2.0 * sw, cell_h - 2.0 * sw)
                } else {
                    (cell_w, cell_h)
                };
                let total_length = glaslat_total_length(
                    gw,
                    gh,
                    kozijn.frame.profile_snapshot.as_ref(),
                    mat,
                    gl,
                );
                glaslat_list.push(GlaslatListItem {
                    piece_id: cell_id.clone(),
                    cell_index: i,
                    position: gl.position.label_nl().to_string(),
                    material: gl.material.clone(),
                    width_mm: gl.width_mm,
                    height_mm: gl.height_mm,
                    total_length_mm: total_length,
                    mitered: gl.mitered,
                    quantity: 4,
                });
            }

            // Sash frame for operable cells — use cell.sash_profile if available
            if cell.panel_type.is_operable() {
                let sash_w = cell_w;
                let sash_h = cell_h;
                let sash_is_miter = is_miter;
                let sash_angle = angle;
                let sash_frame_w = cell.sash_width.unwrap_or(69.0);

                let (sash_stile_net, sash_rail_net) = if sash_is_miter {
                    (sash_h - 2.0 * sash_frame_w, sash_w - 2.0 * sash_frame_w)
                } else {
                    (sash_h, sash_w - 2.0 * sash_frame_w)
                };
                // Alu/PVC verstek-vleugels: gross runs to the long point (sash
                // face width per end) — same systematics as the frame members;
                // wood keeps the historic `gross_length` numbers.
                let sash_gross = |net: f64| {
                    if sash_is_miter {
                        miter_gross_length(net, mat, sash_frame_w)
                    } else {
                        gross_length(net, mat, sash_is_miter)
                    }
                };

                let sash_profile_name = cell.sash_profile.as_ref()
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| format!("Raamhout {}", profile_name));

                // Sash stiles (left + right)
                cut_list.push(CutListItem {
                    piece_id: format!("{}-RSL", cell_id),
                    member_type: MemberType::SashLeft,
                    profile_name: sash_profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: sash_stile_net,
                    gross_length_mm: sash_gross(sash_stile_net),
                    miter_left_deg: sash_angle,
                    miter_right_deg: sash_angle,
                    quantity: 1,
                });
                cut_list.push(CutListItem {
                    piece_id: format!("{}-RSR", cell_id),
                    member_type: MemberType::SashRight,
                    profile_name: sash_profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: sash_stile_net,
                    gross_length_mm: sash_gross(sash_stile_net),
                    miter_left_deg: sash_angle,
                    miter_right_deg: sash_angle,
                    quantity: 1,
                });

                // Sash rails (top + bottom)
                cut_list.push(CutListItem {
                    piece_id: format!("{}-RDB", cell_id),
                    member_type: MemberType::SashTop,
                    profile_name: sash_profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: sash_rail_net,
                    gross_length_mm: sash_gross(sash_rail_net),
                    miter_left_deg: sash_angle,
                    miter_right_deg: sash_angle,
                    quantity: 1,
                });
                cut_list.push(CutListItem {
                    piece_id: format!("{}-RDO", cell_id),
                    member_type: MemberType::SashBottom,
                    profile_name: sash_profile_name.clone(),
                    material: mat_name.clone(),
                    net_length_mm: sash_rail_net,
                    gross_length_mm: sash_gross(sash_rail_net),
                    miter_left_deg: sash_angle,
                    miter_right_deg: sash_angle,
                    quantity: 1,
                });

                // Sash seal gasket
                let sash_perimeter = 2.0 * (sash_w + sash_h) + GASKET_OVERLAP_MM;
                gasket_list.push(GasketListItem {
                    gasket_type: GasketType::SashSeal,
                    length_mm: sash_perimeter,
                    quantity: 1,
                });
            }

            // Hardware list from HardwareSet
            if let Some(ref hw) = cell.hardware_set {
                if let Some(ref hinges) = hw.hinges {
                    hardware_list.push(HardwareListItem {
                        cell_index: i,
                        component: "Scharnier".into(),
                        description: format!("{:?} - draagkracht {:.0} kg", hinges.hinge_type, hinges.load_capacity_kg),
                        quantity: hinges.count as u32,
                    });
                }
                if let Some(ref handle) = hw.handle {
                    hardware_list.push(HardwareListItem {
                        cell_index: i,
                        component: "Greep".into(),
                        description: format!("{:?} - hoogte {} mm", handle.handle_type, handle.height_mm),
                        quantity: 1,
                    });
                }
                if let Some(ref locking) = hw.locking {
                    hardware_list.push(HardwareListItem {
                        cell_index: i,
                        component: "Sluiting".into(),
                        description: format!("{:?} - {} sluitpunten", locking.lock_type, locking.locking_points),
                        quantity: 1,
                    });
                }
            }
        }
    }

    // Frame seal gasket (around the full inner perimeter)
    let frame_perimeter = 2.0 * (inner_w + inner_h) + GASKET_OVERLAP_MM;
    gasket_list.push(GasketListItem {
        gasket_type: GasketType::FrameSeal,
        length_mm: frame_perimeter,
        quantity: 1,
    });

    // ── BOM (aggregate) ────────────────────────────────────────

    let mut bom = Vec::new();

    // Total profile length
    let total_profile_mm: f64 = cut_list.iter().map(|c| c.gross_length_mm * c.quantity as f64).sum();
    bom.push(BomItem {
        category: "Profiel".into(),
        description: format!("{} {} mm", mat_name, profile_name),
        unit: "m".into(),
        quantity: total_profile_mm / 1000.0,
    });

    // Total glass area
    let total_glass_m2: f64 = glass_list.iter().map(|g| g.area_m2 * g.quantity as f64).sum();
    if total_glass_m2 > 0.0 {
        bom.push(BomItem {
            category: "Glas".into(),
            description: glass_list.first().map(|g| g.glass_type.clone()).unwrap_or_default(),
            unit: "m2".into(),
            quantity: total_glass_m2,
        });
    }

    // Total gasket length
    let total_gasket_mm: f64 = gasket_list.iter().map(|g| g.length_mm * g.quantity as f64).sum();
    bom.push(BomItem {
        category: "Rubber".into(),
        description: "EPDM afdichting".into(),
        unit: "m".into(),
        quantity: total_gasket_mm / 1000.0,
    });

    // Hardware count
    let total_hw: u32 = hardware_list.iter().map(|h| h.quantity).sum();
    if total_hw > 0 {
        bom.push(BomItem {
            category: "Beslag".into(),
            description: "Hang- en sluitwerk (diverse)".into(),
            unit: "stuks".into(),
            quantity: total_hw as f64,
        });
    }

    // Panel count
    if !panel_list.is_empty() {
        let total_panels: u32 = panel_list.iter().map(|p| p.quantity).sum();
        bom.push(BomItem {
            category: "Paneel".into(),
            description: "Sandwichpaneel".into(),
            unit: "stuks".into(),
            quantity: total_panels as f64,
        });
    }

    ProductionData {
        kozijn_mark: kozijn.mark.clone(),
        kozijn_name: kozijn.name.clone(),
        cut_list,
        glass_list,
        hardware_list,
        gasket_list,
        panel_list,
        glaslat_list,
        bom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn test_single_fixed_glass() {
        let k = Kozijn::new("Test", "T01", 900.0, 1400.0);
        let prod = compute_production_data(&k);

        // 4 frame pieces (2 stiles + 2 rails)
        assert_eq!(prod.cut_list.len(), 4);

        // 1 glass pane
        assert_eq!(prod.glass_list.len(), 1);
        let glass = &prod.glass_list[0];
        assert!(glass.width_mm > 0.0);
        assert!(glass.height_mm > 0.0);

        // No hardware (fixed glass)
        assert!(prod.hardware_list.is_empty());

        // Fixed glass has no sash seal, but has glazing gaskets + frame seal
        assert!(prod.gasket_list.len() >= 3); // inner + outer + frame seal
    }

    #[test]
    fn test_turn_tilt_with_hardware() {
        let mut k = Kozijn::new("Test", "T02", 900.0, 1400.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].hardware_set = crate::hardware::default_hardware_set(
            PanelType::TurnTilt,
            Some(crate::kozijn::OpeningDirection::Left),
            766.0, 1266.0, 24.0,
            &Material::Wood(crate::kozijn::WoodType::Meranti),
            crate::hardware::SecurityClass::None,
        );
        let prod = compute_production_data(&k);

        // 4 frame + 4 sash pieces = 8
        assert_eq!(prod.cut_list.len(), 8);

        // Hardware should be present
        assert!(!prod.hardware_list.is_empty());

        // Check wood gross length includes pen allowance
        let stile = prod.cut_list.iter().find(|c| c.member_type == MemberType::FrameLeft).unwrap();
        assert!(stile.gross_length_mm > stile.net_length_mm);
        let expected_gross = stile.net_length_mm + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM;
        assert!((stile.gross_length_mm - expected_gross).abs() < 0.01);
    }

    fn find<'a>(prod: &'a ProductionData, mt: MemberType) -> &'a CutListItem {
        prod.cut_list.iter().find(|c| c.member_type == mt).unwrap()
    }

    #[test]
    fn test_empty_corner_joints_matches_legacy_output() {
        // (a) Empty corner_joints must produce exactly the material-based saw
        // list — and be identical to the untouched auto-populated default
        // joint set.
        let k_default = Kozijn::new("Test", "T03", 900.0, 1400.0);
        let mut k_empty = k_default.clone();
        k_empty.frame.corner_joints = vec![];

        let prod_default = compute_production_data(&k_default);
        let prod_empty = compute_production_data(&k_empty);

        assert_eq!(prod_default.cut_list.len(), prod_empty.cut_list.len());
        for (a, b) in prod_default.cut_list.iter().zip(prod_empty.cut_list.iter()) {
            assert_eq!(a.member_type, b.member_type);
            assert!((a.net_length_mm - b.net_length_mm).abs() < 0.01);
            assert!((a.gross_length_mm - b.gross_length_mm).abs() < 0.01);
            assert!((a.miter_left_deg - b.miter_left_deg).abs() < 0.01);
            assert!((a.miter_right_deg - b.miter_right_deg).abs() < 0.01);
        }

        // Wood default per NL-timmerpraktijk (KVT katern 15): dorpels lopen
        // door (full width), stijlen worden ertussen gepend; gross = net +
        // 2 * pen allowance + kerf, square cuts.
        let fw = k_empty.frame.frame_width;
        let stile = find(&prod_empty, MemberType::FrameLeft);
        assert!((stile.net_length_mm - (k_empty.frame.outer_height - 2.0 * fw)).abs() < 0.01);
        assert!((stile.gross_length_mm
            - (stile.net_length_mm + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM)).abs() < 0.01);
        assert!((stile.miter_left_deg - 90.0).abs() < 0.01);
        let rail = find(&prod_empty, MemberType::FrameTop);
        assert!((rail.net_length_mm - k_empty.frame.outer_width).abs() < 0.01);
        let sill = find(&prod_empty, MemberType::FrameBottom);
        assert!((sill.net_length_mm - k_empty.frame.outer_width).abs() < 0.01);
    }

    #[test]
    fn test_pen_slis_joints_drive_cut_list() {
        // (b) Configured pen/slis (stijl through, pen 30 mm): dorpels shorter
        // than the outer width, stijlen run through full height, and the pen
        // allowance sits on the dorpels only.
        let mut k = Kozijn::new("Test", "T04", 900.0, 1400.0);
        let pen = crate::joint::Joint {
            joint_type: JointType::PenSlis,
            through_member: ThroughMember::Stijl,
            angle: 90.0,
            pen_length: 30.0, // non-default → joint-driven path
        };
        k.frame.corner_joints = vec![pen.clone(), pen.clone(), pen.clone(), pen];

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;

        // Stijlen doorlopend: full outer height, no pen allowance (slis side).
        for mt in [MemberType::FrameLeft, MemberType::FrameRight] {
            let stile = find(&prod, mt);
            assert!((stile.net_length_mm - k.frame.outer_height).abs() < 0.01);
            assert!((stile.gross_length_mm - (stile.net_length_mm + SAW_KERF_MM)).abs() < 0.01);
            assert!((stile.miter_left_deg - 90.0).abs() < 0.01);
            assert!((stile.miter_right_deg - 90.0).abs() < 0.01);
        }
        // Dorpels tussen de stijlen + pen allowance on both ends.
        for mt in [MemberType::FrameTop, MemberType::FrameBottom] {
            let rail = find(&prod, mt);
            assert!(rail.net_length_mm < k.frame.outer_width);
            assert!((rail.net_length_mm - (k.frame.outer_width - 2.0 * fw)).abs() < 0.01);
            assert!((rail.gross_length_mm
                - (rail.net_length_mm + 2.0 * 30.0 + SAW_KERF_MM)).abs() < 0.01);
        }
    }

    #[test]
    fn test_verstek_joints_give_45_degree_cuts() {
        // (c) Verstek joints on a wood kozijn: joints beat the material
        // default — 45° cuts on both ends of every member. Netmaat = korte
        // zijde (binnenmaat); bruto = over de LANGE punt (net + 2 × fw) +
        // zaagsnede, want het fysieke stuk loopt door tot de buitenhoek.
        let mut k = Kozijn::new("Test", "T05", 900.0, 1400.0);
        let verstek = crate::joint::Joint {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        k.frame.corner_joints = vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;

        for mt in [
            MemberType::FrameLeft, MemberType::FrameRight,
            MemberType::FrameTop, MemberType::FrameBottom,
        ] {
            let item = find(&prod, mt);
            assert!((item.miter_left_deg - 45.0).abs() < 0.01);
            assert!((item.miter_right_deg - 45.0).abs() < 0.01);
            let outer = match mt {
                MemberType::FrameLeft | MemberType::FrameRight => k.frame.outer_height,
                _ => k.frame.outer_width,
            };
            assert!((item.net_length_mm - (outer - 2.0 * fw)).abs() < 0.01);
            // Wood miter: long point (fw per end) + kerf, no weld overmeasure.
            assert!((item.gross_length_mm
                - (item.net_length_mm + 2.0 * fw + SAW_KERF_MM)).abs() < 0.01);
            // The physical piece covers at least the outer span + kerf.
            assert!(item.gross_length_mm >= outer + SAW_KERF_MM - 0.01);
        }
    }

    #[test]
    fn test_verstek_gross_covers_long_point() {
        // Expliciet (opdracht 3): verstek-lid met buitenmaat 1200 en fw 67 →
        // net = 1200 − 134 = 1066 (korte zijde), gross ≥ 1200 (lange punt)
        // plus zaagsnede.
        let mut k = Kozijn::new("Test", "T15", 1200.0, 1500.0);
        let verstek = crate::joint::Joint {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        k.frame.corner_joints = vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];
        assert!((k.frame.frame_width - 67.0).abs() < 1e-9);

        let prod = compute_production_data(&k);
        let rail = find(&prod, MemberType::FrameTop);
        assert!((rail.net_length_mm - (1200.0 - 2.0 * 67.0)).abs() < 0.01);
        assert!(rail.gross_length_mm >= 1200.0, "gross {} must cover the long point 1200", rail.gross_length_mm);
        assert!((rail.gross_length_mm - (1200.0 + SAW_KERF_MM)).abs() < 0.01);
    }

    #[test]
    fn test_pvc_verstek_adds_weld_overmeasure() {
        let mut k = Kozijn::new("Test", "T06", 900.0, 1400.0);
        k.frame.material = Material::Pvc;
        let verstek = crate::joint::Joint {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        k.frame.corner_joints = vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;
        let stile = find(&prod, MemberType::FrameLeft);
        // Long point (fw) + weld overmeasure per end, + kerf.
        assert!((stile.gross_length_mm
            - (stile.net_length_mm + 2.0 * (fw + PVC_WELD_OVERMEASURE_MM) + SAW_KERF_MM)).abs() < 0.01);
    }

    #[test]
    fn test_aluminum_untouched_defaults_keep_miter() {
        // Regression guard: alu/PVC kozijnen whose corner_joints are still the
        // auto-populated pen/slis defaults must keep the material-based miter,
        // with the SAME gross systematics as the configured-verstek path: the
        // piece runs to the long point (fw per end) + kerf. Alu 900×1400 with
        // fw 67: stijl net = 1400 − 134 = 1266, gross = 1266 + 4 + 134 = 1404.
        let mut k = Kozijn::new("Test", "T07", 900.0, 1400.0);
        k.frame.material = Material::Aluminum;
        // Kozijn::new already populated 4 default joints; leave them untouched.
        assert_eq!(k.frame.corner_joints.len(), 4);

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;
        let stile = find(&prod, MemberType::FrameLeft);
        assert!((stile.net_length_mm - (k.frame.outer_height - 2.0 * fw)).abs() < 0.01);
        assert!((stile.net_length_mm - 1266.0).abs() < 0.01);
        assert!((stile.miter_left_deg - 45.0).abs() < 0.01);
        // Long point + kerf: identical to a configured verstek joint set.
        assert!((stile.gross_length_mm
            - (stile.net_length_mm + 2.0 * fw + SAW_KERF_MM)).abs() < 0.01);
        assert!((stile.gross_length_mm - 1404.0).abs() < 0.01);
        // The physical piece covers the outer span + kerf.
        assert!(stile.gross_length_mm >= k.frame.outer_height + SAW_KERF_MM - 0.01);
    }

    #[test]
    fn test_default_and_configured_verstek_give_identical_gross() {
        // (fix 6) An alu kozijn with untouched default joints and the same
        // kozijn with an explicitly configured verstek set must produce
        // identical net AND gross lengths on every frame member.
        let mut k_default = Kozijn::new("Test", "T17", 900.0, 1400.0);
        k_default.frame.material = Material::Aluminum;
        let mut k_configured = k_default.clone();
        let verstek = crate::joint::Joint {
            joint_type: JointType::Verstek,
            through_member: ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        k_configured.frame.corner_joints =
            vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];

        let prod_a = compute_production_data(&k_default);
        let prod_b = compute_production_data(&k_configured);
        for mt in [
            MemberType::FrameLeft, MemberType::FrameRight,
            MemberType::FrameTop, MemberType::FrameBottom,
        ] {
            let a = find(&prod_a, mt);
            let b = find(&prod_b, mt);
            assert!((a.net_length_mm - b.net_length_mm).abs() < 0.01);
            assert!((a.gross_length_mm - b.gross_length_mm).abs() < 0.01,
                "{:?}: default gross {} != configured gross {}",
                mt, a.gross_length_mm, b.gross_length_mm);
        }
    }

    #[test]
    fn test_mixed_corners_and_dorpel_through() {
        // Bottom corners: dorpel runs through (onderdorpel doorlopend);
        // top corners: default pen/slis with stijl through.
        let mut k = Kozijn::new("Test", "T08", 900.0, 1400.0);
        let top = crate::joint::Joint {
            joint_type: JointType::PenSlis,
            through_member: ThroughMember::Stijl,
            angle: 90.0,
            pen_length: 20.0,
        };
        let bottom = crate::joint::Joint {
            joint_type: JointType::PenSlis,
            through_member: ThroughMember::Dorpel,
            angle: 90.0,
            pen_length: 20.0,
        };
        // Order: [top-left, top-right, bottom-left, bottom-right]
        k.frame.corner_joints = vec![top.clone(), top, bottom.clone(), bottom];

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;

        // Stijlen: through at the top (no reduction), cut short at the bottom
        // where the onderdorpel runs through.
        let stile = find(&prod, MemberType::FrameLeft);
        assert!((stile.net_length_mm - (k.frame.outer_height - fw)).abs() < 0.01);
        // Pen at the bottom end only (stijl is the non-through member there).
        assert!((stile.gross_length_mm - (stile.net_length_mm + 20.0 + SAW_KERF_MM)).abs() < 0.01);

        // Bovendorpel: between the stijlen, pens both ends.
        let top_rail = find(&prod, MemberType::FrameTop);
        assert!((top_rail.net_length_mm - (k.frame.outer_width - 2.0 * fw)).abs() < 0.01);
        assert!((top_rail.gross_length_mm
            - (top_rail.net_length_mm + 2.0 * 20.0 + SAW_KERF_MM)).abs() < 0.01);

        // Onderdorpel: runs through full width, no pen allowance.
        let bottom_rail = find(&prod, MemberType::FrameBottom);
        assert!((bottom_rail.net_length_mm - k.frame.outer_width).abs() < 0.01);
        assert!((bottom_rail.gross_length_mm
            - (bottom_rail.net_length_mm + SAW_KERF_MM)).abs() < 0.01);
    }

    #[test]
    fn test_glaslat_lengths_without_snapshot_keep_legacy_perimeter() {
        // Regression: pre-snapshot kozijnen keep the historic day-size
        // perimeter for the bead cut length.
        let mut k = Kozijn::new("Test", "T09", 900.0, 1400.0);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default()); // 15x17, verstek
        assert!(k.frame.profile_snapshot.is_none());

        let prod = compute_production_data(&k);
        assert_eq!(prod.glaslat_list.len(), 1);
        let item = &prod.glaslat_list[0];
        let gw = 900.0 - 2.0 * 67.0; // 766
        let gh = 1400.0 - 2.0 * 67.0; // 1266
        assert!((item.total_length_mm - 2.0 * (gw + gh)).abs() < 0.01);
        assert_eq!(item.quantity, 4);
    }

    #[test]
    fn test_glaslat_lengths_follow_kvt_when_snapshot_present() {
        // KVT 12.3.2 with a 17 mm sponning (sponningmaat = dagmaat + 34):
        // verstek → all four beads sponningmaat − 1; stomp → horizontals run
        // through, verticals fit between them (− 2 × lathoogte − 1).
        let mut k = Kozijn::new("Test", "T10", 900.0, 1400.0);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default()); // 15x17, verstek
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });

        let gw = 900.0 - 2.0 * 67.0; // dagmaat breed: 766
        let gh = 1400.0 - 2.0 * 67.0; // dagmaat hoog: 1266
        let sm_w = gw + 2.0 * 17.0; // sponningmaat 800
        let sm_h = gh + 2.0 * 17.0; // sponningmaat 1300

        let prod = compute_production_data(&k);
        let verstek = 2.0 * (sm_w - 1.0) + 2.0 * (sm_h - 1.0);
        assert!((prod.glaslat_list[0].total_length_mm - verstek).abs() < 0.01);

        // Stomp (butt): verticale latten = sponningmaat − 2 × lathoogte − 1
        // → met een 17-lat exact dagmaat − 1.
        k.cells[0].glaslat.as_mut().unwrap().mitered = false;
        let prod = compute_production_data(&k);
        let stomp = 2.0 * (sm_w - 1.0) + 2.0 * (sm_h - 2.0 * 17.0 - 1.0);
        assert!((prod.glaslat_list[0].total_length_mm - stomp).abs() < 0.01);
        assert!((sm_h - 2.0 * 17.0 - 1.0 - (gh - 1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_glass_size_without_snapshot_keeps_legacy_clearance() {
        // Regression: pre-snapshot kozijnen keep the historic
        // dagmaat − 2 × GLASS_CLEARANCE_MM glass size.
        let k = Kozijn::new("Test", "T12", 900.0, 1400.0);
        assert!(k.frame.profile_snapshot.is_none());

        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let day_w = 900.0 - 2.0 * 67.0; // 766
        let day_h = 1400.0 - 2.0 * 67.0; // 1266
        assert!((glass.width_mm - (day_w - 2.0 * GLASS_CLEARANCE_MM)).abs() < 0.01);
        assert!((glass.height_mm - (day_h - 2.0 * GLASS_CLEARANCE_MM)).abs() < 0.01);
    }

    #[test]
    fn test_glass_size_follows_r2_when_snapshot_present() {
        // R2 (KVT 12.2 / NPR 3577-uitleg / glasdiscount-inmeetregel):
        // glasmaat = sponningmaat − 2 × omtrekspeling met 5 mm speling per
        // zijde. Bij een 17 mm sponning: dagmaat + 34 − 10 = dagmaat + 24.
        let mut k = Kozijn::new("Test", "T13", 900.0, 1400.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });

        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let day_w = 900.0 - 2.0 * 67.0; // 766
        let day_h = 1400.0 - 2.0 * 67.0; // 1266
        assert!((glass.width_mm - (day_w + 24.0)).abs() < 0.01);
        assert!((glass.height_mm - (day_h + 24.0)).abs() < 0.01);

        // Diepere sponning (PVC-Falzhöhe 28): dagmaat + 56 − 10 = dagmaat
        // + 46 — de aftrek van de sponningmaat blijft 10 (praktijkcheck
        // "sponningmaat − 6 à 10").
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            ..Default::default()
        });
        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        assert!((glass.width_mm - (day_w + 2.0 * 28.0 - 2.0 * GLASS_EDGE_CLEARANCE_MM)).abs() < 0.01);
    }

    #[test]
    fn test_glass_size_operable_uses_sash_day_size() {
        // Draaiend deel met snapshot: dagmaat = celmaat − 2 × raamhout,
        // daarna dezelfde R2-keten (vleugelsponning benaderd met de
        // kozijnsnapshot-waarde).
        let mut k = Kozijn::new("Test", "T14", 900.0, 1400.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].sash_width = Some(69.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });

        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let day_w = (900.0 - 2.0 * 67.0) - 2.0 * 69.0; // 628
        let day_h = (1400.0 - 2.0 * 67.0) - 2.0 * 69.0; // 1128
        assert!((glass.width_mm - (day_w + 24.0)).abs() < 0.01);
        assert!((glass.height_mm - (day_h + 24.0)).abs() < 0.01);
    }

    #[test]
    fn test_melkmeisje_steps_onderdorpel_and_side_stijl() {
        // Melkmeisje: raised side light with wall (buiten) below. The
        // onderdorpel only runs under the raam, and the right stijl only runs
        // down to the side-light step — neither spans the full kozijn.
        let mut k = Kozijn::new("Test", "MM", 1400.0, 1900.0);
        k.layout = Some(crate::layout::melkmeisje1());
        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;

        let bottom = find(&prod, MemberType::FrameBottom);
        let right = find(&prod, MemberType::FrameRight);
        let top = find(&prod, MemberType::FrameTop);
        let left = find(&prod, MemberType::FrameLeft);

        // Onderdorpel steps in: shorter than the full dagmaat width.
        assert!(bottom.net_length_mm > 0.0);
        assert!(bottom.net_length_mm < k.inner_width() - 1.0,
            "onderdorpel {} should be < dagmaat {}", bottom.net_length_mm, k.inner_width());
        // Right stijl steps in: shorter than the full outer height.
        assert!(right.net_length_mm < k.frame.outer_height - 1.0,
            "right stijl {} should be < {}", right.net_length_mm, k.frame.outer_height);
        // Both vakken reach the top and the raam reaches the left: the
        // bovendorpel runs the full width and the left stijl fits between the
        // dorpels (dorpel-through wood default, KVT katern 15).
        assert!((top.net_length_mm - k.frame.outer_width).abs() < 0.5);
        assert!((left.net_length_mm - (k.frame.outer_height - 2.0 * fw)).abs() < 0.5);
    }

    #[test]
    fn test_layout_without_buiten_keeps_full_frame_spans() {
        // A free-subdivision layout without any `Buiten` leaf must keep the
        // full outer frame spans — no melkmeisje stepping, identical to a
        // plain kozijn's frame members (dorpel-through wood default).
        let mut k = Kozijn::new("Test", "FG", 1400.0, 1900.0);
        k.layout = Some(crate::layout::free_grid(2, 2));
        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;
        let bottom = find(&prod, MemberType::FrameBottom);
        let left = find(&prod, MemberType::FrameLeft);
        assert!((bottom.net_length_mm - k.frame.outer_width).abs() < 0.5);
        assert!((left.net_length_mm - (k.frame.outer_height - 2.0 * fw)).abs() < 0.5);
    }

    #[test]
    fn test_layout_melkmeisje_cut_list_has_tussenstijl_and_stepped_glass() {
        // (opdracht 2a) A melkmeisje layout: the tussenstijl runs the FULL
        // binnenhoogte (bovendorpel to onderdorpel — the drawing clip at the
        // side-light step is a picture-only device, the physical member keeps
        // running because the deur borders it over the full height), the
        // side-light onderdorpel comes back as a borstweringdorpel, and no
        // glass pane sits at the full binnenmaat.
        let mut k = Kozijn::new("Test", "MM2", 1400.0, 1900.0);
        k.layout = Some(crate::layout::melkmeisje1());
        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width; // 67
        let inner_w = k.inner_width(); // 1266
        let inner_h = k.inner_height(); // 1766

        let stijlen: Vec<_> = prod.cut_list.iter()
            .filter(|c| c.member_type == MemberType::DividerV)
            .collect();
        assert_eq!(stijlen.len(), 1, "melkmeisje layout yields exactly one tussenstijl");
        assert!((stijlen[0].net_length_mm - inner_h).abs() < 0.01,
            "tussenstijl {} must run the full binnenhoogte {}",
            stijlen[0].net_length_mm, inner_h);

        // Exactly one borstweringdorpel (DividerH): the side-light zone width
        // (melkmeisje1 splits 900/500 over the available 1199 mm) extended by
        // fw on the right where the outer stijl stops op deze dorpel.
        let dorpels: Vec<_> = prod.cut_list.iter()
            .filter(|c| c.member_type == MemberType::DividerH)
            .collect();
        assert_eq!(dorpels.len(), 1, "melkmeisje layout yields exactly one borstweringdorpel");
        let zijlicht_w = (inner_w - fw) * 500.0 / 1400.0; // 428.214…
        assert!((dorpels[0].net_length_mm - (zijlicht_w + fw)).abs() < 0.01,
            "borstweringdorpel {} must be zijlicht width {} + fw {}",
            dorpels[0].net_length_mm, zijlicht_w, fw);

        // Sum of members covers the stepped bottom outline: onderdorpel
        // (under the deur, incl. corners) + borstweringdorpel = outer width.
        let bottom = find(&prod, MemberType::FrameBottom);
        assert!((bottom.net_length_mm + dorpels[0].net_length_mm - k.frame.outer_width).abs() < 0.01,
            "onderdorpel {} + borstweringdorpel {} must span the outer width {}",
            bottom.net_length_mm, dorpels[0].net_length_mm, k.frame.outer_width);

        // Two panes (deur + zijlicht), none at the full binnenmaat.
        assert_eq!(prod.glass_list.len(), 2);
        for g in &prod.glass_list {
            assert!(g.width_mm < inner_w - 1.0,
                "glas {} mag niet de volle binnenbreedte {} zijn", g.width_mm, inner_w);
            assert!(g.height_mm < inner_h - 1.0,
                "glas {} mag niet de volle binnenhoogte {} zijn", g.height_mm, inner_h);
        }
        // Buiten leaf yields nothing: no third pane, no panel.
        assert!(prod.panel_list.is_empty());
    }

    #[test]
    fn test_layout_vulling_types_flow_into_production() {
        // (opdracht 2b) Vulling types from the split tree come through: a
        // raam yields raamhout members + glass, a deur yields deurhout
        // members + glass, a paneel yields a panel item.
        use crate::layout::{deur, paneel, raam, split_row, VakChild};
        let mut k = Kozijn::new("Test", "LT1", 2400.0, 1500.0);
        k.layout = Some(split_row(vec![
            VakChild { size: 1.0, node: raam("draaikiep") },
            VakChild { size: 1.0, node: deur("enkel") },
            VakChild { size: 1.0, node: paneel() },
        ]));
        let prod = compute_production_data(&k);

        // Raamhout members for the raam vak (leaf 0).
        let raamhout: Vec<_> = prod.cut_list.iter()
            .filter(|c| c.profile_name.starts_with("Raamhout"))
            .collect();
        assert_eq!(raamhout.len(), 4, "raam vak yields 4 raamhout members");
        assert!(raamhout.iter().any(|c| c.piece_id.ends_with("-RSL")));

        // Deurhout members for the deur vak (leaf 1) — the deur-item.
        let deurhout: Vec<_> = prod.cut_list.iter()
            .filter(|c| c.profile_name.starts_with("Deurhout"))
            .collect();
        assert_eq!(deurhout.len(), 4, "deur vak yields 4 deurhout members");
        assert!(deurhout.iter().any(|c| c.piece_id.ends_with("-DSL")));
        // Deurhout = kozijnhoutbreedte: vleugelstijl loopt de volle vakhoogte
        // (hout: vleugelstijlen doorlopend).
        let vak_h = k.inner_height();
        assert!((deurhout[0].net_length_mm - vak_h).abs() < 0.5);

        // Glass for raam + deur (not for the paneel), panel item for paneel.
        assert_eq!(prod.glass_list.len(), 2);
        assert_eq!(prod.panel_list.len(), 1);
        assert_eq!(prod.panel_list[0].cell_index, 2);
        assert_eq!(prod.panel_list[0].panel_type, "Sandwichpaneel");
        // cell_index follows the stable depth-first leaf index.
        let glass_cells: Vec<usize> = prod.glass_list.iter().map(|g| g.cell_index).collect();
        assert_eq!(glass_cells, vec![0, 1]);
    }

    #[test]
    fn test_glaslat_lengths_use_material_norm_when_snapshot_empty() {
        // Empty snapshot on a wood kozijn resolves to the 17 mm KVT norm —
        // identical to an explicit 17 — so imported profiles without
        // sponning data still produce plausible bead lengths.
        let mut k = Kozijn::new("Test", "T11", 900.0, 1400.0);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default());
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot::default());

        let gw = 900.0 - 2.0 * 67.0;
        let gh = 1400.0 - 2.0 * 67.0;
        let expected = 2.0 * (gw + 34.0 - 1.0) + 2.0 * (gh + 34.0 - 1.0);
        let prod = compute_production_data(&k);
        assert!((prod.glaslat_list[0].total_length_mm - expected).abs() < 0.01);
    }

    #[test]
    fn test_configured_default_joints_honored_on_aluminum() {
        // (fix 3) A user who deliberately saves the {pen_slis, stijl, 90, 20}
        // set — the same values as the auto-populated default — must be
        // honored once joints_configured is set: the alu kozijn gets
        // stijl-through square corners instead of the material verstek.
        let mut k = Kozijn::new("Test", "T16", 900.0, 1400.0);
        k.frame.material = Material::Aluminum;
        assert_eq!(k.frame.corner_joints.len(), 4); // auto-populated defaults
        k.frame.joints_configured = true;

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;
        // Stijlen doorlopend: full outer height, square cuts, geen verstek.
        for mt in [MemberType::FrameLeft, MemberType::FrameRight] {
            let stile = find(&prod, mt);
            assert!((stile.net_length_mm - k.frame.outer_height).abs() < 0.01);
            assert!((stile.miter_left_deg - 90.0).abs() < 0.01);
            assert!((stile.miter_right_deg - 90.0).abs() < 0.01);
        }
        // Dorpels tussen de stijlen, pen allowance on both ends.
        for mt in [MemberType::FrameTop, MemberType::FrameBottom] {
            let rail = find(&prod, mt);
            assert!((rail.net_length_mm - (k.frame.outer_width - 2.0 * fw)).abs() < 0.01);
            assert!((rail.miter_left_deg - 90.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_layout_single_glas_emits_glaslat_like_grid_path() {
        // (fix 4) A 1-vaks layout with a cell glaslat emits exactly the item
        // the grid path would: same KVT length, position, spec and quantity.
        let mut k = Kozijn::new("Test", "GL1", 900.0, 1400.0);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default());
        let k_grid = k.clone();
        assert!(k_grid.layout.is_none());
        k.layout = Some(crate::layout::glas());

        let prod_layout = compute_production_data(&k);
        let prod_grid = compute_production_data(&k_grid);
        assert_eq!(prod_layout.glaslat_list.len(), 1);
        assert_eq!(prod_grid.glaslat_list.len(), 1);
        let a = &prod_layout.glaslat_list[0];
        let b = &prod_grid.glaslat_list[0];
        assert!((a.total_length_mm - b.total_length_mm).abs() < 0.01,
            "layout bead length {} must match grid path {}",
            a.total_length_mm, b.total_length_mm);
        assert_eq!(a.position, b.position);
        assert_eq!(a.mitered, b.mitered);
        assert_eq!(a.quantity, 4);

        // Same equality with a profile snapshot (KVT 12.3.2 branch).
        let mut k_snap = k.clone();
        k_snap.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });
        let mut k_snap_grid = k_snap.clone();
        k_snap_grid.layout = None;
        let prod_snap = compute_production_data(&k_snap);
        let prod_snap_grid = compute_production_data(&k_snap_grid);
        assert!((prod_snap.glaslat_list[0].total_length_mm
            - prod_snap_grid.glaslat_list[0].total_length_mm).abs() < 0.01);
    }

    #[test]
    fn test_layout_vulling_glaslat_falls_back_to_default_spec() {
        // (fix 4) Without a cell spec the canvas vulling ("binnen"/"buiten")
        // still yields a bead item with the default 15×17 latspec.
        let mut k = Kozijn::new("Test", "GL2", 900.0, 1400.0);
        assert!(k.cells[0].glaslat.is_none());
        k.layout = Some(crate::layout::leaf(crate::layout::Vakvulling::Glas {
            glaslat: Some("buiten".into()),
        }));
        let prod = compute_production_data(&k);
        assert_eq!(prod.glaslat_list.len(), 1);
        let item = &prod.glaslat_list[0];
        assert_eq!(item.position, "Buiten");
        assert!((item.width_mm - 15.0).abs() < 1e-9);
        assert!((item.height_mm - 17.0).abs() < 1e-9);
        assert_eq!(item.quantity, 4);
    }

    #[test]
    fn test_layout_misaligned_cells_fall_back_to_defaults() {
        // (fix 5) Cell data is only leaf-index-aligned when the leaf count
        // (incl. `Buiten`) equals the cell count; otherwise every leaf uses
        // the defaults instead of picking up an unrelated cell's data.
        use crate::layout::{glas, split_row, VakChild};
        let mut k = Kozijn::new("Test", "LG1", 1200.0, 1500.0);
        k.cells[0].glazing.glass_type = "Testglas-misaligned".into();
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default());
        // 2 glass leaves vs 1 cell → misaligned.
        k.layout = Some(split_row(vec![
            VakChild { size: 1.0, node: glas() },
            VakChild { size: 1.0, node: glas() },
        ]));
        let prod = compute_production_data(&k);
        let default_type = crate::kozijn::Glazing::default().glass_type;
        assert_eq!(prod.glass_list.len(), 2);
        for g in &prod.glass_list {
            assert_eq!(g.glass_type, default_type,
                "misaligned cell data must not leak into layout leaves");
        }
        // The cell's glaslat is not picked up either (no vulling fallback set).
        assert!(prod.glaslat_list.is_empty());
    }
}
