use serde::{Deserialize, Serialize};

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

    // ── Frame members ──────────────────────────────────────────

    // For wood: stiles run full height, rails fit between stiles
    // For aluminum/PVC: all mitered, net = outer dimension - 2 * profile width (for rails) or outer dim (for stiles)
    let (stile_net, rail_net) = if is_miter {
        // Mitered: all members cut to outer dimensions minus profile width on each end
        (kozijn.frame.outer_height - 2.0 * fw, kozijn.frame.outer_width - 2.0 * fw)
    } else {
        // Wood: stiles full height, rails between stiles
        (kozijn.frame.outer_height, inner_w)
    };

    // Left stile
    cut_list.push(CutListItem {
        piece_id: format!("{}-SL", mark),
        member_type: MemberType::FrameLeft,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: stile_net,
        gross_length_mm: gross_length(stile_net, mat, is_miter),
        miter_left_deg: angle,
        miter_right_deg: angle,
        quantity: 1,
    });

    // Right stile
    cut_list.push(CutListItem {
        piece_id: format!("{}-SR", mark),
        member_type: MemberType::FrameRight,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: stile_net,
        gross_length_mm: gross_length(stile_net, mat, is_miter),
        miter_left_deg: angle,
        miter_right_deg: angle,
        quantity: 1,
    });

    // Top rail
    cut_list.push(CutListItem {
        piece_id: format!("{}-DB", mark),
        member_type: MemberType::FrameTop,
        profile_name: profile_name.clone(),
        material: mat_name.clone(),
        net_length_mm: rail_net,
        gross_length_mm: gross_length(rail_net, mat, is_miter),
        miter_left_deg: angle,
        miter_right_deg: angle,
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
        net_length_mm: rail_net,
        gross_length_mm: gross_length(rail_net, mat, is_miter),
        miter_left_deg: angle,
        miter_right_deg: angle,
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

        match cell.panel_type {
            PanelType::FixedGlass | PanelType::TurnTilt | PanelType::Turn
            | PanelType::Tilt | PanelType::Sliding | PanelType::Door => {
                // Glass
                let glass_w = cell_w - 2.0 * GLASS_CLEARANCE_MM;
                let glass_h = cell_h - 2.0 * GLASS_CLEARANCE_MM;
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
                panel_list.push(PanelListItem {
                    piece_id: cell_id.clone(),
                    cell_index: i,
                    width_mm: panel_w,
                    height_mm: panel_h,
                    panel_type: "Sandwichpaneel".into(),
                    quantity: 1,
                });
            }
            PanelType::Ventilation => {
                // Ventilation grille — no glass or panel
            }
        }

        // Sash frame for operable cells
        if cell.panel_type.is_operable() {
            let sash_w = cell_w;
            let sash_h = cell_h;
            let sash_is_miter = is_miter;
            let sash_angle = angle;

            let (sash_stile_net, sash_rail_net) = if sash_is_miter {
                (sash_h - 2.0 * fw, sash_w - 2.0 * fw)
            } else {
                (sash_h, sash_w - 2.0 * fw)
            };

            // Sash stiles (left + right)
            cut_list.push(CutListItem {
                piece_id: format!("{}-RSL", cell_id),
                member_type: MemberType::SashLeft,
                profile_name: format!("Raamhout {}", profile_name),
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
                profile_name: format!("Raamhout {}", profile_name),
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
                profile_name: format!("Raamhout {}", profile_name),
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
                profile_name: format!("Raamhout {}", profile_name),
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
}
