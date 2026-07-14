/**
 * frame3d.js — pure 2D geometry helpers for the 3D kozijn builder
 * (Viewer3D.svelte).
 *
 * Everything works in the 2D drawing plane of the geometry payload (mm,
 * y-down); Viewer3D translates to three.js world coordinates itself. No
 * three.js import here: the helpers are pure and independently testable.
 *
 * Joint model implemented by decomposeSteppedFrame (KVT wood rule): merge
 * horizontally first, then vertically — dorpels (rails) run through, stijlen
 * (stiles) fit between them. The stepped melkmeisje frame mass from the
 * payload (`framePolygons` rings per vak) is decomposed into disjoint members
 * so shared edges never yield two coplanar boxes (z-fighting) and no member
 * crosses a `buiten` (masonry) zone.
 */
import { layoutToRects, ensureIds } from "./layout.js";

const EPS = 0.5;

/** Axis-aligned bounding box of a closed ring [[x, y], ...]. */
export function ringBBox(ring) {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const p of ring) {
    if (p[0] < minX) minX = p[0];
    if (p[0] > maxX) maxX = p[0];
    if (p[1] < minY) minY = p[1];
    if (p[1] > maxY) maxY = p[1];
  }
  return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

/**
 * Detect mitered (45° verstek) member ends on a frame-member polygon from
 * `rectangular_frame_member_polygons` (ofs-core geometry.rs). A square-cut end
 * has TWO vertices on the bbox edge at that end; a mitered end has only the
 * outer corner there (the diagonal runs to the inset inner corner).
 * `start` = left end for "h" members, top end for "v" members — matching the
 * extrusion direction in Viewer3D's makeProfileMember.
 */
export function miterEndsFor(poly, orientation) {
  const bb = ringBBox(poly);
  const countAt = (axis, value) => {
    let n = 0;
    for (const p of poly) if (Math.abs(p[axis] - value) < EPS) n++;
    return n;
  };
  return orientation === "h"
    ? { start: countAt(0, bb.x) < 2, end: countAt(0, bb.x + bb.width) < 2 }
    : { start: countAt(1, bb.y) < 2, end: countAt(1, bb.y + bb.height) < 2 };
}

/** True when the layout tree contains a `buiten` (outside-the-frame) leaf. */
export function layoutHasBuiten(node) {
  if (!node) return false;
  if (node.kind === "leaf") return node.vulling?.type === "buiten";
  return (node.children || []).some((c) => layoutHasBuiten(c.node));
}

/**
 * Rects of all `buiten` leaves, laid out exactly like the backend does it
 * (same tree, same inner rect, divider width = frame width).
 */
export function collectBuitenRects(layoutTree, innerRect, dividerW) {
  if (!layoutTree || !innerRect) return [];
  const out = layoutToRects(
    ensureIds(layoutTree),
    { x: innerRect.x, y: innerRect.y, width: innerRect.width, height: innerRect.height },
    dividerW
  );
  return out.leaves
    .filter((l) => l.node?.vulling?.type === "buiten")
    .map((l) => l.rect);
}

/**
 * Extend a `buiten` rect outward by the frame width on every side that
 * touches the inner-opening boundary — there is no frame member there, so the
 * masonry runs to the outer edge of the kozijn instead of leaving a gap.
 */
export function expandBuitenRect(rect, innerRect, fw, eps = 1) {
  let { x, y, width, height } = rect;
  if (Math.abs(rect.x - innerRect.x) < eps) { x -= fw; width += fw; }
  if (Math.abs(rect.x + rect.width - (innerRect.x + innerRect.width)) < eps) width += fw;
  if (Math.abs(rect.y - innerRect.y) < eps) { y -= fw; height += fw; }
  if (Math.abs(rect.y + rect.height - (innerRect.y + innerRect.height)) < eps) height += fw;
  return { x, y, width, height };
}

function dedupSorted(values, eps) {
  const sorted = [...values].sort((a, b) => a - b);
  const out = [];
  for (const v of sorted) if (!out.length || v - out[out.length - 1] >= eps) out.push(v);
  return out;
}

/**
 * Decompose the union of the stepped frame rings (minus vak and divider
 * rects) into disjoint axis-aligned member rects. Horizontal-first merging
 * makes dorpels run through and stijlen fit between them (KVT wood rule);
 * rings that overlap (shared members between adjacent vakken) are deduplicated
 * by construction because every grid cell is emitted exactly once.
 *
 * @param rings axis-aligned rects (bboxes of the payload's stepped polygons)
 * @param holes rects to subtract (vak openings + divider rects)
 * @returns Array<{x, y, width, height}>
 */
