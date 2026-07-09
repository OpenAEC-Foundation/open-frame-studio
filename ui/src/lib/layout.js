/**
 * Free subdivision model — recursive split tree (proof-of-concept, frontend).
 *
 * A kozijn vak is either a LEAF (with a vakvulling) or a SPLIT into child vakken
 * along a direction:
 *   - direction "row"    → children side by side, separated by vertical mullions (tussenstijlen)
 *   - direction "column" → children stacked, separated by horizontal transoms (tussendorpels)
 *
 * Unlike the rigid columns×rows matrix, every branch subdivides independently,
 * so localized mullions, side lights that start higher (melkmeisje), full-width
 * top lights, and arbitrary compositions all follow naturally.
 *
 * This is the source for the locally-verifiable 2D proto; the Rust model in
 * ofs-core mirrors it for the backend/production once the wasm bundle is rebuilt.
 */

// Node ids persist through the Rust roundtrip and into .ofs files, so a bare
// sequential counter would collide with ids loaded from an earlier session
// (every session would regenerate n1, n2, ...). A session-unique prefix keeps
// fresh ids disjoint from any previously persisted ones.
let _idCounter = 0;
const _idSession = Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
const nid = () => `n${_idSession}-${++_idCounter}`;

// ── Vakvulling (cell filling) ────────────────────────────────────

const VULLING_LABELS = {
  glas: "Vast glas",
  raam: "Raam",
  deur: "Deur",
  paneel: "Paneel",
  rooster: "Ventilatie",
  buiten: "Buiten kozijn",
};

export const RAAM_OPENTYPES = {
  draai: "Draairaam",
  draaikiep: "Draaikiepraam",
  valraam: "Valraam",
  uitzet: "Uitzetraam",
  tuimel: "Tuimelraam",
  taats: "Taatsraam",
  schuif: "Schuifraam",
  hefschuif: "Hefschuif",
};

export const DEUR_SOORTEN = {
  enkel: "Enkele deur",
  dubbel: "Dubbele deur (stolp)",
  schuif: "Schuifdeur",
  hefschuif: "Hefschuifdeur",
  vouw: "Vouwdeur",
  taats: "Taatsdeur",
};

function leaf(vulling) {
  return { id: nid(), kind: "leaf", vulling };
}
const glas = () => leaf({ type: "glas" });
const raam = (openType = "draai") => leaf({ type: "raam", openType });
const deur = (doorKind = "enkel") => leaf({ type: "deur", doorKind });
/** Outside the kozijn (wall/masonry) — no vak, no member, makes the outline step. */
const buiten = () => leaf({ type: "buiten" });

const child = (size, node) => ({ size, node });
const splitRow = (...children) => ({ id: nid(), kind: "split", direction: "row", children });
const splitCol = (...children) => ({ id: nid(), kind: "split", direction: "column", children });

export function vullingLabel(v) {
  if (!v) return "—";
  if (v.type === "raam") return RAAM_OPENTYPES[v.openType] || "Raam";
  if (v.type === "deur") return DEUR_SOORTEN[v.doorKind] || "Deur";
  return VULLING_LABELS[v.type] || v.type;
}

// ── Templates ────────────────────────────────────────────────────

/**
 * Side-light zone of a melkmeisje: a glass side light sitting on a low
 * borstwering. The side light (glass) is TALLER than the borstwering — a
 * borstwering is a low wall (~900 mm) and the glass fills up to door height.
 * Below the side light is `buiten`: masonry (gemetseld muurtje), NOT a kozijn
 * vak — the outline steps and the wall is drawn as metselwerk there.
 */
function zijlichtZone(lightH = 1200, borstweringH = 900) {
  return splitCol(child(lightH, glas()), child(borstweringH, buiten()));
}

/**
 * Melkmeisje: a door with a side light that does not reach the floor — a
 * borstwering (low masonry wall) sits below the side light. Named after the
 * milkmaid-with-a-yoke silhouette (nl.wikipedia.org/wiki/Melkmeisje_(bouwkunde)).
 */
export function melkmeisje1() {
  return splitRow(
    child(900, deur("enkel")),
    child(500, zijlichtZone()),
  );
}

