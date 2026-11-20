use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockLength {
    pub profile_id: String,
    pub material: String,
    pub available_length_mm: f64,
    pub cost_per_meter: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CutPlan {
    pub allocations: Vec<StockAllocation>,
    /// Positions made from curved stock, excluded from linear bar nesting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curved_piece_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_piece_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oversize_piece_ids: Vec<String>,
    pub total_waste_mm: f64,
    pub waste_percentage: f64,
    pub total_bars_used: usize,
    pub total_stock_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAllocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub profile_name: String,
    #[serde(default)]
    pub material: String,
    pub stock_length_mm: f64,
    pub cuts: Vec<AllocatedCut>,
    pub remaining_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocatedCut {
    pub piece_id: String,
    pub kozijn_mark: String,
    pub length_mm: f64,
}

/// Nest the actual straight production blanks. Curved stock is ordered by its
/// developed length and cannot be taken from one of these straight bars.
pub fn optimize_frame_cuts(kozijnen: &[crate::kozijn::Kozijn], stock_length_mm: f64) -> CutPlan {
    use std::collections::BTreeMap;
    let mut groups =
        BTreeMap::<(String, Option<String>, String), Vec<(String, String, f64)>>::new();
    let mut curved_piece_ids = Vec::new();
    let mut unresolved_piece_ids = Vec::new();
    for kozijn in kozijnen {
        let prod = crate::production::compute_production_data(kozijn);
        for cut in &prod.cut_list {
            if prod.blank_length_mm(cut).is_none() {
                unresolved_piece_ids.push(cut.piece_id.clone());
            } else if cut.arc.is_some() {
                curved_piece_ids.push(cut.piece_id.clone());
            } else {
                let (profile_id, profile_name) = cut_profile(kozijn, cut);
                groups
                    .entry((cut.material.clone(), profile_id, profile_name))
                    .or_default()
                    .push((
                        cut.piece_id.clone(),
                        prod.kozijn_mark.clone(),
                        cut.gross_length_mm,
                    ));
            }
        }
    }
    let mut plan = empty_plan();
    for ((material, profile_id, profile_name), pieces) in groups {
        // Production already includes the saw kerf once per blank.
        let mut group_plan = optimize_cut_list(pieces, stock_length_mm, 0.0);
        for allocation in &mut group_plan.allocations {
            allocation.profile_id = profile_id.clone();
            allocation.profile_name = profile_name.clone();
            allocation.material = material.clone();
        }
        plan.allocations.extend(group_plan.allocations);
        plan.oversize_piece_ids
            .extend(group_plan.oversize_piece_ids);
    }
    plan.curved_piece_ids = curved_piece_ids;
    plan.unresolved_piece_ids = unresolved_piece_ids;
    plan.total_bars_used = plan.allocations.len();
    plan.total_waste_mm = plan.allocations.iter().map(|a| a.remaining_mm).sum();
    let total_stock = plan.total_bars_used as f64 * stock_length_mm.max(0.0);
    plan.waste_percentage = if total_stock > 0.0 {
        (plan.total_waste_mm / total_stock * 1000.0).round() / 10.0
    } else {
        0.0
    };
    plan
}

fn cut_profile(
    kozijn: &crate::kozijn::Kozijn,
    cut: &crate::production::CutListItem,
) -> (Option<String>, String) {
    use crate::production::MemberType as M;
    let frame = &kozijn.frame;
    if cut.member_type == M::PerimeterStop {
        return (None, cut.profile_name.clone());
    }
    let profile = match cut.member_type {
        M::PerimeterStop => None,
        M::FrameTop => frame.top_profile.as_ref(),
        M::FrameBottom => frame
            .bottom_profile
            .as_ref()
            .or(frame.sill_profile.as_ref()),
        M::FrameLeft => frame.left_profile.as_ref(),
        M::FrameRight => frame.right_profile.as_ref(),
        M::FrameArc => match cut.profile_side {
            Some(crate::curved_timber::TimberProfileSide::Top) => frame.top_profile.as_ref(),
            Some(crate::curved_timber::TimberProfileSide::Bottom) => frame
                .bottom_profile
                .as_ref()
                .or(frame.sill_profile.as_ref()),
            _ => None,
        },
        M::DividerV | M::DividerH if kozijn.layout.is_none() => {
            let marker = if cut.member_type == M::DividerV {
                "-TS"
            } else {
                "-TD"
            };
            cut.piece_id.rfind(marker).and_then(|start| {
                let digits: String = cut.piece_id[start + marker.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let index = digits.parse::<usize>().ok()?;
                let divisions = if cut.member_type == M::DividerV {
                    &kozijn.grid.columns
                } else {
                    &kozijn.grid.rows
                };
                divisions
                    .get(index)
                    .and_then(|d| d.divider_profile.as_ref())
            })
        }
        M::SashArc | M::SashTop | M::SashBottom | M::SashLeft | M::SashRight => {
            cut.piece_id.rsplit_once("-V").and_then(|(_, suffix)| {
                let digits: String = suffix.chars().take_while(|c| c.is_ascii_digit()).collect();
                let cell = digits.parse::<usize>().ok()?.checked_sub(1)?;
                kozijn.cells.get(cell).and_then(|c| c.sash_profile.as_ref())
            })
        }
        _ => None,
    };
    profile
        .map(|p| (Some(p.id.clone()), p.name.clone()))
        .unwrap_or_else(|| (Some(frame.profile.id.clone()), cut.profile_name.clone()))
}

fn empty_plan() -> CutPlan {
    CutPlan {
        allocations: Vec::new(),
        curved_piece_ids: Vec::new(),
        unresolved_piece_ids: Vec::new(),
        oversize_piece_ids: Vec::new(),
        total_waste_mm: 0.0,
        waste_percentage: 0.0,
        total_bars_used: 0,
        total_stock_cost: 0.0,
    }
}

/// First Fit Decreasing algorithm for 1D cutting stock problem
pub fn optimize_cut_list(
    pieces: Vec<(String, String, f64)>, // (piece_id, kozijn_mark, gross_length_mm)
    stock_length_mm: f64,               // standard bar length (e.g., 5800mm for wood)
    saw_kerf_mm: f64,                   // blade width (typically 4mm)
) -> CutPlan {
    // Sort pieces descending by length; no part may overrun its own bar.
    let mut sorted: Vec<_> = pieces.into_iter().collect();
    sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut allocations: Vec<StockAllocation> = Vec::new();
    let mut oversize_piece_ids = Vec::new();

    for (piece_id, mark, length) in sorted {
        if !stock_length_mm.is_finite()
            || stock_length_mm <= 0.0
            || !length.is_finite()
            || length <= 0.0
            || !saw_kerf_mm.is_finite()
            || saw_kerf_mm < 0.0
            || length + saw_kerf_mm > stock_length_mm
        {
            oversize_piece_ids.push(piece_id);
            continue;
        }
        let cut_length = length + saw_kerf_mm;

        // Try to fit in existing bar
        let mut placed = false;
        for alloc in &mut allocations {
            if alloc.remaining_mm >= cut_length {
                alloc.remaining_mm -= cut_length;
                alloc.cuts.push(AllocatedCut {
                    piece_id: piece_id.clone(),
                    kozijn_mark: mark.clone(),
                    length_mm: length,
                });
                placed = true;
                break;
            }
        }

        // Open new bar
        if !placed {
            allocations.push(StockAllocation {
                profile_id: None,
                profile_name: String::new(),
                material: String::new(),
                stock_length_mm,
                cuts: vec![AllocatedCut {
                    piece_id,
                    kozijn_mark: mark,
                    length_mm: length,
                }],
                remaining_mm: stock_length_mm - cut_length,
            });
        }
    }

    let total_waste: f64 = allocations.iter().map(|a| a.remaining_mm).sum();
    let total_stock: f64 = allocations.len() as f64 * stock_length_mm;
    let waste_pct = if total_stock > 0.0 {
        total_waste / total_stock * 100.0
    } else {
        0.0
    };

    CutPlan {
        curved_piece_ids: Vec::new(),
        unresolved_piece_ids: Vec::new(),
        oversize_piece_ids,
        total_bars_used: allocations.len(),
        total_waste_mm: total_waste,
        waste_percentage: (waste_pct * 10.0).round() / 10.0,
        total_stock_cost: 0.0, // TODO: calculate from stock prices
        allocations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_profile_articles_never_share_a_stock_bar() {
        let mut veka = crate::kozijn::Kozijn::new("VEKA", "V01", 900.0, 1400.0);
        veka.frame.material = crate::kozijn::Material::Pvc;
        veka.frame.profile = crate::profile::ProfileRef {
            id: "veka-softline82".into(),
            name: "VEKA Softline 82".into(),
        };
        veka.frame.sill_profile = None;
        veka.frame.bottom_profile = None;
        let mut kommerling = crate::kozijn::Kozijn::new("Kömmerling", "K01", 900.0, 1400.0);
        kommerling.frame.material = crate::kozijn::Material::Pvc;
        kommerling.frame.profile = crate::profile::ProfileRef {
            id: "kommerling-76ad".into(),
            name: "Kömmerling 76 AD".into(),
        };
        kommerling.frame.sill_profile = None;
        kommerling.frame.bottom_profile = None;

        let plan = optimize_frame_cuts(&[veka, kommerling], 5800.0);
        let articles: Vec<_> = plan
            .allocations
            .iter()
            .filter_map(|bar| bar.profile_id.as_deref())
            .collect();
        assert!(articles.contains(&"veka-softline82"));
        assert!(articles.contains(&"kommerling-76ad"));
        for bar in &plan.allocations {
            let expected_mark = match bar.profile_id.as_deref() {
                Some("veka-softline82") => "V01",
                Some("kommerling-76ad") => "K01",
                _ => panic!("unexpected article {:?}", bar.profile_id),
            };
            assert!(bar.cuts.iter().all(|cut| cut.kozijn_mark == expected_mark));
        }
    }

    #[test]
    fn oversize_piece_is_reported_without_a_negative_remainder() {
        let plan = optimize_cut_list(vec![("K01-LANG".into(), "K01".into(), 5801.0)], 5800.0, 0.0);
        assert!(plan.allocations.is_empty());
        assert_eq!(plan.oversize_piece_ids, ["K01-LANG"]);
        assert_eq!(plan.total_bars_used, 0);
    }
}
