//! Free subdivision model — recursive split tree (mirrors ui/src/lib/layout.js).
//!
//! A vak is either a LEAF (with a vakvulling) or a SPLIT into child vakken along
//! a direction (row = side by side / vertical mullions; column = stacked /
//! horizontal transoms). Unlike the rigid columns×rows matrix every branch
//! subdivides independently, so localized mullions, side lights that start
//! higher (melkmeisje) and arbitrary compositions all follow. A `Buiten` leaf
//! marks an area outside the kozijn, which makes the frame outline step.
//!
//! Additive: attached to `Kozijn` via `layout: Option<VakNode>` (`#[serde(default)]`);
//! when present it supersedes the rectangular grid for geometry/production. JSON
//! is identical to the frontend tree; node `id`s are preserved on the roundtrip
//! (the canvas editor selects/splits/merges by id) but omitted when absent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Vakvulling {
    Glas,
    Raam {
        #[serde(default, rename = "openType")]
        open_type: String,
        /// Hinge side ("links"/"rechts"), set by the canvas editor. Optional
        /// with `skip_serializing_if` so payloads without it stay byte-equal
        /// (old bundles/.ofs files roundtrip unchanged).
        #[serde(default, rename = "hingeSide", skip_serializing_if = "Option::is_none")]
        hinge_side: Option<String>,
    },
    Deur {
        #[serde(default, rename = "doorKind")]
        door_kind: String,
        /// Hinge side ("links"/"rechts"); optional, see `Raam::hinge_side`.
        #[serde(default, rename = "hingeSide", skip_serializing_if = "Option::is_none")]
        hinge_side: Option<String>,
    },
    Paneel,
    Rooster,
    /// Outside the kozijn (wall) — no vak, no member; makes the outline step.
    Buiten,
}

impl Vakvulling {
    /// A real vak contributes frame/geometry; `Buiten` does not.
    pub fn is_buiten(&self) -> bool {
        matches!(self, Vakvulling::Buiten)
    }

