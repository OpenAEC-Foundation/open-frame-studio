//! Per-supplier purchase orders — module 8 (procurement).
//!
//! Wraps the category purchase proposals (`procurement::generate_purchase_proposals`)
//! into formal, per-supplier purchase-order documents: order reference, project /
//! delivery header, line items, and a VAT (BTW) breakdown. This is the **offline**
//! document that precedes any online ordering — actually transmitting an order to
//! a supplier is an outward-facing action and is intentionally not performed here.
//!
//! There is no supplier registry in the project model yet, so the supplier name is
//! emitted as a per-category placeholder to be filled in.

use serde::{Deserialize, Serialize};

use crate::kozijn::Project;
use crate::procurement::generate_purchase_proposals;

const DEFAULT_BTW_PCT: f64 = 21.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderLine {
    pub description: String,
    pub article_number: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub unit_price: f64,
    pub total: f64,
    pub kozijn_marks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrder {
    pub order_number: String,
    pub category: String,
    pub supplier_name: String,
    pub project_name: String,
    pub project_number: String,
    pub delivery_address: String,
    pub lines: Vec<PurchaseOrderLine>,
    pub line_count: usize,
    pub subtotal_excl_btw: f64,
    pub btw_pct: f64,
    pub btw_amount: f64,
    pub total_incl_btw: f64,
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Build per-supplier purchase orders for the project (one per material category).
pub fn generate_purchase_orders(project: &Project) -> Vec<PurchaseOrder> {
    let info = &project.project_info;
    let prefix = if info.number.trim().is_empty() {
        "OFS".to_string()
    } else {
        info.number.clone()
    };

    generate_purchase_proposals(project)
        .into_iter()
        .map(|proposal| {
            let category = proposal.category;
            let lines: Vec<PurchaseOrderLine> = proposal
                .items
                .into_iter()
                .map(|it| PurchaseOrderLine {
                    description: it.description,
                    article_number: it.article_number,
                    quantity: round2(it.quantity),
                    unit: it.unit,
                    unit_price: round2(it.unit_price),
                    total: round2(it.total),
                    kozijn_marks: it.kozijn_marks,
                })
                .collect();

            let subtotal = round2(proposal.total_excl_btw);
            let btw_amount = round2(subtotal * DEFAULT_BTW_PCT / 100.0);

            PurchaseOrder {
                order_number: format!("{}-PO-{}", prefix, category),
                supplier_name: format!("[Leverancier {} invullen]", category),
                project_name: info.name.clone(),
                project_number: info.number.clone(),
                delivery_address: info.address.clone(),
                line_count: lines.len(),
                lines,
                subtotal_excl_btw: subtotal,
                btw_pct: DEFAULT_BTW_PCT,
                btw_amount,
                total_incl_btw: round2(subtotal + btw_amount),
                category,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn orders_have_btw_breakdown() {
        let mut project = Project::new("Testproject", "2026-001");
        project.kozijnen.push(Kozijn::new("A", "A01", 1000.0, 1500.0));
        let orders = generate_purchase_orders(&project);

        assert!(!orders.is_empty());
        for o in &orders {
            assert!(o.order_number.starts_with("2026-001-PO-"));
            assert_eq!(o.btw_pct, 21.0);
            // total = subtotal + btw (within rounding)
            assert!((o.total_incl_btw - (o.subtotal_excl_btw + o.btw_amount)).abs() < 0.05);
            assert_eq!(o.line_count, o.lines.len());
            assert_eq!(o.project_number, "2026-001");
        }
    }

    #[test]
    fn empty_project_has_no_orders() {
        let project = Project::new("Leeg", "");
        assert!(generate_purchase_orders(&project).is_empty());
    }
}
