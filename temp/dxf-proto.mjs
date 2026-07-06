// Prototype DXF profile-contour extractor — validate the rebuild approach on the
// real supplier samples before mirroring to Rust. Handles LINE/ARC/CIRCLE/
// ELLIPSE/LWPOLYLINE(bulge) and R12-style POLYLINE→VERTEX(bulge)→SEQEND;
// layer-filters annotations; tessellates curves; chains segments into closed
// loops; picks the largest-area loop as the outer contour (NO convex hull).
import { readFileSync } from "fs";

export const ANNOT = /dim|maat|tekst|text|arc?ering|hatch|as$|^as|center/i; // annotation layers to skip
export const TOL = 0.05; // mm endpoint-match tolerance
export const SEG_CAP = 16000; // above this it's a whole-window assembly — chaining would hang (same cap as dxf-cmp.mjs)

function pairs(txt) {
  const ls = txt.split(/\r\n|\r|\n/);
  const out = [];
  for (let i = 0; i + 1 < ls.length; i += 2) out.push([ls[i].trim(), ls[i + 1].trim()]);
  return out;
}

function arcPts(cx, cy, r, a0, a1) {
  // CCW from a0 to a1 (degrees)
  let s = a0, e = a1;
  while (e < s) e += 360;
  const sweep = e - s;
  const n = Math.max(2, Math.ceil(sweep / 8));
  const pts = [];
  for (let i = 0; i <= n; i++) {
    const a = (s + (sweep * i) / n) * Math.PI / 180;
    pts.push([cx + r * Math.cos(a), cy + r * Math.sin(a)]);
  }
  return pts;
}

function bulgePts(p0, p1, bulge) {
  if (!bulge) return [p0, p1];
  const theta = 4 * Math.atan(bulge); // included angle (signed)
  const dx = p1[0] - p0[0], dy = p1[1] - p0[1];
  const chord = Math.hypot(dx, dy);
  if (chord < 1e-9) return [p0, p1];
  const r = chord / (2 * Math.sin(Math.abs(theta) / 2));
  // center
  const mx = (p0[0] + p1[0]) / 2, my = (p0[1] + p1[1]) / 2;
  const h = r * Math.cos(theta / 2);
  const nx = -dy / chord, ny = dx / chord;
  const sign = bulge > 0 ? 1 : -1;
  const cx = mx + sign * h * nx, cy = my + sign * h * ny;
  let a0 = Math.atan2(p0[1] - cy, p0[0] - cx) * 180 / Math.PI;
  let a1 = Math.atan2(p1[1] - cy, p1[0] - cx) * 180 / Math.PI;
  const pts = bulge > 0 ? arcPts(cx, cy, r, a0, a1) : arcPts(cx, cy, r, a1, a0).reverse();
  return pts;
}

export function extractPolylines(txt) {
  const ps = pairs(txt);
  const polys = []; // { layer, pts: [[x,y],...], closed }
  const used = {};
  const st = { pending: null }; // R12 POLYLINE whose VERTEX entities are being collected
  let i = 0;
  // index entity starts
  while (i < ps.length) {
    if (ps[i][0] === "0") {
      const type = ps[i][1];
      // gather attributes until next code 0
      let j = i + 1;
      const attrs = [];
      while (j < ps.length && ps[j][0] !== "0") { attrs.push(ps[j]); j++; }
      handle(type, attrs, polys, used, st);
      i = j;
    } else i++;
  }
  finalizePolyline(st, polys); // a POLYLINE cut off by EOF still yields its vertices
  return { polys, used };
}

function getLayer(attrs) {
  const l = attrs.find((a) => a[0] === "8");
  return l ? l[1] : "0";
}
function num(attrs, code) {
  const a = attrs.find((x) => x[0] === code);
  return a ? parseFloat(a[1]) : undefined;
}

// Close out a pending R12 POLYLINE: tessellate collected vertices (with bulge).
function finalizePolyline(st, polys) {
  const p = st.pending;
  st.pending = null;
  if (!p) return;
  const pts = polyWithBulge(p.verts, p.closed);
  if (pts.length >= 2) polys.push({ layer: p.layer, pts, closed: !!p.closed });
}

