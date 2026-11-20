use serde::{Deserialize, Serialize};

use crate::curved_contour::{Contour, SashLeaf, Segment};
use crate::curved_timber::{
    ArchJoint, CurvedTimberMember, DividerTenon, FrameMemberRef, TenonIssue, TenonReceiver,
    TimberArc, TimberProfileSide, TimberRole,
};
use crate::glazing_fit::compatible_profile;
use crate::joint::{effective_corner_joints, Joint, JointType, ThroughMember};
use crate::kozijn::{Kozijn, Material, PanelType};
use crate::layout::{intersect_interval_unions, Vakvulling};
mod system;

// ── Cut list constants ─────────────────────────────────────────────

const WOOD_PEN_ALLOWANCE_MM: f64 = 20.0; // tenon allowance per side
const SAW_KERF_MM: f64 = 4.0; // saw blade width
const PVC_WELD_OVERMEASURE_MM: f64 = 4.0; // weld overmeasure per side
const GLASS_CLEARANCE_MM: f64 = 4.0; // legacy glass/panel clearance per side (kozijnen zonder profielsnapshot; panelen altijd)
const GASKET_OVERLAP_MM: f64 = 20.0; // gasket overlap at corners

// ── Data structures ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionData {
    pub kozijn_mark: String,
    pub kozijn_name: String,
    pub cut_list: Vec<CutListItem>,
    /// One entry per wooden divider end, with both workshop position codes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub divider_joints: Vec<DividerJointItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arch_joints: Vec<ArchJointItem>,
    pub glass_list: Vec<GlassListItem>,
    pub hardware_list: Vec<HardwareListItem>,
    pub gasket_list: Vec<GasketListItem>,
    pub panel_list: Vec<PanelListItem>,
    pub glaslat_list: Vec<GlaslatListItem>,
    pub bom: Vec<BomItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DividerJointEnd {
    Start,
    End,
}

