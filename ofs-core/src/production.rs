use serde::{Deserialize, Serialize};

use crate::joint::{Joint, JointType, ThroughMember};
use crate::kozijn::{Kozijn, Material, PanelType};

// ── Cut list constants ─────────────────────────────────────────────

const WOOD_PEN_ALLOWANCE_MM: f64 = 20.0;   // tenon allowance per side
const SAW_KERF_MM: f64 = 4.0;              // saw blade width
const PVC_WELD_OVERMEASURE_MM: f64 = 4.0;  // weld overmeasure per side
const GLASS_CLEARANCE_MM: f64 = 4.0;       // glass clearance per side
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

/// `Kozijn::new` auto-populates four `Joint::default()` entries (pen/slis,
/// stijl through, 90 deg, pen 20 mm) regardless of material. An untouched
/// default set therefore carries no user intent: treat it as "not configured"
/// so aluminum/PVC frames keep their miter default and wood frames keep the
/// historic gross lengths.
fn is_untouched_default_joint(j: &Joint) -> bool {
    j.joint_type == JointType::PenSlis
        && j.through_member == ThroughMember::Stijl
        && (j.angle - 90.0).abs() < 1e-6
        && (j.pen_length - 20.0).abs() < 1e-6
}

/// The four corner joints in [TL, TR, BL, BR] order, or `None` when the
/// material-based fallback should be used.
fn effective_corner_joints(kozijn: &Kozijn) -> Option<[&Joint; 4]> {
    let joints = &kozijn.frame.corner_joints;
    if joints.len() < 4 {
        return None;
    }
    if joints[..4].iter().all(is_untouched_default_joint) {
        return None;
    }
    Some([&joints[0], &joints[1], &joints[2], &joints[3]])
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

/// Extra gross-length allowance at one end (tenon length, weld overmeasure).
fn joint_end_allowance(orientation: FrameOrientation, joint: &Joint, material: &Material) -> f64 {
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
        JointType::Verstek => match material {
            Material::Pvc => PVC_WELD_OVERMEASURE_MM,
            _ => 0.0,
        },
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
                + joint_end_allowance(orientation, start, material)
                + joint_end_allowance(orientation, end, material);
            MemberCut {
                net,
                gross,
                angle_start: joint_end_angle(start),
                angle_end: joint_end_angle(end),
            }
        }
        None => {
            // Historic behaviour: mitered members (alu/PVC) lose fw at both
            // ends; wood stijlen run full height, wood dorpels fit between.
            let net = match (orientation, default_is_miter) {
                (FrameOrientation::Stijl, false) => outer_span,
                _ => outer_span - 2.0 * fw,
            };
            MemberCut {
                net,
                gross: gross_length(net, material, default_is_miter),
                angle_start: default_angle,
                angle_end: default_angle,
            }
        }
    }
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
    // rails between; alu/PVC: all mitered at 45°).
    let corner_joints = effective_corner_joints(kozijn);

    let left_stile = frame_member_cut(
        FrameOrientation::Stijl,
        kozijn.frame.outer_height,
        corner_joints.map(|j| (j[0], j[2])), // top-left, bottom-left
        mat, fw, is_miter, angle,
    );
    let right_stile = frame_member_cut(
        FrameOrientation::Stijl,
        kozijn.frame.outer_height,
        corner_joints.map(|j| (j[1], j[3])), // top-right, bottom-right
        mat, fw, is_miter, angle,
    );
    let top_rail = frame_member_cut(
        FrameOrientation::Dorpel,
        kozijn.frame.outer_width,
        corner_joints.map(|j| (j[0], j[1])), // top-left, top-right
        mat, fw, is_miter, angle,
    );
    let bottom_rail = frame_member_cut(
        FrameOrientation::Dorpel,
        kozijn.frame.outer_width,
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

    // ── Dividers ───────────────────────────────────────────────

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

    // ── Cells: glass, panels, hardware, gaskets ────────────────

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
                // Glass — for operable cells, subtract sash frame width
                let glass_w = if has_sash && sash_fw > 0.0 {
                    cell_w - 2.0 * sash_fw - 2.0 * GLASS_CLEARANCE_MM
                } else {
                    cell_w - 2.0 * GLASS_CLEARANCE_MM
                };
                let glass_h = if has_sash && sash_fw > 0.0 {
                    cell_h - 2.0 * sash_fw - 2.0 * GLASS_CLEARANCE_MM
                } else {
                    cell_h - 2.0 * GLASS_CLEARANCE_MM
                };
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
        // that has one. Cut lengths per KVT 12.3.2 when the frame profile
        // snapshot is present: sponningmaat = dagmaat + 2 × sponninghoogte;
        // horizontale latten lopen door (sponningmaat − 1), verticale latten
        // passen ertussen (sponningmaat − 2 × lathoogte − 1, binnenbeglazing);
        // in verstek (mitered) lopen alle vier tot de hoek (sponningmaat − 1).
        // Voor draaiende delen benaderen we de vleugelsponning met de
        // kozijnsnapshot-waarde (een eigen vleugelprofiel-snapshot is een
        // gedocumenteerde follow-up). Kozijnen zonder snapshot houden de
        // historische dagmaat-omtrek, zodat oude projecten identiek blijven.
        if let Some(gl) = cell.glaslat.as_ref() {
            let (gw, gh) = if cell.panel_type.is_operable() {
                let sw = cell.sash_width.unwrap_or(67.0);
                (cell_w - 2.0 * sw, cell_h - 2.0 * sw)
            } else {
                (cell_w, cell_h)
            };
            let total_length = match kozijn.frame.profile_snapshot.as_ref() {
                Some(snap) => {
                    let sh = snap.resolved_sponning_hoogte(mat);
                    let sponningmaat_w = gw.max(0.0) + 2.0 * sh;
                    let sponningmaat_h = gh.max(0.0) + 2.0 * sh;
                    let horizontaal = (sponningmaat_w - 1.0).max(0.0);
                    let verticaal = if gl.mitered {
                        (sponningmaat_h - 1.0).max(0.0)
                    } else {
                        (sponningmaat_h - 2.0 * gl.height_mm - 1.0).max(0.0)
                    };
                    2.0 * horizontaal + 2.0 * verticaal
                }
                None => 2.0 * (gw.max(0.0) + gh.max(0.0)),
            };
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
            let sash_frame_w = cell.sash_width.unwrap_or(54.0);

            let (sash_stile_net, sash_rail_net) = if sash_is_miter {
                (sash_h - 2.0 * sash_frame_w, sash_w - 2.0 * sash_frame_w)
            } else {
                (sash_h, sash_w - 2.0 * sash_frame_w)
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
                gross_length_mm: gross_length(sash_stile_net, mat, sash_is_miter),
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
                gross_length_mm: gross_length(sash_stile_net, mat, sash_is_miter),
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
                gross_length_mm: gross_length(sash_rail_net, mat, sash_is_miter),
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
                gross_length_mm: gross_length(sash_rail_net, mat, sash_is_miter),
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
        // (a) Empty corner_joints must produce exactly the historic
        // material-based saw list — and be identical to the untouched
        // auto-populated default joint set.
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

        // Historic wood numbers: stiles full height, rails between stiles,
        // gross = net + 2 * pen allowance + kerf, square cuts.
        let fw = k_empty.frame.frame_width;
        let stile = find(&prod_empty, MemberType::FrameLeft);
        assert!((stile.net_length_mm - k_empty.frame.outer_height).abs() < 0.01);
        assert!((stile.gross_length_mm
            - (stile.net_length_mm + 2.0 * WOOD_PEN_ALLOWANCE_MM + SAW_KERF_MM)).abs() < 0.01);
        assert!((stile.miter_left_deg - 90.0).abs() < 0.01);
        let rail = find(&prod_empty, MemberType::FrameTop);
        assert!((rail.net_length_mm - (k_empty.frame.outer_width - 2.0 * fw)).abs() < 0.01);
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
        // default — 45° cuts on both ends of every member, net = short edge.
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
            // Wood miter: no weld overmeasure, just kerf.
            assert!((item.gross_length_mm - (item.net_length_mm + SAW_KERF_MM)).abs() < 0.01);
        }
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
        let stile = find(&prod, MemberType::FrameLeft);
        assert!((stile.gross_length_mm
            - (stile.net_length_mm + 2.0 * PVC_WELD_OVERMEASURE_MM + SAW_KERF_MM)).abs() < 0.01);
    }

    #[test]
    fn test_aluminum_untouched_defaults_keep_miter() {
        // Regression guard: alu/PVC kozijnen whose corner_joints are still the
        // auto-populated pen/slis defaults must keep the material-based miter.
        let mut k = Kozijn::new("Test", "T07", 900.0, 1400.0);
        k.frame.material = Material::Aluminum;
        // Kozijn::new already populated 4 default joints; leave them untouched.
        assert_eq!(k.frame.corner_joints.len(), 4);

        let prod = compute_production_data(&k);
        let fw = k.frame.frame_width;
        let stile = find(&prod, MemberType::FrameLeft);
        assert!((stile.net_length_mm - (k.frame.outer_height - 2.0 * fw)).abs() < 0.01);
        assert!((stile.miter_left_deg - 45.0).abs() < 0.01);
        // Historic alu gross: net + 2 * kerf.
        assert!((stile.gross_length_mm - (stile.net_length_mm + 2.0 * SAW_KERF_MM)).abs() < 0.01);
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
}
