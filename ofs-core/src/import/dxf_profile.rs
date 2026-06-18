//! Parse a DXF file containing a profile cross-section.
//!
//! Reads DXF (ASCII) and extracts the real profile contour: it scopes to the
//! ENTITIES section, filters out annotation layers (dimensions/text), tessellates
//! LINE / ARC / CIRCLE / ELLIPSE / LWPOLYLINE(bulge) / POLYLINE+VERTEX(bulge) into
//! point runs, and chains those runs into closed loops — the largest-area loop is
//! the outer contour. (The previous convex-hull approach destroyed the concave
//! multi-chamber shape + glazing rebate.) Algorithm validated against real
//! supplier samples; see docs/profiel-dxf-dwg-import-onderzoek.md.
//!
//! Not yet handled: SPLINE and INSERT/BLOCK (rare in the samples) and DWG (binary,
//! must be converted to DXF first).

/// Result of parsing a DXF profile cross-section.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedProfile {
    pub id: String,
    pub name: String,
    pub material: String,
    pub material_subtype: Option<String>,
    pub width: f64,
    pub depth: f64,
    pub sightline: f64,
    pub glazing_rebate: f64,
    pub cross_section: Vec<[f64; 2]>,
    pub uf_value: f64,
    pub applicable_as: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponning: Option<Sponning>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sponning {
    pub width: f64,
    pub depth: f64,
    pub position: String,
}

/// Endpoint match tolerance (mm) for chaining segments into loops.
const TOL: f64 = 0.05;

/// Snap tolerances (mm) for the planar-graph fallback, tried tightest-first —
/// bridges the small endpoint gaps between separately drawn entities. Only
/// files that fail the tighter snap climb to a wider one, so clean profiles
/// stay precise and over-collapse is limited to stubborn fragmented files.
const SNAP_TOLS: [f64; 3] = [0.5, 1.0, 2.0];

/// Above this segment count the file is a whole-window assembly, not a single
/// cross-section; skip the (heavier) graph fallback.
const FACE_SEG_CAP: usize = 20_000;

/// Minimum area/bbox fill for a graph-traced contour to be accepted; a folded
/// or collinear face nets ~0 area and is rejected so chaining stands.
const MIN_FILL: f64 = 0.12;

struct Poly {
    layer: String,
    pts: Vec<(f64, f64)>,
    closed: bool,
}

/// Parse a DXF file and extract a profile cross-section.
pub fn parse_dxf_profile(filepath: &str) -> Result<ImportedProfile, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Kan DXF bestand niet openen: {}", e))?;
    let pairs = read_pairs(&content);
    let polys = extract_polylines(&pairs);

    if polys.is_empty() {
        return Err("Geen contour-geometrie gevonden in DXF (alleen maatlijnen/annotaties?)".into());
    }

    // Prefer an explicit closed polyline; otherwise chain the loose segments,
    // and if the chain stays open, fall back to the planar-graph outer-face
    // trace (recovers fragmented contours that chaining can't close).
    let closed_loops: Vec<Vec<(f64, f64)>> = polys
        .iter()
        .filter(|p| p.closed && p.pts.len() > 3)
        .map(|p| p.pts.clone())
        .collect();
    let contour = if !closed_loops.is_empty() {
        closed_loops
            .into_iter()
            .max_by(|a, b| polygon_area(a).partial_cmp(&polygon_area(b)).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap()
    } else {
        let mut loops = chain(&polys);
        // Largest-area loop is the outer contour.
        loops.sort_by(|a, b| polygon_area(b).partial_cmp(&polygon_area(a)).unwrap_or(std::cmp::Ordering::Equal));
        let primary = loops
            .into_iter()
            .next()
            .ok_or("Geen gesloten contour gevonden in DXF")?;
        if is_closed_loop(&primary) || polys.len() > FACE_SEG_CAP {
            primary
        } else {
            // Escalate the snap tolerance until a non-degenerate face is found.
            SNAP_TOLS
                .iter()
                .find_map(|&snap| outer_contour(&polys, snap))
                .unwrap_or(primary)
        }
    };

    // Bounding box
    let min_x = contour.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = contour.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = contour.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = contour.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let width = round1(max_x - min_x);
    let depth = round1(max_y - min_y);

    // Normalize to origin
    let normalized: Vec<(f64, f64)> = contour.iter().map(|p| (p.0 - min_x, p.1 - min_y)).collect();
    let cross_section: Vec<[f64; 2]> = normalized.iter().map(|p| [round2(p.0), round2(p.1)]).collect();

    let sponning = detect_sponning(&normalized, width, depth);

    // Estimates where geometry doesn't give it directly.
    let sightline = round1(width * 0.8);
    let glazing_rebate = sponning.as_ref().map(|s| s.depth).unwrap_or_else(|| round1(width * 0.36));

    let path = std::path::Path::new(filepath);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Profiel")
        .to_string();
    let id = format!("imported-{}", name.to_lowercase().replace(' ', "-"));

    Ok(ImportedProfile {
        id,
        name,
        material: "unknown".into(),
        material_subtype: None,
        width,
        depth,
        sightline,
        glazing_rebate,
        cross_section,
        uf_value: 2.0,
        applicable_as: vec!["frame".into(), "sash".into(), "divider".into()],
        sponning,
    })
}

