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

let _idCounter = 0;
const nid = () => `n${++_idCounter}`;

// ── Vakvulling (cell filling) ────────────────────────────────────

export const VULLING_LABELS = {
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

export function leaf(vulling) {
  return { id: nid(), kind: "leaf", vulling };
}
export const glas = () => leaf({ type: "glas" });
export const raam = (openType = "draai") => leaf({ type: "raam", openType });
export const deur = (doorKind = "enkel") => leaf({ type: "deur", doorKind });
export const paneel = () => leaf({ type: "paneel" });
export const rooster = () => leaf({ type: "rooster" });
/** Outside the kozijn (wall/masonry) — no vak, no member, makes the outline step. */
export const buiten = () => leaf({ type: "buiten" });

const child = (size, node) => ({ size, node });
export const splitRow = (...children) => ({ id: nid(), kind: "split", direction: "row", children });
export const splitCol = (...children) => ({ id: nid(), kind: "split", direction: "column", children });

export function vullingLabel(v) {
  if (!v) return "—";
  if (v.type === "raam") return RAAM_OPENTYPES[v.openType] || "Raam";
  if (v.type === "deur") return DEUR_SOORTEN[v.doorKind] || "Deur";
  return VULLING_LABELS[v.type] || v.type;
}

// ── Templates ────────────────────────────────────────────────────

/**
 * Side-light zone: a glass side light on top that starts higher; BELOW it is
 * outside the kozijn (the outline steps up) — no panel, no continuous bottom
 * transom/stile.
 */
function zijlichtZone(lightH = 900, outsideH = 1000) {
  return splitCol(child(lightH, glas()), child(outsideH, buiten()));
}

/** Melkmeisje: main full-height casement with a side light that begins higher. */
export function melkmeisje1() {
  return splitRow(
    child(900, raam("draaikiep")),
    child(500, zijlichtZone()),
  );
}

/** Melkmeisje with side lights on both sides. */
export function melkmeisje2() {
  return splitRow(
    child(500, zijlichtZone()),
    child(1000, raam("draaikiep")),
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
    layoutToRects(c.node, childRect, dividerW, out);
    pos += len;
    if (i < n - 1) {
      out.dividers.push({
        rect: horiz
          ? { x: pos, y: rect.y, width: dividerW, height: rect.height }
          : { x: rect.x, y: pos, width: rect.width, height: dividerW },
        direction: horiz ? "v" : "h",
      });
      pos += dividerW;
    }
  });
  return out;
}

// ── Tree editing helpers (immutable-ish: returns a new root) ──────

function clone(node) {
  if (node.kind === "leaf") return { ...node, vulling: { ...node.vulling } };
  return { ...node, children: node.children.map((c) => ({ size: c.size, node: clone(c.node) })) };
}

/** Replace the node with id `id` using transform `fn(node) -> node`. */
export function transformNode(root, id, fn) {
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

/** Count leaves (vakken). */
export function countLeaves(node) {
  if (!node) return 0;
  if (node.kind === "leaf") return 1;
  return node.children.reduce((s, c) => s + countLeaves(c.node), 0);
}