impl DividerJointEnd {
    pub fn label_nl(self) -> &'static str {
        match self {
            Self::Start => "Begin",
            Self::End => "Eind",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchJointItem {
    pub piece_id: String,
    pub end: DividerJointEnd,
    pub receiver_piece_id: String,
    pub receiver_end: DividerJointEnd,
    #[serde(flatten)]
    pub joint: ArchJoint,
}

impl ArchJointItem {
    pub fn side_nl(&self) -> &'static str {
        if self.joint.corner == 0 {
            "Links"
        } else {
            "Rechts"
        }
    }
}

pub(crate) fn z_band_text(bands: &[[f64; 2]]) -> String {
    bands
        .iter()
        .map(|band| format!("{:.1}-{:.1}", band[0], band[1]))
        .collect::<Vec<_>>()
        .join(" / ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DividerJointItem {
    pub piece_id: String,
    pub end: DividerJointEnd,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver_piece_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenon: Option<DividerTenon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<TenonIssue>,
}

impl DividerJointItem {
    pub(crate) fn z_bands(&self) -> String {
        self.tenon
            .as_ref()
            .map(|t| z_band_text(&t.z_from_inside_mm))
            .unwrap_or_default()
    }

    pub fn issue_text(&self) -> String {
        self.issue
            .map(|reason| reason.to_string())
            .unwrap_or_default()
    }
}

/// Index by actual cut position so exports can show both sides of every joint.
#[derive(Default)]
pub(crate) struct CutJoinery<'a> {
    pub ends: [Option<&'a DividerJointItem>; 2],
    pub mortises: Vec<&'a DividerJointItem>,
}

impl CutJoinery<'_> {
    pub fn received_text(&self) -> String {
        self.mortises
            .iter()
            .filter_map(|j| {
                j.tenon.as_ref().map(|t| {
                    format!(
                        "{} {} ({} x {:.1} mm)",
                        j.piece_id,
                        j.end.label_nl(),
                        t.z_from_inside_mm.len(),
                        t.mortise_depth_mm
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

impl ProductionData {
    pub fn blank_length_mm(&self, cut: &CutListItem) -> Option<f64> {
        let unresolved = self
            .arch_joints
            .iter()
            .any(|row| row.piece_id == cut.piece_id && row.joint.issue.is_some());
        (!unresolved).then_some(cut.gross_length_mm)
    }

    pub(crate) fn arch_joint_text(
        &self,
        piece_id: &str,
        end: DividerJointEnd,
        kind: crate::curved_timber::ArchJointKind,
    ) -> Option<String> {
        self.arch_joints
            .iter()
            .filter(|row| row.joint.kind == kind)
            .find_map(|row| {
                let (part, partner, partner_end, pen) =
                    if row.piece_id == piece_id && row.end == end {
                        ("Pen", &row.receiver_piece_id, row.receiver_end, true)
                    } else if row.receiver_piece_id == piece_id && row.receiver_end == end {
                        ("Gat", &row.piece_id, row.end, false)
                    } else {
                        return None;
                    };
                let measure = row
                    .joint
                    .tenon
                    .as_ref()
                    .map(|t| {
                        format!(
                            "{:.1} mm",
                            if pen {
                                t.tenon_length_mm
                            } else {
                                t.mortise_depth_mm
                            }
                        )
                    })
                    .or_else(|| row.joint.issue.map(|issue| issue.to_string()))
                    .unwrap_or_default();
                Some(format!(
                    "{part} {measure} / {partner} {}",
                    partner_end.label_nl()
                ))
            })
    }

    pub(crate) fn joint_index(&self) -> std::collections::HashMap<&str, CutJoinery<'_>> {
        let mut index = std::collections::HashMap::<&str, CutJoinery>::new();
        for joint in &self.divider_joints {
            let side = usize::from(joint.end == DividerJointEnd::End);
            index.entry(&joint.piece_id).or_default().ends[side] = Some(joint);
            if let Some(receiver) = &joint.receiver_piece_id {
                index.entry(receiver).or_default().mortises.push(joint);
            }
        }
        index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberType {
    PerimeterStop,
    FrameArc,
    FrameTop,
    FrameBottom,
    FrameLeft,
    FrameRight,
    DividerH,
    DividerV,
    SashArc,
    SashTop,
    SashBottom,
    SashLeft,
    SashRight,
}

impl MemberType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::PerimeterStop => "Aanslagprofiel",
            Self::FrameArc => "Boogstuk",
            Self::FrameTop => "Bovendorpel",
            Self::FrameBottom => "Onderdorpel",
            Self::FrameLeft => "Stijl links",
            Self::FrameRight => "Stijl rechts",
            Self::DividerH => "Tussendorpel",
            Self::DividerV => "Tussenstijl",
            Self::SashArc => "Vleugelboog",
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
    /// Finished outer and centre-line dimensions of a circular member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<TimberArc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_side: Option<TimberProfileSide>,
    /// A circular joint surface replaces the planar end cut at this end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_cope_radius_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_cope_radius_mm: Option<f64>,
    /// Angles between the sides of a shaped divider are chord references.
    /// The contour and edge setbacks specify the actual finished end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divider_start: Option<crate::curved_timber::DividerEnd>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divider_end: Option<crate::curved_timber::DividerEnd>,
    /// Masonry cutouts in the elevation plane of the entire frame (mm).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub masonry_notches: Vec<crate::geometry::Rect2D>,
}

impl CutListItem {
    pub(crate) fn has_curved_cut(&self) -> bool {
        self.arc.is_some()
            || self.divider_start.is_some()
            || self.start_cope_radius_mm.is_some()
            || self.end_cope_radius_mm.is_some()
    }

    pub fn member_label_nl(&self) -> &'static str {
        match (self.member_type, self.profile_side) {
            (MemberType::FrameArc, Some(TimberProfileSide::Top)) => "Boogstuk boven",
            (MemberType::FrameArc, Some(TimberProfileSide::Bottom)) => "Boogstuk onder",
            (MemberType::SashArc, Some(TimberProfileSide::Top)) => "Vleugelboog boven",
            (MemberType::SashArc, Some(TimberProfileSide::Bottom)) => "Vleugelboog onder",
            (MemberType::SashArc, Some(TimberProfileSide::Left)) => "Vleugelboog links",
            (MemberType::SashArc, Some(TimberProfileSide::Right)) => "Vleugelboog rechts",
            _ => self.member_type.label_nl(),
        }
    }

    pub fn end_text(angle: f64, radius: Option<f64>, decimals: usize) -> String {
        match radius {
            Some(r) => format!("{angle:.decimals$} / R{r:.1}"),
            None => format!("{angle:.decimals$}"),
        }
    }

    pub fn notch_text(r: &crate::geometry::Rect2D) -> String {
        format!(
            "x {:.1}, y {:.1}, b {:.1}, h {:.1}",
            r.x, r.y, r.width, r.height
        )
    }
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
    /// False when the selected system article combination or glass thickness
    /// fails a verified product limit; the pane remains visible for correction.
    #[serde(default = "glass_ready_default")]
    pub production_ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production_issue: Option<String>,
    /// Finished cut outline in mm, relative to the top-left of the pane's
    /// bounding box (x right, y down). Rectangular panes need only width/height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut_contour: Option<Contour>,
}

fn glass_ready_default() -> bool {
    true
}

impl GlassListItem {
    pub fn dimension_text(&self, mm: f64) -> String {
        if self.cut_contour.is_some() {
            format!("{mm:.1}")
        } else {
            format!("{mm:.0}")
        }
    }

    pub fn shape_label_nl(&self) -> String {
        let Some(contour) = &self.cut_contour else {
            return "Rechthoek".into();
        };
        if contour
            .segments
            .iter()
            .all(|e| matches!(e, Segment::Arc { .. }))
        {
            return format!("Rond Ø{:.1}", self.width_mm);
        }
        contour
            .segments
            .iter()
            .find_map(|e| match e {
                Segment::Arc { radius, .. } => Some(format!("Boog R{radius:.1}")),
                _ => None,
            })
            .unwrap_or_else(|| "Veelhoek".into())
    }

    /// Ordered cutting instructions in the same local x-right, y-down plane as
    /// the drawing. Including both endpoints distinguishes mirrored panes.
    pub fn cut_steps(&self) -> Vec<String> {
        let Some(cut) = &self.cut_contour else {
            return Vec::new();
        };
        cut.segments.iter().map(cut_step).collect()
    }
}

pub(crate) fn cut_step(segment: &Segment) -> String {
    let a = segment.from();
    let b = segment.to();
    match segment {
        Segment::Line { .. } => format!("L ({:.1},{:.1}) -> ({:.1},{:.1})", a[0], a[1], b[0], b[1]),
        Segment::Arc {
            center,
            radius,
            sweep_angle,
            ..
        } => format!(
            "R{radius:.1} M({:.1},{:.1}) ({:.1},{:.1}) -> ({:.1},{:.1}) {:.2}°",
            center[0],
            center[1],
            a[0],
            a[1],
            b[0],
            b[1],
            sweep_angle.to_degrees(),
        ),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareListItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piece_id: Option<String>,
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
    ClosingInner,
    ClosingCentre,
    ClosingOuter,
    ClosingSingle,
    ClosingUnspecified,
}

impl GasketType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::GlazingInner => "Binnenrubber beglazing",
            Self::GlazingOuter => "Buitenrubber beglazing",
            Self::SashSeal => "Vleugelafdichting",
            Self::FrameSeal => "Kozijnafdichting",
            Self::ClosingInner => "Binnenaanslag",
            Self::ClosingCentre => "Middendichting",
            Self::ClosingOuter => "Buitenaanslag",
            Self::ClosingSingle => "Aanslagdichting",
            Self::ClosingUnspecified => "Sluitdichting volgens systeemopgave",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasketListItem {
    pub gasket_type: GasketType,
    pub length_mm: f64,
    pub quantity: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<SealReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealReference {
    pub piece_id: String,
    pub profile_name: String,
    pub line: SealMeasureLine,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SealMeasureLine {
    GlazingDay,
    BeadOpening,
    SashOuter,
}

impl SealMeasureLine {
    pub fn label_nl(self) -> &'static str {
        match self {
            Self::GlazingDay => "Dagmaat vaste glasaanslag",
            Self::BeadOpening => "Binnenomtrek glaslat",
            Self::SashOuter => "Buitenomtrek vleugel",
        }
    }
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
    /// Combined cut length for rectangular stock, or net developed length
    /// along the outside of the bead for a shaped opening (mm).
    pub total_length_mm: f64,
    pub mitered: bool,
    pub quantity: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contour_runs: Vec<BeadRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeadRun {
    pub length_mm: f64,
    /// Outside bending radius. Straight lengths have no radius.
    pub radius_mm: Option<f64>,
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
        Material::Pvc => net + 2.0 * PVC_WELD_OVERMEASURE_MM + SAW_KERF_MM,
        Material::WoodAluminum => {
            // Wood rules for the wood part
            net + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM
        }
    }
}

fn divider_gross(net: f64, material: &Material) -> f64 {
    // A mechanical T-junction has square butt ends; only the saw kerf consumes stock.
    if material.is_profile_system() {
        net + SAW_KERF_MM
    } else {
        gross_length(net, material, false)
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

/// Total for all four beads. Square-cut verticals fit between the horizontals.
fn glaslat_total_length(fit: &crate::glazing_fit::GlazingFit) -> f64 {
    let horizontal = (fit.rebate.width - 1.0).max(0.0);
    let end_reduction = if fit.bead_mitered {
        0.0
    } else {
        2.0 * fit.bead_height
    };
    let vertical = (fit.rebate.height - end_reduction - 1.0).max(0.0);
    2.0 * (horizontal + vertical)
}

fn is_rectangular_contour(contour: &Contour) -> bool {
    contour.segments.iter().all(|s| {
        matches!(s, Segment::Line { from, to }
        if (from[0] - to[0]).abs() < 1e-7 || (from[1] - to[1]).abs() < 1e-7)
    })
}

fn piece_cut_contour(contour: &Contour) -> Option<Contour> {
    if is_rectangular_contour(contour) {
        return None;
    }
    let bounds = contour.bounds();
    let local = |p: &mut [f64; 2]| {
        p[0] -= bounds.x;
        p[1] -= bounds.y;
    };
    let mut cut = contour.clone();
    for edge in &mut cut.segments {
        match edge {
            Segment::Line { from, to } => {
                local(from);
                local(to);
            }
            Segment::Arc {
                from, to, center, ..
            } => {
                local(from);
                local(to);
                local(center);
            }
        }
    }
    Some(cut)
}

fn bead_runs(contour: &Contour) -> Vec<BeadRun> {
    if is_rectangular_contour(contour) {
        return Vec::new();
    }
    // The outline is cut from a single circle and straight clipping edges.
    // Split arcs at the circle's parameter seam form one continuous bend.
    let mut runs: Vec<BeadRun> = Vec::new();
    for edge in &contour.segments {
        let radius_mm = match edge {
            Segment::Arc { radius, .. } => Some(*radius),
            _ => None,
        };
        if let Some(last) = runs.last_mut() {
            if radius_mm.is_some() && last.radius_mm == radius_mm {
                last.length_mm += edge.length();
                continue;
            }
        }
        runs.push(BeadRun {
            length_mm: edge.length(),
            radius_mm,
        });
    }
    if runs.len() > 1
        && runs[0].radius_mm.is_some()
        && runs[0].radius_mm == runs.last().unwrap().radius_mm
    {
        let end = runs.pop().unwrap();
        runs[0].length_mm += end.length_mm;
    }
    runs
}

/// Timber sash stiles run through; mitred systems measure net lengths
/// between short points. Both start from the assembled leaf's outer size.
#[allow(clippy::too_many_arguments)]
fn push_sash_members(
    cut_list: &mut Vec<CutListItem>,
    cell_id: &str,
    id_prefix: &str,
    sash_profile_name: &str,
    material: &Material,
    material_name: &str,
    is_miter: bool,
    angle: f64,
    outer_width: f64,
    outer_height: f64,
    sash_w: f64,
) {
    let stile_net = if is_miter {
        (outer_height - 2.0 * sash_w).max(0.0)
    } else {
        outer_height
    };
    let rail_net = (outer_width - 2.0 * sash_w).max(0.0);
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
            arc: None,
            profile_side: None,
            start_cope_radius_mm: None,
            end_cope_radius_mm: None,
            divider_start: None,
            divider_end: None,
            masonry_notches: Vec::new(),
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
        crate::layout::LayoutRect {
            x: fw,
            y: fw,
            width: ow - 2.0 * fw,
            height: oh - 2.0 * fw,
        },
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
    let span =
        |acc: Option<(f64, f64)>, full: f64| acc.map_or(full, |(lo, hi)| (hi - lo) + 2.0 * fw);
    (
        span(left, oh),
        span(right, oh),
        span(top, ow),
        span(bottom, ow),
    )
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

fn system_stock_name(
    frame: &crate::kozijn::Frame,
    material: &Material,
    stock: Option<&crate::profile::ProfileRef>,
) -> String {
    stock
        .filter(|p| compatible_profile(&p.id, material))
        .or_else(|| compatible_profile(&frame.profile.id, material).then_some(&frame.profile))
        .map_or_else(
            || {
                format!(
                    "{} {}x{} mm",
                    material_name(material),
                    frame.frame_width,
                    frame.frame_depth
                )
            },
            |p| p.name.clone(),
        )
}

/// Both fixed and moving timber are cut from the same finished faces used in the elevation.
fn formed_timber_cut(
    member: &CurvedTimberMember,
    assembly: &str,
    suffix: &str,
    member_type: MemberType,
    profile: &str,
    material: &Material,
    gross_length_mm: f64,
) -> CutListItem {
    let piece_id = if member.arc.is_some() {
        format!("{assembly}-{suffix}{}", member.index + 1)
    } else if member.index > 0 {
        format!("{assembly}-{suffix}-{}", member.index + 1)
    } else {
        format!("{assembly}-{suffix}")
    };
    CutListItem {
        piece_id,
        member_type,
        profile_name: profile.into(),
        material: material_name(material).into(),
        net_length_mm: member.net_length_mm,
        gross_length_mm,
        miter_left_deg: member.start_angle_deg,
        miter_right_deg: member.end_angle_deg,
        quantity: 1,
        arc: member.arc.clone(),
        profile_side: member.arc.as_ref().map(|_| member.profile_side),
        start_cope_radius_mm: None,
        end_cope_radius_mm: None,
        divider_start: None,
        divider_end: None,
        masonry_notches: member.masonry_notches.clone(),
    }
}

fn curved_sash_cut(
    member: &CurvedTimberMember,
    assembly: &str,
    prefix: &str,
    profile: &str,
    material: &Material,
) -> CutListItem {
    let (kind, position) = match member.role {
        TimberRole::Arc => (MemberType::SashArc, "BO"),
        TimberRole::Top => (MemberType::SashTop, "DB"),
        TimberRole::Bottom => (MemberType::SashBottom, "DO"),
        TimberRole::Left => (MemberType::SashLeft, "SL"),
        TimberRole::Right => (MemberType::SashRight, "SR"),
    };
    let allowance = match material {
        Material::Wood(_) | Material::WoodAluminum => {
            let tenons = u8::from(member.start_tenon) + u8::from(member.end_tenon);
            f64::from(tenons) * WOOD_PEN_ALLOWANCE_MM
        }
        Material::Pvc => 2.0 * PVC_WELD_OVERMEASURE_MM,
        Material::Aluminum => 0.0,
    };
    let gross = member.net_length_mm + allowance + SAW_KERF_MM;
    formed_timber_cut(
        member,
        assembly,
        &format!("{prefix}{position}"),
        kind,
        profile,
        material,
        gross,
    )
}

/// Curved stock is dimensioned on the finished outer arc. Its gross value is
/// developed length plus the joint allowances and one kerf, in millimetres.
fn curved_frame_cuts(
    kozijn: &Kozijn,
    geometry: &crate::geometry::KozijnGeometry2D,
) -> Vec<CutListItem> {
    let frame = &kozijn.frame;
    let joints = effective_corner_joints(&frame.corner_joints, frame.joints_configured);
    let round = frame.shape.shape_type == crate::kozijn::ShapeType::Round;
    geometry
        .curved_timber
        .iter()
        .map(|member| {
            let (kind, suffix, profile, material, orientation) = match member.role {
                TimberRole::Top => (
                    MemberType::FrameTop,
                    "DB",
                    &frame.top_profile,
                    &frame.top_material,
                    FrameOrientation::Dorpel,
                ),
                TimberRole::Left => (
                    MemberType::FrameLeft,
                    "SL",
                    &frame.left_profile,
                    &frame.left_material,
                    FrameOrientation::Stijl,
                ),
                TimberRole::Right => (
                    MemberType::FrameRight,
                    "SR",
                    &frame.right_profile,
                    &frame.right_material,
                    FrameOrientation::Stijl,
                ),
                TimberRole::Bottom => (
                    MemberType::FrameBottom,
                    "DO",
                    &frame.bottom_profile,
                    &frame.bottom_material,
                    FrameOrientation::Dorpel,
                ),
                TimberRole::Arc if matches!(member.profile_side, TimberProfileSide::Bottom) => (
                    MemberType::FrameArc,
                    "BO",
                    &frame.bottom_profile,
                    &frame.bottom_material,
                    FrameOrientation::Dorpel,
                ),
                TimberRole::Arc => (
                    MemberType::FrameArc,
                    "BO",
                    &frame.top_profile,
                    &frame.top_material,
                    FrameOrientation::Dorpel,
                ),
            };
            let material = material.as_ref().unwrap_or(&frame.material);
            let fallback_profile = if kind == MemberType::FrameBottom {
                frame.sill_profile.as_ref()
            } else {
                None
            };
            let profile = profile.as_ref().or(fallback_profile);
            let stock_name = system_stock_name(frame, material, profile);
            // Each tongue belongs to one physical blank. Its developed extension
            // already covers every Z-band, including a double mortise and tenon.
            let extension: f64 = geometry
                .arch_joints
                .iter()
                .filter(|j| j.source.role == member.role && j.source.index == member.index)
                .filter_map(|j| j.tenon.as_ref())
                .map(|t| t.blank_extension_mm)
                .sum();
            let mut gross = member.net_length_mm + extension + SAW_KERF_MM;
            for corner in [member.start_corner, member.end_corner]
                .into_iter()
                .flatten()
            {
                if round {
                    if matches!(material, Material::Pvc) {
                        gross += PVC_WELD_OVERMEASURE_MM;
                    }
                    continue;
                }
                if corner < 2 && matches!(material, Material::Wood(_) | Material::WoodAluminum) {
                    continue;
                }
                // Net already reaches the long point, so there is no extra miter allowance.
                gross += if let Some(joints) = joints {
                    joint_end_allowance(orientation, joints[corner], material, 0.0)
                } else {
                    match (material, orientation) {
                        (Material::Wood(_) | Material::WoodAluminum, FrameOrientation::Stijl) => {
                            WOOD_PEN_ALLOWANCE_MM
                        }
                        (Material::Pvc, _) => PVC_WELD_OVERMEASURE_MM,
                        _ => 0.0,
                    }
                };
            }
            formed_timber_cut(
                member,
                &kozijn.mark,
                suffix,
                kind,
                &stock_name,
                material,
                gross,
            )
        })
        .collect()
}

fn arch_joints(
    geometry: &crate::geometry::KozijnGeometry2D,
    cuts: &[CutListItem],
) -> Vec<ArchJointItem> {
    let mut result = Vec::new();
    for joint in &geometry.arch_joints {
        let find_end = |part: &FrameMemberRef| {
            let (piece, cut) = geometry
                .curved_timber
                .iter()
                .zip(cuts)
                .find(|(p, _)| p.role == part.role && p.index == part.index)?;
            let end = if piece.start_corner == Some(joint.corner) {
                DividerJointEnd::Start
            } else if piece.end_corner == Some(joint.corner) {
                DividerJointEnd::End
            } else {
                return None;
            };
            Some((cut.piece_id.clone(), end))
        };
        if let (Some((piece_id, end)), Some((receiver_piece_id, receiver_end))) =
            (find_end(&joint.source), find_end(&joint.receiver))
        {
            result.push(ArchJointItem {
                piece_id,
                end,
                receiver_piece_id,
                receiver_end,
                joint: joint.clone(),
            });
        }
    }
    result
}

fn curved_divider_cuts(
    kozijn: &Kozijn,
    geometry: &crate::geometry::KozijnGeometry2D,
) -> Vec<CutListItem> {
    let members = &geometry.curved_divider_timber;
    let mut source_counts = std::collections::HashMap::new();
    for member in members {
        *source_counts
            .entry((member.vertical, member.source_index))
            .or_insert(0) += 1;
    }
    members
        .iter()
        .map(|member| {
            let (kind, suffix, divisions) = if member.vertical {
                (MemberType::DividerV, "TS", &kozijn.grid.columns)
            } else {
                (MemberType::DividerH, "TD", &kozijn.grid.rows)
            };
            let grid = kozijn.layout.is_none();
            let slot = if grid {
                member.source_index + 1
            } else {
                member.source_index
            };
            let profile = if grid {
                divisions.get(slot).and_then(|d| d.divider_profile.as_ref())
            } else {
                None
            };
            let mut id = format!("{}-{suffix}{slot}", kozijn.mark);
            if source_counts[&(member.vertical, member.source_index)] > 1 {
                id.push_str(&format!(".{}", member.piece_index + 1));
            }
            CutListItem {
                piece_id: id,
                member_type: kind,
                profile_name: system_stock_name(&kozijn.frame, &kozijn.frame.material, profile),
                material: material_name(&kozijn.frame.material).into(),
                net_length_mm: member.net_length_mm,
                gross_length_mm: if matches!(
                    kozijn.frame.material,
                    Material::Wood(_) | Material::WoodAluminum
                ) {
                    member.net_length_mm
                        + SAW_KERF_MM
                        + member
                            .start
                            .tenon
                            .as_ref()
                            .map_or(0.0, |t| t.blank_extension_mm)
                        + member
                            .end
                            .tenon
                            .as_ref()
                            .map_or(0.0, |t| t.blank_extension_mm)
                } else {
                    divider_gross(member.net_length_mm, &kozijn.frame.material)
                },
                miter_left_deg: member.start.angle_deg,
                miter_right_deg: member.end.angle_deg,
                start_cope_radius_mm: member.start.radius_mm,
                end_cope_radius_mm: member.end.radius_mm,
                divider_start: Some(member.start.clone()),
                divider_end: Some(member.end.clone()),
                quantity: 1,
                arc: None,
                profile_side: None,
                masonry_notches: Vec::new(),
            }
        })
        .collect()
}

fn divider_joints(
    geometry: &crate::geometry::KozijnGeometry2D,
    cuts: &[CutListItem],
) -> Vec<DividerJointItem> {
    if geometry.curved_divider_timber.is_empty() {
        return vec![];
    }
    // Both functions emitting these cut rows follow the core's piece order.
    // Reuse the resulting IDs, including suffixes for splits and free layouts.
    let (frame_rows, divider_rows) = cuts.split_at(geometry.curved_timber.len());
    let frame = geometry.curved_timber.iter().zip(frame_rows);
    let dividers = geometry.curved_divider_timber.iter().zip(divider_rows);
    let mut joints = Vec::new();
    for (piece, cut) in dividers.clone() {
        for (end, shoulder) in [
            (DividerJointEnd::Start, &piece.start),
            (DividerJointEnd::End, &piece.end),
        ] {
            if shoulder.tenon.is_none() && shoulder.tenon_issue.is_none() {
                continue;
            }
            let receiver = shoulder.tenon.as_ref().and_then(|t| match &t.receiver {
                TenonReceiver::Frame { role, index } => frame
                    .clone()
                    .find(|(p, _)| p.role == *role && p.index == *index)
                    .map(|(_, c)| &c.piece_id),
                TenonReceiver::Divider {
                    vertical,
                    source_index,
                    piece_index,
                } => dividers
                    .clone()
                    .find(|(p, _)| {
                        (p.vertical, p.source_index, p.piece_index)
                            == (*vertical, *source_index, *piece_index)
                    })
                    .map(|(_, c)| &c.piece_id),
            });
            joints.push(DividerJointItem {
                piece_id: cut.piece_id.clone(),
                end,
                receiver_piece_id: receiver.cloned(),
                tenon: shoulder.tenon.clone(),
                issue: shoulder.tenon_issue.clone(),
            });
        }
    }
    joints
}

fn rectangular_frame_cuts(kozijn: &Kozijn, profile_name: &str) -> Vec<CutListItem> {
    let frame = &kozijn.frame;
    let fw = frame.frame_width;
    let mat = &frame.material;
    let corners = effective_corner_joints(&frame.corner_joints, frame.joints_configured);
    let (left, right, top, bottom) = stepped_frame_spans(kozijn, fw);
    let rows = [
        (
            MemberType::FrameLeft,
            "SL",
            FrameOrientation::Stijl,
            left,
            0,
            2,
        ),
        (
            MemberType::FrameRight,
            "SR",
            FrameOrientation::Stijl,
            right,
            1,
            3,
        ),
        (
            MemberType::FrameTop,
            "DB",
            FrameOrientation::Dorpel,
            top,
            0,
            1,
        ),
        (
            MemberType::FrameBottom,
            "DO",
            FrameOrientation::Dorpel,
            bottom,
            2,
            3,
        ),
    ];
    rows.into_iter()
        .map(|(member_type, suffix, orient, span, start, end)| {
            let (own_profile, own_material) = match member_type {
                MemberType::FrameLeft => (&frame.left_profile, &frame.left_material),
                MemberType::FrameRight => (&frame.right_profile, &frame.right_material),
                MemberType::FrameTop => (&frame.top_profile, &frame.top_material),
                _ => (&frame.bottom_profile, &frame.bottom_material),
            };
            let mat = own_material.as_ref().unwrap_or(mat);
            let is_miter = mat.is_profile_system();
            let cut = frame_member_cut(
                orient,
                span,
                corners.map(|j| (j[start], j[end])),
                mat,
                fw,
                is_miter,
                miter_angle(mat),
            );
            let sill = if member_type == MemberType::FrameBottom {
                frame.sill_profile.as_ref()
            } else {
                None
            };
            let own_profile = own_profile.as_ref().or(sill);
            let profile_name = if frame.material.is_profile_system() {
                system_stock_name(frame, mat, own_profile)
            } else if !frame.material.is_profile_system()
                && member_type == MemberType::FrameBottom
                && frame.sill_profile.is_some()
            {
                format!("Dorpel {}", profile_name)
            } else {
                profile_name.to_owned()
            };
            CutListItem {
                piece_id: format!("{}-{}", kozijn.mark, suffix),
                member_type,
                profile_name,
                material: material_name(mat).to_owned(),
                net_length_mm: cut.net,
                gross_length_mm: cut.gross,
                miter_left_deg: cut.angle_start,
                miter_right_deg: cut.angle_end,
                quantity: 1,
                arc: None,
                profile_side: None,
                start_cope_radius_mm: None,
                end_cope_radius_mm: None,
                divider_start: None,
                divider_end: None,
                masonry_notches: Vec::new(),
            }
        })
        .collect()
}

// ── Main computation ───────────────────────────────────────────────

fn glass_production_issue(
    kozijn: &Kozijn,
    cell: Option<&crate::kozijn::Cell>,
    fit: &crate::glazing_fit::GlazingFit,
) -> Option<String> {
    if let Some(cell) = cell {
        if let Some(sash) = &cell.sash_profile {
            let frame_snapshot = kozijn.frame.profile_snapshot.as_ref().or_else(|| {
                crate::glazing_fit::catalog_snapshot(
                    &kozijn.frame.profile.id,
                    &kozijn.frame.material,
                )
            });
            if !crate::glazing_fit::system_profile_pair_compatible(
                &kozijn.frame.material,
                &kozijn.frame.profile.id,
                frame_snapshot,
                &sash.id,
                cell.sash_profile_snapshot.as_ref(),
            ) {
                return Some(format!(
                    "Vleugelprofiel '{}' hoort niet bij kozijnsysteem '{}'",
                    sash.name, kozijn.frame.profile.name,
                ));
            }
        }
    }
    if let Some(minimum) = fit.min_glass_thickness {
        if fit.glass_thickness + 1e-6 < minimum {
            return Some(format!(
                "Glasdikte {:.1} mm is kleiner dan systeemminimum {:.1} mm",
                fit.glass_thickness, minimum
            ));
        }
    }
    if let Some(maximum) = fit.max_glass_thickness {
        if fit.glass_thickness > maximum + 1e-6 {
            return Some(format!(
                "Glas {} mm overschrijdt systeemmaximum {} mm",
                fit.glass_thickness, maximum
            ));
        }
    }
    None
}

pub fn compute_production_data(kozijn: &Kozijn) -> ProductionData {
    let mark = &kozijn.mark;
    let mat = &kozijn.frame.material;
    let fw = kozijn.frame.frame_width;
    let is_miter = matches!(mat, Material::Aluminum | Material::Pvc);
    let angle = miter_angle(mat);
    let mat_name = material_name(mat).to_string();
    let is_system = mat.is_profile_system();
    let profile_name = if is_system {
        system_stock_name(&kozijn.frame, mat, None)
    } else {
        format!("{}x{}", kozijn.frame.frame_width, kozijn.frame.frame_depth)
    };

    let inner_w = kozijn.inner_width();
    let inner_h = kozijn.inner_height();

    let geometry = crate::geometry::compute_2d_geometry(kozijn);
    let mut cut_list = if geometry.curved_frame.is_some() {
        let mut pieces = curved_frame_cuts(kozijn, &geometry);
        pieces.extend(curved_divider_cuts(kozijn, &geometry));
        pieces
    } else {
        rectangular_frame_cuts(kozijn, &profile_name)
    };
    if let Some(stop) = &geometry.perimeter_stop {
        cut_list.extend(stop.cuts(mark, &mat_name, SAW_KERF_MM));
    }
    let divider_joints = divider_joints(&geometry, &cut_list);
    let arch_joints = arch_joints(&geometry, &cut_list);
    let mut glass_list = Vec::new();
    let mut hardware_list = Vec::new();
    let mut gasket_list = Vec::new();
    let mut panel_list = Vec::new();
    let mut glaslat_list = Vec::new();

    // ── Dividers + vakken ──────────────────────────────────────
    //
    // With a free-subdivision layout (`kozijn.layout`) the tussenstijlen/
    // tussendorpels and the vak contents come from the SAME layout geometry
    // the drawing uses (`compute_layout_geometry`), so the saw list matches
    // the picture. Without a layout the historic grid path in the `else`
    // branch runs byte-identical (regression-tested).
    let mut cells_aligned = true;
    if let Some(node) = kozijn.layout.as_ref() {
        // Raw (unclipped) layout geometry: the drawing clips its dividers
        // against `Buiten` (a mullion visually stops at the melkmeisje step),
        // but that clip is for the PICTURE — production resolves its own
        // member lengths below. Leaves are identical in both variants, so the
        // vak contents still match the drawing.
        let lg = crate::layout::compute_layout_geometry_unclipped(
            node,
            crate::layout::LayoutRect {
                x: fw,
                y: fw,
                width: inner_w,
                height: inner_h,
            },
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
        for d in lg
            .dividers
            .iter()
            .filter(|_| geometry.curved_frame.is_none())
        {
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
                        gross_length_mm: divider_gross(div_net, mat),
                        miter_left_deg: 90.0,
                        miter_right_deg: 90.0,
                        quantity: 1,
                        arc: None,
                        profile_side: None,
                        start_cope_radius_mm: None,
                        end_cope_radius_mm: None,
                        divider_start: None,
                        divider_end: None,
                        masonry_notches: Vec::new(),
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
                        gross_length_mm: divider_gross(div_net, mat),
                        miter_left_deg: 90.0,
                        miter_right_deg: 90.0,
                        quantity: 1,
                        arc: None,
                        profile_side: None,
                        start_cope_radius_mm: None,
                        end_cope_radius_mm: None,
                        divider_start: None,
                        divider_end: None,
                        masonry_notches: Vec::new(),
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
        cells_aligned = lg.leaves.len() == kozijn.cells.len();
        for l in &lg.leaves {
            if l.vulling.is_buiten() {
                continue;
            }
            let cell = if cells_aligned {
                kozijn.cells.get(l.leaf_index)
            } else {
                None
            };
            let cell_id = format!("{}-V{}", mark, l.leaf_index + 1);
            let vak_w = l.rect.width;
            let vak_h = l.rect.height;

            match &l.vulling {
                Vakvulling::Glas { .. } | Vakvulling::Raam { .. } | Vakvulling::Deur { .. } => {
                    // Hardware van de uitgelijnde cel (raam/deur-vullingen).
                    if !is_system && !matches!(&l.vulling, Vakvulling::Glas { .. }) {
                        if let Some(hw) = cell.and_then(|c| c.hardware_set.as_ref()) {
                            if let Some(ref hinges) = hw.hinges {
                                hardware_list.push(HardwareListItem {
                                    piece_id: None,
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
                                    piece_id: None,
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
                                    piece_id: None,
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
        for (i, col) in kozijn
            .grid
            .columns
            .iter()
            .enumerate()
            .filter(|_| geometry.curved_frame.is_none())
        {
            if i > 0 && (is_system || col.divider_profile.is_some()) {
                let div_net = inner_h;
                cut_list.push(CutListItem {
                    piece_id: format!("{}-TS{}", mark, i),
                    member_type: MemberType::DividerV,
                    profile_name: col
                        .divider_profile
                        .as_ref()
                        .filter(|p| is_system && compatible_profile(&p.id, mat))
                        .map_or_else(|| profile_name.clone(), |p| p.name.clone()),
                    material: mat_name.clone(),
                    net_length_mm: div_net,
                    gross_length_mm: divider_gross(div_net, mat),
                    miter_left_deg: 90.0,
                    miter_right_deg: 90.0,
                    quantity: 1,
                    arc: None,
                    profile_side: None,
                    start_cope_radius_mm: None,
                    end_cope_radius_mm: None,
                    divider_start: None,
                    divider_end: None,
                    masonry_notches: Vec::new(),
                });
            }
        }

        // Horizontal dividers (tussendorpels)
        for (i, row) in kozijn
            .grid
            .rows
            .iter()
            .enumerate()
            .filter(|_| geometry.curved_frame.is_none())
        {
            if i > 0 && (is_system || row.divider_profile.is_some()) {
                let split = is_system && kozijn.grid.columns.len() > 1;
                let widths: Vec<_> = if split {
                    kozijn.grid.columns.iter().map(|c| c.size).collect()
                } else {
                    vec![inner_w]
                };
                for (segment, div_net) in widths.into_iter().enumerate() {
                    let id = format!("{mark}-TD{i}");
                    cut_list.push(CutListItem {
                        piece_id: if split {
                            format!("{id}.{}", segment + 1)
                        } else {
                            id
                        },
                        member_type: MemberType::DividerH,
                        profile_name: row
                            .divider_profile
                            .as_ref()
                            .filter(|p| is_system && compatible_profile(&p.id, mat))
                            .map_or_else(|| profile_name.clone(), |p| p.name.clone()),
                        material: mat_name.clone(),
                        net_length_mm: div_net,
                        gross_length_mm: divider_gross(div_net, mat),
                        miter_left_deg: 90.0,
                        miter_right_deg: 90.0,
                        quantity: 1,
                        arc: None,
                        profile_side: None,
                        start_cope_radius_mm: None,
                        end_cope_radius_mm: None,
                        divider_start: None,
                        divider_end: None,
                        masonry_notches: Vec::new(),
                    });
                }
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

            match cell.panel_type {
                PanelType::FixedGlass
                | PanelType::TurnTilt
                | PanelType::Turn
                | PanelType::Tilt
                | PanelType::Sliding
                | PanelType::Door
                | PanelType::TopHung
                | PanelType::BottomHung
                | PanelType::LiftSlide
                | PanelType::Pivot => {}
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

            // Hardware list from HardwareSet
            if let Some(hw) = cell.hardware_set.as_ref().filter(|_| !is_system) {
                if let Some(ref hinges) = hw.hinges {
                    hardware_list.push(HardwareListItem {
                        piece_id: None,
                        cell_index: i,
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
                        piece_id: None,
                        cell_index: i,
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
                        piece_id: None,
                        cell_index: i,
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

    // Read the same per-leaf fit as the drawing, including both halves of a stolp.

    let default_glazing = crate::kozijn::Glazing::default();
    for cr in &geometry.cell_rects {
        let cell = kozijn.cells.get(cr.cell_index).filter(|_| cells_aligned);
        if is_system {
            system::hardware(cr, cell, mark, &mut hardware_list);
        }
        let glazing = cell.map(|c| &c.glazing).unwrap_or(&default_glazing);
        for (index, fit) in cr.fit.iter().chain(&cr.second_fit).enumerate() {
            let mut piece_id = format!("{}-V{}", mark, cr.cell_index + 1);
            if let Some(leaf) = fit.contours.as_ref().and_then(|c| c.sash_leaf.as_ref()) {
                piece_id.push_str(match leaf {
                    SashLeaf::Left => "-L",
                    SashLeaf::Right => "-R",
                });
            } else if cr.second_fit.is_some() {
                piece_id.push_str(if index == 0 { "-L" } else { "-R" });
            }
            if let Some(outer) = &fit.sash_outer {
                let sash_width = fit
                    .contours
                    .as_ref()
                    .map_or_else(|| (fit.day.x - outer.x).max(0.0), |c| c.sash_width);
                let door = matches!(&cr.vulling, Some(Vakvulling::Deur { .. }))
                    || (fit.contours.is_some()
                        && cr.vulling.is_none()
                        && cell.is_some_and(|c| c.panel_type == PanelType::Door));
                let prefix = if door { "D" } else { "R" };
                let stock = if door { "Deurhout" } else { "Raamhout" };
                let profile = cell
                    .and_then(|c| c.sash_profile.as_ref())
                    .filter(|p| compatible_profile(&p.id, mat))
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| {
                        if is_system {
                            return format!("{profile_name} (vleugel)");
                        }
                        if fit.contours.is_none() {
                            return format!("{stock} {profile_name}");
                        }
                        match cell
                            .and_then(|c| c.sash_depth)
                            .filter(|d| d.is_finite() && *d > 0.0)
                        {
                            Some(depth) => format!("{stock} {sash_width}x{depth} mm"),
                            None => format!("{stock} {sash_width} mm"),
                        }
                    });
                if let Some(contours) = &fit.contours {
                    cut_list.extend(
                        contours.sash_timber.iter().map(|member| {
                            curved_sash_cut(member, &piece_id, prefix, &profile, mat)
                        }),
                    );
                } else {
                    push_sash_members(
                        &mut cut_list,
                        &piece_id,
                        prefix,
                        &profile,
                        mat,
                        &mat_name,
                        is_miter,
                        angle,
                        outer.width,
                        outer.height,
                        sash_width,
                    );
                }
                if !is_system {
                    gasket_list.push(GasketListItem {
                        reference: None,
                        gasket_type: GasketType::SashSeal,
                        length_mm: fit
                            .contours
                            .as_ref()
                            .and_then(|c| c.sash_outer.as_ref())
                            .map_or_else(|| 2.0 * (outer.width + outer.height), Contour::perimeter)
                            + GASKET_OVERLAP_MM,
                        quantity: 1,
                    });
                }
            }
            let (w, h) = (fit.glass.width, fit.glass.height);
            let outline = fit.contours.as_ref().map(|c| &c.glass);
            let glass_perimeter = outline.map_or(2.0 * (w + h), Contour::perimeter);
            let production_issue = glass_production_issue(kozijn, cell, fit);
            glass_list.push(GlassListItem {
                piece_id: piece_id.clone(),
                cell_index: cr.cell_index,
                glass_type: glazing.glass_type.clone(),
                width_mm: w,
                height_mm: h,
                thickness_mm: glazing.thickness_mm,
                ug_value: glazing.ug_value,
                area_m2: outline.map_or(w * h, Contour::area) / 1_000_000.0,
                quantity: 1,
                production_ready: production_issue.is_none(),
                production_issue,
                cut_contour: outline.and_then(piece_cut_contour),
            });
            if is_system {
                let carrier = cell
                    .filter(|_| fit.sash_outer.is_some())
                    .and_then(|c| c.sash_profile.as_ref())
                    .filter(|p| compatible_profile(&p.id, mat))
                    .map_or(profile_name.as_str(), |p| &p.name);
                system::gaskets(fit, &piece_id, carrier, &mut gasket_list);
            } else {
                for gasket_type in [GasketType::GlazingInner, GasketType::GlazingOuter] {
                    gasket_list.push(GasketListItem {
                        reference: None,
                        gasket_type,
                        length_mm: glass_perimeter + GASKET_OVERLAP_MM,
                        quantity: 1,
                    });
                }
            }
            let contour_runs = fit
                .contours
                .as_ref()
                .map_or_else(Vec::new, |c| bead_runs(&c.rebate));
            let (total_length_mm, quantity) = if contour_runs.is_empty() {
                (glaslat_total_length(fit), 4)
            } else {
                (
                    contour_runs.iter().map(|r| r.length_mm).sum(),
                    contour_runs.len() as u32,
                )
            };
            glaslat_list.push(GlaslatListItem {
                piece_id,
                cell_index: cr.cell_index,
                position: fit.bead_position.label_nl().into(),
                material: cell
                    .and_then(|c| c.glaslat.as_ref())
                    .map(|b| b.material.clone())
                    .filter(|m| {
                        !is_system
                            || !matches!(
                                m.trim().to_ascii_lowercase().as_str(),
                                "hout" | "wood" | "meranti"
                            )
                    })
                    .unwrap_or_else(|| mat_name.clone()),
                width_mm: fit.bead_width,
                height_mm: fit.bead_height,
                total_length_mm,
                mitered: fit.bead_mitered,
                quantity,
                contour_runs,
            });
        }
    }

    // Frame seal gasket (around the full inner perimeter)
    if !is_system {
        let frame_perimeter = geometry
            .curved_frame
            .as_ref()
            .map_or(2.0 * (inner_w + inner_h), |f| f.day.perimeter())
            + GASKET_OVERLAP_MM;
        gasket_list.push(GasketListItem {
            reference: None,
            gasket_type: GasketType::FrameSeal,
            length_mm: frame_perimeter,
            quantity: 1,
        });
    }

    // ── BOM (aggregate) ────────────────────────────────────────

    let mut bom = Vec::new();

    // Formed leaves and dividers with a selected profile have their own stock.
    let mut frame_stock_mm = 0.0;
    let mut other_stock_mm = std::collections::BTreeMap::<(&str, &str), f64>::new();
    for cut in &cut_list {
        if arch_joints
            .iter()
            .any(|j| j.piece_id == cut.piece_id && j.joint.issue.is_some())
        {
            bom.push(BomItem {
                category: "Profiel".into(),
                description: format!("{} - brutomaat onbepaald", cut.piece_id),
                unit: "stuk".into(),
                quantity: f64::from(cut.quantity),
            });
            continue;
        }
        let stock_length = cut.gross_length_mm * cut.quantity as f64;
        let sash = matches!(
            cut.member_type,
            MemberType::SashArc
                | MemberType::SashTop
                | MemberType::SashBottom
                | MemberType::SashLeft
                | MemberType::SashRight
        );
        let custom_divider =
            cut.divider_start.is_some() && cut.profile_name != kozijn.frame.profile.name;
        let custom_ring = kozijn.frame.shape.shape_type == crate::kozijn::ShapeType::Round
            && cut.member_type == MemberType::FrameArc
            && (cut.profile_name != kozijn.frame.profile.name || cut.material != mat_name);
        if is_system || geometry.curved_frame.is_some() && (sash || custom_divider || custom_ring) {
            *other_stock_mm
                .entry((&cut.profile_name, &cut.material))
                .or_default() += stock_length;
        } else {
            frame_stock_mm += stock_length;
        }
    }
    if frame_stock_mm > 0.0 {
        bom.push(BomItem {
            category: "Profiel".into(),
            description: format!("{} {} mm", mat_name, profile_name),
            unit: "m".into(),
            quantity: frame_stock_mm / 1000.0,
        });
    }
    for ((profile, material), mm) in other_stock_mm {
        bom.push(BomItem {
            category: "Profiel".into(),
            description: format!("{profile} ({material})"),
            unit: "m".into(),
            quantity: mm / 1000.0,
        });
    }

    // Total glass area
    let total_glass_m2: f64 = glass_list
        .iter()
        .map(|g| g.area_m2 * g.quantity as f64)
        .sum();
    if total_glass_m2 > 0.0 {
        bom.push(BomItem {
            category: "Glas".into(),
            description: glass_list
                .first()
                .map(|g| g.glass_type.clone())
                .unwrap_or_default(),
            unit: "m2".into(),
            quantity: total_glass_m2,
        });
    }

    // Total gasket length
    let total_gasket_mm: f64 = gasket_list
        .iter()
        .map(|g| g.length_mm * g.quantity as f64)
        .sum();
    if is_system {
        let mut seals = std::collections::BTreeMap::new();
        for gasket in &gasket_list {
            if let Some(reference) = gasket.reference.as_ref().filter(|_| gasket.quantity > 0) {
                *seals
                    .entry((&reference.profile_name, gasket.gasket_type.label_nl()))
                    .or_insert(0.0) += gasket.length_mm * f64::from(gasket.quantity);
            }
        }
        for ((profile, kind), length) in seals {
            bom.push(BomItem {
                category: "Rubber".into(),
                description: format!("{profile} - {kind} (netto meetlijn)"),
                unit: "m".into(),
                quantity: length / 1000.0,
            });
        }
    } else {
        bom.push(BomItem {
            category: "Rubber".into(),
            description: "EPDM afdichting".into(),
            unit: "m".into(),
            quantity: total_gasket_mm / 1000.0,
        });
    }

    // Hardware count
    if is_system {
        let mut components = std::collections::BTreeMap::new();
        for hardware in hardware_list.iter().filter(|h| h.quantity > 0) {
            *components
                .entry((&hardware.component, &hardware.description))
                .or_insert(0) += hardware.quantity;
        }
        for ((name, detail), count) in components {
            bom.push(BomItem {
                category: "Beslag".into(),
                description: format!("{name} - {detail}"),
                unit: "stuks".into(),
                quantity: f64::from(count),
            });
        }
    } else if !hardware_list.is_empty() {
        let total_hw: u32 = hardware_list.iter().map(|h| h.quantity).sum();
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
        divider_joints,
        arch_joints,
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
    fn material_switch_updates_divider_drawing_glass_and_cut_articles() {
        let mut k = Kozijn::new("Wisseltest", "W01", 1200.0, 1500.0);
        k.add_column(500.0);
        let old_geometry = crate::geometry::compute_2d_geometry(&k);
        assert_eq!(old_geometry.v_dividers[0].width, 67.0);
        let old_glass_width = old_geometry.cell_rects[0].fit.as_ref().unwrap().glass.width;
        k.frame.joints_configured = true;
        let snapshot =
            crate::glazing_fit::catalog_snapshot("veka-softline82", &Material::Pvc).unwrap();
        let snapshot_json = serde_json::to_string(snapshot).unwrap();
        k.apply_frame_profile(
            crate::profile::ProfileRef {
                id: "veka-softline82".into(),
                name: "VEKA Softline 82 MD".into(),
            },
            Some(73.0),
            Some(82.0),
            Some(&snapshot_json),
            Some("\"pvc\""),
        )
        .unwrap();

        let geometry = crate::geometry::compute_2d_geometry(&k);
        assert!(!k.frame.joints_configured);
        assert_eq!(
            geometry.corner_settings[0].joint_type,
            crate::joint::JointType::Verstek
        );
        assert_eq!(geometry.v_dividers[0].width, 94.0);
        let cells_total =
            k.grid.columns.iter().map(|c| c.size).sum::<f64>() + geometry.v_dividers[0].width;
        assert!((cells_total - k.inner_width()).abs() < 1e-6);

        let production = compute_production_data(&k);
        assert_eq!(production.glass_list.len(), 2);
        for (index, glass) in production.glass_list.iter().enumerate() {
            let fit = geometry.cell_rects[index].fit.as_ref().unwrap();
            assert!((glass.width_mm - fit.glass.width).abs() < 1e-6);
            assert!(glass.production_ready);
        }
        assert_ne!(production.glass_list[0].width_mm, old_glass_width);
        let divider_cut = production
            .cut_list
            .iter()
            .find(|c| c.member_type == MemberType::DividerV)
            .unwrap();
        assert_eq!(
            divider_cut.profile_name,
            k.grid.columns[1].divider_profile.as_ref().unwrap().name
        );
        let plan = crate::optimization::optimize_frame_cuts(std::slice::from_ref(&k), 5800.0);
        assert!(plan
            .allocations
            .iter()
            .all(|bar| bar.profile_id.as_deref() == Some("veka-softline82")));
        let stock_articles: std::collections::BTreeSet<_> = plan
            .allocations
            .iter()
            .map(|bar| bar.profile_name.as_str())
            .collect();
        assert!(stock_articles.len() >= 2, "kozijn- en tussenstijlartikelen moeten aparte staven krijgen");
    }

    #[test]
    fn system_glass_limits_are_visible_in_the_glass_list_and_validation() {
        let mut k = Kozijn::new("VEKA", "V80", 900.0, 1400.0);
        let snapshot =
            crate::glazing_fit::catalog_snapshot("veka-softline82", &Material::Pvc).unwrap();
        let snapshot_json = serde_json::to_string(snapshot).unwrap();
        k.apply_frame_profile(
            crate::profile::ProfileRef {
                id: "veka-softline82".into(),
                name: "VEKA Softline 82 MD".into(),
            },
            Some(73.0),
            Some(82.0),
            Some(&snapshot_json),
            Some("\"pvc\""),
        )
        .unwrap();
        k.cells[0].glazing.thickness_mm = 80.0;
        let glass = &compute_production_data(&k).glass_list[0];
        assert!(!glass.production_ready);
        assert!(glass.production_issue.as_deref().unwrap().contains("52"));
        assert!(crate::validation::validate(&k).iter().any(|e| matches!(
            e,
            crate::validation::ValidationError::GlassTooThick(1, 80.0, 52.0)
        )));
    }

    #[test]
    fn mixed_system_sash_profile_is_rejected_without_mutating_the_cell() {
        let mut k = Kozijn::new("VEKA", "V01", 900.0, 1400.0);
        let veka = crate::glazing_fit::catalog_snapshot("veka-softline82", &Material::Pvc).unwrap();
        let veka_json = serde_json::to_string(veka).unwrap();
        k.apply_frame_profile(
            crate::profile::ProfileRef {
                id: "veka-softline82".into(),
                name: "VEKA Softline 82 MD".into(),
            },
            Some(73.0),
            Some(82.0),
            Some(&veka_json),
            Some("\"pvc\""),
        )
        .unwrap();
        let result = k.update_cell_sash_profile(
            0,
            crate::profile::ProfileRef {
                id: "kommerling-76ad".into(),
                name: "Kömmerling 76 AD".into(),
            },
            70.0,
            76.0,
            crate::glazing_fit::catalog_snapshot("kommerling-76ad", &Material::Pvc).cloned(),
        );
        assert!(result.unwrap_err().contains("profielsysteem"));
        assert!(k.cells[0].sash_profile.is_none());
        k.cells[0].sash_profile = Some(crate::profile::ProfileRef {
            id: "kommerling-76ad".into(),
            name: "Kömmerling 76 AD".into(),
        });
        assert!(crate::validation::validate(&k).iter().any(|e| matches!(
            e,
            crate::validation::ValidationError::IncompatibleSashSystem(1, _, _)
        )));
    }

    #[test]
    fn test_turn_tilt_with_hardware() {
        let mut k = Kozijn::new("Test", "T02", 900.0, 1400.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].hardware_set = crate::hardware::default_hardware_set(
            PanelType::TurnTilt,
            Some(crate::kozijn::OpeningDirection::Left),
            766.0,
            1266.0,
            24.0,
            &Material::Wood(crate::kozijn::WoodType::Meranti),
            crate::hardware::SecurityClass::None,
        );
        let prod = compute_production_data(&k);

        // 4 frame + 4 sash pieces = 8
        assert_eq!(prod.cut_list.len(), 8);

        // Hardware should be present
        assert!(!prod.hardware_list.is_empty());

        // Check wood gross length includes pen allowance
        let stile = prod
            .cut_list
            .iter()
            .find(|c| c.member_type == MemberType::FrameLeft)
            .unwrap();
        assert!(stile.gross_length_mm > stile.net_length_mm);
        let expected_gross = stile.net_length_mm + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM;
        assert!((stile.gross_length_mm - expected_gross).abs() < 0.01);
    }

    fn find<'a>(prod: &'a ProductionData, mt: MemberType) -> &'a CutListItem {
        prod.cut_list.iter().find(|c| c.member_type == mt).unwrap()
    }

    #[test]
    fn round_frame_lists_two_finished_half_rings_with_radial_end_cuts() {
        let mut k = Kozijn::new("Rond", "R1", 1200.0, 1500.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        let mut bottom = k.frame.profile.clone();
        bottom.name = "Onderste ringprofiel".to_owned();
        k.frame.bottom_profile = Some(bottom);
        let p = compute_production_data(&k);
        assert_eq!(p.cut_list.len(), 2);
        assert_eq!(p.arch_joints.len(), 2);
        let ext = 600.0 * (20.0_f64 / (600.0 - 67.0 / 2.0 - 25.0 / 2.0)).atan();
        for (i, cut) in p.cut_list.iter().enumerate() {
            assert_eq!(cut.piece_id, format!("R1-BO{}", i + 1));
            assert_eq!(cut.member_type, MemberType::FrameArc);
            assert_eq!(cut.quantity, 1);
            let arc = cut.arc.as_ref().unwrap();
            assert_eq!(arc.outer_radius_mm, 600.0);
            assert_eq!(arc.centerline_radius_mm, 566.5);
            assert!((arc.outer_length_mm - 600.0 * std::f64::consts::PI).abs() < 1e-8);
            assert!((arc.centerline_length_mm - 566.5 * std::f64::consts::PI).abs() < 1e-8);
            assert!((cut.miter_left_deg - 90.0).abs() < 1e-8);
            assert!((cut.miter_right_deg - 90.0).abs() < 1e-8);
            let extensions = if i == 1 { 2.0 * ext } else { 0.0 };
            assert!(
                (cut.gross_length_mm - arc.outer_length_mm - SAW_KERF_MM - extensions).abs() < 1e-7
            );
        }
        for (i, row) in p.arch_joints.iter().enumerate() {
            assert_eq!(
                row.joint.kind,
                crate::curved_timber::ArchJointKind::RingSeam
            );
            assert_eq!(row.joint.corner, i);
            assert_eq!(row.piece_id, "R1-BO2");
            assert_eq!(row.receiver_piece_id, "R1-BO1");
            let t = row.joint.tenon.as_ref().unwrap();
            assert_eq!(t.z_from_inside_mm.len(), 2);
            assert!((t.blank_extension_mm - ext).abs() < 1e-7);
            assert_eq!(
                row.end,
                if i == 0 {
                    DividerJointEnd::End
                } else {
                    DividerJointEnd::Start
                }
            );
            assert_ne!(row.end, row.receiver_end);
        }
        assert_eq!(p.cut_list[0].profile_name, k.frame.profile.name);
        assert_eq!(p.cut_list[1].profile_name, "Onderste ringprofiel");
        let stock = p
            .bom
            .iter()
            .find(|r| r.description == "Onderste ringprofiel (Meranti)")
            .unwrap();
        assert!((stock.quantity * 1000. - p.cut_list[1].gross_length_mm).abs() < 1e-7);
    }

    #[test]
    fn ring_blanks_feed_cnc_and_procurement_only_with_a_resolved_developed_length() {
        use crate::kozijn::{Project, ShapeType};
        let mut k = Kozijn::new("Rond", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        k.frame.joints_configured = true;
        k.frame.corner_joints[0].through_member = ThroughMember::Dorpel;
        k.frame.corner_joints[1].through_member = ThroughMember::Stijl;
        k.frame.corner_joints[1].pen_length = 26.;
        let p = compute_production_data(&k);
        let parts = crate::cnc::generate_cnc_parts(&k);
        let plan = crate::optimization::optimize_frame_cuts(std::slice::from_ref(&k), 5800.);
        assert!(plan.allocations.is_empty());
        assert_eq!(plan.curved_piece_ids, ["R1-BO1", "R1-BO2"]);
        for (i, (cut, cnc)) in p.cut_list.iter().zip(parts).enumerate() {
            let length = if i == 0 { 26.0_f64 } else { 20. };
            let expected = cut.net_length_mm + SAW_KERF_MM + 600. * (length / 554.).atan();
            assert!((cut.gross_length_mm - expected).abs() < 1e-8);
            assert_eq!(cnc.gross_length_mm, Some(cut.gross_length_mm));
            assert_eq!(cnc.arch_joints.len(), 2);
            assert!(cnc.operations.is_empty());
        }

        k.frame.corner_joints[0].pen_length = 1000.;
        let mut project = Project::new("Ringen", "R");
        project.kozijnen.push(k);
        let plan = crate::optimization::optimize_frame_cuts(&project.kozijnen, 5800.);
        assert_eq!(plan.curved_piece_ids, ["R1-BO1"]);
        assert_eq!(plan.unresolved_piece_ids, ["R1-BO2"]);
        let cnc = crate::cnc::generate_cnc_parts(&project.kozijnen[0]);
        assert!(cnc[0].gross_length_mm.is_some() && cnc[1].gross_length_mm.is_none());
        let orders = crate::purchase_order::generate_purchase_orders(&project);
        let wood = orders.iter().find(|p| p.category == "Hout").unwrap();
        assert_eq!(wood.unresolved_piece_ids, ["R1-BO2"]);
        assert_eq!(wood.lines.len(), 1);
        assert!(wood.lines[0].description.contains("Boogstuk boven"));
    }

    #[test]
    fn round_turning_leaf_has_two_half_rings_and_no_rectangular_sash_members() {
        let mut k = Kozijn::new("Rond raam", "R1", 1200.0, 1500.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        k.cells[0].panel_type = PanelType::Turn;
        let prod = compute_production_data(&k);
        let sash: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|i| i.piece_id.starts_with("R1-V1-"))
            .collect();
        assert_eq!(sash.len(), 2);
        let sash_stock = prod
            .bom
            .iter()
            .find(|row| row.description == "Raamhout 69 mm (Meranti)")
            .unwrap();
        assert!((sash_stock.quantity - 2.0 * sash[0].gross_length_mm / 1000.0).abs() < 1e-8);
        for (i, row) in sash.iter().enumerate() {
            assert_eq!(row.member_type, MemberType::SashArc);
            assert_eq!(row.piece_id, format!("R1-V1-RBO{}", i + 1));
            assert_eq!(row.profile_name, "Raamhout 69 mm");
            let arc = row.arc.as_ref().unwrap();
            let outer_r = 600.0 - 67.0 + 14.0;
            assert_eq!(arc.outer_radius_mm, outer_r);
            assert_eq!(arc.centerline_radius_mm, outer_r - 69.0 / 2.0);
            assert!((row.net_length_mm - std::f64::consts::PI * outer_r).abs() < 1e-6);
            assert!((row.gross_length_mm - row.net_length_mm - 4.0).abs() < 1e-6);
            assert!((row.miter_left_deg - 90.0).abs() < 1e-7);
            assert!((row.miter_right_deg - 90.0).abs() < 1e-7);
        }
    }

    #[test]
    fn arched_sash_cut_lengths_and_angles_follow_the_moving_shoulders() {
        let mut k = Kozijn::new("Getoogd raam", "B1", 1200.0, 1500.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].sash_profile = Some(crate::profile::ProfileRef {
            id: "raamhout-69x90".into(),
            name: "Raamhout 69x90".into(),
        });
        let prod = compute_production_data(&k);
        let head = find(&prod, MemberType::SashArc);
        let stile = find(&prod, MemberType::SashLeft);
        let sill = find(&prod, MemberType::SashBottom);
        let radius: f64 = 750.0 - 67.0 + 14.0;
        let half_width: f64 = 600.0 - 67.0 + 14.0;
        let half_sweep = (half_width / radius).asin();
        let shoulder_y = |d: f64| 750.0 - ((radius - d).powi(2) - (half_width - d).powi(2)).sqrt();
        let shoulder_tilt = (shoulder_y(69.0) - shoulder_y(0.0)).atan2(69.0);
        assert_eq!(head.piece_id, "B1-V1-RBO1");
        assert_eq!(head.profile_name, "Raamhout 69x90");
        assert_eq!(head.arc.as_ref().unwrap().outer_radius_mm, radius);
        assert!((head.net_length_mm - 2.0 * radius * half_sweep).abs() < 1e-6);
        assert!((head.miter_left_deg - (half_sweep + shoulder_tilt).to_degrees()).abs() < 1e-6);
        assert!((head.miter_right_deg - head.miter_left_deg).abs() < 1e-6);
        assert!((stile.miter_left_deg - (90.0 - shoulder_tilt.to_degrees())).abs() < 1e-6);
        assert!((stile.net_length_mm - (1447.0 - shoulder_y(0.0))).abs() < 1e-6);
        assert_eq!(stile.miter_right_deg, 90.0);
        assert_eq!(sill.net_length_mm, 2.0 * (half_width - 69.0));
        // Each end of the head and sill carries a tenon into a stile.
        assert!((head.gross_length_mm - head.net_length_mm - 44.0).abs() < 1e-6);
        assert!((sill.gross_length_mm - sill.net_length_mm - 44.0).abs() < 1e-6);
        assert!((stile.gross_length_mm - stile.net_length_mm - 4.0).abs() < 1e-6);
        let sash_stock = prod
            .bom
            .iter()
            .find(|row| row.description == "Raamhout 69x90 (Meranti)")
            .unwrap();
        let gross_mm = head.gross_length_mm + 2.0 * stile.gross_length_mm + sill.gross_length_mm;
        assert!((sash_stock.quantity - gross_mm / 1000.0).abs() < 1e-8);
    }

    #[test]
    fn round_pair_excludes_meeting_gap_and_labels_each_leaf_separately() {
        let mut k = Kozijn::new("Rond stolp", "P1", 1200.0, 1200.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        k.layout = Some(crate::layout::deur("dubbel"));
        let prod = compute_production_data(&k);
        let arcs: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|cut| cut.member_type == MemberType::SashArc)
            .collect();
        assert_eq!(arcs.len(), 2);
        let radius = 547.0_f64;
        let sweep = 2.0 * (3.0 / radius).acos();
        for (i, id) in ["P1-V1-L-DBO1", "P1-V1-R-DBO1"].iter().enumerate() {
            assert_eq!(arcs[i].piece_id, *id);
            assert!((arcs[i].net_length_mm - radius * sweep).abs() < 1e-6);
        }
        assert_eq!(arcs[0].profile_side, Some(TimberProfileSide::Left));
        assert_eq!(arcs[1].profile_side, Some(TimberProfileSide::Right));
        assert_eq!(
            prod.glass_list
                .iter()
                .map(|g| g.piece_id.as_str())
                .collect::<Vec<_>>(),
            ["P1-V1-L", "P1-V1-R"]
        );
    }

    #[test]
    fn round_crossing_lists_separate_blanks_with_a_chord_angle_and_handed_setbacks() {
        let mut k = Kozijn::new("Rond kruis", "R1", 1200.0, 1200.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        k.add_column(300.0);
        k.add_row(450.0);
        k.set_grid_sizes_fitted(&[273.0, 726.0], &[363.0, 636.0]);
        k.grid.columns[1].divider_profile = Some(crate::profile::ProfileRef {
            id: "ts-profiel".into(),
            name: "Stijlprofiel".into(),
        });
        let p = compute_production_data(&k);
        let heads: Vec<_> = p
            .cut_list
            .iter()
            .filter(|i| i.divider_start.is_some())
            .collect();
        assert_eq!(heads.len(), 3);
        let by_id = |id| *heads.iter().find(|i| i.piece_id == id).unwrap();
        let upper = by_id("R1-TS1.1");
        let lower = by_id("R1-TS1.2");
        let rail = by_id("R1-TD1");
        let at = |x: f64| (533.0_f64.powi(2) - (600.0 - x).powi(2)).sqrt();
        let near = |a: f64, b: f64| assert!((a - b).abs() < 1e-7, "{a} vs {b}");
        let h = at(407.0) - at(340.0);
        let angle = 67.0_f64.atan2(h).to_degrees();
        near(upper.net_length_mm, 430.0 - (600.0 - at(407.0)));
        near(lower.net_length_mm, 600.0 + at(407.0) - 497.0);
        for i in &heads {
            let vertical = i.member_type == MemberType::DividerV;
            let axis = usize::from(vertical);
            let start = i.divider_start.as_ref().unwrap();
            let end = i.divider_end.as_ref().unwrap();
            let lo = start.segments[0].from()[axis] - start.edge_offsets_mm[0];
            let hi = lo + i.net_length_mm;
            let first = start.tenon.as_ref().map_or(lo, |t| lo.min(t.tip_axis_mm));
            let last = end.tenon.as_ref().map_or(hi, |t| hi.max(t.tip_axis_mm));
            near(i.gross_length_mm, last - first + SAW_KERF_MM);
        }
        assert_eq!(upper.profile_name, "Stijlprofiel");
        assert_eq!(lower.profile_name, "Stijlprofiel");
        let stock = p
            .bom
            .iter()
            .find(|row| row.description == "Stijlprofiel (Meranti)")
            .unwrap();
        near(
            stock.quantity,
            (upper.gross_length_mm + lower.gross_length_mm) / 1000.0,
        );
        near(upper.miter_left_deg, angle);
        near(lower.miter_right_deg, angle);
        for e in [
            upper.divider_start.as_ref().unwrap(),
            lower.divider_end.as_ref().unwrap(),
        ] {
            near(e.edge_offsets_mm[0], h);
            near(e.edge_offsets_mm[1], 0.0);
            assert_eq!(e.radius_mm, Some(533.0));
        }
        near(upper.miter_right_deg, 90.0);
        near(lower.miter_left_deg, 90.0);
        assert_eq!(
            (upper.end_cope_radius_mm, lower.start_cope_radius_mm),
            (None, None)
        );
        near(rail.net_length_mm, 2.0 * at(497.0));
        let angle = 67.0_f64.atan2(at(497.0) - at(430.0)).to_degrees();
        near(rail.miter_left_deg, angle);
        near(rail.miter_right_deg, angle);
        assert_eq!(
            (rail.start_cope_radius_mm, rail.end_cope_radius_mm),
            (Some(533.0), Some(533.0))
        );
        let preview = crate::cnc::generate_cnc_parts(&k);
        assert!(preview
            .iter()
            .find(|i| i.piece_id == rail.piece_id)
            .unwrap()
            .operations
            .is_empty());
        assert!(preview
            .iter()
            .find(|i| i.piece_id == upper.piece_id)
            .unwrap()
            .operations
            .is_empty());
    }

    #[test]
    fn curved_joint_positions_link_each_hole_to_its_piece_and_budget_the_actual_tip() {
        let mut k = Kozijn::new("Rond met kruis", "R", 1200., 1200.);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        k.add_column(300.);
        k.add_row(450.);
        k.set_grid_sizes_fitted(&[273., 726.], &[363., 636.]);
        let prod = compute_production_data(&k);
        let index = prod.joint_index();
        let upper = &index["R-TS1.1"];
        let top = upper.ends[0].unwrap();
        assert_eq!(top.receiver_piece_id.as_deref(), Some("R-BO1"));
        assert_eq!(
            upper.ends[1].unwrap().receiver_piece_id.as_deref(),
            Some("R-TD1")
        );
        assert_eq!(
            index["R-TS1.2"].ends[1]
                .unwrap()
                .receiver_piece_id
                .as_deref(),
            Some("R-BO2")
        );
        assert_eq!(index["R-TD1"].mortises.len(), 2);
        assert!(index["R-BO1"]
            .mortises
            .iter()
            .any(|j| j.piece_id == "R-TS1.1"));
        let t = top.tenon.as_ref().unwrap();
        assert!(t.tenon_length_mm > t.blank_extension_mm + 0.1);
        assert!((t.mortise_depth_mm - t.tenon_length_mm - t.bottom_clearance_mm).abs() < 1e-7);
        let cut = prod
            .cut_list
            .iter()
            .find(|i| i.piece_id == "R-TS1.1")
            .unwrap();
        let bottom = upper.ends[1].unwrap().tenon.as_ref().unwrap();
        assert!(
            (cut.gross_length_mm - bottom.tip_axis_mm + t.tip_axis_mm - SAW_KERF_MM).abs() < 1e-7
        );
        let gate = crate::cnc::generate_cnc_parts(&k);
        assert!(gate
            .iter()
            .find(|p| p.piece_id == "R-BO1")
            .unwrap()
            .divider_joints
            .iter()
            .any(|j| j.piece_id == "R-TS1.1"));
    }

    #[test]
    fn segmental_head_and_jambs_report_the_saw_angle_to_each_piece_and_joint_allowance() {
        let mut k = Kozijn::new("Toog", "B1", 1200.0, 1500.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.0);
        k.frame.joints_configured = true;
        for j in &mut k.frame.corner_joints {
            j.through_member = ThroughMember::Dorpel;
        }
        k.frame.corner_joints[0].pen_length = 16.0;
        k.frame.corner_joints[2].pen_length = 27.0;
        k.add_column(410.0);
        let p = compute_production_data(&k);
        assert!(p
            .cut_list
            .iter()
            .all(|c| c.member_type != MemberType::FrameTop));
        let head = find(&p, MemberType::FrameArc);
        let arc = head.arc.as_ref().unwrap();
        assert_eq!(arc.outer_radius_mm, 750.0);
        let outside_sweep = 2.0 * 0.8_f64.asin();
        assert!((arc.sweep_deg - outside_sweep.to_degrees()).abs() < 1e-8);
        assert!((head.net_length_mm - 750.0 * outside_sweep).abs() < 1e-8);
        assert!((head.miter_left_deg - 72.01298753).abs() < 1e-6);
        assert!((head.miter_right_deg - 72.01298753).abs() < 1e-6);
        let jamb = find(&p, MemberType::FrameLeft);
        assert!((jamb.miter_left_deg - 71.11711482).abs() < 1e-6);
        assert_eq!(jamb.miter_right_deg, 90.0);
        assert_eq!(jamb.net_length_mm, 1500.0 - 67.0 - 300.0);
        let top = &p
            .arch_joints
            .iter()
            .find(|j| j.piece_id == jamb.piece_id)
            .unwrap()
            .joint;
        let t = top.tenon.as_ref().unwrap();
        assert_eq!(t.tenon_length_mm, 16.0);
        let upper_tip = t.tip[0][1].min(t.tip[1][1]);
        assert!((jamb.gross_length_mm - (1433.0 - upper_tip + 27.0 + SAW_KERF_MM)).abs() < 1e-7);
        let divider = find(&p, MemberType::DividerV);
        assert_eq!(divider.start_cope_radius_mm, Some(683.0));
        assert_eq!(divider.end_cope_radius_mm, None);
        let far_edge = 67.0 + k.grid.columns[0].size + 67.0;
        let highest = 750.0 - (683.0_f64.powi(2) - (600.0 - far_edge).powi(2)).sqrt();
        assert!((divider.net_length_mm - (1433.0 - highest)).abs() < 1e-7);
    }

    #[test]
    fn arch_foot_blanks_drive_stock_and_preview_even_when_the_bow_carries_the_tongue() {
        use crate::kozijn::{Project, ShapeType};
        let mut k = Kozijn::new("Boogvoeten", "B1", 1200., 1500.);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.frame.joints_configured = true;
        for j in &mut k.frame.corner_joints {
            j.through_member = ThroughMember::Dorpel;
        }
        k.frame.corner_joints[1].through_member = ThroughMember::Stijl;
        k.frame.corner_joints[1].pen_length = 24.;
        let p = compute_production_data(&k);
        assert_eq!(p.arch_joints.len(), 2);
        let (left, right) = (&p.arch_joints[0], &p.arch_joints[1]);
        assert_eq!(
            (
                &*left.piece_id,
                left.end,
                &*left.receiver_piece_id,
                left.receiver_end
            ),
            (
                "B1-SL",
                DividerJointEnd::Start,
                "B1-BO1",
                DividerJointEnd::Start
            )
        );
        assert_eq!(
            (
                &*right.piece_id,
                right.end,
                &*right.receiver_piece_id,
                right.receiver_end
            ),
            (
                "B1-BO1",
                DividerJointEnd::End,
                "B1-SR",
                DividerJointEnd::Start
            )
        );
        let head = find(&p, MemberType::FrameArc);
        let extension = right.joint.tenon.as_ref().unwrap().blank_extension_mm;
        assert!((head.gross_length_mm - head.net_length_mm - extension - SAW_KERF_MM).abs() < 1e-7);
        assert!(p.arch_joints.iter().all(|j| j
            .joint
            .tenon
            .as_ref()
            .unwrap()
            .z_from_inside_mm
            .len()
            == 2));
        let plan = crate::optimization::optimize_frame_cuts(std::slice::from_ref(&k), 5800.);
        assert_eq!(plan.curved_piece_ids, ["B1-BO1"]);
        let nested = plan
            .allocations
            .iter()
            .flat_map(|a| &a.cuts)
            .find(|i| i.piece_id == "B1-SL")
            .unwrap();
        assert_eq!(
            nested.length_mm,
            find(&p, MemberType::FrameLeft).gross_length_mm
        );
        let parts = crate::cnc::generate_cnc_parts(&k);
        let crown = parts.iter().find(|c| c.piece_id == "B1-BO1").unwrap();
        assert_eq!(crown.arch_joints.len(), 2);
        assert!(crown.operations.is_empty());
        let jamb = parts.iter().find(|c| c.piece_id == "B1-SL").unwrap();
        assert!(
            matches!(jamb.operations.as_slice(), [crate::cnc::CncOperation::CrossCut { position_mm, .. }] if *position_mm > 0.)
        );

        k.frame.corner_joints[1].pen_length = 1000.;
        let p = compute_production_data(&k);
        let joint = &p.arch_joints[1];
        assert!(joint.joint.tenon.is_none() && joint.joint.issue.is_some());
        assert_eq!(joint.receiver_piece_id, "B1-SR");
        assert!(p.blank_length_mm(find(&p, MemberType::FrameArc)).is_none());
        let plan = crate::optimization::optimize_frame_cuts(std::slice::from_ref(&k), 5800.);
        assert_eq!(plan.unresolved_piece_ids, ["B1-BO1"]);
        assert!(plan.curved_piece_ids.is_empty());
        let mut project = Project::new("Voeten", "B");
        project.kozijnen.push(k);
        let orders = crate::purchase_order::generate_purchase_orders(&project);
        let wood = orders.iter().find(|p| p.category == "Hout").unwrap();
        assert_eq!(wood.unresolved_piece_ids, ["B1-BO1"]);
        assert!(wood
            .lines
            .iter()
            .all(|l| !l.description.contains("Boogstuk")));
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
        assert!(
            (stile.gross_length_mm
                - (stile.net_length_mm + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM))
                .abs()
                < 0.01
        );
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
            assert!(
                (rail.gross_length_mm - (rail.net_length_mm + 2.0 * 30.0 + SAW_KERF_MM)).abs()
                    < 0.01
            );
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
            MemberType::FrameLeft,
            MemberType::FrameRight,
            MemberType::FrameTop,
            MemberType::FrameBottom,
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
            assert!(
                (item.gross_length_mm - (item.net_length_mm + 2.0 * fw + SAW_KERF_MM)).abs() < 0.01
            );
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
        assert!(
            rail.gross_length_mm >= 1200.0,
            "gross {} must cover the long point 1200",
            rail.gross_length_mm
        );
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
        assert!(
            (stile.gross_length_mm
                - (stile.net_length_mm + 2.0 * (fw + PVC_WELD_OVERMEASURE_MM) + SAW_KERF_MM))
                .abs()
                < 0.01
        );
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
        assert!(
            (stile.gross_length_mm - (stile.net_length_mm + 2.0 * fw + SAW_KERF_MM)).abs() < 0.01
        );
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
            MemberType::FrameLeft,
            MemberType::FrameRight,
            MemberType::FrameTop,
            MemberType::FrameBottom,
        ] {
            let a = find(&prod_a, mt);
            let b = find(&prod_b, mt);
            assert!((a.net_length_mm - b.net_length_mm).abs() < 0.01);
            assert!(
                (a.gross_length_mm - b.gross_length_mm).abs() < 0.01,
                "{:?}: default gross {} != configured gross {}",
                mt,
                a.gross_length_mm,
                b.gross_length_mm
            );
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
        assert!(
            (top_rail.gross_length_mm - (top_rail.net_length_mm + 2.0 * 20.0 + SAW_KERF_MM)).abs()
                < 0.01
        );

        // Onderdorpel: runs through full width, no pen allowance.
        let bottom_rail = find(&prod, MemberType::FrameBottom);
        assert!((bottom_rail.net_length_mm - k.frame.outer_width).abs() < 0.01);
        assert!(
            (bottom_rail.gross_length_mm - (bottom_rail.net_length_mm + SAW_KERF_MM)).abs() < 0.01
        );
    }

    #[test]
    fn test_glaslat_lengths_without_snapshot_use_material_norm_sponning() {
        // Kozijnen zonder profielsnapshot (elk nieuw kozijn/sjabloon) rekenen
        // met de materiaalnorm (hout 17 mm) i.p.v. de kale dagmaat-omtrek, zodat
        // de lat bij de glasmaat past. Identiek aan een expliciete 17 mm snapshot.
        let mut k = Kozijn::new("Test", "T09", 900.0, 1400.0);
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat::default()); // 15x17, verstek
        assert!(k.frame.profile_snapshot.is_none());

        let prod = compute_production_data(&k);
        assert_eq!(prod.glaslat_list.len(), 1);
        let item = &prod.glaslat_list[0];
        let sm_w = 766.0 + 2.0 * 17.0; // sponningmaat 800
        let sm_h = 1266.0 + 2.0 * 17.0; // sponningmaat 1300
        assert!((item.total_length_mm - (2.0 * (sm_w - 1.0) + 2.0 * (sm_h - 1.0))).abs() < 0.01);
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
    fn test_glass_size_without_snapshot_follows_material_norm() {
        // Een nieuw kozijn heeft nog geen profielsnapshot. Het glas moet dan
        // toch in de sponning vallen (norm hout 17 mm → dagmaat + 24), niet de
        // oude dagmaat − 8 mm: een ruit KLEINER dan de opening.
        let k = Kozijn::new("Test", "T12", 900.0, 1400.0);
        assert!(k.frame.profile_snapshot.is_none());

        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let day_w = 900.0 - 2.0 * 67.0; // 766
        let day_h = 1400.0 - 2.0 * 67.0; // 1266
        assert!((glass.width_mm - (day_w + 24.0)).abs() < 0.01);
        assert!((glass.height_mm - (day_h + 24.0)).abs() < 0.01);
        // Identiek aan een expliciete snapshot met dezelfde 17 mm sponning.
        let mut with_snap = k.clone();
        with_snap.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });
        let g2 = &compute_production_data(&with_snap).glass_list[0];
        assert!((g2.width_mm - glass.width_mm).abs() < 1e-9);
        assert!((g2.height_mm - glass.height_mm).abs() < 1e-9);
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
        assert!((glass.width_mm - (day_w + 46.0)).abs() < 0.01);
    }

    #[test]
    fn test_glass_size_operable_uses_sash_day_size() {
        // A 69 mm sash with a 14 mm overlap leaves 55 mm of visible sash.
        let mut k = Kozijn::new("Test", "T14", 900.0, 1400.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].sash_width = Some(69.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });

        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let day_w = (900.0 - 2.0 * 67.0) - 2.0 * 55.0;
        let day_h = (1400.0 - 2.0 * 67.0) - 2.0 * 55.0;
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
        assert!(
            bottom.net_length_mm < k.inner_width() - 1.0,
            "onderdorpel {} should be < dagmaat {}",
            bottom.net_length_mm,
            k.inner_width()
        );
        // Right stijl steps in: shorter than the full outer height.
        assert!(
            right.net_length_mm < k.frame.outer_height - 1.0,
            "right stijl {} should be < {}",
            right.net_length_mm,
            k.frame.outer_height
        );
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

        let stijlen: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|c| c.member_type == MemberType::DividerV)
            .collect();
        assert_eq!(
            stijlen.len(),
            1,
            "melkmeisje layout yields exactly one tussenstijl"
        );
        assert!(
            (stijlen[0].net_length_mm - inner_h).abs() < 0.01,
            "tussenstijl {} must run the full binnenhoogte {}",
            stijlen[0].net_length_mm,
            inner_h
        );

        // Exactly one borstweringdorpel (DividerH): the side-light zone width
        // (melkmeisje1 splits 900/500 over the available 1199 mm) extended by
        // fw on the right where the outer stijl stops op deze dorpel.
        let dorpels: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|c| c.member_type == MemberType::DividerH)
            .collect();
        assert_eq!(
            dorpels.len(),
            1,
            "melkmeisje layout yields exactly one borstweringdorpel"
        );
        let zijlicht_w = (inner_w - fw) * 500.0 / 1400.0; // 428.214…
        assert!(
            (dorpels[0].net_length_mm - (zijlicht_w + fw)).abs() < 0.01,
            "borstweringdorpel {} must be zijlicht width {} + fw {}",
            dorpels[0].net_length_mm,
            zijlicht_w,
            fw
        );

        // Sum of members covers the stepped bottom outline: onderdorpel
        // (under the deur, incl. corners) + borstweringdorpel = outer width.
        let bottom = find(&prod, MemberType::FrameBottom);
        assert!(
            (bottom.net_length_mm + dorpels[0].net_length_mm - k.frame.outer_width).abs() < 0.01,
            "onderdorpel {} + borstweringdorpel {} must span the outer width {}",
            bottom.net_length_mm,
            dorpels[0].net_length_mm,
            k.frame.outer_width
        );

        // Two panes (deur + zijlicht), none at the full binnenmaat.
        assert_eq!(prod.glass_list.len(), 2);
        for g in &prod.glass_list {
            assert!(
                g.width_mm < inner_w - 1.0,
                "glas {} mag niet de volle binnenbreedte {} zijn",
                g.width_mm,
                inner_w
            );
            assert!(
                g.height_mm < inner_h - 1.0,
                "glas {} mag niet de volle binnenhoogte {} zijn",
                g.height_mm,
                inner_h
            );
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
            VakChild {
                size: 1.0,
                node: raam("draaikiep"),
            },
            VakChild {
                size: 1.0,
                node: deur("enkel"),
            },
            VakChild {
                size: 1.0,
                node: paneel(),
            },
        ]));
        let prod = compute_production_data(&k);

        // Raamhout members for the raam vak (leaf 0).
        let raamhout: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|c| c.profile_name.starts_with("Raamhout"))
            .collect();
        assert_eq!(raamhout.len(), 4, "raam vak yields 4 raamhout members");
        assert!(raamhout.iter().any(|c| c.piece_id.ends_with("-RSL")));

        // Deurhout members for the deur vak (leaf 1) — the deur-item.
        let deurhout: Vec<_> = prod
            .cut_list
            .iter()
            .filter(|c| c.profile_name.starts_with("Deurhout"))
            .collect();
        assert_eq!(deurhout.len(), 4, "deur vak yields 4 deurhout members");
        assert!(deurhout.iter().any(|c| c.piece_id.ends_with("-DSL")));
        // The through stiles of the timber door overlap the head and sill.
        assert_eq!(deurhout[0].net_length_mm, k.inner_height() + 2.0 * 14.0);

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
        assert!(
            (a.total_length_mm - b.total_length_mm).abs() < 0.01,
            "layout bead length {} must match grid path {}",
            a.total_length_mm,
            b.total_length_mm
        );
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
        assert!(
            (prod_snap.glaslat_list[0].total_length_mm
                - prod_snap_grid.glaslat_list[0].total_length_mm)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn test_layout_vulling_glaslat_falls_back_to_default_spec() {
        // Without a cell spec the leaf side and the catalog's 17×17 bead apply.
        let mut k = Kozijn::new("Test", "GL2", 900.0, 1400.0);
        assert!(k.cells[0].glaslat.is_none());
        k.layout = Some(crate::layout::leaf(crate::layout::Vakvulling::Glas {
            glaslat: Some("buiten".into()),
        }));
        let prod = compute_production_data(&k);
        assert_eq!(prod.glaslat_list.len(), 1);
        let item = &prod.glaslat_list[0];
        assert_eq!(item.position, "Buiten");
        assert!((item.width_mm - 17.0).abs() < 1e-9);
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
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat {
            height_mm: 28.0,
            ..Default::default()
        });
        // 2 glass leaves vs 1 cell → misaligned.
        k.layout = Some(split_row(vec![
            VakChild {
                size: 1.0,
                node: glas(),
            },
            VakChild {
                size: 1.0,
                node: glas(),
            },
        ]));
        let prod = compute_production_data(&k);
        let default_type = crate::kozijn::Glazing::default().glass_type;
        assert_eq!(prod.glass_list.len(), 2);
        for g in &prod.glass_list {
            assert_eq!(
                g.glass_type, default_type,
                "misaligned cell data must not leak into layout leaves"
            );
        }
        // Both leaves use the profile default, not the unrelated 28 mm bead.
        assert_eq!(prod.glaslat_list.len(), 2);
        assert!(prod.glaslat_list.iter().all(|b| b.height_mm == 17.0));
    }

    #[test]
    fn system_edge_cover_and_sash_rebate_set_glass_and_bead_sizes() {
        use crate::geometry::compute_2d_geometry;
        use crate::glaslat::GlaslatPosition;
        use crate::profile::ProfileSnapshot;
        let mut k = Kozijn::new("Sponning", "S1", 1000.0, 1200.0);
        k.frame.material = Material::Pvc;
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            glasinval: Some(20.0),
            sash_overlap: Some(6.0),
            ..Default::default()
        });
        let prod = compute_production_data(&k);
        assert_eq!(prod.glass_list[0].width_mm, k.inner_width() + 40.0);
        let fixed = compute_2d_geometry(&k).cell_rects.remove(0).fit.unwrap();
        assert_eq!(fixed.glass.x - fixed.rebate.x, 8.0);

        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].sash_width = Some(60.0);
        k.cells[0].sash_profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(18.0),
            sponning_diepte: Some(52.0),
            glaslat_hoogte: Some(22.0),
            glaslat_breedte: Some(16.0),
            bead_position: Some(GlaslatPosition::Buiten),
            ..Default::default()
        });
        let fit = compute_2d_geometry(&k).cell_rects.remove(0).fit.unwrap();
        assert_eq!(
            fit.sash_outer.as_ref().unwrap().x,
            k.frame.frame_width - 6.0
        );
        assert_eq!(fit.day.x - fit.glass.x, 13.0);
        assert_eq!(fit.glass.x - fit.rebate.x, 5.0);
        assert_eq!(fit.rebate_depth, Some(52.0));
        let prod = compute_production_data(&k);
        assert_eq!(prod.glass_list[0].width_mm, fit.glass.width);
        assert_eq!(prod.glaslat_list[0].height_mm, 22.0);
        assert_eq!(prod.glaslat_list[0].width_mm, 16.0);
        assert_eq!(prod.glaslat_list[0].position, "Buiten");
    }

    #[test]
    fn double_door_has_two_fitted_leaves_and_two_matching_panes() {
        let mut k = Kozijn::new("Stolp", "D1", 1800.0, 2100.0);
        k.layout = Some(crate::layout::deur("dubbel"));
        let cr = crate::geometry::compute_2d_geometry(&k)
            .cell_rects
            .remove(0);
        let left = cr.fit.unwrap();
        let right = cr.second_fit.unwrap();
        let outer_left = left.sash_outer.unwrap();
        let outer_right = right.sash_outer.unwrap();
        assert_eq!(outer_left.x, cr.rect.x - 14.0);
        assert_eq!(outer_right.x - outer_left.x - outer_left.width, 6.0);
        assert_eq!(outer_left.width, outer_right.width);
        let prod = compute_production_data(&k);
        assert_eq!(prod.glass_list.len(), 2);
        assert_eq!(prod.glass_list[0].width_mm, left.glass.width);
        assert_eq!(prod.glass_list[1].width_mm, right.glass.width);
        assert_ne!(prod.glass_list[0].piece_id, prod.glass_list[1].piece_id);
        assert_eq!(prod.glaslat_list.len(), 2);
        assert_eq!(prod.cut_list.len(), 4 + 8);
    }

    #[test]
    fn round_glass_and_bead_use_concentric_radii() {
        let mut k = Kozijn::new("Rond", "R1", 1000.0, 1000.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        let pi = std::f64::consts::PI;
        assert_eq!((glass.width_mm, glass.height_mm), (890.0, 890.0));
        assert_eq!(glass.shape_label_nl(), "Rond Ø890.0");
        let bounds = glass.cut_contour.as_ref().unwrap().bounds();
        assert_eq!((bounds.x, bounds.y), (0.0, 0.0));
        let area = pi * 445.0_f64.powi(2) / 1e6;
        assert!((glass.area_m2 - area).abs() < 1e-9);
        assert!(
            (prod
                .bom
                .iter()
                .find(|b| b.category == "Glas")
                .unwrap()
                .quantity
                - area)
                .abs()
                < 1e-9
        );
        let rubber = prod
            .gasket_list
            .iter()
            .find(|g| g.gasket_type == GasketType::GlazingInner)
            .unwrap();
        assert!((rubber.length_mm - (2.0 * pi * 445.0 + 20.0)).abs() < 1e-7);
        let bead = &prod.glaslat_list[0];
        assert_eq!(bead.quantity, 1);
        assert_eq!(bead.contour_runs[0].radius_mm, Some(450.0));
        assert!((bead.total_length_mm - 2.0 * pi * 450.0).abs() < 1e-7);
    }

    #[test]
    fn segmental_arch_glass_area_and_bead_lengths_use_offset_intersections() {
        let mut k = Kozijn::new("Getoogd", "A1", 1200.0, 1500.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.0);
        let prod = compute_production_data(&k);
        let glass = &prod.glass_list[0];
        // The outer circle has R750 and its centre is 750 mm below the crown.
        // Glass and bead terminate at their own intersection with the jamb.
        let r: f64 = 695.0;
        let half: f64 = 545.0;
        let spring = 750.0 - (r * r - half * half).sqrt();
        let angle = 2.0 * (half / r).asin();
        let area = (1090.0 * (1445.0 - spring) + 0.5 * r * r * (angle - angle.sin())) / 1e6;
        assert!((glass.width_mm - 1090.0).abs() < 1e-7);
        assert!((glass.height_mm - 1390.0).abs() < 1e-7);
        assert!((glass.area_m2 - area).abs() < 1e-9);
        assert_eq!(glass.shape_label_nl(), "Boog R695.0");
        let bead = &prod.glaslat_list[0];
        let bead_angle = 2.0 * (550.0_f64 / 700.0).asin();
        let bead_spring = 750.0 - (700.0_f64.powi(2) - 550.0_f64.powi(2)).sqrt();
        let perimeter = 1100.0 + 2.0 * (1450.0 - bead_spring) + 700.0 * bead_angle;
        assert_eq!(bead.quantity, 4);
        assert_eq!(
            bead.contour_runs
                .iter()
                .filter(|run| run.radius_mm == Some(700.0))
                .count(),
            1
        );
        assert!((bead.total_length_mm - perimeter).abs() < 1e-7);
    }

    #[test]
    fn divided_round_glass_has_the_width_height_and_area_of_its_circular_segment() {
        let mut k = Kozijn::new("Gedeeld rond", "R2", 1000.0, 1000.0);
        k.frame.shape.shape_type = crate::kozijn::ShapeType::Round;
        k.add_column(300.0);
        k.grid.columns[0].size = 300.0;
        k.grid.columns[1].size = 499.0;
        let prod = compute_production_data(&k);
        assert_eq!(prod.glass_list.len(), 2);
        let left = &prod.glass_list[0];
        // The divider begins at x=367. The glass reaches 12 mm behind it.
        let r: f64 = 445.0;
        let distance: f64 = 500.0 - 379.0;
        let half_chord = (r * r - distance * distance).sqrt();
        let segment_area = r * r * (distance / r).acos() - distance * half_chord;
        assert!((left.width_mm - 324.0).abs() < 1e-7);
        assert!((left.height_mm - 2.0 * half_chord).abs() < 1e-7);
        assert!((left.area_m2 - segment_area / 1e6).abs() < 1e-9);
        assert_eq!(prod.glaslat_list[0].quantity, 2);
        assert_eq!(prod.glaslat_list[1].quantity, 2);
        let outline = left.cut_contour.as_ref().unwrap();
        let bounds = outline.bounds();
        assert!(bounds.x.abs() < 1e-7 && bounds.y.abs() < 1e-7);
        assert!(outline
            .segments
            .iter()
            .any(|s| matches!(s, Segment::Line { from, to }
            if (from[0] - 324.0).abs() < 1e-7 && (to[0] - 324.0).abs() < 1e-7)));
    }

    #[test]
    fn catalog_profile_by_id_fits_glass_and_clip_bead_without_saved_snapshot() {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        assert_eq!(sj.stijl_profile.id, "veka-softline82");
        let mut k = Kozijn::new_with_sjabloon("VEKA", "K1", 1200.0, 1000.0, &sj);
        k.cells[0].glazing.thickness_mm = 48.0;
        assert!(k.frame.profile_snapshot.is_none());
        let fit = crate::geometry::compute_2d_geometry(&k)
            .cell_rects
            .remove(0)
            .fit
            .unwrap();
        assert_eq!(fit.day.x - fit.glass.x, 20.0);
        assert_eq!(fit.day.x - fit.rebate.x, 28.0);
        assert_eq!(fit.bead_width, 11.5);
    }
}
