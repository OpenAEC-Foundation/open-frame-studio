<script>
  import { selectedCellIndex, selectedMember, selectedLayoutLeafId, selectedTimberJoints, selectDividerPiece, updateCellType, updateDimensions, updateGridSizes, currentKozijn, setKozijnLayout } from "../../stores/kozijn.js";
  import { get } from "svelte/store";
  import { layoutToRects, gridRowHeights, vullingLabel, findNode, setSplitChildSizes, splitLeaf, mergeAt, swapVullingen, ensureIds } from "../../lib/layout.js";
  import { ralToHex } from "../../lib/ral-colors.js";
  import { cellFits, insetOpening, moveFit } from "../../lib/glazingFit.js";
  import { battenElevationPaths, circleDiameter, contourPath, openingPath, sawAllowancePath } from "../../lib/contour2d.js";
  import { timberMemberType, memberLabel, dividerPieceLabel } from "../../lib/labels.js";
  import { jointPaths } from "../../lib/dividerJoinery.js";
  import { fittingDrawing } from "../../lib/hardwareDrawing.js";
  import { _ } from "svelte-i18n";

  let { geometry, kozijn, zoom = 0.35, oncellcontextmenu, dividerClipId } = $props();

  // #8: reflect the chosen RAL frame colour in the drawing instead of a fixed
  // token. The front elevation shows the inside face colour (colorInside); the
  // selection highlight stays a separate stroke overlay (see below) so colour
  // and selection coexist. Falls back to the editor token when no RAL is set.
  let frameColor = $derived(kozijn?.frame?.colorInside ? ralToHex(kozijn.frame.colorInside) : "var(--editor-frame)");
  let seamColor = $derived.by(() => {
    if (!frameColor.startsWith("#")) return "var(--bg-surface)";
    const [r, g, b] = frameColor.slice(1).match(/../g).map(v => parseInt(v, 16));
    return 0.213 * r + 0.715 * g + 0.072 * b > 135 ? "#645f59" : "#c4c1bc";
  });

  // ── Free-subdivision layout: editable render of the split tree ──────────────
  // While a divider is being dragged we render a local working copy so the drag
  // is smooth and history/backend are only touched once, on release.
  let workingTree = $state(null);
  let layoutDrag = $state(null);
  // Leaf selection lives in the store so the side panels (vulling editor) see it.
  let selectedLeafId = $derived($selectedLayoutLeafId);
  // #9: "vakken uitwisselen". When a swap is armed (via the ⇄ button) this holds
  // the source vak id; the next vak click swaps the two vakken's contents.
  let swapSourceId = $state(null);
  // ensureIds is defensive: old .ofs files or a stale wasm bundle may hand us a
  // tree without node ids, while all canvas editing is id-based. It is memoized
  // so the same input yields the same object (stable selection).
  let activeTree = $derived(workingTree ?? ensureIds(kozijn?.layout ?? null));

  let layoutGeom = $derived.by(() => {
    if (!activeTree || !geometry?.innerRect) return null;
    const width = geometry.layoutDividerWidth ?? kozijn.frame.frameWidth;
    return layoutToRects(activeTree, geometry.innerRect, width, undefined, !geometry.curvedDividerTimber);
  });
  let selectedLeafRect = $derived.by(() => {
    const index = layoutGeom?.leaves.findIndex(l => l.node.id === selectedLeafId) ?? -1;
    if (index < 0) return null;
    return geometry?.curvedFrame ? fittedCells.get(index)?.openingBounds : layoutGeom.leaves[index].rect;
  });

  function vullingFill(t) {
    return t === "deur" ? "#b98b5e" : t === "paneel" ? "#c9c2b2" : t === "rooster" ? "#9aa0a8" : "var(--editor-glass)";
  }

  function openingLabel(rect, contour) {
    const diameter = circleDiameter(contour);
    return diameter == null ? `${Math.round(rect.width)}×${Math.round(rect.height)}` : `Ø ${Math.round(diameter)}`;
  }

  function beadSeams(rim, inner, mitered) {
    const { x, y, width, height } = inner;
    const right = rim.x + rim.width, bottom = rim.y + rim.height;
    const startY = mitered ? rim.y : y;
    const endY = mitered ? bottom : y + height;
    return `M${rim.x} ${startY}L${x} ${y}M${right} ${startY}L${x + width} ${y}
            M${rim.x} ${endY}L${x} ${y + height}M${right} ${endY}L${x + width} ${y + height}`;
  }

  let fittedCells = $derived(new Map((geometry?.cellRects ?? []).map(c => [c.cellIndex, c])));
  function layoutFits(leaf, index) {
    const cell = layoutGeom.leaves.length === kozijn.cells?.length ? kozijn.cells[index] : null;
    const source = fittedCells.get(index);
    const target = leaf.rect;
    if (geometry?.curvedFrame) return source ? cellFits(source, cell, kozijn.frame, geometry) : [];
    if (!source || source.vulling?.type !== leaf.node.vulling.type) {
      return cellFits({ rect: target, vulling: leaf.node.vulling }, cell, kozijn.frame, geometry);
    }
    const fits = cellFits(source, cell, kozijn.frame, geometry);
    const base = source.rect;
    if (target.x === base.x && target.y === base.y && target.width === base.width && target.height === base.height) return fits;
    return fits.map((fit, i) => {
      if (fits.length === 1) return moveFit(fit, base, target);
      const half = (r) => ({ ...r, x: r.x + i * r.width / 2, width: r.width / 2 });
      return moveFit(fit, half(base), half(target));
    });
  }

  function cellHitPath(shape, rect, fits) {
    // The lip of an inward-opening leaf is in front of the fixed frame.
    if (shape && fits.length && fits.every(f => f.contours?.sashOuter && !f.contours.sashOpensOutward)) {
      return fits.map(f => contourPath(f.contours.sashOuter)).join(" ");
    }
    return openingPath(shape, rect);
  }

  // ── Draaisymbolen op het layout-pad (NL-conventie, VKG 1.8) ─────────────────
  // Buitenaanzicht: brede zijde van de driehoek = scharnierzijde, punt =
  // kruk-/slotzijde; doorgetrokken lijn = naar buiten draaiend, streeplijn =
  // naar binnen draaiend; schuivend deel = pijl in de schuifrichting.
  // Returns [{ d, solid }] paths in vak-coördinaten.
  function vullingSymbols(v, r) {
    if (!v || (v.type !== "raam" && v.type !== "deur")) return [];
    const { x, y, width: w, height: h } = r;
    const mx = w * 0.14, my = h * 0.14;
    const left = (v.hingeSide || "links") === "links";
    const outward = v.opensOutward ?? (v.type === "raam" && v.openType === "uitzet");
    const turn = (rx, rw) => left
      ? `M${rx + rw * 0.14},${y + my} L${rx + rw * 0.86},${y + h / 2} L${rx + rw * 0.14},${y + h - my}`
      : `M${rx + rw * 0.86},${y + my} L${rx + rw * 0.14},${y + h / 2} L${rx + rw * 0.86},${y + h - my}`;
    const val = `M${x + mx},${y + h - my} L${x + w / 2},${y + my} L${x + w - mx},${y + h - my}`;
    const uitzet = `M${x + mx},${y + my} L${x + w / 2},${y + h - my} L${x + w - mx},${y + my}`;
    const arrow = (() => {
      const yMid = y + h / 2, x1 = x + w * 0.25, x2 = x + w * 0.75;
      const hd = Math.min(w, h) * 0.06;
      return left
        ? `M${x1},${yMid} L${x2},${yMid} M${x2 - hd},${yMid - hd} L${x2},${yMid} L${x2 - hd},${yMid + hd}`
        : `M${x2},${yMid} L${x1},${yMid} M${x1 + hd},${yMid - hd} L${x1},${yMid} L${x1 + hd},${yMid + hd}`;
    })();
    const syms = [];
    if (v.type === "deur") {
      if (v.doorKind === "dubbel") {
        // Stolp: twee vleugels, scharnieren op de buitenstijlen, punten op de naald.
        syms.push({ d: `M${x + w * 0.07},${y + my} L${x + w * 0.46},${y + h / 2} L${x + w * 0.07},${y + h - my}`, solid: outward });
        syms.push({ d: `M${x + w * 0.93},${y + my} L${x + w * 0.54},${y + h / 2} L${x + w * 0.93},${y + h - my}`, solid: outward });
      } else if (v.doorKind === "schuif" || v.doorKind === "hefschuif") {
        syms.push({ d: arrow, solid: true });
      } else {
        syms.push({ d: turn(x, w), solid: outward });
      }
      return syms;
    }
    switch (v.openType) {
      case "draaikiep":
        syms.push({ d: turn(x, w), solid: outward });
        syms.push({ d: val, solid: false });
        break;
      case "valraam":
        syms.push({ d: val, solid: false });
        break;
      case "uitzet":
        syms.push({ d: uitzet, solid: true });
        break;
      case "tuimel":
        syms.push({ d: uitzet, solid: outward });
        syms.push({ d: `M${x + mx},${y + h / 2} L${x + w - mx},${y + h / 2}`, solid: false });
        break;
      case "taats":
        syms.push({ d: turn(x, w), solid: outward });
        syms.push({ d: `M${x + w / 2},${y + my} L${x + w / 2},${y + h - my}`, solid: false });
        break;
      case "schuif":
      case "hefschuif":
        syms.push({ d: arrow, solid: true });
        break;
      default: // draai
        syms.push({ d: turn(x, w), solid: outward });
    }
    return syms;
  }

  // Borstwering (`buiten`-vak): strek het metselwerk door tot de buitenrand aan
  // zijden waar geen kozijnhout meer zit (de getrapte omtrek slaat die kant
  // over), anders blijft daar een kale strook editor-achtergrond als "gat".
  function buitenRect(r) {
    const fw = kozijn?.frame?.frameWidth || 67;
    const ow = kozijn?.frame?.outerWidth || 0;
    const oh = kozijn?.frame?.outerHeight || 0;
    const x0 = Math.abs(r.x - fw) < 1 ? 0 : r.x;
    const y0 = Math.abs(r.y - fw) < 1 ? 0 : r.y;
    const x1 = Math.abs(r.x + r.width - (ow - fw)) < 1 ? ow : r.x + r.width;
    const y1 = Math.abs(r.y + r.height - (oh - fw)) < 1 ? oh : r.y + r.height;
    return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
  }

  // ── Drag a layout divider (tussenstijl/-dorpel) to resize the two vakken ────
  // Mirrors GridHandles' screen→mm mapping (deltaMm = deltaPx / zoom); the vak
  // sizes are relative within their split, so a mm delta maps to a size delta
  // via the divider's avail/sizeSum (carried by layoutToRects).
  function startLayoutDrag(d, e, piece = null) {
    if (!d || e.ctrlKey) return;
    e.stopPropagation();
    e.preventDefault();
    const base = ensureIds(kozijn?.layout ?? null);
    const split = findNode(base, d.splitId);
    if (!split || split.kind !== "split") return;
    layoutDrag = {
      base, moved: false, piece,
      splitId: d.splitId, childIndex: d.childIndex, axis: d.direction,
      avail: d.avail, sizeSum: d.sizeSum,
      startX: e.clientX, startY: e.clientY,
      origI: split.children[d.childIndex].size,
      origNext: split.children[d.childIndex + 1].size,
    };
    workingTree = base;
    window.addEventListener("mousemove", onLayoutDragMove);
    window.addEventListener("mouseup", onLayoutDragUp);
  }
  function onLayoutDragMove(e) {
    if (!layoutDrag) return;
    const deltaPx = layoutDrag.axis === "v" ? e.clientX - layoutDrag.startX : e.clientY - layoutDrag.startY;
    if (!layoutDrag.moved && Math.abs(deltaPx) < 2) return;
    layoutDrag.moved = true;
    const deltaMm = deltaPx / (zoom || 1);
    let deltaSize = deltaMm * layoutDrag.sizeSum / Math.max(1, layoutDrag.avail);
    const min = 0.05 * layoutDrag.sizeSum;
    deltaSize = Math.max(-(layoutDrag.origI - min), Math.min(layoutDrag.origNext - min, deltaSize));
    workingTree = setSplitChildSizes(
      layoutDrag.base, layoutDrag.splitId, layoutDrag.childIndex,
      layoutDrag.origI + deltaSize, layoutDrag.origNext - deltaSize,
    );
  }
  function onLayoutDragUp() {
    window.removeEventListener("mousemove", onLayoutDragMove);
    window.removeEventListener("mouseup", onLayoutDragUp);
    // Only commit when the divider actually moved — a bare mousedown/mouseup
    // would otherwise push a no-op history entry and a redundant IPC roundtrip.
    if (workingTree && layoutDrag?.moved) setKozijnLayout(workingTree);
    else if (layoutDrag?.piece) selectDividerPiece(layoutDrag.piece);
    workingTree = null;
    layoutDrag = null;
  }

  function selectLeaf(id, e) {
    // Ctrl(+Alt)+klik = tussenstijl/-dorpel plaatsen: laat de click doorbubbelen
    // naar de svg-handler van KozijnEditor in plaats van hem hier te vangen.
    if (e?.ctrlKey) return;
    e?.stopPropagation();
    // If a swap is armed and a *different* vak is clicked, complete the swap.
    if (swapSourceId && id !== swapSourceId && activeTree) {
      setKozijnLayout(swapVullingen(activeTree, swapSourceId, id));
      swapSourceId = null;
      selectedLayoutLeafId.set(id);
      selectedCellIndex.set(null);
      selectedMember.set(null);
      return;
    }
    swapSourceId = null; // clicking (the same or any) vak cancels an armed swap
    selectedLayoutLeafId.set(id);
    selectedCellIndex.set(null);
    selectedMember.set(null);
  }
  function doSplitLayout(dir, e) {
    e?.stopPropagation();
    swapSourceId = null;
    if (selectedLeafId && activeTree) setKozijnLayout(splitLeaf(activeTree, selectedLeafId, dir));
  }
  function doMergeLayout(e) {
    e?.stopPropagation();
    swapSourceId = null;
    if (selectedLeafId && activeTree) setKozijnLayout(mergeAt(activeTree, selectedLeafId));
  }
  // Arm a swap: the next vak click will exchange its contents with this vak.
  // Toggling on the same vak disarms it.
  function doSwapLayout(e) {
    e?.stopPropagation();
    if (!selectedLeafId) return;
    swapSourceId = swapSourceId === selectedLeafId ? null : selectedLeafId;
  }

  // ── Round/elliptical frame: donut geometry + inner clip ─────────────────────
  // The donut is drawn UNDER the cell layer and the cells are clipped to the
  // inner opening, so the decoration never paints over vak content and never
  // steals pointer events.
  const clipId = `ofs-inner-clip-${Math.random().toString(36).slice(2, 9)}`;
  // Running-bond brick pattern for the borstwering (masonry) under a melkmeisje
  // side light — makes the `buiten` zone read as a wall instead of an empty hole.
  const brickPatternId = `ofs-brick-${Math.random().toString(36).slice(2, 9)}`;
  let roundClip = $derived.by(() => {
    const f = kozijn?.frame;
    if (geometry?.curvedFrame || !f?.shape) return null;
    if (f.shape.shapeType === "round") {
      const rOuter = Math.min(f.outerWidth, f.outerHeight) / 2;
      const rInner = rOuter - f.frameWidth;
      if (rInner <= 0) return null;
      return { kind: "circle", cx: f.outerWidth / 2, cy: f.outerHeight / 2, rOuter, rInner };
    }
    if (f.shape.shapeType === "elliptical") {
      const rx = f.shape.ellipseRx || f.outerWidth / 2;
      const ry = f.shape.ellipseRy || f.outerHeight / 3;
      return {
        kind: "ellipse", cx: f.outerWidth / 2, cy: f.outerHeight / 2,
        rx, ry, irx: rx - f.frameWidth, iry: ry - f.frameWidth,
      };
    }
    return null;
  });

  // ── Dimension view model ─────────────────────────────────────────────────────
  // The backend bakes dimension levels in as fixed model-mm offsets, and older
  // wasm builds use different offsets/orderings than current ofs-core. So:
  //  - the level index per side is re-derived from the distinct baked offsets,
  //  - every dimension is classified semantically by its span (outer / dagmaat /
  //    stijl / tussenstijl / vak), independent of backend version,
  //  - render positions are re-projected screen-space-aware so the levels never
  //    overlap each other or the kozijn at low zoom,
  //  - labels get a deterministic lane sweep (on/above/below the line) so a
  //    chain of narrow segments stays readable (kozijnstaat practice),
  //  - with a free-subdivision layout the (stale) grid chain is replaced by a
  //    chain generated from layoutGeom, editable via the split tree,
  //  - round/elliptical kozijnen only show the buitenwerkse maten.
  // Rendering and inline editing both consume this single source.
  const DIM_EPS = 1.0;
  const near = (a, b, eps = DIM_EPS) => Math.abs(a - b) < eps;

  let dimsView = $derived.by(() => {
    const k = kozijn;
    if (!k?.frame) return [];
    const dims = geometry?.dimensions || [];
    const ow = k.frame.outerWidth || 0;
    const oh = k.frame.outerHeight || 0;
    const fw = k.frame.frameWidth || 0;
    const z = zoom || 1;
    const fontSize = 12 / z;
    const bgPad = 3 / z;
    const tickSz = 6 / z;
    const labelH = fontSize + bgPad * 2;
    // Screen-aware level spacing: room for the label itself plus an above/below
    // lane without touching the next level (all values are model-mm at this zoom).
    const minStart = labelH + 12 / z;
    const minStep = labelH * 1.5 + 10 / z;
    const laneOffset = labelH / 2 + tickSz + 2 / z;
    const shape = k.frame.shape?.shapeType;
    const roundish = shape === "round" || shape === "elliptical";
    const diameter = shape === "round" ? circleDiameter(geometry?.curvedFrame?.outer) : null;
    if (diameter != null) {
      const [cx, cy] = geometry.curvedFrame.outer.segments[0].center;
      const y = cy + diameter / 2 + minStart;
      const text = `Ø ${Math.round(diameter * 10) / 10}`;
      return [{ key: "outer-diameter", kind: "diameter", editable: true, isH: true,
        value: diameter, text, x1: cx - diameter / 2, x2: cx + diameter / 2, y1: y, y2: y,
        midX: cx, midY: y, labelW: text.length * fontSize * 0.6 + bgPad * 2, labelH, laneOff: 0 }];
    }
    const hasLayout = !!(k.layout && layoutGeom);

    const gridCells = hasLayout ? [] : geometry?.cellRects ?? [];
    const columns = gridCells.filter(c => c.row === 0);
    const rows = gridCells.filter(c => c.col === 0);

    const SIDES = {
      bottom: { isH: true, base: oh, dir: 1 },
      top: { isH: true, base: 0, dir: -1 },
      right: { isH: false, base: ow, dir: 1 },
      left: { isH: false, base: 0, dir: -1 },
    };
    const bySide = { bottom: [], top: [], right: [], left: [] };
    for (const dim of dims) (bySide[dim.side] || bySide.bottom).push(dim);

    // Semantic classification by span — independent of backend level offsets
    function classify(dim, cfg) {
      const a = cfg.isH ? Math.min(dim.x1, dim.x2) : Math.min(dim.y1, dim.y2);
      const b = cfg.isH ? Math.max(dim.x1, dim.x2) : Math.max(dim.y1, dim.y2);
      const total = cfg.isH ? ow : oh;
      if (near(a, 0) && near(b, total)) return { kind: "outer", editable: true, a, b };
      if (near(a, fw) && near(b, total - fw)) return { kind: "dagmaat", editable: true, a, b };
      // Vak match before the edge-touch rule: older wasm builds lay the last
      // column flush against the outer edge, and a genuine vak must stay
      // editable there. Current-backend vak dims never touch the outer edge.
      for (const cell of cfg.isH ? columns : rows) {
        const r = cell.rect;
        if (near(a, cfg.isH ? r.x : r.y) && near(b - a, cfg.isH ? r.width : r.height)) {
          return { kind: "vak", editable: true, a, b, gridAxis: cfg.isH ? "col" : "row", gridIndex: cfg.isH ? cell.col : cell.row };
        }
      }
      if (near(a, 0) || near(b, total)) return { kind: "stijl", editable: false, a, b };
      return { kind: "tussenstijl", editable: false, a, b };
    }
    const isChainKind = (kind) => kind !== "outer" && kind !== "dagmaat";

    // Editing a layout vak = resizing the two children of the split next to it;
    // exact (and thus offered) only when the leaf is a DIRECT child of that split.
    function layoutEditInfo(leaf, cfg, a, b) {
      const wantDir = cfg.isH ? "v" : "h";
      const lo = cfg.isH ? leaf.rect.y : leaf.rect.x;
      const hi = lo + (cfg.isH ? leaf.rect.height : leaf.rect.width);
      for (const d of layoutGeom.dividers) {
        if (d.direction !== wantDir) continue;
        const dPos = cfg.isH ? d.rect.x : d.rect.y;
        const dEnd = dPos + (cfg.isH ? d.rect.width : d.rect.height);
        const dLo = cfg.isH ? d.rect.y : d.rect.x;
        const dHi = dLo + (cfg.isH ? d.rect.height : d.rect.width);
        if (dHi <= lo + 0.5 || dLo >= hi - 0.5) continue; // no cross overlap
        const split = findNode(activeTree, d.splitId);
        if (!split || split.kind !== "split") continue;
        if (near(dPos, b) && split.children[d.childIndex]?.node?.id === leaf.node.id) {
          return {
            splitId: d.splitId, childIndex: d.childIndex, editIsFirst: true,
            origI: split.children[d.childIndex].size,
            origNext: split.children[d.childIndex + 1]?.size ?? 1,
            avail: d.avail, sizeSum: d.sizeSum, leafLen: b - a,
          };
        }
        if (near(dEnd, a) && split.children[d.childIndex + 1]?.node?.id === leaf.node.id) {
          return {
            splitId: d.splitId, childIndex: d.childIndex, editIsFirst: false,
            origI: split.children[d.childIndex].size,
            origNext: split.children[d.childIndex + 1].size,
            avail: d.avail, sizeSum: d.sizeSum, leafLen: b - a,
          };
        }
      }
      return null;
    }

    // Chain from the free-subdivision layout: edges of the leaves that touch
    // the inner edge (bottom for horizontal, right for vertical) → segments.
    function layoutChain(cfg) {
      const total = cfg.isH ? ow : oh;
      const innerEdge = (cfg.isH ? oh : ow) - fw;
      const bounds = [0, fw, total - fw, total];
      const anchored = [];
      for (const l of layoutGeom.leaves) {
        if (l.node.vulling?.type === "buiten") continue;
        const touches = cfg.isH
          ? near(l.rect.y + l.rect.height, innerEdge)
          : near(l.rect.x + l.rect.width, innerEdge);
        if (!touches) continue;
        anchored.push(l);
        const a = cfg.isH ? l.rect.x : l.rect.y;
        bounds.push(a, a + (cfg.isH ? l.rect.width : l.rect.height));
      }
      const uniq = [];
      for (const v of bounds.sort((p, q) => p - q)) {
        if (!uniq.length || v - uniq[uniq.length - 1] >= 0.5) uniq.push(v);
      }
      const segs = [];
      for (let i = 0; i + 1 < uniq.length; i++) {
        const a = uniq[i], b = uniq[i + 1];
        const seg = { a, b, kind: "layoutSeg", editable: false, layout: null };
        if (near(a, 0) || near(b, total)) {
          seg.kind = "stijl";
        } else {
          const leaf = anchored.find((l) => {
            const la = cfg.isH ? l.rect.x : l.rect.y;
            return near(la, a) && near(la + (cfg.isH ? l.rect.width : l.rect.height), b);
          });
          if (leaf) {
            seg.kind = "layoutVak";
            seg.layout = layoutEditInfo(leaf, cfg, a, b);
            seg.editable = !!seg.layout;
          } else if (near(b - a, geometry.layoutDividerWidth ?? fw)) {
            // Divider-width gap between anchored leaves → tussenstijl/-dorpel
            seg.kind = "tussenstijl";
          }
        }
        segs.push(seg);
      }
      return segs;
    }

    const out = [];
    for (const side of Object.keys(SIDES)) {
      const cfg = SIDES[side];
      const list = bySide[side];
      const wantLayoutChain = hasLayout && !roundish && (side === "bottom" || side === "right");
      if (!list.length && !wantLayoutChain) continue;

      // Geometric level index from the distinct baked offsets (0 = closest)
      const offKey = cfg.isH ? "y1" : "x1";
      const offsets = [];
      for (const dim of list) {
        if (!offsets.some((o) => near(o, dim[offKey]))) offsets.push(dim[offKey]);
      }
      offsets.sort((p, q) => cfg.dir * (p - q));
      const origStart = offsets.length ? cfg.dir * (offsets[0] - cfg.base) : 20;
      const origStep = offsets.length > 1 ? cfg.dir * (offsets[1] - offsets[0]) : 35;
      const start = Math.max(origStart, minStart);
      const step = Math.max(origStep, minStep);

      let entries = list.map((dim) => {
        const level = offsets.findIndex((o) => near(o, dim[offKey]));
        const c = classify(dim, cfg);
        const num = Number(dim.label);
        return {
          ...c, side, isH: cfg.isH,
          level: level < 0 ? 0 : level,
          text: num > 0 ? String(Math.round(num)) : String(dim.label ?? ""),
          value: num > 0 ? Math.round(num) : Math.round(c.b - c.a),
        };
      });

      if (roundish) {
        // Rond/ellips: alleen de buitenwerkse breedte- en hoogtemaat tonen
        entries = entries.filter((e) => e.kind === "outer");
      } else if (wantLayoutChain) {
        // Vrije indeling: grid-ketting vervangen door ketting uit de layout.
        // Drop every dim at the chain level too — a vestigial 1×1 grid emits a
        // full-span vak there that classifies as dagmaat and would collide.
        const chainLevels = entries.filter((e) => isChainKind(e.kind)).map((e) => e.level);
        const chainLevel = chainLevels.length ? Math.min(...chainLevels) : 0;
        entries = entries.filter((e) => !isChainKind(e.kind) && e.level !== chainLevel);
        for (const seg of layoutChain(cfg)) {
          entries.push({
            ...seg, side, isH: cfg.isH, level: chainLevel,
            text: String(Math.round(seg.b - seg.a)),
            value: Math.round(seg.b - seg.a),
          });
        }
      }
      if (!entries.length) continue;

      // Compact the level indices after filtering (e.g. round: only outer left)
      const lvls = [...new Set(entries.map((e) => e.level))].sort((p, q) => p - q);
      for (const e of entries) e.level = lvls.indexOf(e.level);

      // Project onto screen-space-aware level positions
      for (const e of entries) {
        const linePos = cfg.base + cfg.dir * (start + e.level * step);
        e.mid = (e.a + e.b) / 2;
        e.x1 = e.isH ? e.a : linePos;
        e.x2 = e.isH ? e.b : linePos;
        e.y1 = e.isH ? linePos : e.a;
        e.y2 = e.isH ? linePos : e.b;
        e.midX = e.isH ? e.mid : linePos;
        e.midY = e.isH ? linePos : e.mid;
        e.labelW = e.text.length * fontSize * 0.6 + bgPad * 2;
        e.labelH = labelH;
      }

      // Deterministic label lanes per level: labels wider than ~85% of their
      // segment go above the line (kozijnstaat practice for houtdiktes);
      // colliding labels fall over to the above/below lanes.
      const byLevel = new Map();
      for (const e of entries) {
        if (!byLevel.has(e.level)) byLevel.set(e.level, []);
        byLevel.get(e.level).push(e);
      }
      for (const group of byLevel.values()) {
        group.sort((p, q) => p.mid - q.mid);
        const lanes = { on: -Infinity, above: -Infinity, below: -Infinity };
        for (const e of group) {
          const fits = e.labelW <= 0.85 * (e.b - e.a);
          const order = fits ? ["on", "above", "below"] : ["above", "below"];
          let lane = order.find((ln) => e.mid - e.labelW / 2 >= lanes[ln] + 4 / z);
          if (!lane) lane = order.reduce((m, ln) => (lanes[ln] < lanes[m] ? ln : m), order[0]);
          lanes[lane] = e.mid + e.labelW / 2;
          e.laneOff = lane === "on" ? 0 : lane === "above" ? -laneOffset : laneOffset;
        }
      }

      let n = 0;
      for (const e of entries) e.key = `${side}:${e.level}:${n++}`;
      out.push(...entries);
    }
    return out;
  });

  // Inline dimension editing state
  let editingDim = $state(null); // { key, value, entry }
  let editInputEl = $state(null);

  function handleDimClick(entry, e) {
    e.stopPropagation();
    if (!entry.editable) return;
    editingDim = { key: entry.key, value: entry.value, entry };

    // Focus input after render
    requestAnimationFrame(() => {
      if (editInputEl) {
        editInputEl.focus();
        editInputEl.select();
      }
    });
  }

  function handleDimKeyDown(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      commitDimEdit();
    }
    if (e.key === "Escape") {
      editingDim = null;
    }
  }

  function commitDimEdit() {
    if (!editingDim) return;
    const entry = editingDim.entry;
    const newVal = parseFloat(editInputEl?.value);
    editingDim = null;
    if (!entry?.editable || !newVal || newVal <= 0) return;

    const k = get(currentKozijn);
    if (!k) return;
    const ow = k.frame.outerWidth;
    const oh = k.frame.outerHeight;
    const fw = k.frame.frameWidth;

    if (entry.kind === "diameter") {
      updateDimensions(newVal, newVal);
    } else if (entry.kind === "outer") {
      // Buitenwerkse maat
      if (entry.isH) updateDimensions(newVal, oh);
      else updateDimensions(ow, newVal);
    } else if (entry.kind === "dagmaat") {
      // Dagmaat → buitenwerkse maat; the backend rescales the vakken proportionally
      if (entry.isH) updateDimensions(newVal + 2 * fw, oh);
      else updateDimensions(ow, newVal + 2 * fw);
    } else if (entry.kind === "vak") {
      // Vakmaat: resize this column/row and compensate the neighbour so the
      // buitenwerkse maat stays fixed
      const colSizes = k.grid.columns.map((c) => c.size);
      const fromDay = entry.gridAxis === "row" && !!geometry.curvedFrame;
      const rowSizes = gridRowHeights(k, fromDay ? geometry : null);
      const arr = entry.gridAxis === "col" ? colSizes : rowSizes;
      const i = entry.gridIndex;
      if (i == null || i < 0 || i >= arr.length) return;
      const size = Math.max(100, arr[i] + newVal - (entry.b - entry.a));
      const nb = i + 1 < arr.length ? i + 1 : i - 1;
      if (nb >= 0) arr[nb] = Math.max(100, arr[nb] - (size - arr[i]));
      arr[i] = size;
      updateGridSizes(colSizes, rowSizes, fromDay);
    } else if (entry.kind === "layoutVak" && entry.layout) {
      // Vrije indeling: pas de twee kinderen van de aangrenzende split aan
      // (same mm→size mapping and minimum as the divider drag)
      const L = entry.layout;
      if (!activeTree) return;
      let delta = (newVal - L.leafLen) * L.sizeSum / Math.max(1, L.avail);
      const min = 0.05 * L.sizeSum;
      if (L.editIsFirst) {
        delta = Math.max(-(L.origI - min), Math.min(L.origNext - min, delta));
        setKozijnLayout(setSplitChildSizes(activeTree, L.splitId, L.childIndex, L.origI + delta, L.origNext - delta));
      } else {
        delta = Math.max(-(L.origNext - min), Math.min(L.origI - min, delta));
        setKozijnLayout(setSplitChildSizes(activeTree, L.splitId, L.childIndex, L.origI - delta, L.origNext + delta));
      }
    }
  }

  const FRAME_MEMBER_NAMES = ["frame_top", "frame_bottom", "frame_left", "frame_right"];

  function handleCellRightClick(cellIndex, e) {
    e.preventDefault();
    e.stopPropagation();
    selectedCellIndex.set(cellIndex);
    selectedMember.set(null);
    if (oncellcontextmenu) {
      oncellcontextmenu(cellIndex, e.clientX, e.clientY);
    }
  }

  function handleMemberClick(memberType, index, e) {
    e.stopPropagation();
    selectedCellIndex.set(null);
    selectedLayoutLeafId.set(null);
    selectedMember.set({ type: memberType, index });
  }

  const PANEL_COLORS = {
    fixed_glass: { fill: "var(--editor-glass)", stroke: "var(--editor-glass-stroke)" },
    turn_tilt: { fill: "#DBEAFE", stroke: "#3B82F6" },
    turn: { fill: "#DBEAFE", stroke: "#60A5FA" },
    tilt: { fill: "#E0E7FF", stroke: "#818CF8" },
    sliding: { fill: "#D1FAE5", stroke: "#34D399" },
    door: { fill: "var(--editor-door)", stroke: "#F97316" },
    panel: { fill: "var(--editor-panel)", stroke: "#A8A29E" },
    ventilation: { fill: "#FEF3C7", stroke: "#F59E0B" },
  };

  function cellColor(cellIndex) {
    const cell = kozijn.cells[cellIndex];
    if (!cell) return PANEL_COLORS.fixed_glass;
    return PANEL_COLORS[cell.panelType] || PANEL_COLORS.fixed_glass;
  }

  function cellLabel(cellIndex) {
    const cell = kozijn.cells[cellIndex];
    if (!cell) return "";
    const labels = {
      fixed_glass: "VG",
      turn_tilt: "DK",
      turn: "D",
      tilt: "K",
      sliding: "S",
      door: "DR",
      panel: "P",
      ventilation: "V",
    };
    return labels[cell.panelType] || "";
  }

  /**
   * EN 12519 opening symbols — triangle with base at hinge side, apex at handle side.
   * Returns array of SVG path data strings for the cell's opening type.
   */
  function openingSymbols(cellIndex, rect) {
    const cell = kozijn.cells[cellIndex];
    if (!cell || !cell.panelType) return [];
    const type = cell.panelType;
    const dir = cell.openingDirection || "right";
    const { x, y, width: w, height: h } = rect;
    const cx = x + w / 2;
    const cy = y + h / 2;
    const paths = [];

    // Turn symbol: triangle, base at hinge side, apex at handle side
    if (type === "turn" || type === "turn_tilt" || type === "door") {
      if (dir === "left") {
        // Hinge on LEFT → apex on RIGHT
        paths.push({ d: `M${x},${y} L${x + w},${cy} L${x},${y + h} Z`, label: "turn" });
      } else {
        // Hinge on RIGHT → apex on LEFT
        paths.push({ d: `M${x + w},${y} L${x},${cy} L${x + w},${y + h} Z`, label: "turn" });
      }
    }

    // Tilt symbol: triangle from bottom, apex at top center
    if (type === "tilt" || type === "turn_tilt") {
      paths.push({ d: `M${x},${y + h} L${cx},${y} L${x + w},${y + h} Z`, label: "tilt" });
    }

    // Sliding: horizontal arrows
    if (type === "sliding") {
      const ay = cy;
      if (dir === "left") {
        paths.push({ d: `M${x + w * 0.8},${ay} L${x + w * 0.2},${ay} M${x + w * 0.3},${ay - h * 0.08} L${x + w * 0.2},${ay} L${x + w * 0.3},${ay + h * 0.08}`, label: "slide" });
      } else {
        paths.push({ d: `M${x + w * 0.2},${ay} L${x + w * 0.8},${ay} M${x + w * 0.7},${ay - h * 0.08} L${x + w * 0.8},${ay} L${x + w * 0.7},${ay + h * 0.08}`, label: "slide" });
      }
    }

    return paths;
  }

  /**
   * Hinge and handle positions for operable cells.
   */
  function hardwarePositions(cellIndex, rect) {
    const cell = kozijn.cells[cellIndex];
    if (!cell || ["pvc","aluminum"].includes(kozijn.frame?.material)) return { hinges: [], handle: null };
    const type = cell.panelType;
    const dir = cell.openingDirection || "right";
    if (!["turn", "turn_tilt", "tilt", "door"].includes(type)) return { hinges: [], handle: null };

    const { x, y, width: w, height: h } = rect;
    const hinges = [];
    const hingeR = Math.min(4, w * 0.02);

    if (type === "tilt") {
      // Bottom-hung: hinges at bottom-left and bottom-right
      hinges.push({ cx: x + w * 0.15, cy: y + h });
      hinges.push({ cx: x + w * 0.85, cy: y + h });
      return { hinges, handle: { x: x + w / 2, y: y, dir: "top" } };
    }

    // Turn / turn_tilt / door: hinges on the hinge side
    const hingeX = dir === "left" ? x : x + w;
    const numHinges = h > 1000 ? 3 : 2;
    const spacing = h / (numHinges + 1);
    for (let i = 1; i <= numHinges; i++) {
      hinges.push({ cx: hingeX, cy: y + spacing * i });
    }

    // Handle on opposite side, at mid-height
    const handleX = dir === "left" ? x + w : x;
    return { hinges, handle: { x: handleX, y: y + h * 0.45, dir: dir === "left" ? "right" : "left" } };
  }

  function handleCellClick(cellIndex, e) {
    if (e.ctrlKey) return;
    e.stopPropagation();
    selectedMember.set(null);
    selectedCellIndex.set(cellIndex);
  }