// ── DXF tokenizer ──────────────────────────────────────────────

/// Read DXF into (group code, value) pairs (trimmed).
fn read_pairs(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut pairs = Vec::with_capacity(lines.len() / 2);
    let mut i = 0;
    while i + 1 < lines.len() {
        pairs.push((lines[i].trim().to_string(), lines[i + 1].trim().to_string()));
        i += 2;
    }
    pairs
}

/// Annotation layers whose geometry is not part of the profile contour.
fn is_annotation_layer(layer: &str) -> bool {
    let l = layer.to_lowercase();
    ["dim", "maat", "text", "tekst", "hatch", "arcering", "center", "as-", "-as", "hidden"]
        .iter()
        .any(|k| l.contains(k))
        || l == "as"
}

/// Extract geometry as point-runs, scoped to the ENTITIES section, layer-filtered.
fn extract_polylines(pairs: &[(String, String)]) -> Vec<Poly> {
    // Scope to ENTITIES section if present.
    let start = pairs
        .iter()
        .position(|(c, v)| c == "2" && v == "ENTITIES")
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = pairs[start..]
        .iter()
        .position(|(c, v)| c == "0" && v == "ENDSEC")
        .map(|i| start + i)
        .unwrap_or(pairs.len());

    let mut polys = Vec::new();
    let mut i = start;
    while i < end {
        if pairs[i].0 == "0" {
            let etype = pairs[i].1.clone();
            // Gather attributes until the next code-0 (or section end).
            let mut j = i + 1;
            while j < end && pairs[j].0 != "0" {
                j += 1;
            }
            handle_entity(&etype, &pairs[i + 1..j], &mut polys);
            i = j;
        } else {
            i += 1;
        }
    }
    polys
}

fn attr_f(attrs: &[(String, String)], code: &str) -> Option<f64> {
    attrs.iter().find(|(c, _)| c == code).and_then(|(_, v)| v.parse().ok())
}
fn attr_layer(attrs: &[(String, String)]) -> String {
    attrs.iter().find(|(c, _)| c == "8").map(|(_, v)| v.clone()).unwrap_or_else(|| "0".into())
}