export function decomposeSteppedFrame(rings, holes, eps = EPS) {
  const coordsX = [];
  const coordsY = [];
  for (const r of [...rings, ...holes]) {
    coordsX.push(r.x, r.x + r.width);
    coordsY.push(r.y, r.y + r.height);
  }
  const X = dedupSorted(coordsX, eps);
  const Y = dedupSorted(coordsY, eps);
  const nx = X.length - 1;
  const ny = Y.length - 1;
  if (nx < 1 || ny < 1) return [];

  const covers = (r, px, py) =>
    px > r.x - 0.1 && px < r.x + r.width + 0.1 &&
    py > r.y - 0.1 && py < r.y + r.height + 0.1;

  const filled = [];
  for (let j = 0; j < ny; j++) {
    const row = new Array(nx).fill(false);
    const cy = (Y[j] + Y[j + 1]) / 2;
    for (let i = 0; i < nx; i++) {
      const cx = (X[i] + X[i + 1]) / 2;
      row[i] = rings.some((r) => covers(r, cx, cy)) && !holes.some((r) => covers(r, cx, cy));
    }
    filled.push(row);
  }

  // 1) Maximal horizontal runs per row (dorpels run through).
  const strips = filled.map((row) => {
    const runs = [];
    let start = -1;
    for (let i = 0; i <= row.length; i++) {
      const on = i < row.length && row[i];
      if (on && start < 0) start = i;
      if (!on && start >= 0) { runs.push({ i0: start, i1: i }); start = -1; }
    }
    return runs;
  });

  // 2) Merge identical runs vertically (stijlen between the dorpels).
  const consumed = strips.map((runs) => runs.map(() => false));
  const members = [];
  for (let j = 0; j < ny; j++) {
    strips[j].forEach((run, k) => {
      if (consumed[j][k]) return;
      consumed[j][k] = true;
      let j2 = j + 1;
      for (; j2 < ny; j2++) {
        const idx = strips[j2].findIndex(
          (r2, k2) => !consumed[j2][k2] && r2.i0 === run.i0 && r2.i1 === run.i1
        );
        if (idx < 0) break;
        consumed[j2][idx] = true;
      }
      members.push({
        x: X[run.i0],
        y: Y[j],
        width: X[run.i1] - X[run.i0],
        height: Y[j2] - Y[j],
      });
    });
  }
  return members;
}

/**
 * Which sides of member rect `m` border one of `rects` (edge contact with
 * overlap along the edge). `top` means a rect sits above `m` in 2D (its
 * bottom edge equals m's top edge) — 2D is y-down.
 */
export function rectAdjacency(m, rects, eps = 1) {
  const adj = { top: false, bottom: false, left: false, right: false };
  for (const r of rects) {
    const xOverlap = Math.min(m.x + m.width, r.x + r.width) - Math.max(m.x, r.x) > eps;
    const yOverlap = Math.min(m.y + m.height, r.y + r.height) - Math.max(m.y, r.y) > eps;
    if (xOverlap && Math.abs(r.y + r.height - m.y) < eps) adj.top = true;
    if (xOverlap && Math.abs(r.y - (m.y + m.height)) < eps) adj.bottom = true;
    if (yOverlap && Math.abs(r.x + r.width - m.x) < eps) adj.left = true;
    if (yOverlap && Math.abs(r.x - (m.x + m.width)) < eps) adj.right = true;
  }
  return adj;
}

/**
 * Symmetric divider (tussenstijl/tussendorpel) cross-section: rectangular body
 * with a glazing rebate on BOTH vak sides — same form as the isDivider branch
 * in profileContour.js woodCrossSection, generalized so PVC/alu T-profiles get
 * a plausible double-sided channel too. Coordinates follow the library
 * convention: u 0..w across the sight width, v 0 = outside .. d = inside.
 *
 * @param w cross width (divider sight width)
 * @param d build depth
 * @param h rebate height (glasinval) per side
 * @param ledgeV v where the rebate starts (wood: d - sponningbreedte)
 */
export function dividerSectionContour(w, d, h, ledgeV) {
  const hh = Math.max(2, Math.min(h, w * 0.45));
  const lv = Math.max(2, Math.min(ledgeV, d - 4));
  return [
    [0, 0], [w, 0], [w, lv], [w - hh, lv],
    [w - hh, d], [hh, d], [hh, lv], [0, lv],
  ];
}

/** Wrap a raw contour as a section object compatible with Viewer3D's cache. */
export function sectionFromPoints(id, pts) {
  let minU = Infinity, maxU = -Infinity, minV = Infinity, maxV = -Infinity;
  for (const [u, v] of pts) {
    if (u < minU) minU = u;
    if (u > maxU) maxU = u;
    if (v < minV) minV = v;
    if (v > maxV) maxV = v;
  }
  return { id, def: null, pts, minU, minV, w: maxU - minU, d: maxV - minV };
}