function handle(type, attrs, polys, used, st) {
  used[type] = (used[type] || 0) + 1;
  if (type === "VERTEX") {
    if (st.pending) {
      const flags = num(attrs, "70") || 0;
      // skip spline-frame control points (16) and polyface face records (128 without 64)
      if (!(flags & 16) && !((flags & 128) && !(flags & 64))) {
        const x = num(attrs, "10"), y = num(attrs, "20");
        if (x !== undefined && y !== undefined) st.pending.verts.push({ x, y, bulge: num(attrs, "42") || 0 });
      }
    }
    return;
  }
  if (type === "SEQEND") { finalizePolyline(st, polys); return; }
  finalizePolyline(st, polys); // any other entity implicitly ends an open vertex run
  const layer = getLayer(attrs);
  const skip = ANNOT.test(layer);
  const add = (pts, closed = false) => { if (!skip && pts.length >= 2) polys.push({ layer, pts, closed }); };
  if (type === "LINE") {
    const x1 = num(attrs, "10"), y1 = num(attrs, "20"), x2 = num(attrs, "11"), y2 = num(attrs, "21");
    if ([x1, y1, x2, y2].every((v) => v !== undefined)) add([[x1, y1], [x2, y2]]);
  } else if (type === "ARC") {
    const cx = num(attrs, "10"), cy = num(attrs, "20"), r = num(attrs, "40"), a0 = num(attrs, "50"), a1 = num(attrs, "51");
    if ([cx, cy, r, a0, a1].every((v) => v !== undefined)) add(arcPts(cx, cy, r, a0, a1));
  } else if (type === "CIRCLE") {
    const cx = num(attrs, "10"), cy = num(attrs, "20"), r = num(attrs, "40");
    if ([cx, cy, r].every((v) => v !== undefined)) add(arcPts(cx, cy, r, 0, 360), true);
  } else if (type === "ELLIPSE") {
    const cx = num(attrs, "10"), cy = num(attrs, "20"), mx = num(attrs, "11"), my = num(attrs, "21"), ratio = num(attrs, "40");
    let s = num(attrs, "41") ?? 0, e = num(attrs, "42") ?? 2 * Math.PI;
    if ([cx, cy, mx, my, ratio].every((v) => v !== undefined)) {
      const major = Math.hypot(mx, my), rot = Math.atan2(my, mx), minor = major * ratio;
      const pts = []; const n = 48;
      while (e < s) e += 2 * Math.PI;
      for (let k = 0; k <= n; k++) {
        const t = s + (e - s) * k / n;
        const ex = major * Math.cos(t), ey = minor * Math.sin(t);
        pts.push([cx + ex * Math.cos(rot) - ey * Math.sin(rot), cy + ex * Math.sin(rot) + ey * Math.cos(rot)]);
      }
      add(pts, Math.abs(e - s - 2 * Math.PI) < 1e-3);
    }
  } else if (type === "LWPOLYLINE") {
    const closed = (num(attrs, "70") || 0) & 1;
    // collect vertices in order with bulge
    const verts = [];
    let cur = null;
    for (const [c, v] of attrs) {
      if (c === "10") { if (cur) verts.push(cur); cur = { x: parseFloat(v), bulge: 0 }; }
      else if (c === "20" && cur) cur.y = parseFloat(v);
      else if (c === "42" && cur) cur.bulge = parseFloat(v);
    }
    if (cur) verts.push(cur);
    const pts = polyWithBulge(verts, closed);
    if (pts.length >= 2) add(pts, !!closed);
  } else if (type === "POLYLINE") {
    // R12 polyline: points follow as separate VERTEX entities until SEQEND.
    // Code 70 bit 1 = closed. Annotation-layer polylines never become pending.
    if (!skip) st.pending = { layer, closed: (num(attrs, "70") || 0) & 1, verts: [] };
  }
}

function polyWithBulge(verts, closed) {
  const out = [];
  const n = verts.length;
  const lim = closed ? n : n - 1;
  for (let k = 0; k < lim; k++) {
    const a = verts[k], b = verts[(k + 1) % n];
    if (a.x === undefined || b.x === undefined) continue;
    const seg = bulgePts([a.x, a.y], [b.x, b.y], a.bulge || 0);
    if (k === 0) out.push(...seg); else out.push(...seg.slice(1));
  }
  return out;
}