fn handle_entity(etype: &str, attrs: &[(String, String)], polys: &mut Vec<Poly>) {
    let layer = attr_layer(attrs);
    if is_annotation_layer(&layer) {
        return;
    }
    let mut push = |pts: Vec<(f64, f64)>, closed: bool| {
        if pts.len() >= 2 {
            polys.push(Poly { layer: layer.clone(), pts, closed });
        }
    };
    match etype {
        "LINE" => {
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
                (attr_f(attrs, "10"), attr_f(attrs, "20"), attr_f(attrs, "11"), attr_f(attrs, "21"))
            {
                push(vec![(x1, y1), (x2, y2)], false);
            }
        }
        "ARC" => {
            if let (Some(cx), Some(cy), Some(r), Some(a0), Some(a1)) = (
                attr_f(attrs, "10"), attr_f(attrs, "20"), attr_f(attrs, "40"),
                attr_f(attrs, "50"), attr_f(attrs, "51"),
            ) {
                push(arc_pts(cx, cy, r, a0, a1), false);
            }
        }
        "CIRCLE" => {
            if let (Some(cx), Some(cy), Some(r)) = (attr_f(attrs, "10"), attr_f(attrs, "20"), attr_f(attrs, "40")) {
                push(arc_pts(cx, cy, r, 0.0, 360.0), true);
            }
        }
        "ELLIPSE" => {
            if let (Some(cx), Some(cy), Some(mx), Some(my), Some(ratio)) = (
                attr_f(attrs, "10"), attr_f(attrs, "20"), attr_f(attrs, "11"),
                attr_f(attrs, "21"), attr_f(attrs, "40"),
            ) {
                let s = attr_f(attrs, "41").unwrap_or(0.0);
                let e = attr_f(attrs, "42").unwrap_or(std::f64::consts::TAU);
                let pts = ellipse_pts(cx, cy, mx, my, ratio, s, e);
                let closed = (e - s - std::f64::consts::TAU).abs() < 1e-3;
                push(pts, closed);
            }
        }
        "LWPOLYLINE" => {
            let closed = (attr_f(attrs, "70").unwrap_or(0.0) as i64) & 1 == 1;
            let verts = collect_vertices(attrs);
            push(poly_with_bulge(&verts, closed), closed);
        }
        _ => {}
    }
}

/// LWPOLYLINE vertices in order, each with its optional bulge (code 42).
fn collect_vertices(attrs: &[(String, String)]) -> Vec<(f64, f64, f64)> {
    let mut verts: Vec<(f64, f64, f64)> = Vec::new();
    let mut cur: Option<(f64, Option<f64>, f64)> = None; // (x, y, bulge)
    for (c, v) in attrs {
        match c.as_str() {
            "10" => {
                if let Some((x, Some(y), b)) = cur.take() {
                    verts.push((x, y, b));
                }
                cur = v.parse().ok().map(|x| (x, None, 0.0));
            }
            "20" => {
                if let (Some(slot), Ok(y)) = (cur.as_mut(), v.parse::<f64>()) {
                    slot.1 = Some(y);
                }
            }
            "42" => {
                if let (Some(slot), Ok(b)) = (cur.as_mut(), v.parse::<f64>()) {
                    slot.2 = b;
                }
            }
            _ => {}
        }
    }
    if let Some((x, Some(y), b)) = cur {
        verts.push((x, y, b));
    }
    verts
}

fn poly_with_bulge(verts: &[(f64, f64, f64)], closed: bool) -> Vec<(f64, f64)> {
    let n = verts.len();
    if n < 2 {
        return verts.iter().map(|v| (v.0, v.1)).collect();
    }
    let mut out: Vec<(f64, f64)> = Vec::new();
    let limit = if closed { n } else { n - 1 };
    for k in 0..limit {
        let a = verts[k];
        let b = verts[(k + 1) % n];
        let seg = bulge_pts((a.0, a.1), (b.0, b.1), a.2);
        if k == 0 {
            out.extend(seg);
        } else {
            out.extend(seg.into_iter().skip(1));
        }
    }
    out
}

// ── Tessellation ───────────────────────────────────────────────