/** Melkmeisje with a side light (on a borstwering) on both sides of the door. */
export function melkmeisje2() {
  return splitRow(
    child(500, zijlichtZone()),
    child(1000, deur("enkel")),
    child(500, zijlichtZone()),
  );
}

/** Door with a full-width top light (bovenlicht), door below as a double (stolp). */
export function deurMetBovenlicht() {
  return splitCol(
    child(500, glas()),
    child(2100, deur("dubbel")),
  );
}

// ── Convert a rectangular matrix kozijn → split tree ─────────────

const OPENTYPE_FROM_PANELTYPE = {
  turn_tilt: "draaikiep", turn: "draai", tilt: "valraam", sliding: "schuif",
  top_hung: "uitzet", bottom_hung: "tuimel", lift_slide: "hefschuif", pivot: "taats",
};

function cellToVulling(cell) {
  if (!cell) return { type: "glas" };
  const pt = cell.panelType || "fixed_glass";
  const extra = {};
  if (cell.glaslat?.position) extra.glaslat = String(cell.glaslat.position).toLowerCase();
  const dir = cell.openingDirection;
  if (dir === "left" || dir === "right") extra.hingeSide = dir === "left" ? "links" : "rechts";
  if (dir === "outward") extra.opensOutward = true;
  if (cell.hardwareSet) extra.hardware = true;
  if (pt === "fixed_glass") return { type: "glas", ...extra };
  if (pt === "panel") return { type: "paneel" };
  if (pt === "ventilation") return { type: "rooster" };
  if (pt === "door") return { type: "deur", doorKind: "enkel", ...extra };
  return { type: "raam", openType: OPENTYPE_FROM_PANELTYPE[pt] || "draai", ...extra };
}

/** Build a split tree equivalent to a kozijn's rectangular grid + cells. */
export function gridToLayout(kozijn) {
  const cols = kozijn?.grid?.columns || [];
  const rows = kozijn?.grid?.rows || [];
  const cells = kozijn?.cells || [];
  const numCols = cols.length || 1;
  if (cols.length === 0 || rows.length === 0) return glas();
  const rowChildren = rows.map((row, r) => {
    const colChildren = cols.map((col, c) =>
      child(col.size || 1, leaf(cellToVulling(cells[r * numCols + c]))));
    return child(row.size || 1, colChildren.length > 1 ? splitRow(...colChildren) : colChildren[0].node);
  });
  return rowChildren.length > 1 ? splitCol(...rowChildren) : rowChildren[0].node;
}

/** Arbitrary free grid: `cols` columns × `rows` rows of glass (e.g. 9×4 = 8 stijlen, 3 dorpels). */
export function freeGrid(cols = 9, rows = 4) {
  const makeRow = () => splitRow(...Array.from({ length: cols }, () => child(1, glas())));
  return splitCol(...Array.from({ length: rows }, () => child(1, makeRow())));
}

// ── Geometry: lay the tree out into rects ────────────────────────

/**
 * Recursively partition `rect` according to the split tree.
 * @returns {{leaves: Array<{rect, node}>, dividers: Array<{rect, direction}>}}
 */
export function layoutToRects(node, rect, dividerW = 20, out = { leaves: [], dividers: [] }) {
  layoutRec(node, rect, dividerW, out);
  clipDividersAgainstBuiten(out);
  return out;
}

function layoutRec(node, rect, dividerW, out) {
  if (!node) return out;
  if (node.kind === "leaf") {
    out.leaves.push({ rect, node });
    return out;
  }
  const horiz = node.direction === "row";
  const total = horiz ? rect.width : rect.height;
  const n = node.children.length;
  const dividerSpace = Math.max(0, n - 1) * dividerW;
  const avail = Math.max(0, total - dividerSpace);
  const sizeSum = node.children.reduce((s, c) => s + (c.size || 1), 0) || 1;
  let pos = horiz ? rect.x : rect.y;
  node.children.forEach((c, i) => {
    const len = avail * (c.size || 1) / sizeSum;
    const childRect = horiz
      ? { x: pos, y: rect.y, width: len, height: rect.height }
      : { x: rect.x, y: pos, width: rect.width, height: len };
    layoutRec(c.node, childRect, dividerW, out);
    pos += len;
    if (i < n - 1) {
      out.dividers.push({
        rect: horiz
          ? { x: pos, y: rect.y, width: dividerW, height: rect.height }
          : { x: rect.x, y: pos, width: rect.width, height: dividerW },
        direction: horiz ? "v" : "h",
        splitId: node.id,
        childIndex: i,
        avail,
        sizeSum,
      });
      pos += dividerW;
    }
  });
  return out;
}

