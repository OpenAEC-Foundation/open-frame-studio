<script>
  import { selectedCellIndex, selectedMember, updateCellType, updateDimensions, updateGridSizes, currentKozijn, setKozijnLayout } from "../../stores/kozijn.js";
  import { get } from "svelte/store";
  import { layoutToRects, vullingLabel, findNode, setSplitChildSizes, splitLeaf, mergeAt, ensureIds } from "../../lib/layout.js";

  let { geometry, kozijn, zoom = 0.35, oncellcontextmenu } = $props();

  // ── Free-subdivision layout: editable render of the split tree ──────────────
  // While a divider is being dragged we render a local working copy so the drag
  // is smooth and history/backend are only touched once, on release.
  let workingTree = $state(null);
  let layoutDrag = $state(null);
  let selectedLeafId = $state(null);
  // ensureIds is defensive: old .ofs files or a stale wasm bundle may hand us a
  // tree without node ids, while all canvas editing is id-based. It is memoized
  // so the same input yields the same object (stable selection).
  let activeTree = $derived(workingTree ?? ensureIds(kozijn?.layout ?? null));

  let layoutGeom = $derived.by(() => {
    const k = kozijn;
    if (!activeTree || !k?.frame) return null;
    const fw = k.frame.frameWidth || 67;
    const inner = { x: fw, y: fw, width: (k.frame.outerWidth || 0) - 2 * fw, height: (k.frame.outerHeight || 0) - 2 * fw };
    return layoutToRects(activeTree, inner, fw);
  });
  let selectedLeafRect = $derived(layoutGeom?.leaves.find((l) => l.node.id === selectedLeafId)?.rect || null);

  function vullingFill(t) {
    return t === "deur" ? "#b98b5e" : t === "paneel" ? "#c9c2b2" : t === "rooster" ? "#9aa0a8" : "var(--editor-glass)";
  }

  // ── Drag a layout divider (tussenstijl/-dorpel) to resize the two vakken ────
  // Mirrors GridHandles' screen→mm mapping (deltaMm = deltaPx / zoom); the vak
  // sizes are relative within their split, so a mm delta maps to a size delta
  // via the divider's avail/sizeSum (carried by layoutToRects).
  function startLayoutDrag(d, e) {
    e.stopPropagation();
    e.preventDefault();
    const base = ensureIds(kozijn?.layout ?? null);
    const split = findNode(base, d.splitId);
    if (!split || split.kind !== "split") return;
    layoutDrag = {
      base, moved: false,
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
    if (deltaPx !== 0) layoutDrag.moved = true;
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
    workingTree = null;
    layoutDrag = null;
  }

  function selectLeaf(id, e) {
    e?.stopPropagation();
    selectedLeafId = id;
    selectedCellIndex.set(null);
    selectedMember.set(null);
  }
  function doSplitLayout(dir, e) {
    e?.stopPropagation();
    if (selectedLeafId && activeTree) setKozijnLayout(splitLeaf(activeTree, selectedLeafId, dir));
  }
  function doMergeLayout(e) {
    e?.stopPropagation();
    if (selectedLeafId && activeTree) setKozijnLayout(mergeAt(activeTree, selectedLeafId));
  }

  // Inline dimension editing state
  let editingDim = $state(null); // { index, value, x, y, isH, width, height }
  let editInputEl = $state(null);

  function handleDimClick(dimIndex, e) {
    e.stopPropagation();
    const dim = geometry.dimensions[dimIndex];
    const isH = dim.side === "bottom" || dim.side === "top";
    const midX = (dim.x1 + dim.x2) / 2;
    const midY = (dim.y1 + dim.y2) / 2;
    const val = Math.round(Number(dim.label));

    editingDim = { index: dimIndex, value: val, x: midX, y: midY, isH };

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
    const newVal = parseFloat(editInputEl?.value);
    if (!newVal || newVal <= 0) { editingDim = null; return; }

    const dim = geometry.dimensions[editingDim.index];
    const k = get(currentKozijn);
    if (!k) { editingDim = null; return; }

    const ow = k.frame.outerWidth;
    const oh = k.frame.outerHeight;
    const fw = k.frame.frameWidth;

    // Determine what this dimension controls based on position
    const dimOffset = 25;
    const y1 = dim.y1;
    const x1 = dim.x1;

    if (dim.side === "bottom") {
      // Level 1: overall width
      if (Math.abs(y1 - (oh + dimOffset)) < 5) {
        updateDimensions(newVal, oh);
      }
      // Level 2: dagmaat — compute outer from inner
      else if (Math.abs(y1 - (oh + dimOffset * 2)) < 5) {
        updateDimensions(newVal + 2 * fw, oh);
      }
      // Level 3: column sizes or frame width
      else if (Math.abs(y1 - (oh + dimOffset * 3)) < 5) {
        // Check if it's a frame width or column size
        if (Math.abs(dim.x1) < 1 || Math.abs(dim.x2 - ow) < 1) {
          // Frame width — skip (profile-dependent)
        } else {
          // Column size — find which column
          const cols = [...k.grid.columns];
          let cx = fw;
          for (let i = 0; i < cols.length; i++) {
            if (Math.abs(dim.x1 - cx) < 2) {
              const diff = newVal - cols[i].size;
              cols[i] = { ...cols[i], size: newVal };
              // Adjust adjacent column to compensate
              if (i + 1 < cols.length) {
                cols[i + 1] = { ...cols[i + 1], size: Math.max(100, cols[i + 1].size - diff) };
              }
              updateGridSizes(cols.map(c => c.size), k.grid.rows.map(r => r.size));
              break;
            }
            cx += cols[i].size;
            if (cols[i].dividerProfile || i < cols.length - 1) cx += fw;
          }
        }
      }
    } else if (dim.side === "right") {
      // Level 1: overall height
      if (Math.abs(x1 - (ow + dimOffset)) < 5) {
        updateDimensions(ow, newVal);
      }
      // Level 2: dagmaat hoogte
      else if (Math.abs(x1 - (ow + dimOffset * 2)) < 5) {
        updateDimensions(ow, newVal + 2 * fw);
      }
      // Level 3: row sizes
      else if (Math.abs(x1 - (ow + dimOffset * 3)) < 5) {
        const rows = [...k.grid.rows];
        let cy = fw;
        for (let i = 0; i < rows.length; i++) {
          if (Math.abs(dim.y1 - cy) < 2) {
            const diff = newVal - rows[i].size;
            rows[i] = { ...rows[i], size: newVal };
            if (i + 1 < rows.length) {
              rows[i + 1] = { ...rows[i + 1], size: Math.max(100, rows[i + 1].size - diff) };
            }
            updateGridSizes(k.grid.columns.map(c => c.size), rows.map(r => r.size));
            break;
          }
          cy += rows[i].size;
          if (i < rows.length - 1) cy += fw;
        }
      }
    }

    editingDim = null;
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
    if (!cell) return { hinges: [], handle: null };
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
    e.stopPropagation();
    selectedMember.set(null);
    selectedCellIndex.set(cellIndex);
  }
</script>

<g>
  <!-- Frame members background (drawn first, behind cells) -->
  {#each geometry.frameRects as rect}
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
    />
  {/each}

  {#if kozijn?.layout && layoutGeom}
    <!-- Free-subdivision layout (editable: click a vak, drag the dividers) -->
    {#each layoutGeom.dividers as d}
      <rect x={d.rect.x} y={d.rect.y} width={d.rect.width} height={d.rect.height} fill="var(--editor-frame)" pointer-events="none" />
    {/each}
    {#each layoutGeom.leaves as l}
      {#if l.node.vulling.type !== "buiten"}
        {@const r = l.rect}
        {@const v = l.node.vulling}
        {@const lcx = r.x + r.width / 2}
        {@const lcy = r.y + r.height / 2}
        {@const sb = (kozijn.frame.frameWidth || 67) * 0.4}
        <rect x={r.x} y={r.y} width={r.width} height={r.height} fill={vullingFill(v.type)} stroke="var(--editor-frame)" stroke-width={1} pointer-events="none" />
        {#if v.type === "raam" || v.type === "deur"}
          <rect x={r.x + sb} y={r.y + sb} width={Math.max(1, r.width - sb * 2)} height={Math.max(1, r.height - sb * 2)} fill="none" stroke="var(--amber)" stroke-width={1.2} opacity="0.7" pointer-events="none" />
        {/if}
        {#if v.glaslat}
          {@const gi = sb + 6}
          <rect x={r.x + gi} y={r.y + gi} width={Math.max(1, r.width - gi * 2)} height={Math.max(1, r.height - gi * 2)} fill="none" stroke={v.glaslat === "buiten" ? "#3B82F6" : "var(--amber)"} stroke-width={0.8} opacity="0.6" stroke-dasharray="4 3" pointer-events="none" />
        {/if}
        <text x={lcx} y={lcy - 7 / zoom} text-anchor="middle" dominant-baseline="central" fill="var(--text-secondary)" font-size={12 / zoom} font-weight="700" opacity="0.55" pointer-events="none">{vullingLabel(v)}</text>
        <text x={lcx} y={lcy + 9 / zoom} text-anchor="middle" dominant-baseline="central" fill="#DC2626" font-size={8 / zoom} opacity="0.7" pointer-events="none">{Math.round(r.width)}×{Math.round(r.height)}</text>
        <!-- transparent hit layer for selecting the vak -->
        <rect x={r.x} y={r.y} width={r.width} height={r.height} fill="transparent" class="vak-hit"
              onclick={(e) => selectLeaf(l.node.id, e)} role="button" tabindex="0"
              aria-label="Selecteer vak" onkeydown={(e) => e.key === "Enter" && selectLeaf(l.node.id, e)} />
        {#if l.node.id === selectedLeafId}
          <rect x={r.x + 2} y={r.y + 2} width={Math.max(1, r.width - 4)} height={Math.max(1, r.height - 4)}
                fill="none" stroke="var(--editor-selected, #D97706)" stroke-width={3 / zoom} pointer-events="none" />
        {/if}
      {/if}
    {/each}

    <!-- divider drag handles (tussenstijl/-dorpel) -->
    {#each layoutGeom.dividers as d, i (i)}
      <rect x={d.rect.x} y={d.rect.y} width={d.rect.width} height={d.rect.height}
            fill="transparent" class="layout-divider {d.direction}"
            onmousedown={(e) => startLayoutDrag(d, e)}
            role="separator" aria-label="Versleep deellijn" tabindex="-1" />
    {/each}

    <!-- in-canvas split / merge controls for the selected vak -->
    {#if selectedLeafRect}
      {@const bs = 30 / zoom}
      {@const gp = 5 / zoom}
      {@const tw = bs * 3 + gp * 2}
      {@const bx = selectedLeafRect.x + selectedLeafRect.width / 2 - tw / 2}
      {@const by = selectedLeafRect.y + 7 / zoom}
      {#each [{ k: "row", t: "⟷", title: "Splits met tussenstijl" }, { k: "column", t: "↕", title: "Splits met tussendorpel" }, { k: "merge", t: "⤬", title: "Samenvoegen" }] as b, bi}
        {@const x0 = bx + bi * (bs + gp)}
        <g class="lay-btn" role="button" tabindex="0" aria-label={b.title}
           onclick={(e) => b.k === "merge" ? doMergeLayout(e) : doSplitLayout(b.k, e)}
           onkeydown={(e) => e.key === "Enter" && (b.k === "merge" ? doMergeLayout(e) : doSplitLayout(b.k, e))}>
          <rect x={x0} y={by} width={bs} height={bs} rx={5 / zoom}
                fill="var(--bg-surface-alt, #2a2a38)" stroke="var(--amber, #D97706)" stroke-width={1.4 / zoom} />
          <text x={x0 + bs / 2} y={by + bs / 2} text-anchor="middle" dominant-baseline="central"
                font-size={17 / zoom} fill="var(--amber, #D97706)" pointer-events="none">{b.t}</text>
        </g>
      {/each}
    {/if}
  {/if}

  {#if !kozijn?.layout}
  <!-- Cell fills (glazing/panel areas) -->
  {#each geometry.cellRects as cellRect}
    {@const colors = cellColor(cellRect.cellIndex)}
    {@const isSelected = $selectedCellIndex === cellRect.cellIndex}
    <rect
      x={cellRect.rect.x} y={cellRect.rect.y}
      width={cellRect.rect.width} height={cellRect.rect.height}
      fill={colors.fill}
      stroke={isSelected ? "var(--editor-selected)" : colors.stroke}
      stroke-width={isSelected ? 3 : 1}
      class="cell"
      onclick={(e) => handleCellClick(cellRect.cellIndex, e)}
      oncontextmenu={(e) => handleCellRightClick(cellRect.cellIndex, e)}
      role="button"
      tabindex="0"
    />

    <!-- EN 12519 opening symbols -->
    {#each openingSymbols(cellRect.cellIndex, cellRect.rect) as sym}
      <path
        d={sym.d}
        fill="none"
        stroke={colors.stroke}
        stroke-width={sym.label === "tilt" ? 0.8 : 1}
        stroke-dasharray={sym.label === "tilt" ? "3 4" : "4 3"}
        opacity={sym.label === "tilt" ? 0.5 : 0.7}
        pointer-events="none"
      />
    {/each}

    <!-- Hardware indicators (hinges + handle) -->
    {@const hw = hardwarePositions(cellRect.cellIndex, cellRect.rect)}
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
    {@const cell = kozijn.cells?.[cellRect.cellIndex]}
    {@const cellFs = 10 / zoom}
    {@const smallFs = 8 / zoom}
    {@const cx = cellRect.rect.x + cellRect.rect.width / 2}
    {@const cy = cellRect.rect.y + cellRect.rect.height / 2}
    {@const vakW = Math.round(cellRect.rect.width)}
    {@const vakH = Math.round(cellRect.rect.height)}
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
      {vakW}×{vakH}
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
  {/each}
  {/if}

  <!-- Frame members (clickable overlay, drawn ON TOP of cells so onderdorpel is visible) -->
  {#each geometry.frameRects as rect, i}
    {@const memberType = FRAME_MEMBER_NAMES[i]}
    {@const isSelected = $selectedMember?.type === memberType}
    {#if rect.width > 0 && rect.height > 0}
      <rect
        x={rect.x} y={rect.y}
        width={rect.width} height={rect.height}
        fill="var(--editor-frame)"
        stroke={isSelected ? "var(--editor-selected)" : "none"}
        stroke-width={isSelected ? 3 : 0}
        class="member"
        onclick={(e) => handleMemberClick(memberType, i, e)}
        role="button"
        tabindex="0"
      />
      <!-- Sponning line (inner edge of rebate, ~17mm offset from glass side) -->
      {@const sp = 17}
      {#if memberType === "frame_top" && rect.height > sp}
        <line x1={rect.x} y1={rect.y + rect.height - sp} x2={rect.x + rect.width} y2={rect.y + rect.height - sp}
          stroke="var(--text-muted)" stroke-width={0.5} opacity="0.3" pointer-events="none" />
      {:else if memberType === "frame_bottom" && rect.height > sp}
        <line x1={rect.x} y1={rect.y + sp} x2={rect.x + rect.width} y2={rect.y + sp}
          stroke="var(--text-muted)" stroke-width={0.5} opacity="0.3" pointer-events="none" />
      {:else if memberType === "frame_left" && rect.width > sp}
        <line x1={rect.x + rect.width - sp} y1={rect.y} x2={rect.x + rect.width - sp} y2={rect.y + rect.height}
          stroke="var(--text-muted)" stroke-width={0.5} opacity="0.3" pointer-events="none" />
      {:else if memberType === "frame_right" && rect.width > sp}
        <line x1={rect.x + sp} y1={rect.y} x2={rect.x + sp} y2={rect.y + rect.height}
          stroke="var(--text-muted)" stroke-width={0.5} opacity="0.3" pointer-events="none" />
      {/if}
    {/if}
  {/each}

  <!-- Corner joint indicators (color-coded by type) -->
  {#if kozijn.frame?.cornerJoints?.length > 0}
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
  {#if geometry.trapezoidOuter && geometry.trapezoidOuter.length >= 3}
    <polygon
      points={geometry.trapezoidOuter.map(p => `${p[0]},${p[1]}`).join(' ')}
      fill="none"
      stroke="var(--editor-frame)"
      stroke-width={kozijn.frame.frameWidth}
      stroke-linejoin="miter"
    />
  {/if}

  <!-- Round frame: filled donut between outer and inner circle -->
  {#if kozijn.frame?.shape?.shapeType === "round"}
    {@const rOuter = Math.min(kozijn.frame.outerWidth, kozijn.frame.outerHeight) / 2}
    {@const rInner = rOuter - kozijn.frame.frameWidth}
    {@const rcx = kozijn.frame.outerWidth / 2}
    {@const rcy = kozijn.frame.outerHeight / 2}
    {#if rInner > 0}
      <!-- Frame fill using two circles with clip path (donut) -->
      <circle cx={rcx} cy={rcy} r={rOuter}
        fill="var(--editor-frame)" stroke="var(--editor-frame)" stroke-width="1" />
      <circle cx={rcx} cy={rcy} r={rInner}
        fill="var(--editor-glass)" stroke="var(--editor-frame)" stroke-width="1" />
    {/if}
  {/if}

  <!-- Elliptical frame: filled donut between outer and inner ellipse -->
  {#if kozijn.frame?.shape?.shapeType === "elliptical"}
    {@const erx = kozijn.frame.shape.ellipseRx || kozijn.frame.outerWidth / 2}
    {@const ery = kozijn.frame.shape.ellipseRy || kozijn.frame.outerHeight / 3}
    {@const ecx = kozijn.frame.outerWidth / 2}
    {@const ecy = kozijn.frame.outerHeight / 2}
    {@const eirx = erx - kozijn.frame.frameWidth}
    {@const eiry = ery - kozijn.frame.frameWidth}
    <ellipse cx={ecx} cy={ecy} rx={erx} ry={ery}
      fill="var(--editor-frame)" stroke="var(--editor-frame)" stroke-width="1" />
    {#if eirx > 0 && eiry > 0}
      <ellipse cx={ecx} cy={ecy} rx={eirx} ry={eiry}
        fill="var(--editor-glass)" stroke="var(--editor-frame)" stroke-width="1" />
    {/if}
  {/if}

  <!-- Arcs for arched/round kozijnen -->
  {#each (geometry.arcs || []) as arc}
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

  {#if !kozijn?.layout}
  <!-- Vertical dividers -->
  {#each geometry.vDividers as rect, i}
    {@const isSelected = $selectedMember?.type === "divider_v" && $selectedMember?.index === i}
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
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
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
      stroke={isSelected ? "var(--editor-selected)" : "none"}
      stroke-width={isSelected ? 3 : 0}
      class="member"
      onclick={(e) => handleMemberClick("divider_h", i, e)}
      role="button"
      tabindex="0"
    />
  {/each}
  {/if}

  <!-- Profile codes on frame members (GA Kozijn style - green text) -->
  {#each geometry.frameRects as rect, i}
    {@const memberName = FRAME_MEMBER_NAMES[i]}
    {@const isVertical = memberName === "frame_left" || memberName === "frame_right"}
    {@const profileLabel = `${Math.round(kozijn.frame.frameWidth)}×${Math.round(kozijn.frame.frameDepth)}`}
    {@const labelFs = 10 / zoom}
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
  {/each}

  {#if !kozijn?.layout}
  <!-- Sash frame (raamhout/deurhout) with sponning detail -->
  {#each geometry.cellRects as cellRect}
    {@const cell = kozijn.cells?.[cellRect.cellIndex]}
    {#if cell && cell.sashWidth && cell.sashWidth > 0 && ["turn_tilt", "turn", "tilt", "sliding", "door", "top_hung", "bottom_hung", "lift_slide", "pivot"].includes(cell.panelType)}
      {@const sw = cell.sashWidth || 54}
      {@const sp = 17}
      {@const opdek = 13}
      {@const gap = 2}
      {@const r = cellRect.rect}
      {@const isDoor = cell.panelType === "door"}
      <!-- Sash sits in frame sponning with small gap (sponningspeling) -->
      {@const sx = r.x + gap}
      {@const sy = r.y + gap}
      {@const sashW = Math.max(1, r.width - gap * 2)}
      {@const sashH = Math.max(1, r.height - gap * 2)}
      <!-- Sash outer edge (raamhout buitenkant) -->
      <rect x={sx} y={sy} width={sashW} height={sashH}
        fill="none" stroke="var(--amber)" stroke-width={1.2} opacity="0.7" pointer-events="none" />
      <!-- Sash sponning line (inner rebate where glass sits, ~17mm inset) -->
      {@const spInset = Math.min(sp, sw * 0.4)}
      <rect x={sx + spInset} y={sy + spInset}
        width={Math.max(1, sashW - spInset * 2)} height={Math.max(1, sashH - spInset * 2)}
        fill="none" stroke="var(--amber)" stroke-width={0.4} opacity="0.4" pointer-events="none" />
      <!-- Glaslat line (strip holding glass, ~5mm inside sponning) — clearer + position-tinted when configured -->
      {@const glInset = spInset + 5}
      {@const hasGl = !!cell.glaslat}
      {@const glColor = cell.glaslat?.position === "buiten" ? "#3B82F6" : "var(--amber)"}
      <rect x={sx + glInset} y={sy + glInset}
        width={Math.max(1, sashW - glInset * 2)} height={Math.max(1, sashH - glInset * 2)}
        fill="none" stroke={hasGl ? glColor : "var(--amber)"}
        stroke-width={hasGl ? 0.8 : 0.25} opacity={hasGl ? 0.6 : 0.25}
        stroke-dasharray={hasGl ? "" : "3 2"} pointer-events="none" />
      {#if isDoor}
        <!-- Door: thicker bottom rail (onderdorpel ~150mm) -->
        {@const doorBottomH = Math.min(150, sashH * 0.15)}
        <rect x={sx} y={sy + sashH - doorBottomH}
          width={sashW} height={doorBottomH}
          fill="var(--editor-frame)" fill-opacity="0.3"
          stroke="var(--amber)" stroke-width={0.8} opacity="0.5" pointer-events="none" />
        <!-- Threshold slope indicator -->
        <line x1={sx} y1={sy + sashH} x2={sx + sashW} y2={sy + sashH}
          stroke="var(--amber)" stroke-width={3} opacity="0.7" pointer-events="none" />
      {/if}
    {/if}
  {/each}
  {/if}

  <!-- Edge/spouwlat indicators (dashed outlines along frame edges) -->
  {#if kozijn.edges?.length > 0}
    {#each kozijn.edges as edge, ei}
      {#if edge.spouwlat}
        {@const eow = geometry.outerRect?.width || 0}
        {@const eoh = geometry.outerRect?.height || 0}
        {@const spW = edge.spouwlat.width || 100}
        {@const spH = edge.spouwlat.height || 30}
        <!-- Side-specific spouwlat: 0=top, 1=bottom, 2=left, 3=right -->
        {#if ei === 0}
          <rect x={0} y={-spH} width={eow} height={spH}
            fill="none" stroke="#8b5cf6" stroke-width={1 / zoom}
            stroke-dasharray="{3 / zoom} {2 / zoom}" opacity="0.5" pointer-events="none" />
        {:else if ei === 1}
          <rect x={0} y={eoh} width={eow} height={spH}
            fill="none" stroke="#8b5cf6" stroke-width={1 / zoom}
            stroke-dasharray="{3 / zoom} {2 / zoom}" opacity="0.5" pointer-events="none" />
        {:else if ei === 2}
          <rect x={-spW} y={0} width={spW} height={eoh}
            fill="none" stroke="#8b5cf6" stroke-width={1 / zoom}
            stroke-dasharray="{3 / zoom} {2 / zoom}" opacity="0.5" pointer-events="none" />
        {:else if ei === 3}
          <rect x={eow} y={0} width={spW} height={eoh}
            fill="none" stroke="#8b5cf6" stroke-width={1 / zoom}
            stroke-dasharray="{3 / zoom} {2 / zoom}" opacity="0.5" pointer-events="none" />
        {/if}
      {/if}
    {/each}
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

  <!-- Dimension lines — rendered in model-space with inverse-zoom for constant screen size -->
  {#each geometry.dimensions as dim, dimIdx}
    {@const fontSize = 12 / zoom}
    {@const sw = 1 / zoom}
    {@const tick = 6 / zoom}
    {@const bgPad = 3 / zoom}
    {@const isH = dim.side === "bottom" || dim.side === "top"}
    {@const midX = (dim.x1 + dim.x2) / 2}
    {@const midY = (dim.y1 + dim.y2) / 2}
    {@const dimVal = Number(dim.label) > 0 ? Math.round(Number(dim.label)) : dim.label}

    <!-- Dimension line -->
    <line x1={dim.x1} y1={dim.y1} x2={dim.x2} y2={dim.y2}
      stroke="var(--editor-dimension)" stroke-width={sw} />

    <!-- Tick marks -->
    {#if isH}
      <line x1={dim.x1} y1={dim.y1 - tick} x2={dim.x1} y2={dim.y1 + tick} stroke="var(--editor-dimension)" stroke-width={sw}/>
      <line x1={dim.x2} y1={dim.y2 - tick} x2={dim.x2} y2={dim.y2 + tick} stroke="var(--editor-dimension)" stroke-width={sw}/>
    {:else}
      <line x1={dim.x1 - tick} y1={dim.y1} x2={dim.x1 + tick} y2={dim.y1} stroke="var(--editor-dimension)" stroke-width={sw}/>
      <line x1={dim.x2 - tick} y1={dim.y2} x2={dim.x2 + tick} y2={dim.y2} stroke="var(--editor-dimension)" stroke-width={sw}/>
    {/if}

    <!-- Label background for readability -->
    {@const labelW = String(dimVal).length * fontSize * 0.6 + bgPad * 2}
    {@const labelH = fontSize + bgPad * 2}

    {#if editingDim && editingDim.index === dimIdx}
      <!-- Inline edit input -->
      <foreignObject
        x={isH ? midX - 50 / zoom : midX + bgPad}
        y={midY - labelH * 0.7}
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
      {#if isH}
        <!-- Horizontal: label centered above dimension line -->
        <rect
          x={midX - labelW / 2} y={midY - labelH / 2}
          width={labelW} height={labelH}
          fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
          style="cursor: pointer"
          onclick={(e) => handleDimClick(dimIdx, e)}
        />
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <text
          x={midX} y={midY}
          text-anchor="middle" dominant-baseline="central"
          fill="var(--editor-dimension)" font-size={fontSize}
          font-family="var(--font-body)" font-weight="600"
          style="cursor: pointer"
          onclick={(e) => handleDimClick(dimIdx, e)}
        >{dimVal}</text>
      {:else}
        <!-- Vertical (right side): label rotated 90° along the dimension line -->
        <g transform="translate({midX}, {midY}) rotate(-90)">
          <rect
            x={-labelW / 2} y={-labelH / 2}
            width={labelW} height={labelH}
            fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
            style="cursor: pointer"
            onclick={(e) => handleDimClick(dimIdx, e)}
          />
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <text
            x={0} y={0}
            text-anchor="middle" dominant-baseline="central"
            fill="var(--editor-dimension)" font-size={fontSize}
            font-family="var(--font-body)" font-weight="600"
            style="cursor: pointer"
            onclick={(e) => handleDimClick(dimIdx, e)}
          >{dimVal}</text>
        </g>
      {/if}
    {/if}
  {/each}
</g>

<style>
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
