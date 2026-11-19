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
    pub total_waste_mm: f64,
    pub waste_percentage: f64,
    pub total_bars_used: usize,
    pub total_stock_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAllocation {
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
    let mut pieces = Vec::new();
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
                pieces.push((cut.piece_id.clone(), prod.kozijn_mark.clone(), cut.gross_length_mm));
            }
        }
    }
    // Production already includes the saw kerf once per blank.
    let mut plan = optimize_cut_list(pieces, stock_length_mm, 0.0);
    plan.curved_piece_ids = curved_piece_ids;
    plan.unresolved_piece_ids = unresolved_piece_ids;
    plan
}

/// First Fit Decreasing algorithm for 1D cutting stock problem
pub fn optimize_cut_list(
    pieces: Vec<(String, String, f64)>,  // (piece_id, kozijn_mark, gross_length_mm)
    stock_length_mm: f64,                // standard bar length (e.g., 5800mm for wood)
    saw_kerf_mm: f64,                    // blade width (typically 4mm)
) -> CutPlan {
    // Sort pieces descending by length
    let mut sorted: Vec<_> = pieces.into_iter().collect();
    sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut allocations: Vec<StockAllocation> = Vec::new();

    for (piece_id, mark, length) in sorted {
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
    let waste_pct = if total_stock > 0.0 { total_waste / total_stock * 100.0 } else { 0.0 };

    CutPlan {
        curved_piece_ids: Vec::new(),
        unresolved_piece_ids: Vec::new(),
        total_bars_used: allocations.len(),
        total_waste_mm: total_waste,
        waste_percentage: (waste_pct * 10.0).round() / 10.0,
        total_stock_cost: 0.0, // TODO: calculate from stock prices
        allocations,
    }
}