/**
 * Clip divider segments so a mullion/transom only covers the overlap of two
 * adjacent NON-`buiten` regions (issue 10). A divider that borders the wall
 * (`buiten`) on either side loses that part: the melkmeisje tussenstijl stops
 * at the bottom of the side light instead of running down through the wall, and
 * the side-light onderdorpel (wall directly below it) drops out entirely.
 * Layouts without any `buiten` leaf are left untouched. Mirrors the same pass
 * in ofs-core (layout.rs `clip_dividers_against_buiten`) — keep them in sync.
 * Divider drag/edit metadata (splitId/childIndex/avail/sizeSum) is preserved on
 * every kept segment, so resizing still works on the clipped handle.
 */
function clipDividersAgainstBuiten(out) {
  const EPS = 1;
  const buiten = out.leaves
    .filter((l) => l.node?.vulling?.type === "buiten")
    .map((l) => l.rect);
  if (!buiten.length) return;
  const kept = [];
  for (const d of out.dividers) {
    const r = d.rect;
    if (d.direction === "v") {
      const lo = r.y, hi = r.y + r.height;
      const leftX = r.x, rightX = r.x + r.width;
      const blocked = [];
      for (const b of buiten) {
        if (Math.abs(b.x + b.width - leftX) < EPS || Math.abs(b.x - rightX) < EPS) {
          blocked.push([Math.max(b.y, lo), Math.min(b.y + b.height, hi)]);
        }
      }
      for (const [a, c] of subtractIntervals(lo, hi, blocked, EPS)) {
        kept.push({ ...d, rect: { x: r.x, y: a, width: r.width, height: c - a } });
      }
    } else {
      const lo = r.x, hi = r.x + r.width;
      const topY = r.y, botY = r.y + r.height;
      const blocked = [];
      for (const b of buiten) {
        if (Math.abs(b.y + b.height - topY) < EPS || Math.abs(b.y - botY) < EPS) {
          blocked.push([Math.max(b.x, lo), Math.min(b.x + b.width, hi)]);
        }
      }
      for (const [a, c] of subtractIntervals(lo, hi, blocked, EPS)) {
        kept.push({ ...d, rect: { x: a, y: r.y, width: c - a, height: r.height } });
      }
    }
  }
  out.dividers = kept;
}

/** `[lo, hi]` minus the union of `blocked` [a,b] intervals → remaining sub-intervals. */
function subtractIntervals(lo, hi, blocked, eps = 1) {
  const b = blocked
    .map(([a, c]) => [Math.max(a, lo), Math.min(c, hi)])
    .filter(([a, c]) => c > a + eps)
    .sort((p, q) => p[0] - q[0]);
  const out = [];
  let cur = lo;
  for (const [bs, be] of b) {
    if (bs > cur + eps) out.push([cur, bs]);
    cur = Math.max(cur, be);
  }
  if (hi > cur + eps) out.push([cur, hi]);
  return out;
}

// ── Tree editing helpers (immutable-ish: returns a new root) ──────

function clone(node) {
  if (node.kind === "leaf") return { ...node, vulling: { ...node.vulling } };
  return { ...node, children: node.children.map((c) => ({ size: c.size, node: clone(c.node) })) };
}

/** Replace the node with id `id` using transform `fn(node) -> node`. */
function transformNode(root, id, fn) {
  const r = clone(root);
  const walk = (node) => {
    if (node.id === id) return fn(node);
    if (node.kind === "split") {
      node.children = node.children.map((c) => ({ size: c.size, node: walk(c.node) }));
    }
    return node;
  };
  return walk(r);
}