</script>

{#snippet sashSeat(fit, opening)}
  {#if fit?.contours?.sashSeat && !fit.contours.sashOpensOutward && opening}
    <path class="sash-seat" d={contourPath(fit.contours.sashSeat) + " " + contourPath(opening)}
      fill-rule="evenodd" fill={seamColor} pointer-events="none" />
  {/if}
{/snippet}

{#snippet sealElevation(fit, source, rect, outward)}
  {#if fit.systemSeals}
    {@const paths = fit.contours}
    {@const bead = insetOpening(fit.rebate, fit.beadHeight)}
    {@const beadCovers = paths ? fit.beadHeight >= paths.rebateHeight : bead.width <= fit.day.width}
    {#if (fit.sealThickness ?? 2) > 0}
      <path class="glass-gasket" d={openingPath(beadCovers ? paths?.beadInner : paths?.day, beadCovers ? bead : fit.day)}
        fill="none" stroke="#343536" stroke-width="2" pointer-events="none" />
    {/if}
    {#if fit.sashOuter && fit.systemSeals.hinged && fit.systemSeals.sealingLevels}
      <!-- In the elevation the outermost lip hides the deeper sealing levels. -->
      <path class="sash-gasket" d={outward ? openingPath(source?.openingContour,rect) : openingPath(paths?.sashOuter,fit.sashOuter)}
        fill="none" stroke="#343536" stroke-width="2" pointer-events="none" />
    {/if}
  {/if}
{/snippet}

{#snippet glazingFace(fit, glassColor, cellIndex)}
  {#if fit.contours}
    {@const c = fit.contours}
    {@const inside = fit.beadPosition !== "buiten"}
    <g class="glazing-face curved" stroke={seamColor} pointer-events="none"
       clip-path={c.sashOpensOutward && c.sashOuter ? `url(#${clipId}-cell-${cellIndex})` : null}>
      {#if c.sashTimber?.length}
        {#each c.sashTimber as piece}
          <path class="sash-piece" d={contourPath(piece.outline)} fill={frameColor} />
        {/each}
      {:else if c.sashOuter}
        <path d={contourPath(c.sashOuter)} fill={frameColor} class="sash-outline" />
      {/if}
      <path d={contourPath(inside ? c.rebate : c.day)} fill={frameColor} />
      {#if inside && fit.beadHeight < c.rebateHeight}
        <path d={contourPath(c.beadInner)} fill={frameColor} />
      {/if}
      <path d={contourPath(fit.beadHeight >= c.rebateHeight ? c.beadInner : c.day)} fill={glassColor} />
    </g>
  {:else}
  {@const beadInner = insetOpening(fit.rebate, fit.beadHeight)}
  {@const inside = fit.beadPosition !== "buiten"}
  {@const rim = inside ? fit.rebate : fit.day}
  <g class="glazing-face" stroke={seamColor} pointer-events="none"
    clip-path={fit.systemSeals && fittedCells.get(cellIndex)?.hardwareLeaves?.[0]?.opensOutward ? `url(#${clipId}-cell-${cellIndex})` : null}>
    {#if fit.sashOuter}
      <rect {...fit.sashOuter} fill={frameColor} class="sash-outline" />
    {/if}
    <!-- Inside elevation. The bead shoulder is visible on its installation
         side. An exterior bead is visible only where it projects past the stop. -->
    <rect {...rim} fill={frameColor} />
    {#if inside || beadInner.width < rim.width}
      <rect {...beadInner} fill={beadInner.width > fit.day.width ? frameColor : glassColor} />
      {#if beadInner.width > fit.day.width}
        <rect {...fit.day} fill={glassColor} />
      {/if}
      {#if inside}
        <path fill="none" d={beadSeams(rim, beadInner, fit.beadMitered)} />
      {/if}
    {:else}
      <rect {...fit.day} fill={glassColor} />
    {/if}
  </g>
  {/if}
{/snippet}

<g>
  <defs>
    {#if geometry.curvedFrame}
      <clipPath id={clipId}><path d={contourPath(geometry.curvedFrame.outer)} /></clipPath>
      <clipPath id={clipId + "-day"}><path d={contourPath(geometry.curvedFrame.day)} /></clipPath>
      {#if geometry.curvedDividerTimber && dividerClipId}
        {@const leaves = geometry.cellRects.flatMap(cr => [cr.fit, cr.secondFit]).filter(f => f?.contours?.sashOuter && !f.contours.sashOpensOutward)}
        <clipPath id={dividerClipId}><path clip-rule="evenodd"
          d={[contourPath(geometry.curvedFrame.day), ...leaves.map(f => contourPath(f.contours.sashOuter))].join(" ")} /></clipPath>
      {/if}
      {#each geometry.cellRects as cr}
        <clipPath id={clipId + "-cell-" + cr.cellIndex}><path d={contourPath(cr.openingContour)} /></clipPath>
        {#if cr.fit?.contours?.sashOuter}
          <clipPath id={clipId + "-glass-" + cr.cellIndex}>
            {#each [cr.fit, cr.secondFit].filter(Boolean) as fit}
              <path d={contourPath(fit.beadHeight >= fit.contours.rebateHeight ? fit.contours.beadInner : fit.contours.day)} />
            {/each}
          </clipPath>
        {/if}
      {/each}
    {:else}
      {#each geometry.cellRects as cr}
        {#if cr.hardwareLeaves?.[0]?.opensOutward}
          <clipPath id={clipId + "-cell-" + cr.cellIndex}><rect {...cr.rect} /></clipPath>
        {/if}
      {/each}
    {/if}
    <!-- Metselwerk (borstwering) — running bond, in mm user-space so it scales. -->
    <pattern id={brickPatternId} width="120" height="60" patternUnits="userSpaceOnUse">
      <rect width="120" height="60" fill="#9c876d" />
      <g stroke="#cabfa8" stroke-width="4" stroke-linecap="square">
        <line x1="0" y1="0" x2="120" y2="0" />
        <line x1="0" y1="30" x2="120" y2="30" />
        <line x1="0" y1="0" x2="0" y2="30" />
        <line x1="60" y1="30" x2="60" y2="60" />
      </g>
    </pattern>
  </defs>
  {#if geometry.perimeterStop}
    <path class="perimeter-stop" d={contourPath(geometry.perimeterStop.outer) + " " + contourPath(geometry.perimeterStop.inner)}
      fill-rule="evenodd" fill={kozijn.frame?.colorOutside ? ralToHex(kozijn.frame.colorOutside) : frameColor}
      stroke={seamColor} stroke-width="1" pointer-events="none" />
  {/if}
  <!-- Frame members background (drawn first, behind cells). Mitered (verstek)
       and stepped (melkmeisje) frames arrive as closed polygons in
       geometry.framePolygons; fall back to the plain rects otherwise (older
       wasm bundle / plain butt frame) so the canvas renders either shape. -->
  {#if geometry.curvedTimber?.length}
    <g class="curved-frame">
      {#each geometry.curvedTimber as piece}
        {@const type = timberMemberType(piece)}
        {@const selected = $selectedMember?.type === type && $selectedMember.index === piece.index}
        {@const label = memberLabel($_, piece.arc ? "frame_arc" : type, piece.profileSide)}
        <path class="member timber-piece" d={contourPath(piece.outline)} fill={frameColor}
          stroke={selected ? "var(--editor-selected)" : seamColor} stroke-width={selected ? 2 : 0.8}
          vector-effect="non-scaling-stroke" role="button" tabindex="0" aria-label={label}
          onclick={(e) => handleMemberClick(type, piece.index, e)}
          onkeydown={(e) => e.key === "Enter" && handleMemberClick(type, piece.index, e)}>
          <title>{label}</title>
        </path>
      {/each}
    </g>
  {:else if geometry.curvedFrame}
    <path class="curved-frame" d={contourPath(geometry.curvedFrame.outer) + " " + contourPath(geometry.curvedFrame.day)}
      fill-rule="evenodd" fill={frameColor} stroke={seamColor} stroke-width="1" />
  {:else if geometry.framePolygons?.length}
    {#each geometry.framePolygons as poly}
      <!-- Stroke maakt de naad tussen de leden zichtbaar: de 45°-lasnaad bij
           PVC/alu-verstek, de haakse stuiknaad bij hout — zonder stroke oogt
           het kozijn als één massieve rand ("geen echte verbindingen"). -->
      <polygon points={poly.map((p) => `${p[0]},${p[1]}`).join(" ")} fill={frameColor}
               stroke="var(--editor-frame)" stroke-width="1" />
    {/each}
  {:else}
    {#each geometry.frameRects as rect}
      <rect
        x={rect.x} y={rect.y}
        width={rect.width} height={rect.height}
        fill={frameColor}
      />
    {/each}
  {/if}

  <!-- Round/elliptical frame: donut drawn UNDER the cell layer (decoration only,
       no pointer events); the cell layer below is clipped to the inner opening -->
  {#if roundClip}
    <clipPath id={clipId}>
      {#if roundClip.kind === "circle"}
        <circle cx={roundClip.cx} cy={roundClip.cy} r={Math.max(0, roundClip.rInner)} />
      {:else}
        <ellipse cx={roundClip.cx} cy={roundClip.cy} rx={Math.max(0, roundClip.irx)} ry={Math.max(0, roundClip.iry)} />
      {/if}
    </clipPath>
    {#if roundClip.kind === "circle"}
      <circle cx={roundClip.cx} cy={roundClip.cy} r={roundClip.rOuter}
        fill={frameColor} stroke="var(--editor-frame)" stroke-width="1" pointer-events="none" />
      <circle cx={roundClip.cx} cy={roundClip.cy} r={roundClip.rInner}
        fill="var(--editor-glass)" stroke="var(--editor-frame)" stroke-width="1" pointer-events="none" />
    {:else}
      <ellipse cx={roundClip.cx} cy={roundClip.cy} rx={roundClip.rx} ry={roundClip.ry}
        fill={frameColor} stroke="var(--editor-frame)" stroke-width="1" pointer-events="none" />
      {#if roundClip.irx > 0 && roundClip.iry > 0}
        <ellipse cx={roundClip.cx} cy={roundClip.cy} rx={roundClip.irx} ry={roundClip.iry}
          fill="var(--editor-glass)" stroke="var(--editor-frame)" stroke-width="1" pointer-events="none" />
      {/if}
    {/if}
  {/if}

  {#if geometry.curvedDividerTimber}
    {#each geometry.curvedDividerTimber as piece}
      {@const selected = $selectedMember?.type === (piece.vertical ? "divider_v" : "divider_h") && $selectedMember.index === piece.sourceIndex && $selectedMember.pieceIndex === piece.pieceIndex}
      <path class="curved-divider" class:selected d={contourPath(piece.outline)} fill={frameColor}
        stroke={selected ? "var(--editor-selected)" : seamColor} stroke-width={selected ? 2 : 0.8}
        vector-effect="non-scaling-stroke" pointer-events="none" />
    {/each}
  {:else if geometry.curvedFrame}
    {#each geometry.curvedDividers as divider}
      <path class="curved-divider" d={contourPath(divider)} fill={frameColor} pointer-events="none" />
    {/each}
  {:else if !kozijn?.layout}
    {#each [...geometry.vDividers, ...geometry.hDividers] as rect}
      <rect {...rect} fill={frameColor} pointer-events="none" />
    {/each}
  {/if}

  <g clip-path={roundClip || geometry.curvedFrame ? `url(#${clipId})` : null}>
  {#if kozijn?.layout && layoutGeom}
    <!-- Free-subdivision layout (editable: click a vak, drag the dividers) -->
    {#if !geometry.curvedFrame}
      {#each layoutGeom.dividers as d}
        <rect {...d.rect} fill={frameColor} pointer-events="none" />
      {/each}
    {/if}
    {#each layoutGeom.leaves as l, leafIndex}
      {#if l.node.vulling.type !== "buiten"}
        {@const source = fittedCells.get(leafIndex)}
        {@const shape = source?.openingContour}
        {@const r = source?.openingBounds ?? l.rect}
        {@const v = l.node.vulling}
        {@const lcx = r.x + r.width / 2}
        {@const lcy = r.y + r.height / 2}
        {@const fits = layoutFits(l, leafIndex)}
        {@const hitPath = cellHitPath(shape, r, fits)}
        <path d={openingPath(shape, r)} fill={vullingFill(v.type)} stroke="var(--editor-frame)" stroke-width={1} pointer-events="none" />
        {@render sashSeat(fits[0], shape)}
        {#each fits as fit}
          {@render glazingFace(fit, "var(--editor-glass)", leafIndex)}
          {@render sealElevation(fit, source, r, fit.contours?.sashOpensOutward ?? source?.hardwareLeaves?.[0]?.opensOutward ?? v.opensOutward)}
        {/each}
        <g clip-path={shape ? `url(#${clipId}-cell-${leafIndex})` : null}>
        {#each vullingSymbols(v, r) as sym}
          <path d={sym.d} fill="none" stroke="var(--editor-frame)" stroke-width={1.2}
                clip-path={fits[0]?.contours?.sashOuter ? `url(#${clipId}-glass-${leafIndex})` : null}
                stroke-dasharray={sym.solid ? null : "7 5"} opacity="0.8" pointer-events="none" />
        {/each}
        <text x={lcx} y={lcy - 7 / zoom} text-anchor="middle" dominant-baseline="central" fill="var(--text-secondary)" font-size={12 / zoom} font-weight="700" opacity="0.55" pointer-events="none">{vullingLabel(v)}</text>
        <text x={lcx} y={lcy + 9 / zoom} text-anchor="middle" dominant-baseline="central" fill="#DC2626" font-size={8 / zoom} opacity="0.7" pointer-events="none">{openingLabel(r, shape)}</text>
        </g>
        <!-- transparent hit layer for selecting the vak -->
        <path d={hitPath} fill="transparent" class="vak-hit"
              onclick={(e) => selectLeaf(l.node.id, e)} role="button" tabindex="0"
              aria-label="Selecteer vak" onkeydown={(e) => e.key === "Enter" && selectLeaf(l.node.id, e)} />
        {#if l.node.id === selectedLeafId}
          <path d={shape ? hitPath : openingPath(null, insetOpening(r, 2))}
                fill="none" stroke="var(--editor-selected, #D97706)" stroke-width={3 / zoom} pointer-events="none" />
        {/if}
      {:else}
        <!-- Borstwering: het lage muurtje onder een melkmeisje-zijlicht. Geen
             kozijnvak maar metselwerk, doorgetrokken tot de buitenrand waar de
             getrapte omtrek geen hout meer heeft — muur i.p.v. een gat. -->
        {@const r = buitenRect(l.rect)}
        <rect x={r.x} y={r.y} width={r.width} height={r.height}
              fill="url(#{brickPatternId})" stroke="var(--editor-frame)" stroke-width={1} pointer-events="none" />
        <text x={r.x + r.width / 2} y={r.y + r.height / 2} text-anchor="middle" dominant-baseline="central"
              fill="#F5EFE3" font-size={11 / zoom} font-weight="700" opacity="0.85" pointer-events="none">Borstwering</text>
        <!-- Ook een borstwering blijft selecteerbaar, anders is de keuze
             'buiten' onomkeerbaar (vullingpaneel onbereikbaar). -->
        <rect x={r.x} y={r.y} width={r.width} height={r.height} fill="transparent" class="vak-hit"
              onclick={(e) => selectLeaf(l.node.id, e)} role="button" tabindex="0"
              aria-label="Selecteer borstwering" onkeydown={(e) => e.key === "Enter" && selectLeaf(l.node.id, e)} />
        {#if l.node.id === selectedLeafId}
          <rect x={r.x + 2} y={r.y + 2} width={Math.max(1, r.width - 4)} height={Math.max(1, r.height - 4)}
                fill="none" stroke="var(--editor-selected, #D97706)" stroke-width={3 / zoom} pointer-events="none" />
        {/if}
      {/if}
    {/each}

    <!-- divider drag handles (tussenstijl/-dorpel) -->
    <g clip-path={geometry.curvedFrame ? `url(#${geometry.curvedDividerTimber && dividerClipId ? dividerClipId : clipId + "-day"})` : null}>
    {#if geometry.curvedDividerTimber}
      {@const vertical = layoutGeom.dividers.filter(d => d.direction === "v")}
      {@const horizontal = layoutGeom.dividers.filter(d => d.direction === "h")}
      {#each geometry.curvedDividerTimber as piece}
        {@const d = (piece.vertical ? vertical : horizontal)[piece.sourceIndex]}
        {@const label = dividerPieceLabel($_, piece, geometry.curvedDividerTimber)}
        <path d={contourPath(piece.outline)} fill="transparent" class="layout-divider divider-piece-hit" class:v={piece.vertical} class:h={!piece.vertical}
          onmousedown={(e) => startLayoutDrag(d, e, piece)} role="button" aria-label={label} tabindex="0"
          onkeydown={(e) => e.key === "Enter" && selectDividerPiece(piece)} />
        {#if layoutDrag?.moved && d && layoutDrag.splitId === d.splitId && layoutDrag.childIndex === d.childIndex}
          <rect {...d.rect} fill="none" stroke="var(--editor-selected)" pointer-events="none" />
        {/if}
      {/each}
    {:else}
    {#each layoutGeom.dividers as d, i (i)}
      <rect {...d.rect}
            stroke={geometry.curvedFrame && layoutDrag?.moved && d.splitId === layoutDrag.splitId && d.childIndex === layoutDrag.childIndex ? "var(--editor-selected)" : "none"}
            fill="transparent" class="layout-divider {d.direction}"
            onmousedown={(e) => startLayoutDrag(d, e)}
            role="separator" aria-label="Versleep deellijn" tabindex="-1" />
    {/each}
    {/if}
    </g>

    <!-- in-canvas split / merge controls for the selected vak -->
    {#if selectedLeafRect}
      {@const bs = 30 / zoom}
      {@const gp = 5 / zoom}
      {@const tw = bs * 4 + gp * 3}
      {@const bx = selectedLeafRect.x + selectedLeafRect.width / 2 - tw / 2}
      {@const by = geometry.curvedFrame ? selectedLeafRect.y + selectedLeafRect.height / 2 - bs / 2 : selectedLeafRect.y + 7 / zoom}
      {#each [{ k: "row", t: "⟷", title: "Splits met tussenstijl" }, { k: "column", t: "↕", title: "Splits met tussendorpel" }, { k: "merge", t: "⤬", title: "Samenvoegen" }, { k: "swap", t: "⇄", title: "Verwissel met ander vak" }] as b, bi}
        {@const x0 = bx + bi * (bs + gp)}
        {@const armed = b.k === "swap" && swapSourceId === selectedLeafId}
        <g class="lay-btn" role="button" tabindex="0" aria-label={b.title}
           onclick={(e) => b.k === "merge" ? doMergeLayout(e) : b.k === "swap" ? doSwapLayout(e) : doSplitLayout(b.k, e)}
           onkeydown={(e) => e.key === "Enter" && (b.k === "merge" ? doMergeLayout(e) : b.k === "swap" ? doSwapLayout(e) : doSplitLayout(b.k, e))}>
          <rect x={x0} y={by} width={bs} height={bs} rx={5 / zoom}
                fill={armed ? "var(--amber, #D97706)" : "var(--bg-surface-alt, #2a2a38)"} stroke="var(--amber, #D97706)" stroke-width={1.4 / zoom} />
          <text x={x0 + bs / 2} y={by + bs / 2} text-anchor="middle" dominant-baseline="central"
                font-size={17 / zoom} fill={armed ? "#1a1a1a" : "var(--amber, #D97706)"} pointer-events="none">{b.t}</text>
        </g>
      {/each}
      {#if swapSourceId === selectedLeafId}
        <text x={selectedLeafRect.x + selectedLeafRect.width / 2} y={by + bs + 14 / zoom}
              text-anchor="middle" dominant-baseline="central" fill="var(--amber, #D97706)"
              font-size={11 / zoom} font-weight="600" pointer-events="none">→ klik het vak om mee te wisselen</text>
      {/if}
    {/if}
  {/if}

  {#if !kozijn?.layout}
  <!-- Cell fills (glazing/panel areas) -->
  {#each geometry.cellRects as cellRect}
    {@const r = cellRect.openingBounds ?? cellRect.rect}
    {@const cell = kozijn.cells?.[cellRect.cellIndex]}
    {@const fits = cellFits(cellRect, cell, kozijn.frame, geometry)}
    {@const colors = cellColor(cellRect.cellIndex)}
    {@const isSelected = $selectedCellIndex === cellRect.cellIndex}
    {@render sashSeat(fits[0], cellRect.openingContour)}
    {#each fits as fit}
      {@render glazingFace(fit, cell?.panelFilling ? "var(--editor-panel)" : "var(--editor-glass)", cellRect.cellIndex)}
      {@render sealElevation(fit, cellRect, r, fit.contours?.sashOpensOutward ?? cellRect.hardwareLeaves?.[0]?.opensOutward ?? cell?.openingDirection === "outward")}
    {/each}
    <path
      d={cellHitPath(cellRect.openingContour, r, fits)}
      fill={fits.length ? "transparent" : colors.fill}
      stroke={isSelected ? "var(--editor-selected)" : fits.length ? "none" : colors.stroke}
      stroke-width={isSelected ? 3 : 1}
      class="cell"
      onclick={(e) => handleCellClick(cellRect.cellIndex, e)}
      oncontextmenu={(e) => handleCellRightClick(cellRect.cellIndex, e)}
      role="button"
      tabindex="0"
    />

    {@const hw = hardwarePositions(cellRect.cellIndex, r)}
    {@const cellFs = 10 / zoom}
    {@const smallFs = 8 / zoom}
    {@const cx = r.x + r.width / 2}
    {@const cy = r.y + r.height / 2}
    <g clip-path={cellRect.openingContour ? `url(#${clipId}-cell-${cellRect.cellIndex})` : null}>
    <!-- EN 12519 opening symbols -->
    {#each openingSymbols(cellRect.cellIndex, fits[0]?.day ?? r) as sym}
      <path
        d={sym.d}
        clip-path={fits[0]?.contours?.sashOuter ? `url(#${clipId}-glass-${cellRect.cellIndex})` : null}
        fill="none"
        stroke={colors.stroke}
        stroke-width={sym.label === "tilt" ? 0.8 : 1}
        stroke-dasharray={sym.label === "tilt" ? "3 4" : "4 3"}
        opacity={sym.label === "tilt" ? 0.5 : 0.7}
        pointer-events="none"
      />
    {/each}

    <!-- Hardware indicators (hinges + handle) -->
    {#each hw.hinges as hinge}
      <circle
        cx={hinge.cx} cy={hinge.cy}
        r={3 / zoom}
        fill="var(--editor-frame)"
        stroke={colors.stroke}
        stroke-width={0.5 / zoom}
        pointer-events="none"
      />
    {/each}
    {#if hw.handle}
      <line
        x1={hw.handle.x} y1={hw.handle.y - 8 / zoom}
        x2={hw.handle.x} y2={hw.handle.y + 8 / zoom}
        stroke={colors.stroke}
        stroke-width={1.5 / zoom}
        pointer-events="none"
      />
    {/if}

    <!-- Cell label (GA Kozijn style: type + vakmaat + glasspec) -->
    <!-- Type label (DK, VG, etc) -->
    <text
      x={cx} y={cy - cellFs * 1.5}
      text-anchor="middle" dominant-baseline="central"
      fill="var(--text-secondary)"
      font-size={cellFs * 1.4}
      font-family="var(--font-heading)"
      font-weight="700"
      opacity="0.5"
      pointer-events="none"
    >
      {cellLabel(cellRect.cellIndex)}
    </text>
    <!-- Vakmaat -->
    <text
      x={cx} y={cy}
      text-anchor="middle" dominant-baseline="central"
      fill="#DC2626"
      font-size={smallFs}
      font-family="var(--font-body)"
      font-weight="600"
      opacity="0.7"
      pointer-events="none"
    >
      {openingLabel(r, cellRect.openingContour)}
    </text>
    <!-- Glass spec -->
    {#if cell?.glazing}
      <text
        x={cx} y={cy + cellFs * 1.2}
        text-anchor="middle" dominant-baseline="central"
        fill="#16A34A"
        font-size={smallFs * 0.9}
        font-family="var(--font-body)"
        font-weight="500"
        opacity="0.6"
        pointer-events="none"
      >
        {cell.glazing.glassType} {cell.glazing.thicknessMm}mm
      </text>
    {/if}
    </g>
  {/each}
  {/if}
  </g>

  {#if !layoutDrag?.moved}
    {#each geometry.cellRects as cr}
      {#each cr.hardwareLeaves ?? [] as leaf}
        {#each leaf.components ?? [] as part}
          {@const drawing = fittingDrawing(part,leaf.set)}
          {#if drawing && drawing.face !== "rebate" && (drawing.face !== "opening" || !leaf.opensOutward)}
            <g class="hardware-fitting" transform={`translate(${part.position.join(" ")}) rotate(${Math.atan2(part.tangent[1],part.tangent[0])*180/Math.PI})`}
               fill="#aab0b4" stroke="#3e4446" stroke-width="1" pointer-events="none">
              {#each drawing.boxes as box}
                <rect x={box[0]-box[3]/2} y={box[1]-box[4]/2} width={box[3]} height={box[4]} rx={box[6]==="disc" ? box[4]/2 : 2}/>
              {/each}
            </g>
          {/if}
        {/each}
      {/each}
    {/each}
  {/if}

  <!-- Transparent member hit areas preserve the visible bead and sash overlap. -->
  {#if !geometry.curvedTimber?.length && !geometry.framePolygons?.length}
  {#each geometry.frameRects as rect, i}
    {@const memberType = FRAME_MEMBER_NAMES[i]}
    {@const isSelected = $selectedMember?.type === memberType}
    {#if rect.width > 0 && rect.height > 0}
      <rect
        x={rect.x} y={rect.y}
        width={rect.width} height={rect.height}
        fill="transparent"
        stroke={isSelected ? "var(--editor-selected)" : "none"}
        stroke-width={isSelected ? 3 : 0}
        class="member"
        onclick={(e) => handleMemberClick(memberType, i, e)}
        role="button"
        tabindex="0"
      />
    {/if}
  {/each}
  {/if}

  <!-- Corner joint indicators (color-coded by type) -->
  {#if !geometry.curvedFrame && kozijn.frame?.cornerJoints?.length > 0}
    {#each kozijn.frame.cornerJoints as joint, ji}
      {#if ji < 4}
        {@const jfw2 = kozijn.frame.frameWidth}
        {@const jow2 = geometry.outerRect?.width || 0}
        {@const joh2 = geometry.outerRect?.height || 0}
        {@const cpx = ji % 2 === 0 ? jfw2 / 2 : jow2 - jfw2 / 2}
        {@const cpy = ji < 2 ? jfw2 / 2 : joh2 - jfw2 / 2}
        {@const jc = joint.jointType === "pen_slis" ? "#22c55e" : joint.jointType === "verstek" ? "#3b82f6" : joint.jointType === "contramal" ? "#f59e0b" : "#ef4444"}
        <circle
          cx={cpx} cy={cpy}
          r={5 / zoom}
          fill={jc}
          opacity="0.7"
          pointer-events="none"
        />
        {#if joint.jointType === "verstek"}
          {@const mLen = jfw2 * 0.6}
          {@const dx = ji % 2 === 0 ? 1 : -1}
          {@const dy = ji < 2 ? 1 : -1}
          <line
            x1={cpx - dx * mLen / 2} y1={cpy - dy * mLen / 2}
            x2={cpx + dx * mLen / 2} y2={cpy + dy * mLen / 2}
            stroke={jc} stroke-width={1.5 / zoom}
            opacity="0.6" pointer-events="none"
          />
        {/if}
      {/if}
    {/each}
  {/if}

  <!-- Trapezoid outline -->
  {#if !geometry.curvedFrame && geometry.trapezoidOuter && geometry.trapezoidOuter.length >= 3}
    <polygon
      points={geometry.trapezoidOuter.map(p => `${p[0]},${p[1]}`).join(' ')}
      fill="none"
      stroke="var(--editor-frame)"
      stroke-width={kozijn.frame.frameWidth}
      stroke-linejoin="miter"
    />
  {/if}

  <!-- Arcs for arched/round kozijnen -->
  {#each (geometry.curvedFrame ? [] : geometry.arcs || []) as arc}
    {@const r = arc.radius}
    {@const startRad = arc.startAngle * Math.PI / 180}
    {@const endRad = arc.endAngle * Math.PI / 180}
    {@const x1 = arc.cx + r * Math.cos(startRad)}
    {@const y1 = arc.cy - r * Math.sin(startRad)}
    {@const x2 = arc.cx + r * Math.cos(endRad)}
    {@const y2 = arc.cy - r * Math.sin(endRad)}
    {@const largeArc = (arc.endAngle - arc.startAngle) > 180 ? 1 : 0}
    <path
      d="M {x1} {y1} A {r} {r} 0 {largeArc} 1 {x2} {y2}"
      fill="none"
      stroke="var(--editor-frame)"
      stroke-width={arc.strokeWidth}
    />
  {/each}

  {#if !kozijn?.layout && !geometry.curvedDividerTimber}
  <!-- Vertical dividers -->
  {#each geometry.vDividers as rect, i}
    {@const isSelected = $selectedMember?.type === "divider_v" && $selectedMember?.index === i}
    <path
      d={openingPath(geometry.curvedDividers?.[i], rect)}
      fill="transparent"
      stroke={isSelected ? "var(--editor-selected)" : "none"}
      stroke-width={isSelected ? 3 : 0}
      class="member"
      onclick={(e) => handleMemberClick("divider_v", i, e)}
      role="button"
      tabindex="0"
    />
  {/each}

  <!-- Horizontal dividers -->
  {#each geometry.hDividers as rect, i}
    {@const isSelected = $selectedMember?.type === "divider_h" && $selectedMember?.index === i}
    <path
      d={openingPath(geometry.curvedDividers?.[geometry.vDividers.length + i], rect)}
      fill="transparent"
      stroke={isSelected ? "var(--editor-selected)" : "none"}
      stroke-width={isSelected ? 3 : 0}
      class="member"
      onclick={(e) => handleMemberClick("divider_h", i, e)}
      role="button"
      tabindex="0"
    />
  {/each}
  {/if}

  {#if $selectedTimberJoints.length}
    <g class="divider-joints" pointer-events="none" aria-hidden="true">
      {#each $selectedTimberJoints as joint}
        {@const paths = jointPaths(joint)}
        {#if joint.shoulderFace}
          <path class="arch-shoulder" d={`M${joint.shoulderFace[0]} L${joint.shoulderFace[1]}`}
            fill="none" stroke="var(--editor-selected)" stroke-width="1.5" vector-effect="non-scaling-stroke" />
        {/if}
        {#if paths}
          <path class="tenon-footprint" d={paths.tenon} fill="var(--editor-selected, #D97706)" fill-opacity="0.25"
            stroke={seamColor} stroke-width="1" vector-effect="non-scaling-stroke" />
          <path class="mortise-bottom" d={paths.socket} fill="none" stroke={seamColor}
            stroke-width="1.25" stroke-dasharray="3 2" vector-effect="non-scaling-stroke" />
        {/if}
      {/each}
    </g>
  {/if}

  <!-- Profile codes on frame members (GA Kozijn style - green text) —
       skipped for zero-size members (round/elliptical: all four; arched: top),
       and for polygon frames (verstek/melkmeisje) whose members don't map to
       the 4 rect slots. -->
  {#if !geometry.framePolygons?.length && kozijn.frame.shape?.shapeType !== "arched_trapezoid"}
  {#each geometry.frameRects as rect, i}
    {@const memberName = FRAME_MEMBER_NAMES[i]}
    {@const isVertical = memberName === "frame_left" || memberName === "frame_right"}
    {@const profileLabel = `${Math.round(kozijn.frame.frameWidth)}×${Math.round(kozijn.frame.frameDepth)}`}
    {@const labelFs = 10 / zoom}
    {#if rect.width > 0 && rect.height > 0}
    <text
      x={isVertical ? rect.x + rect.width / 2 : rect.x + rect.width / 2}
      y={isVertical ? rect.y + rect.height / 2 : rect.y + rect.height / 2}
      text-anchor="middle"
      dominant-baseline="central"
      fill="#16A34A"
      font-size={labelFs}
      font-family="var(--font-body)"
      font-weight="600"
      opacity="0.7"
      pointer-events="none"
      transform={isVertical ? `rotate(-90, ${rect.x + rect.width / 2}, ${rect.y + rect.height / 2})` : ""}
    >
      {profileLabel}
    </text>
    {/if}
  {/each}
  {/if}

  <!-- Four straight strips follow the rectangular outside faces. A shaped
       frame's bounding box is not its wall connection contour. -->
  {#if (kozijn.frame.shape?.shapeType ?? "rectangular") === "rectangular" && kozijn.frame.edges?.length}
    <g class="spouwlatten" fill="none" stroke="#8b5cf6" stroke-width={1 / zoom}
      stroke-dasharray="{3 / zoom} {2 / zoom}" opacity="0.5" pointer-events="none">
      {#each battenElevationPaths(geometry.outerRect, kozijn.frame.edges) as batten (batten.side)}
        <path data-side={batten.side} d={batten.path} />
      {/each}
    </g>
  {/if}

  <!-- Frame extensions (free-form members beyond the grid) -->
  {#each (kozijn.extensions || []) as ext, i}
    {@const dx = ext.endX - ext.startX}
    {@const dy = ext.endY - ext.startY}
    {@const len = Math.sqrt(dx * dx + dy * dy)}
    {@const hw = (ext.memberWidth || 67) / 2}
    {#if len > 0}
      {@const nx = -dy / len * hw}
      {@const ny = dx / len * hw}
      <polygon
        points="{ext.startX + nx},{ext.startY + ny} {ext.endX + nx},{ext.endY + ny} {ext.endX - nx},{ext.endY - ny} {ext.startX - nx},{ext.startY - ny}"
        fill="var(--editor-frame)"
        fill-opacity="0.4"
        stroke="var(--amber)"
        stroke-width={1.5 / zoom}
        stroke-dasharray="{4 / zoom} {3 / zoom}"
        pointer-events="none"
      />
      <text
        x={(ext.startX + ext.endX) / 2}
        y={(ext.startY + ext.endY) / 2}
        text-anchor="middle" dominant-baseline="central"
        fill="var(--amber)" font-size={9 / zoom} opacity="0.7"
        pointer-events="none"
      >
        {ext.extensionType || "ext"}
      </text>
    {/if}
  {/each}

  {#each geometry.curvedDividerTimber ?? [] as piece}
    {#if $selectedMember?.type === (piece.vertical ? "divider_v" : "divider_h") && $selectedMember.index === piece.sourceIndex && $selectedMember.pieceIndex === piece.pieceIndex}
      {#each ["start", "end"] as side}
        {@const path = sawAllowancePath(piece, side)}
        {#if path}
          <path d={path} class="saw-allowance" fill="none" stroke="var(--editor-selected)" stroke-width="1.5"
            stroke-dasharray="4 3" vector-effect="non-scaling-stroke" pointer-events="none" />
        {/if}
      {/each}
    {/if}
  {/each}

  <!-- Dimension lines — positions, labels and editability come from dimsView
       (levels re-projected screen-space-aware, deterministic label lanes) -->
  {#each dimsView as entry (entry.key)}
    {@const fontSize = 12 / zoom}
    {@const sw = 1 / zoom}
    {@const tick = 6 / zoom}
    {@const bgPad = 3 / zoom}
    {@const labelW = entry.labelW}
    {@const labelH = entry.labelH}
    {@const dimTitle = entry.editable
      ? "Klik om de maat te bewerken"
      : entry.kind === "stijl" || entry.kind === "tussenstijl"
        ? "Profielmaat — volgt de profielkeuze"
        : "Niet direct bewerkbaar — versleep de deellijn in de tekening"}

    <!-- Dimension line -->
    <line x1={entry.x1} y1={entry.y1} x2={entry.x2} y2={entry.y2}
      stroke="var(--editor-dimension)" stroke-width={sw} />

    <!-- Tick marks -->
    {#if entry.isH}
      <line x1={entry.x1} y1={entry.y1 - tick} x2={entry.x1} y2={entry.y1 + tick} stroke="var(--editor-dimension)" stroke-width={sw}/>
      <line x1={entry.x2} y1={entry.y2 - tick} x2={entry.x2} y2={entry.y2 + tick} stroke="var(--editor-dimension)" stroke-width={sw}/>
    {:else}
      <line x1={entry.x1 - tick} y1={entry.y1} x2={entry.x1 + tick} y2={entry.y1} stroke="var(--editor-dimension)" stroke-width={sw}/>
      <line x1={entry.x2 - tick} y1={entry.y2} x2={entry.x2 + tick} y2={entry.y2} stroke="var(--editor-dimension)" stroke-width={sw}/>
    {/if}

    {#if editingDim && editingDim.key === entry.key}
      <!-- Inline edit input -->
      <foreignObject
        x={entry.isH ? entry.midX - 50 / zoom : entry.midX + bgPad}
        y={entry.midY - labelH * 0.7}
        width={100 / zoom}
        height={labelH * 1.4}
      >
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:this={editInputEl}
          type="number"
          value={editingDim.value}
          onkeydown={handleDimKeyDown}
          onblur={() => { editingDim = null; }}
          style="
            width: 100%;
            height: 100%;
            background: var(--amber, #D97706);
            color: #fff;
            border: none;
            border-radius: 4px;
            text-align: center;
            font-size: {fontSize * 0.8}px;
            font-weight: 700;
            font-family: var(--font-body);
            outline: none;
            padding: 2px 4px;
          "
        />
      </foreignObject>
    {:else}
      {#if entry.isH}
        <!-- Horizontal: label on/above/below the line (deterministic lane) -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <rect
          x={entry.midX - labelW / 2} y={entry.midY + entry.laneOff - labelH / 2}
          width={labelW} height={labelH}
          fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
          style="cursor: {entry.editable ? 'pointer' : 'default'}"
          onclick={(e) => handleDimClick(entry, e)}
        >
          <title>{dimTitle}</title>
        </rect>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <text
          x={entry.midX} y={entry.midY + entry.laneOff}
          text-anchor="middle" dominant-baseline="central"
          fill="var(--editor-dimension)" font-size={fontSize}
          font-family="var(--font-body)" font-weight="600"
          style="cursor: {entry.editable ? 'pointer' : 'default'}"
          onclick={(e) => handleDimClick(entry, e)}
        >{entry.text}</text>
      {:else}
        <!-- Vertical: label rotated 90° along the line; laneOff is the local
             perpendicular offset (negative = toward the kozijn) -->
        <g transform="translate({entry.midX}, {entry.midY}) rotate(-90)">
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <rect
            x={-labelW / 2} y={entry.laneOff - labelH / 2}
            width={labelW} height={labelH}
            fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
            style="cursor: {entry.editable ? 'pointer' : 'default'}"
            onclick={(e) => handleDimClick(entry, e)}
          >
            <title>{dimTitle}</title>
          </rect>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <text
            x={0} y={entry.laneOff}
            text-anchor="middle" dominant-baseline="central"
            fill="var(--editor-dimension)" font-size={fontSize}
            font-family="var(--font-body)" font-weight="600"
            style="cursor: {entry.editable ? 'pointer' : 'default'}"
            onclick={(e) => handleDimClick(entry, e)}
          >{entry.text}</text>
        </g>
      {/if}
    {/if}
  {/each}
</g>

<style>
  .glazing-face {
    stroke-linejoin: round;
  }

  .glazing-face rect, .glazing-face path {
    vector-effect: non-scaling-stroke;
    stroke-width: 0.75px;
  }

  .glazing-face .sash-outline {
    stroke-width: 1.1px;
  }

  .cell {
    cursor: default;
    transition: stroke 0.15s, stroke-width 0.15s;
  }

  .cell:hover {
    stroke: var(--amber);
    stroke-width: 2;
  }

  .member {
    cursor: default;
    transition: stroke 0.15s, stroke-width 0.15s;
  }

  .member:hover {
    stroke: var(--amber);
    stroke-width: 2;
  }

  /* Free-subdivision layout editing */
  .vak-hit { cursor: pointer; }
  .layout-divider.v { cursor: col-resize; }
  .layout-divider.h { cursor: row-resize; }
  .layout-divider:hover { fill: rgba(217, 119, 6, 0.18) !important; }
  .lay-btn { cursor: pointer; }
  .lay-btn:hover rect { fill: var(--amber); }
  .lay-btn:hover text { fill: #1a1a1a; }
</style>