fn arc_pts(cx: f64, cy: f64, r: f64, a0_deg: f64, a1_deg: f64) -> Vec<(f64, f64)> {
    let s = a0_deg;
    let mut e = a1_deg;
    while e < s {
        e += 360.0;
    }
    let sweep = e - s;
    let n = (sweep / 8.0).ceil().max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = (s + sweep * i as f64 / n as f64).to_radians();
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

fn bulge_pts(p0: (f64, f64), p1: (f64, f64), bulge: f64) -> Vec<(f64, f64)> {
    if bulge.abs() < 1e-9 {
        return vec![p0, p1];
    }
    let theta = 4.0 * bulge.atan();
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let chord = (dx * dx + dy * dy).sqrt();
    if chord < 1e-9 {
        return vec![p0, p1];
    }
    let r = chord / (2.0 * (theta.abs() / 2.0).sin());
    let mx = (p0.0 + p1.0) / 2.0;
    let my = (p0.1 + p1.1) / 2.0;
    let h = r * (theta / 2.0).cos();
    let nx = -dy / chord;
    let ny = dx / chord;
    let sign = if bulge > 0.0 { 1.0 } else { -1.0 };
    let cx = mx + sign * h * nx;
    let cy = my + sign * h * ny;
    let a0 = (p0.1 - cy).atan2(p0.0 - cx).to_degrees();
    let a1 = (p1.1 - cy).atan2(p1.0 - cx).to_degrees();
    if bulge > 0.0 {
        arc_pts(cx, cy, r, a0, a1)
    } else {
        let mut v = arc_pts(cx, cy, r, a1, a0);
        v.reverse();
        v
    }
}

#[allow(clippy::too_many_arguments)]
fn ellipse_pts(cx: f64, cy: f64, mx: f64, my: f64, ratio: f64, s: f64, mut e: f64) -> Vec<(f64, f64)> {
    let major = (mx * mx + my * my).sqrt();
    let rot = my.atan2(mx);
    let minor = major * ratio;
    while e < s {
        e += std::f64::consts::TAU;
    }
    let n = 48usize;
    (0..=n)
        .map(|k| {
            let t = s + (e - s) * k as f64 / n as f64;
            let ex = major * t.cos();
            let ey = minor * t.sin();
            (cx + ex * rot.cos() - ey * rot.sin(), cy + ex * rot.sin() + ey * rot.cos())
        })
        .collect()
}

// ── Segment chaining ───────────────────────────────────────────

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// How the nearest segment joins onto the growing chain.
enum Join {
    /// tail → s[0]: append s.
    AppendFwd,
    /// tail → s[last]: append s reversed.
    AppendRev,
    /// head ← s[last]: prepend s.
    PrependFwd,
    /// head ← s[0]: prepend s reversed.
    PrependRev,
}

/// Chain disjoint point-runs into loops by matching endpoints.
///
/// At each step the chain's head and tail look for the *nearest* free segment
/// endpoint within `TOL` across all candidates (best-match, not first-match).
/// Best-match matters at junctions — where a chamber wall meets the outer
/// contour, three+ segments share a node and first-match can route the chain
/// into a chamber, leaving the outer contour open. Picking the closest endpoint
/// keeps the walk on the true continuation.
fn chain(polys: &[Poly]) -> Vec<Vec<(f64, f64)>> {
    let mut remaining: Vec<Vec<(f64, f64)>> =
        polys.iter().filter(|p| p.pts.len() >= 2).map(|p| p.pts.clone()).collect();
    let mut loops = Vec::new();
    while let Some(first) = remaining.pop() {
        let mut chain_pts = first;
        loop {
            let head = chain_pts[0];
            let tail = chain_pts[chain_pts.len() - 1];
            // Nearest endpoint within TOL across all remaining segments.
            let mut best: Option<(usize, Join, f64)> = None;
            for (i, s) in remaining.iter().enumerate() {
                let (s0, s1) = (s[0], s[s.len() - 1]);
                for (d, join) in [
                    (dist(tail, s0), Join::AppendFwd),
                    (dist(tail, s1), Join::AppendRev),
                    (dist(head, s1), Join::PrependFwd),
                    (dist(head, s0), Join::PrependRev),
                ] {
                    if d <= TOL && best.as_ref().map_or(true, |(_, _, bd)| d < *bd) {
                        best = Some((i, join, d));
                    }
                }
            }
            let Some((i, join, _)) = best else { break };
            let s = remaining.remove(i);
            match join {
                Join::AppendFwd => chain_pts.extend(s.into_iter().skip(1)),
                Join::AppendRev => chain_pts.extend(s.into_iter().rev().skip(1)),
                Join::PrependFwd => {
                    let mut new_head = s;
                    new_head.pop();
                    new_head.extend(chain_pts.into_iter());
                    chain_pts = new_head;
                }
                Join::PrependRev => {
                    let mut new_head: Vec<(f64, f64)> = s.into_iter().rev().collect();
                    new_head.pop();
                    new_head.extend(chain_pts.into_iter());
                    chain_pts = new_head;
                }
            }
        }
        loops.push(chain_pts);
    }
    loops
}

/// A chained loop counts as closed when its ends meet within `TOL`.
fn is_closed_loop(pts: &[(f64, f64)]) -> bool {
    pts.len() > 3 && dist(pts[0], pts[pts.len() - 1]) <= TOL
}

// ── Planar-graph outer-contour fallback ────────────────────────
//
// Greedy chaining mis-routes at junctions and leaves fragmented contours open.
// This builds a planar graph from all segments — snapping near-coincident
// endpoints to bridge sub-millimetre gaps — prunes dangling spurs, keeps the
// largest connected component, then traces every face via the angular
// next-edge rule and returns the outer face (most-negative signed area). A
// folded / collinear trace (fill < MIN_FILL) is rejected so chaining stands.
// Mirrors `outerContour` in temp/dxf-face.mjs.

fn node_angle(nodes: &[(f64, f64)], a: usize, b: usize) -> f64 {
    (nodes[b].1 - nodes[a].1).atan2(nodes[b].0 - nodes[a].0)
}

fn signed_area_ids(nodes: &[(f64, f64)], ids: &[usize]) -> f64 {
    let n = ids.len();
    if n < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..n {
        let p = nodes[ids[i]];
        let q = nodes[ids[(i + 1) % n]];
        a += p.0 * q.1 - q.0 * p.1;
    }
    a / 2.0
}

/// Snap a point to an existing node within `snap_tol` (grid-hashed), else add it.
fn snap_node(
    pt: (f64, f64),
    cell: f64,
    snap_tol: f64,
    nodes: &mut Vec<(f64, f64)>,
    grid: &mut std::collections::HashMap<(i64, i64), Vec<usize>>,
) -> usize {
    let gx = (pt.0 / cell).round() as i64;
    let gy = (pt.1 / cell).round() as i64;
    for dx in -1..=1 {
        for dy in -1..=1 {
            if let Some(bucket) = grid.get(&(gx + dx, gy + dy)) {
                for &id in bucket {
                    if dist(nodes[id], pt) <= snap_tol {
                        return id;
                    }
                }
            }
        }
    }
    let id = nodes.len();
    nodes.push(pt);
    grid.entry((gx, gy)).or_default().push(id);
    id
}

fn outer_contour(polys: &[Poly], snap_tol: f64) -> Option<Vec<(f64, f64)>> {
    use std::collections::{HashMap, HashSet};

    // 1. undirected edges from consecutive points of each poly
    let cell = snap_tol.max(1e-6);
    let mut nodes: Vec<(f64, f64)> = Vec::new();
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
    for p in polys {
        for w in p.pts.windows(2) {
            let a = snap_node(w[0], cell, snap_tol, &mut nodes, &mut grid);
            let b = snap_node(w[1], cell, snap_tol, &mut nodes, &mut grid);
            if a == b {
                continue;
            }
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        }
    }
    if adj.is_empty() {
        return None;
    }

    // 2. prune dangling spurs (degree ≤ 1) iteratively
    loop {
        let leaves: Vec<usize> =
            adj.iter().filter(|(_, nb)| nb.len() <= 1).map(|(&n, _)| n).collect();
        if leaves.is_empty() {
            break;
        }
        for n in leaves {
            if let Some(nb) = adj.remove(&n) {
                for o in nb {
                    if let Some(s) = adj.get_mut(&o) {
                        s.remove(&n);
                    }
                }
            }
        }
    }
    if adj.is_empty() {
        return None;
    }

    // 3. keep only the largest connected component (the profile)
    {
        let mut seen: HashSet<usize> = HashSet::new();
        let mut best_comp: Option<HashSet<usize>> = None;
        let mut best_extent = -1.0_f64;
        let keys: Vec<usize> = adj.keys().copied().collect();
        for start in keys {
            if seen.contains(&start) {
                continue;
            }
            let mut stack = vec![start];
            seen.insert(start);
            let mut comp: HashSet<usize> = HashSet::new();
            let (mut minx, mut maxx, mut miny, mut maxy) =
                (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
            while let Some(u) = stack.pop() {
                comp.insert(u);
                let nn = nodes[u];
                minx = minx.min(nn.0);
                maxx = maxx.max(nn.0);
                miny = miny.min(nn.1);
                maxy = maxy.max(nn.1);
                if let Some(nb) = adj.get(&u) {
                    for &w in nb {
                        if seen.insert(w) {
                            stack.push(w);
                        }
                    }
                }
            }
            let extent = (maxx - minx) * (maxy - miny);
            if extent > best_extent {
                best_extent = extent;
                best_comp = Some(comp);
            }
        }
        let bc = best_comp?;
        let drop: Vec<usize> = adj.keys().copied().filter(|u| !bc.contains(u)).collect();
        for u in drop {
            adj.remove(&u);
        }
    }
    if adj.is_empty() {
        return None;
    }

    // 4. trace every face via the next-edge rule (interior on left, CCW; the
    //    outer face is traced CW → most-negative signed area).
    let two_pi = std::f64::consts::TAU;
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut outer: Option<Vec<usize>> = None;
    let mut outer_area = 0.0_f64;
    let keys: Vec<usize> = adj.keys().copied().collect();
    for u in keys {
        let starts: Vec<usize> = match adj.get(&u) {
            Some(nb) => nb.iter().copied().collect(),
            None => continue,
        };
        for v in starts {
            if visited.contains(&(u, v)) {
                continue;
            }
            let mut cycle = vec![u];
            let (mut pu, mut pv) = (u, v);
            let mut guard = 0u32;
            loop {
                visited.insert((pu, pv));
                cycle.push(pv);
                // next half-edge clockwise from the reverse of (pu → pv)
                let back = node_angle(&nodes, pv, pu);
                let mut nw: i64 = -1;
                let mut best_delta = f64::INFINITY;
                if let Some(nb) = adj.get(&pv) {
                    for &w in nb {
                        let aw = node_angle(&nodes, pv, w);
                        let mut delta = (back - aw) % two_pi;
                        if delta <= 1e-9 {
                            delta += two_pi; // never an immediate U-turn unless forced
                        }
                        if delta < best_delta {
                            best_delta = delta;
                            nw = w as i64;
                        }
                    }
                }
                if nw < 0 {
                    break;
                }
                pu = pv;
                pv = nw as usize;
                if pu == u && pv == v {
                    break; // closed the face
                }
                guard += 1;
                if guard > 200_000 {
                    break;
                }
            }
            cycle.pop(); // last == first
            let sa = signed_area_ids(&nodes, &cycle);
            if sa < outer_area {
                outer_area = sa;
                outer = Some(cycle);
            }
        }
    }

    let outer = outer?;
    let mut pts: Vec<(f64, f64)> = outer.iter().map(|&id| nodes[id]).collect();
    pts.reverse(); // CW outer → CCW output
    if pts.len() < 4 {
        return None;
    }
    let (mut minx, mut maxx, mut miny, mut maxy) =
        (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for &(x, y) in &pts {
        minx = minx.min(x);
        maxx = maxx.max(x);
        miny = miny.min(y);
        maxy = maxy.max(y);
    }
    let fill = outer_area.abs() / ((maxx - minx) * (maxy - miny)).max(1e-9);
    if fill < MIN_FILL {
        return None;
    }
    Some(pts)
}

fn polygon_area(pts: &[(f64, f64)]) -> f64 {
    if pts.len() < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..pts.len() {
        let p = pts[i];
        let q = pts[(i + 1) % pts.len()];
        a += p.0 * q.1 - q.0 * p.1;
    }
    a.abs() / 2.0
}

// ── Sponning detection ─────────────────────────────────────────

fn detect_sponning(contour: &[(f64, f64)], width: f64, depth: f64) -> Option<Sponning> {
    if contour.len() < 6 {
        return None;
    }
    let n = contour.len();
    for i in 0..n {
        let p0 = contour[i];
        let p1 = contour[(i + 1) % n];
        let p2 = contour[(i + 2) % n];
        let dx1 = (p1.0 - p0.0).abs();
        let dy1 = (p1.1 - p0.1).abs();
        let dx2 = (p2.0 - p1.0).abs();
        let dy2 = (p2.1 - p1.1).abs();

        if dx1 > 2.0 && dy1 < 1.0 && dx2 < 1.0 && dy2 > 2.0 {
            let sp_w = round1(dx1);
            let sp_d = round1(dy2);
            if (5.0..=30.0).contains(&sp_w) && (5.0..=40.0).contains(&sp_d) {
                let mid_x = (p0.0 + p1.0) / 2.0;
                let position = if mid_x > width / 2.0 { "buiten" } else { "binnen" };
                return Some(Sponning { width: sp_w, depth: sp_d, position: position.into() });
            }
        }
        if dy1 > 2.0 && dx1 < 1.0 && dy2 < 1.0 && dx2 > 2.0 {
            let sp_w = round1(dx2);
            let sp_d = round1(dy1);
            if (5.0..=30.0).contains(&sp_w) && (5.0..=40.0).contains(&sp_d) {
                let mid_y = (p0.1 + p1.1) / 2.0;
                let position = if mid_y > depth / 2.0 { "buiten" } else { "binnen" };
                return Some(Sponning { width: sp_w, depth: sp_d, position: position.into() });
            }
        }
    }
    None
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_tessellation_spans_endpoints() {
        let pts = arc_pts(0.0, 0.0, 10.0, 0.0, 90.0);
        assert!(pts.len() >= 3);
        assert!((pts[0].0 - 10.0).abs() < 1e-6 && pts[0].1.abs() < 1e-6);
        let last = pts[pts.len() - 1];
        assert!(last.0.abs() < 1e-6 && (last.1 - 10.0).abs() < 1e-6);
    }

    #[test]
    fn chains_lines_into_closed_square() {
        let polys = vec![
            Poly { layer: "0".into(), pts: vec![(0.0, 0.0), (10.0, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 0.0), (10.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 10.0), (0.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(0.0, 10.0), (0.0, 0.0)], closed: false },
        ];
        let loops = chain(&polys);
        assert_eq!(loops.len(), 1);
        assert!((polygon_area(&loops[0]) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn annotation_layers_are_skipped() {
        assert!(is_annotation_layer("DIMENSIONS"));
        assert!(is_annotation_layer("Maatvoering"));
        assert!(!is_annotation_layer("0"));
        assert!(!is_annotation_layer("CONTOUR"));
    }

    #[test]
    fn outer_contour_traces_square_and_prunes_spur() {
        // Square plus a dangling spur off a corner — the spur is degree-1 and
        // must be pruned, leaving the square as the outer face (area 100).
        let polys = vec![
            Poly { layer: "0".into(), pts: vec![(0.0, 0.0), (10.0, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 0.0), (10.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 10.0), (0.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(0.0, 10.0), (0.0, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 10.0), (15.0, 13.0)], closed: false },
        ];
        let c = outer_contour(&polys, 0.5).expect("contour");
        assert!((polygon_area(&c) - 100.0).abs() < 1e-6, "area was {}", polygon_area(&c));
    }

    #[test]
    fn outer_contour_bridges_subtolerance_gap() {
        // A 0.3 mm gap on the bottom edge defeats chaining (TOL 0.05) but is
        // bridged by snap_tol 0.5, so the graph fallback still closes the loop.
        let polys = vec![
            Poly { layer: "0".into(), pts: vec![(0.0, 0.0), (4.85, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(5.15, 0.0), (10.0, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 0.0), (10.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 10.0), (0.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(0.0, 10.0), (0.0, 0.0)], closed: false },
        ];
        // Chaining alone leaves the largest loop open (gap > TOL).
        let mut loops = chain(&polys);
        loops.sort_by(|a, b| polygon_area(b).partial_cmp(&polygon_area(a)).unwrap());
        assert!(!is_closed_loop(&loops[0]));
        // The graph fallback bridges the gap and recovers the square.
        let c = outer_contour(&polys, 0.5).expect("contour");
        assert!(polygon_area(&c) > 90.0, "area was {}", polygon_area(&c));
    }

    #[test]
    fn outer_contour_escalates_snap_for_wider_gap() {
        // A 0.8 mm gap is too wide for snap 0.5 (loop stays open → pruned to
        // nothing → None) but is bridged at snap 1.0 — the premise behind the
        // escalating SNAP_TOLS fallback.
        let polys = vec![
            Poly { layer: "0".into(), pts: vec![(0.0, 0.0), (4.6, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(5.4, 0.0), (10.0, 0.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 0.0), (10.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(10.0, 10.0), (0.0, 10.0)], closed: false },
            Poly { layer: "0".into(), pts: vec![(0.0, 10.0), (0.0, 0.0)], closed: false },
        ];
        assert!(outer_contour(&polys, 0.5).is_none());
        assert!(outer_contour(&polys, 1.0).is_some());
    }
}
