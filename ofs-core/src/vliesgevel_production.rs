use serde::{Deserialize, Serialize};

use crate::production::{BomItem, CutListItem, GasketListItem, GasketType, GlassListItem, MemberType};
use crate::vliesgevel::Vliesgevel;

const SAW_KERF_MM: f64 = 4.0;
const GLASS_CLEARANCE_MM: f64 = 4.0;
const GASKET_OVERLAP_MM: f64 = 20.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VliesgevalProductionData {
    pub mark: String,
    pub name: String,
    pub mullion_list: Vec<CutListItem>,
    pub transom_list: Vec<CutListItem>,
    pub glass_list: Vec<GlassListItem>,
    pub gasket_list: Vec<GasketListItem>,
    pub bom: Vec<BomItem>,
}

pub fn compute_vliesgevel_production(vg: &Vliesgevel) -> VliesgevalProductionData {
    let mark = &vg.mark;
    let mut mullion_list = Vec::new();
    let mut transom_list = Vec::new();
    let mut glass_list = Vec::new();
    let mut gasket_list = Vec::new();

    // Mullions: full height, miter cut at 45 degrees (aluminum)
    for (i, m) in vg.mullions.iter().enumerate() {
        let net = vg.overall_height;
        mullion_list.push(CutListItem {
            piece_id: format!("{}-M{}", mark, i + 1),
            member_type: MemberType::DividerV,
            profile_name: m.profile.as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| vg.mullion_profile.name.clone()),
            material: "Aluminium".into(),
            net_length_mm: net,
            gross_length_mm: net + 2.0 * SAW_KERF_MM,
            miter_left_deg: 90.0,
            miter_right_deg: 90.0,
            quantity: 1,
        });
    }

    // Edge mullions (left + right border)
    for (id_suffix, member_type) in [("ML", MemberType::FrameLeft), ("MR", MemberType::FrameRight)] {
        let net = vg.overall_height;
        mullion_list.push(CutListItem {
            piece_id: format!("{}-{}", mark, id_suffix),
            member_type,
            profile_name: vg.mullion_profile.name.clone(),
            material: "Aluminium".into(),
            net_length_mm: net,
            gross_length_mm: net + 2.0 * SAW_KERF_MM,
            miter_left_deg: 90.0,
            miter_right_deg: 90.0,
            quantity: 1,
        });
    }

    // Transoms: span between mullions in each zone
    let col_zones = vg.column_zones();
    for (i, t) in vg.transoms.iter().enumerate() {
        for (col, (x1, x2)) in col_zones.iter().enumerate() {
            let net = x2 - x1;
            if net > 0.0 {
                transom_list.push(CutListItem {
                    piece_id: format!("{}-T{}-{}", mark, i + 1, col + 1),
                    member_type: MemberType::DividerH,
                    profile_name: t.profile.as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| vg.transom_profile.name.clone()),
                    material: "Aluminium".into(),
                    net_length_mm: net,
                    gross_length_mm: net + 2.0 * SAW_KERF_MM,
                    miter_left_deg: 90.0,
                    miter_right_deg: 90.0,
                    quantity: 1,
                });
            }
        }
    }

    // Top + bottom transoms (edge transoms)
    for (id_suffix, member_type) in [("TB", MemberType::FrameBottom), ("TT", MemberType::FrameTop)] {
        for (col, (x1, x2)) in col_zones.iter().enumerate() {
            let net = x2 - x1;
            if net > 0.0 {
                transom_list.push(CutListItem {
                    piece_id: format!("{}-{}-{}", mark, id_suffix, col + 1),
                    member_type,
                    profile_name: vg.transom_profile.name.clone(),
                    material: "Aluminium".into(),
                    net_length_mm: net,
                    gross_length_mm: net + 2.0 * SAW_KERF_MM,
                    miter_left_deg: 90.0,
                    miter_right_deg: 90.0,
                    quantity: 1,
                });
            }
        }
    }

    // Panels: glass or opaque
    let row_zones = vg.row_zones();
    for panel in &vg.panels {
        if panel.col >= col_zones.len() || panel.row >= row_zones.len() {
            continue;
        }
        let (x1, x2) = col_zones[panel.col];
        let (y1, y2) = row_zones[panel.row];
        let pw = (x2 - x1).max(0.0);
        let ph = (y2 - y1).max(0.0);

        if panel.panel_type.is_glazed() {
            let gw = pw - 2.0 * GLASS_CLEARANCE_MM;
            let gh = ph - 2.0 * GLASS_CLEARANCE_MM;
            if gw > 0.0 && gh > 0.0 {
                let glazing = panel.glazing.as_ref();
                glass_list.push(GlassListItem {
                    piece_id: format!("{}-G{}-{}", vg.mark, panel.row + 1, panel.col + 1),
                    cell_index: panel.row * vg.num_cols() + panel.col,
                    glass_type: glazing.map(|g| g.glass_type.clone()).unwrap_or("HR++".into()),
                    width_mm: gw,
                    height_mm: gh,
                    thickness_mm: glazing.map(|g| g.thickness_mm).unwrap_or(24.0),
                    ug_value: glazing.map(|g| g.ug_value).unwrap_or(1.0),
                    area_m2: (gw * gh) / 1e6,
                    quantity: 1,
                });

                // Glazing gaskets per panel
                let perimeter = 2.0 * (gw + gh) + GASKET_OVERLAP_MM;
                gasket_list.push(GasketListItem {
                    gasket_type: GasketType::GlazingInner,
                    length_mm: perimeter,
                    quantity: 1,
                });
                gasket_list.push(GasketListItem {
                    gasket_type: GasketType::GlazingOuter,
                    length_mm: perimeter,
                    quantity: 1,
                });
            }
        }
    }

    // BOM
    let mut bom = Vec::new();
    let total_mullion_m: f64 = mullion_list.iter().map(|c| c.gross_length_mm / 1000.0).sum();
    let total_transom_m: f64 = transom_list.iter().map(|c| c.gross_length_mm / 1000.0).sum();
    bom.push(BomItem {
        category: "Stijlprofiel".into(),
        description: vg.mullion_profile.name.clone(),
        unit: "m".into(),
        quantity: total_mullion_m,
    });
    bom.push(BomItem {
        category: "Regelprofiel".into(),
        description: vg.transom_profile.name.clone(),
        unit: "m".into(),
        quantity: total_transom_m,
    });

    let total_glass_m2: f64 = glass_list.iter().map(|g| g.area_m2).sum();
    if total_glass_m2 > 0.0 {
        bom.push(BomItem {
            category: "Glas".into(),
            description: glass_list.first().map(|g| g.glass_type.clone()).unwrap_or_default(),
            unit: "m2".into(),
            quantity: total_glass_m2,
        });
    }

    let total_gasket_m: f64 = gasket_list.iter().map(|g| g.length_mm / 1000.0).sum();
    bom.push(BomItem {
        category: "Rubber".into(),
        description: "EPDM afdichting".into(),
        unit: "m".into(),
        quantity: total_gasket_m,
    });

    VliesgevalProductionData {
        mark: vg.mark.clone(),
        name: vg.name.clone(),
        mullion_list,
        transom_list,
        glass_list,
        gasket_list,
        bom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vliesgevel_grid::create_regular_grid;

    #[test]
    fn test_production_regular_grid() {
        let vg = create_regular_grid(6000.0, 3600.0, 1500.0, 1200.0);
        let prod = compute_vliesgevel_production(&vg);

        // 3 inner mullions + 2 edge mullions = 5
        assert_eq!(prod.mullion_list.len(), 5);

        // 2 inner transoms * 4 zones + 2 edge transoms * 4 zones = 16
        assert_eq!(prod.transom_list.len(), 16);

        // 12 glass panels (4 x 3)
        assert_eq!(prod.glass_list.len(), 12);

        // Each glass has inner + outer gasket = 24
        assert_eq!(prod.gasket_list.len(), 24);

        assert!(!prod.bom.is_empty());
    }
}
