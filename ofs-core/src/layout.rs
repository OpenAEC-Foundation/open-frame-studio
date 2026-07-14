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
    Glas {
        /// Glaslat position ("binnen"/"buiten"), set by the canvas editor.
        /// Optional with `skip_serializing_if` so `{"type":"glas"}` payloads
        /// stay byte-equal (old bundles/.ofs files roundtrip unchanged).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        glaslat: Option<String>,
    },
    Raam {
        #[serde(default, rename = "openType")]
        open_type: String,
        /// Hinge side ("links"/"rechts"), set by the canvas editor. Optional
        /// with `skip_serializing_if` so payloads without it stay byte-equal
        /// (old bundles/.ofs files roundtrip unchanged).
        #[serde(default, rename = "hingeSide", skip_serializing_if = "Option::is_none")]
        hinge_side: Option<String>,
        /// Glaslat position ("binnen"/"buiten"); optional, see `Glas::glaslat`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        glaslat: Option<String>,
        /// Opening direction: `Some(true)` = buitendraaiend, `Some(false)` =
        /// binnendraaiend; optional so an unset value never appears in JSON.
        #[serde(default, rename = "opensOutward", skip_serializing_if = "Option::is_none")]
        opens_outward: Option<bool>,
        /// Hang- en sluitwerk toggle from the canvas editor; optional, an
        /// explicit `false` roundtrips as `false` (never silently dropped).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hardware: Option<bool>,
    },
    Deur {
        #[serde(default, rename = "doorKind")]
        door_kind: String,
        /// Hinge side ("links"/"rechts"); optional, see `Raam::hinge_side`.
        #[serde(default, rename = "hingeSide", skip_serializing_if = "Option::is_none")]
        hinge_side: Option<String>,
        /// Glaslat position ("binnen"/"buiten") for glazed doors — the
        /// grid→layout conversion (ui/src/lib/layout.js `cellToVulling`)
        /// carries it onto door vullingen too; optional, see `Glas::glaslat`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        glaslat: Option<String>,
        /// Opening direction; optional, see `Raam::opens_outward`.
        #[serde(default, rename = "opensOutward", skip_serializing_if = "Option::is_none")]
        opens_outward: Option<bool>,
        /// Hang- en sluitwerk toggle; optional, see `Raam::hardware`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hardware: Option<bool>,
    },
    Paneel {
        /// Infill spec (sandwich/massief/…), same shape as
        /// `Cell::panel_filling`; optional so `{"type":"paneel"}` stays
        /// byte-equal on the roundtrip.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filling: Option<crate::panel_filling::PanelFilling>,
    },
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
            Vakvulling::Glas { .. } => "glas",
            Vakvulling::Raam { .. } => "raam",
            Vakvulling::Deur { .. } => "deur",
            Vakvulling::Paneel { .. } => "paneel",
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
    let mut out = compute_layout_geometry_unclipped(node, rect, divider_w);
    clip_dividers_against_buiten(&mut out);
    out
}

/// Same partitioning as `compute_layout_geometry` but WITHOUT the `Buiten`
/// divider-clipping pass: dividers keep their full drawn spans. The clip
/// serves the DRAWING (a mullion visually stops at a melkmeisje step);
/// production resolves its own member lengths from these raw dividers, so a
/// stretch clipped away for the picture (the side-light onderdorpel on the
/// borstwering) is still available as a member to cut. Leaves are identical
/// in both variants. Backend-only — the JS mirror keeps the clipped form.
pub fn compute_layout_geometry_unclipped(
    node: &VakNode,
    rect: LayoutRect,
    divider_w: f64,
) -> LayoutGeometry {
    let mut out = LayoutGeometry::default();
    layout_rec(node, rect, divider_w, &mut out);
    out
}