/** Split a leaf into two along `direction`, keeping the original as the first child. */
export function splitLeaf(root, id, direction) {
  return transformNode(root, id, (node) => {
    if (node.kind !== "leaf") return node;
    return {
      id: nid(),
      kind: "split",
      direction,
      children: [child(1, node), child(1, glas())],
    };
  });
}

/** Set the vulling of a leaf. */
export function setVulling(root, id, vulling) {
  return transformNode(root, id, (node) =>
    node.kind === "leaf" ? { ...node, vulling: { ...vulling } } : node);
}

/** Patch fields onto a leaf's vulling (e.g. hinge side, glaslat). */
export function patchVulling(root, id, patch) {
  return transformNode(root, id, (node) =>
    node.kind === "leaf" ? { ...node, vulling: { ...node.vulling, ...patch } } : node);
}

/** Find a node by id (returns the live reference within `root`). */
export function findNode(root, id) {
  if (!root) return null;
  if (root.id === id) return root;
  if (root.kind === "split") {
    for (const c of root.children) {
      const r = findNode(c.node, id);
      if (r) return r;
    }
  }
  return null;
}

/** Set absolute sizes of two adjacent children of a split (used while dragging). */
export function setSplitChildSizes(root, splitId, childIndex, sizeI, sizeNext) {
  return transformNode(root, splitId, (node) => {
    if (node.kind !== "split") return node;
    const children = node.children.map((c, idx) => {
      if (idx === childIndex) return { size: sizeI, node: c.node };
      if (idx === childIndex + 1) return { size: sizeNext, node: c.node };
      return c;
    });
    return { ...node, children };
  });
}

/** Merge: collapse the split that directly contains `leafId` back into that single vak. */
export function mergeAt(root, leafId) {
  const walk = (node) => {
    if (node.kind !== "split") return node;
    const hit = node.children.find((c) => c.node.id === leafId);
    if (hit && node.children.length >= 2) return clone(hit.node);
    return { ...node, children: node.children.map((c) => ({ size: c.size, node: walk(c.node) })) };
  };
  return walk(clone(root));
}

/**
 * Swap the vulling (content) of two leaves, keeping tree structure, sizes and
 * node ids intact. This is "vakken uitwisselen": e.g. exchange a glass vak with
 * a door vak, or two draaikiep vakken, without moving the dividers.
 */
export function swapVullingen(root, idA, idB) {
  if (!root || idA === idB) return root;
  const a = findNode(root, idA);
  const b = findNode(root, idB);
  if (!a || !b || a.kind !== "leaf" || b.kind !== "leaf") return root;
  const va = { ...a.vulling };
  const vb = { ...b.vulling };
  return setVulling(setVulling(root, idA, vb), idB, va);
}

/** Count leaves (vakken). */
export function countLeaves(node) {
  if (!node) return 0;
  if (node.kind === "leaf") return 1;
  return node.children.reduce((s, c) => s + countLeaves(c.node), 0);
}

// ── Id hygiene ────────────────────────────────────────────────────

/** True when every node in the tree carries an id. */
function hasAllIds(node) {
  if (!node?.id) return false;
  if (node.kind === "split") return node.children.every((c) => hasAllIds(c.node));
  return true;
}

const _ensureIdsCache = new WeakMap();

/**
 * Return a tree in which every node has an id (missing ones get a fresh nid()).
 * Defensive against trees that lost their ids on a roundtrip (old .ofs files,
 * stale wasm bundles). A tree that already has ids everywhere is returned
 * as-is; otherwise the result is memoized per input object (WeakMap) so
 * repeated calls with the same tree yield the identical object — this keeps
 * id-based selection stable across re-renders.
 */
export function ensureIds(tree) {
  if (!tree) return tree;
  if (hasAllIds(tree)) return tree;
  const cached = _ensureIdsCache.get(tree);
  if (cached) return cached;
  const fill = (node) => {
    const n = { ...node, id: node.id || nid() };
    if (n.kind === "split") n.children = node.children.map((c) => ({ size: c.size, node: fill(c.node) }));
    return n;
  };
  const out = fill(tree);
  _ensureIdsCache.set(tree, out);
  return out;
}