// Chain disjoint polylines into closed loops by matching endpoints. At each step
// the chain's head/tail join the *nearest* free segment endpoint within TOL
// (best-match, not first-match) — best-match keeps the walk on the true outer
// contour at junctions where a chamber wall meets the contour. Mirrors the Rust
// `chain()` in ofs-core/src/import/dxf_profile.rs.
export function chain(polys) {
  const d = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
  const loops = [];
  const remaining = polys.map((p) => p.pts.slice()).filter((s) => s.length >= 2);
  while (remaining.length) {
    let chainPts = remaining.shift().slice();
    let extended = true;
    while (extended) {
      extended = false;
      const head = chainPts[0], tail = chainPts[chainPts.length - 1];
      let bi = -1, bop = 0, bd = TOL;
      for (let i = 0; i < remaining.length; i++) {
        const s = remaining[i], s0 = s[0], s1 = s[s.length - 1];
        const c0 = d(tail, s0); if (c0 <= bd && (bi < 0 || c0 < bd)) { bd = c0; bi = i; bop = 0; }
        const c1 = d(tail, s1); if (c1 <= bd && (bi < 0 || c1 < bd)) { bd = c1; bi = i; bop = 1; }
        const c2 = d(head, s1); if (c2 <= bd && (bi < 0 || c2 < bd)) { bd = c2; bi = i; bop = 2; }
        const c3 = d(head, s0); if (c3 <= bd && (bi < 0 || c3 < bd)) { bd = c3; bi = i; bop = 3; }
      }
      if (bi >= 0) {
        const s = remaining[bi];
        if (bop === 0) chainPts.push(...s.slice(1));
        else if (bop === 1) chainPts.push(...s.slice().reverse().slice(1));
        else if (bop === 2) chainPts.unshift(...s.slice(0, -1));
        else chainPts.unshift(...s.slice().reverse().slice(0, -1));
        remaining.splice(bi, 1);
        extended = true;
      }
    }
    const closed = chainPts.length > 2 && d(chainPts[0], chainPts[chainPts.length - 1]) <= TOL;
    loops.push({ pts: chainPts, closed });
  }
  return loops;
}

export function area(pts) {
  let a = 0;
  for (let i = 0; i < pts.length; i++) {
    const p = pts[i], q = pts[(i + 1) % pts.length];
    a += p[0] * q[1] - q[0] * p[1];
  }
  return Math.abs(a) / 2;
}

export function analyze(file, segCap = Infinity) {
  const txt = readFileSync(file, "utf8");
  const { polys, used } = extractPolylines(txt);
  if (!polys.length) return { file, contourLoss: true, used };
  if (polys.length > segCap) return { file: file.split(/[\\/]/).pop(), huge: true, segments: polys.length };
  // if there's a single closed polyline, prefer it; else chain
  let loops = polys.filter((p) => p.closed && p.pts.length > 3).map((p) => ({ pts: p.pts, closed: true }));
  if (!loops.length) loops = chain(polys);
  loops.sort((a, b) => area(b.pts) - area(a.pts));
  const c = loops[0];
  const xs = c.pts.map((p) => p[0]), ys = c.pts.map((p) => p[1]);
  return {
    file: file.split(/[\\/]/).pop(),
    used: Object.entries(used).filter(([k]) => /LINE|ARC|POLYLINE|ELLIPSE|CIRCLE|SPLINE|HATCH|INSERT/.test(k)).map(([k, v]) => `${k}:${v}`).join(" "),
    loops: loops.length,
    contourPts: c.pts.length,
    closed: c.closed,
    width: +(Math.max(...xs) - Math.min(...xs)).toFixed(1),
    depth: +(Math.max(...ys) - Math.min(...ys)).toFixed(1),
    areaMm2: +area(c.pts).toFixed(0),
  };
}

import { readdirSync } from "fs";
import { pathToFileURL } from "url";

const isMain = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;
const args = isMain ? process.argv.slice(2) : [];
if (isMain && args[0] === "--all") {
  const dir = "samples";
  const files = readdirSync(dir).filter((f) => f.toLowerCase().endsWith(".dxf"));
  let closed = 0, open = 0, none = 0, err = 0, huge = 0;
  const failEntities = {};
  for (const f of files) {
    try {
      const r = analyze(`${dir}/${f}`, SEG_CAP);
      if (r.contourLoss) { none++; }
      else if (r.huge) { huge++; }
      else if (r.closed) { closed++; }
      else {
        open++;
        for (const t of (r.used || "").split(" ")) { const k = t.split(":")[0]; if (k) failEntities[k] = (failEntities[k] || 0) + 1; }
      }
    } catch { err++; }
  }
  console.log(JSON.stringify({ total: files.length, closed, open, noContour: none, huge, error: err, openFileEntityHist: failEntities }, null, 2));
} else if (isMain) {
  for (const f of args) {
    try { console.log(JSON.stringify(analyze(f))); }
    catch (e) { console.log(JSON.stringify({ file: f, error: String(e) })); }
  }
}