/// Clip divider segments so a mullion/transom only spans the overlap of two
/// adjacent NON-`Buiten` regions (issue 10). A divider that borders the wall
/// (`Buiten`) on either side loses that part: the melkmeisje tussenstijl stops
/// at the bottom of the side-light instead of running down through the wall,
/// and the side-light onderdorpel (which has wall directly below it) drops out
/// entirely. Divider stretches away from any `Buiten` region are untouched, so
/// every non-melkmeisje layout stays byte-identical. Mirrors the same pass in
/// `layoutToRects` (ui/src/lib/layout.js) — keep the two in sync.
fn clip_dividers_against_buiten(out: &mut LayoutGeometry) {
    const EPS: f64 = 1.0;
    let buiten: Vec<LayoutRect> = out
        .leaves
        .iter()
        .filter(|l| l.vulling.is_buiten())
        .map(|l| l.rect)
        .collect();
    if buiten.is_empty() {
        return;
    }
    let mut kept: Vec<LayoutDivider> = Vec::new();
    for d in out.dividers.drain(..) {
        let r = d.rect;
        if d.direction == "v" {
            // Vertical mullion: spans Y, its left/right edges border regions.
            let (lo, hi) = (r.y, r.y + r.height);
            let (left_x, right_x) = (r.x, r.x + r.width);
            let mut blocked: Vec<(f64, f64)> = Vec::new();
            for b in &buiten {
                let adjacent = (b.x + b.width - left_x).abs() < EPS || (b.x - right_x).abs() < EPS;
                if adjacent {
                    blocked.push((b.y.max(lo), (b.y + b.height).min(hi)));
                }
            }
            for (a, c) in subtract_intervals(lo, hi, &blocked, EPS) {
                kept.push(LayoutDivider {
                    rect: LayoutRect { x: r.x, y: a, width: r.width, height: c - a },
                    direction: "v".into(),
                });
            }
        } else {
            // Horizontal transom: spans X, its top/bottom edges border regions.
            let (lo, hi) = (r.x, r.x + r.width);
            let (top_y, bot_y) = (r.y, r.y + r.height);
            let mut blocked: Vec<(f64, f64)> = Vec::new();
            for b in &buiten {
                let adjacent = (b.y + b.height - top_y).abs() < EPS || (b.y - bot_y).abs() < EPS;
                if adjacent {
                    blocked.push((b.x.max(lo), (b.x + b.width).min(hi)));
                }
            }
            for (a, c) in subtract_intervals(lo, hi, &blocked, EPS) {
                kept.push(LayoutDivider {
                    rect: LayoutRect { x: a, y: r.y, width: c - a, height: r.height },
                    direction: "h".into(),
                });
            }
        }
    }
    out.dividers = kept;
}