    /// Serialized tag of this vulling ("glas", "raam", "deur", "paneel",
    /// "rooster", "buiten") — the same string the JSON payload carries in
    /// `type`, for consumers that only need the kind.
    pub fn type_str(&self) -> &'static str {
        match self {
            Vakvulling::Glas => "glas",
            Vakvulling::Raam { .. } => "raam",
            Vakvulling::Deur { .. } => "deur",
            Vakvulling::Paneel => "paneel",
            Vakvulling::Rooster => "rooster",
            Vakvulling::Buiten => "buiten",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VakChild {
    /// Desired size along the split direction (mm or relative weight).
    pub size: f64,
    pub node: VakNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum VakNode {
    Split {
        /// Frontend node id (canvas editing is id-based); preserved verbatim.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        direction: SplitDirection,
        children: Vec<VakChild>,
    },
    Leaf {
        /// Frontend node id (canvas editing is id-based); preserved verbatim.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        vulling: Vakvulling,
    },
}

// ── Geometry ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutLeaf {
    pub rect: LayoutRect,
    pub vulling: Vakvulling,
    /// Stable depth-first index of this leaf in the split tree. Every leaf
    /// counts (`Buiten` included), so the index depends only on the tree
    /// structure — changing a vulling never renumbers. Downstream geometry
    /// uses it as `cell_index` (with gaps where `Buiten` leaves sit).
    #[serde(default)]
    pub leaf_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutDivider {
    pub rect: LayoutRect,
    /// "v" = vertical mullion (tussenstijl), "h" = horizontal transom (tussendorpel).
    pub direction: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutGeometry {
    pub leaves: Vec<LayoutLeaf>,
    pub dividers: Vec<LayoutDivider>,
}

/// Recursively partition `rect` according to the split tree (mirrors layoutToRects).
pub fn compute_layout_geometry(node: &VakNode, rect: LayoutRect, divider_w: f64) -> LayoutGeometry {
    let mut out = LayoutGeometry::default();
    layout_rec(node, rect, divider_w, &mut out);
    out
}

fn layout_rec(node: &VakNode, rect: LayoutRect, divider_w: f64, out: &mut LayoutGeometry) {
    match node {
        VakNode::Leaf { vulling, .. } => {
            // Leaves arrive here in depth-first order, so the running length
            // of the output is exactly the stable depth-first leaf index.
            let leaf_index = out.leaves.len();
            out.leaves.push(LayoutLeaf { rect, vulling: vulling.clone(), leaf_index });
        }
        VakNode::Split { direction, children, .. } => {
            let horiz = matches!(direction, SplitDirection::Row);
            let total = if horiz { rect.width } else { rect.height };
            let n = children.len();
            let divider_space = if n > 1 { (n - 1) as f64 * divider_w } else { 0.0 };
            let avail = (total - divider_space).max(0.0);
            let size_sum: f64 = children.iter().map(|c| c.size).sum::<f64>().max(1e-9);
            let mut pos = if horiz { rect.x } else { rect.y };
            for (i, c) in children.iter().enumerate() {
                let len = avail * c.size / size_sum;
                let child_rect = if horiz {
                    LayoutRect { x: pos, y: rect.y, width: len, height: rect.height }
                } else {
                    LayoutRect { x: rect.x, y: pos, width: rect.width, height: len }
                };
                layout_rec(&c.node, child_rect, divider_w, out);
                pos += len;
                if i + 1 < n {
                    let div_rect = if horiz {
                        LayoutRect { x: pos, y: rect.y, width: divider_w, height: rect.height }
                    } else {
                        LayoutRect { x: rect.x, y: pos, width: rect.width, height: divider_w }
                    };
                    out.dividers.push(LayoutDivider {
                        rect: div_rect,
                        direction: if horiz { "v".into() } else { "h".into() },
                    });
                    pos += divider_w;
                }
            }
        }
    }
}

/// Number of real (non-buiten) vakken.
pub fn count_vakken(node: &VakNode) -> usize {
    match node {
        VakNode::Leaf { vulling, .. } => if vulling.is_buiten() { 0 } else { 1 },
        VakNode::Split { children, .. } => children.iter().map(|c| count_vakken(&c.node)).sum(),
    }
}

// ── Builders / templates ────────────────────────────────────────

pub fn leaf(vulling: Vakvulling) -> VakNode {
    VakNode::Leaf { id: None, vulling }
}
pub fn glas() -> VakNode {
    leaf(Vakvulling::Glas)
}
pub fn raam(open_type: &str) -> VakNode {
    leaf(Vakvulling::Raam { open_type: open_type.to_string(), hinge_side: None })
}
pub fn deur(door_kind: &str) -> VakNode {
    leaf(Vakvulling::Deur { door_kind: door_kind.to_string(), hinge_side: None })
}
pub fn paneel() -> VakNode {
    leaf(Vakvulling::Paneel)
}
pub fn buiten() -> VakNode {
    leaf(Vakvulling::Buiten)
}
fn child(size: f64, node: VakNode) -> VakChild {
    VakChild { size, node }
}
pub fn split_row(children: Vec<VakChild>) -> VakNode {
    VakNode::Split { id: None, direction: SplitDirection::Row, children }
}
pub fn split_col(children: Vec<VakChild>) -> VakNode {
    VakNode::Split { id: None, direction: SplitDirection::Column, children }
}

/// Side-light zone: glass side light on top (starts higher) over outside-the-kozijn.
fn zijlicht_zone() -> VakNode {
    split_col(vec![child(900.0, glas()), child(1000.0, buiten())])
}

/// Melkmeisje: full-height casement with a raised side light (stepped outline).
pub fn melkmeisje1() -> VakNode {
    split_row(vec![child(900.0, raam("draaikiep")), child(500.0, zijlicht_zone())])
}

/// Melkmeisje with side lights on both sides.
pub fn melkmeisje2() -> VakNode {
    split_row(vec![
        child(500.0, zijlicht_zone()),
        child(1000.0, raam("draaikiep")),
        child(500.0, zijlicht_zone()),
    ])
}

/// Arbitrary free grid of glass (e.g. 9×4 = 8 mullions + 3 transoms).
pub fn free_grid(cols: usize, rows: usize) -> VakNode {
    let make_row = || split_row((0..cols).map(|_| child(1.0, glas())).collect());
    split_col((0..rows).map(|_| child(1.0, make_row())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: LayoutRect = LayoutRect { x: 0.0, y: 0.0, width: 1400.0, height: 1900.0 };

    #[test]
    fn melkmeisje_side_light_sits_above_outside() {
        let geom = compute_layout_geometry(&melkmeisje1(), FULL, 90.0);
        // raam (full height) + glas (side light) + buiten (below) = 3 leaves
        assert_eq!(geom.leaves.len(), 3);
        let glas_leaf = geom.leaves.iter().find(|l| matches!(l.vulling, Vakvulling::Glas)).unwrap();
        let buiten_leaf = geom.leaves.iter().find(|l| matches!(l.vulling, Vakvulling::Buiten)).unwrap();
        // side light is above the outside (stepped outline), no panel
        assert!(glas_leaf.rect.y < buiten_leaf.rect.y);
        assert!(!geom.leaves.iter().any(|l| matches!(l.vulling, Vakvulling::Paneel)));
        // 2 real vakken (raam + glas), buiten does not count
        assert_eq!(count_vakken(&melkmeisje1()), 2);
    }

    #[test]
    fn free_grid_partitions_fully() {
        let geom = compute_layout_geometry(&free_grid(9, 4), FULL, 60.0);
        assert_eq!(geom.leaves.len(), 36);
        // total leaf width in the first row ≈ available width after 8 mullions
        assert_eq!(count_vakken(&free_grid(9, 4)), 36);
    }

    #[test]
    fn row_split_places_children_left_to_right() {
        let n = split_row(vec![child(1.0, glas()), child(1.0, raam("draai"))]);
        let geom = compute_layout_geometry(&n, FULL, 100.0);
        assert_eq!(geom.dividers.len(), 1);
        assert_eq!(geom.dividers[0].direction, "v");
        assert!(geom.leaves[0].rect.x < geom.leaves[1].rect.x);
    }

    #[test]
    fn leaves_carry_stable_depth_first_indices() {
        // melkmeisje1: raam (0), then zijlicht glas (1), then buiten (2) —
        // buiten counts in the numbering so vulling changes never renumber.
        let geom = compute_layout_geometry(&melkmeisje1(), FULL, 90.0);
        let idx: Vec<usize> = geom.leaves.iter().map(|l| l.leaf_index).collect();
        assert_eq!(idx, vec![0, 1, 2]);
        assert!(matches!(geom.leaves[0].vulling, Vakvulling::Raam { .. }));
        assert!(matches!(geom.leaves[2].vulling, Vakvulling::Buiten));
        assert_eq!(geom.leaves[0].vulling.type_str(), "raam");
    }

    #[test]
    fn vulling_hinge_side_roundtrips_and_stays_optional() {
        // hingeSide from the canvas editor survives the Rust roundtrip
        let json = r#"{"kind":"leaf","vulling":{"type":"raam","openType":"draai","hingeSide":"links"}}"#;
        let node: VakNode = serde_json::from_str(json).unwrap();
        match &node {
            VakNode::Leaf { vulling: Vakvulling::Raam { open_type, hinge_side }, .. } => {
                assert_eq!(open_type, "draai");
                assert_eq!(hinge_side.as_deref(), Some("links"));
            }
            other => panic!("expected raam leaf, got {:?}", other),
        }
        assert!(serde_json::to_string(&node).unwrap().contains("\"hingeSide\":\"links\""));
        // ...and payloads without it stay byte-equal (no null field appears)
        let plain = serde_json::to_string(&raam("draai")).unwrap();
        assert!(!plain.contains("hingeSide"), "plain raam grew a hingeSide field: {}", plain);
    }
}
