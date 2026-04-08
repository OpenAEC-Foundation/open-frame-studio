<script>
  import { selectedCellIndex, selectedMember, updateCellType, updateDimensions, updateGridSizes, currentKozijn } from "../../stores/kozijn.js";
  import { get } from "svelte/store";

  let { geometry, kozijn, zoom = 0.35, oncellcontextmenu } = $props();

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

  function openingSymbol(cellIndex, rect) {
    const cell = kozijn.cells[cellIndex];
    if (!cell || !cell.panelType) return null;
    const type = cell.panelType;
    const dir = cell.openingDirection;
    const cx = rect.x + rect.width / 2;
    const cy = rect.y + rect.height / 2;

    // Draw opening triangle for turn/turn_tilt
    if (type === "turn_tilt" || type === "turn") {
      if (dir === "left") {
        return `M${rect.x},${rect.y} L${rect.x + rect.width},${cy} L${rect.x},${rect.y + rect.height} Z`;
      } else {
        return `M${rect.x + rect.width},${rect.y} L${rect.x},${cy} L${rect.x + rect.width},${rect.y + rect.height} Z`;
      }
    }
    // Tilt: triangle from bottom
    if (type === "tilt") {
      return `M${rect.x},${rect.y + rect.height} L${cx},${rect.y} L${rect.x + rect.width},${rect.y + rect.height} Z`;
    }
    return null;
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

    <!-- Opening symbol -->
    {@const symbol = openingSymbol(cellRect.cellIndex, cellRect.rect)}
    {#if symbol}
      <path
        d={symbol}
        fill="none"
        stroke={colors.stroke}
        stroke-width="1"
        stroke-dasharray="4 3"
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

  <!-- Frame members (clickable overlay, drawn ON TOP of cells so onderdorpel is visible) -->
  {#each geometry.frameRects as rect, i}
    {@const memberType = FRAME_MEMBER_NAMES[i]}
    {@const isSelected = $selectedMember?.type === memberType}
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
  {/each}

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
      d="M {x1} {y1} A {r} {r} 0 {largeArc} 0 {x2} {y2}"
      fill="none"
      stroke="var(--editor-frame)"
      stroke-width={arc.strokeWidth}
    />
  {/each}

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

  <!-- Sash frame (raamhout) for operable cells -->
  {#each geometry.cellRects as cellRect}
    {@const cell = kozijn.cells?.[cellRect.cellIndex]}
    {#if cell && cell.sashWidth && cell.sashWidth > 0 && ["turn_tilt", "turn", "tilt", "sliding", "door"].includes(cell.panelType)}
      {@const sw = cell.sashWidth}
      {@const r = cellRect.rect}
      <rect
        x={r.x + sw * 0.3} y={r.y + sw * 0.3}
        width={Math.max(1, r.width - sw * 0.6)} height={Math.max(1, r.height - sw * 0.6)}
        fill="none"
        stroke="var(--amber)"
        stroke-width={sw * 0.15}
        opacity="0.5"
        pointer-events="none"
      />
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
        x={isH ? midX - 30 / zoom : midX + bgPad}
        y={isH ? midY - labelH - bgPad : midY - labelH / 2}
        width={60 / zoom}
        height={labelH * 1.2}
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
            border-radius: 2px;
            text-align: center;
            font-size: {Math.min(14, fontSize * zoom)}px;
            font-weight: 700;
            font-family: var(--font-body);
            outline: none;
            padding: 0 2px;
          "
        />
      </foreignObject>
    {:else}
      {#if isH}
        <rect
          x={midX - labelW / 2} y={midY - labelH - bgPad}
          width={labelW} height={labelH}
          fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
          style="cursor: pointer"
          onclick={(e) => handleDimClick(dimIdx, e)}
        />
      {:else}
        <rect
          x={midX + bgPad} y={midY - labelH / 2}
          width={labelW} height={labelH}
          fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
          style="cursor: pointer"
          onclick={(e) => handleDimClick(dimIdx, e)}
        />
      {/if}

      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <text
        x={isH ? midX : midX + bgPad + labelW / 2}
        y={isH ? midY - bgPad - fontSize * 0.15 : midY}
        text-anchor="middle"
        dominant-baseline={isH ? "auto" : "central"}
        fill="var(--editor-dimension)"
        font-size={fontSize}
        font-family="var(--font-body)"
        font-weight="600"
        style="cursor: pointer"
        onclick={(e) => handleDimClick(dimIdx, e)}
      >
        {dimVal}
      </text>
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
</style>