/// `[lo, hi]` minus the union of `blocked` intervals, as a list of remaining
/// sub-intervals (each wider than `eps`). Shared with production's own
/// divider-member resolution (`crate::production`).
pub(crate) fn subtract_intervals(
    lo: f64,
    hi: f64,
    blocked: &[(f64, f64)],
    eps: f64,
) -> Vec<(f64, f64)> {
    let mut b: Vec<(f64, f64)> = blocked
        .iter()
        .map(|&(a, c)| (a.max(lo), c.min(hi)))
        .filter(|&(a, c)| c > a + eps)
        .collect();
    b.sort_by(|p, q| p.0.total_cmp(&q.0));
    let mut result = Vec::new();
    let mut cur = lo;
    for (bs, be) in b {
        if bs > cur + eps {
            result.push((cur, bs));
        }
        cur = cur.max(be);
    }
    if hi > cur + eps {
        result.push((cur, hi));
    }
    result
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
    leaf(Vakvulling::Glas { glaslat: None })
}
pub fn raam(open_type: &str) -> VakNode {
    leaf(Vakvulling::Raam {
        open_type: open_type.to_string(),
        hinge_side: None,
        glaslat: None,
        opens_outward: None,
        hardware: None,
    })
}
pub fn deur(door_kind: &str) -> VakNode {
    leaf(Vakvulling::Deur {
        door_kind: door_kind.to_string(),
        hinge_side: None,
        glaslat: None,
        opens_outward: None,
        hardware: None,
    })
}
pub fn paneel() -> VakNode {
    leaf(Vakvulling::Paneel { filling: None })
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

/// Side-light zone of a melkmeisje: a glass side light on top sitting on a low
/// borstwering (masonry, `Buiten`). The side light is TALLER than the borstwering
/// (a borstwering is a low wall ~900 mm; the glass fills up to door height), and
/// the outline steps: below the side light is wall, not kozijn.
fn zijlicht_zone() -> VakNode {
    split_col(vec![child(1200.0, glas()), child(900.0, buiten())])
}

/// Melkmeisje: a DOOR with a side light that does not reach the floor — a
/// borstwering (low masonry wall) sits below the side light (stepped outline).
pub fn melkmeisje1() -> VakNode {
    split_row(vec![child(900.0, deur("enkel")), child(500.0, zijlicht_zone())])
}

/// Melkmeisje with a side light (on a borstwering) on both sides of the door.
pub fn melkmeisje2() -> VakNode {
    split_row(vec![
        child(500.0, zijlicht_zone()),
        child(1000.0, deur("enkel")),
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
        let glas_leaf = geom.leaves.iter().find(|l| matches!(l.vulling, Vakvulling::Glas { .. })).unwrap();
        let buiten_leaf = geom.leaves.iter().find(|l| matches!(l.vulling, Vakvulling::Buiten)).unwrap();
        // side light is above the outside (stepped outline), no panel
        assert!(glas_leaf.rect.y < buiten_leaf.rect.y);
        assert!(!geom.leaves.iter().any(|l| matches!(l.vulling, Vakvulling::Paneel { .. })));
        // 2 real vakken (raam + glas), buiten does not count
        assert_eq!(count_vakken(&melkmeisje1()), 2);
    }

    #[test]
    fn melkmeisje_dividers_clipped_against_buiten() {
        // The tussenstijl between the raam and the side light stops at the
        // side-light step (does not run down through the wall), and the
        // side-light onderdorpel (wall directly below) drops out entirely.
        let geom = compute_layout_geometry(&melkmeisje1(), FULL, 90.0);
        let v: Vec<_> = geom.dividers.iter().filter(|d| d.direction == "v").collect();
        let h: Vec<_> = geom.dividers.iter().filter(|d| d.direction == "h").collect();
        assert_eq!(v.len(), 1, "one clipped tussenstijl");
        assert!(
            v[0].rect.height < FULL.height - 1e-6,
            "tussenstijl {} must be shorter than the full height {}",
            v[0].rect.height,
            FULL.height
        );
        assert!(v[0].rect.height > 0.0);
        assert_eq!(h.len(), 0, "side-light onderdorpel borders buiten → dropped");
    }

    #[test]
    fn dividers_without_buiten_are_untouched() {
        // No `Buiten` leaf → clipping is a no-op (byte-identical dividers).
        let geom = compute_layout_geometry(&free_grid(3, 2), FULL, 60.0);
        // 3×2 = 2 mullions per row (×2 rows via the row splits) + 1 transom.
        let v = geom.dividers.iter().filter(|d| d.direction == "v").count();
        let h = geom.dividers.iter().filter(|d| d.direction == "h").count();
        assert_eq!(v, 4, "two mullions in each of the two rows");
        assert_eq!(h, 1, "one transom between the rows");
        // Every mullion spans a full row height (not clipped).
        assert!(geom
            .dividers
            .iter()
            .filter(|d| d.direction == "v")
            .all(|d| d.rect.height > 0.0));
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
        // melkmeisje1: deur (0), then zijlicht glas (1), then buiten (2) —
        // buiten counts in the numbering so vulling changes never renumber.
        let geom = compute_layout_geometry(&melkmeisje1(), FULL, 90.0);
        let idx: Vec<usize> = geom.leaves.iter().map(|l| l.leaf_index).collect();
        assert_eq!(idx, vec![0, 1, 2]);
        assert!(matches!(geom.leaves[0].vulling, Vakvulling::Deur { .. }));
        assert!(matches!(geom.leaves[2].vulling, Vakvulling::Buiten));
        assert_eq!(geom.leaves[0].vulling.type_str(), "deur");
    }

    #[test]
    fn vulling_hinge_side_roundtrips_and_stays_optional() {
        // hingeSide from the canvas editor survives the Rust roundtrip
        let json = r#"{"kind":"leaf","vulling":{"type":"raam","openType":"draai","hingeSide":"links"}}"#;
        let node: VakNode = serde_json::from_str(json).unwrap();
        match &node {
            VakNode::Leaf { vulling: Vakvulling::Raam { open_type, hinge_side, .. }, .. } => {
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

    #[test]
    fn vulling_extras_roundtrip_through_serde() {
        // All canvas-editor extras (glaslat, opensOutward, hardware, filling)
        // survive the Rust roundtrip instead of being silently dropped.
        let json = r#"{"kind":"split","direction":"row","children":[
            {"size":1.0,"node":{"kind":"leaf","vulling":{"type":"glas","glaslat":"binnen"}}},
            {"size":1.0,"node":{"kind":"leaf","vulling":{"type":"raam","openType":"draaikiep","hingeSide":"rechts","glaslat":"buiten","opensOutward":true,"hardware":true}}},
            {"size":1.0,"node":{"kind":"leaf","vulling":{"type":"deur","doorKind":"enkel","opensOutward":false,"hardware":false}}},
            {"size":1.0,"node":{"kind":"leaf","vulling":{"type":"paneel","filling":{"fillingType":"sandwich","material":"sandwich-pur","thicknessMm":40.0,"uValue":0.6,"color":"RAL9010"}}}}
        ]}"#;
        let node: VakNode = serde_json::from_str(json).unwrap();
        let back = serde_json::to_string(&node).unwrap();
        let reparsed: VakNode = serde_json::from_str(&back).unwrap();
        let children = match &reparsed {
            VakNode::Split { children, .. } => children,
            other => panic!("expected split, got {:?}", other),
        };
        match &children[0].node {
            VakNode::Leaf { vulling: Vakvulling::Glas { glaslat }, .. } => {
                assert_eq!(glaslat.as_deref(), Some("binnen"));
            }
            other => panic!("expected glas leaf, got {:?}", other),
        }
        match &children[1].node {
            VakNode::Leaf {
                vulling: Vakvulling::Raam { open_type, hinge_side, glaslat, opens_outward, hardware },
                ..
            } => {
                assert_eq!(open_type, "draaikiep");
                assert_eq!(hinge_side.as_deref(), Some("rechts"));
                assert_eq!(glaslat.as_deref(), Some("buiten"));
                assert_eq!(*opens_outward, Some(true));
                assert_eq!(*hardware, Some(true));
            }
            other => panic!("expected raam leaf, got {:?}", other),
        }
        match &children[2].node {
            VakNode::Leaf { vulling: Vakvulling::Deur { opens_outward, hardware, .. }, .. } => {
                // Explicit `false` roundtrips as `false` — the canvas editor
                // distinguishes "unchecked" from "never touched".
                assert_eq!(*opens_outward, Some(false));
                assert_eq!(*hardware, Some(false));
            }
            other => panic!("expected deur leaf, got {:?}", other),
        }
        match &children[3].node {
            VakNode::Leaf { vulling: Vakvulling::Paneel { filling }, .. } => {
                let f = filling.as_ref().expect("paneel filling survives roundtrip");
                assert_eq!(f.filling_type, crate::panel_filling::FillingType::Sandwich);
                assert!((f.thickness_mm - 40.0).abs() < 1e-9);
            }
            other => panic!("expected paneel leaf, got {:?}", other),
        }
        // Serialized form keeps the camelCase wire names the frontend uses.
        assert!(back.contains("\"opensOutward\":true"), "payload: {}", back);
        assert!(back.contains("\"glaslat\":\"binnen\""), "payload: {}", back);
        assert!(back.contains("\"hardware\":false"), "payload: {}", back);
        assert!(back.contains("\"filling\":{"), "payload: {}", back);
    }

    #[test]
    fn bare_vullingen_stay_byte_compatible() {
        // Old payloads without the new fields still parse...
        let node: VakNode =
            serde_json::from_str(r#"{"kind":"leaf","vulling":{"type":"glas"}}"#).unwrap();
        assert!(matches!(
            &node,
            VakNode::Leaf { vulling: Vakvulling::Glas { glaslat: None }, .. }
        ));
        let paneel_node: VakNode =
            serde_json::from_str(r#"{"kind":"leaf","vulling":{"type":"paneel"}}"#).unwrap();
        assert!(matches!(
            &paneel_node,
            VakNode::Leaf { vulling: Vakvulling::Paneel { filling: None }, .. }
        ));
        // ...and a bare vulling still serializes without any of the new keys.
        assert_eq!(
            serde_json::to_string(&glas()).unwrap(),
            r#"{"kind":"leaf","vulling":{"type":"glas"}}"#
        );
        assert_eq!(
            serde_json::to_string(&paneel()).unwrap(),
            r#"{"kind":"leaf","vulling":{"type":"paneel"}}"#
        );
    }
}
