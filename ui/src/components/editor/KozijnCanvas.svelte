<script>
  import { selectedCellIndex, selectedMember, updateCellType } from "../../stores/kozijn.js";

  let { geometry, kozijn, zoom = 0.35 } = $props();

  const FRAME_MEMBER_NAMES = ["frame_top", "frame_bottom", "frame_left", "frame_right"];

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
  <!-- Frame members (outer border) -->
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

    <!-- Cell label -->
    <text
      x={cellRect.rect.x + cellRect.rect.width / 2}
      y={cellRect.rect.y + cellRect.rect.height / 2}
      text-anchor="middle"
      dominant-baseline="central"
      fill="var(--text-secondary)"
      font-size="24"
      font-family="var(--font-heading)"
      font-weight="700"
      opacity="0.4"
      pointer-events="none"
    >
      {cellLabel(cellRect.cellIndex)}
    </text>
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

  <!-- Dimension lines — rendered in model-space with inverse-zoom for constant screen size -->
  {#each geometry.dimensions as dim}
    {@const fontSize = 12 / zoom}
    {@const sw = 1 / zoom}
    {@const tick = 6 / zoom}
    {@const bgPad = 3 / zoom}
    {@const isH = dim.side === "bottom" || dim.side === "top"}
    {@const midX = (dim.x1 + dim.x2) / 2}
    {@const midY = (dim.y1 + dim.y2) / 2}

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
    {@const labelW = dim.label.length * fontSize * 0.6 + bgPad * 2}
    {@const labelH = fontSize + bgPad * 2}
    {#if isH}
      <rect
        x={midX - labelW / 2} y={midY - labelH - bgPad}
        width={labelW} height={labelH}
        fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
      />
    {:else}
      <rect
        x={midX + bgPad} y={midY - labelH / 2}
        width={labelW} height={labelH}
        fill="var(--editor-bg, #1a1a2e)" rx={2 / zoom} opacity="0.85"
      />
    {/if}

    <!-- Label text -->
    <text
      x={isH ? midX : midX + bgPad + labelW / 2}
      y={isH ? midY - bgPad - fontSize * 0.15 : midY}
      text-anchor="middle"
      dominant-baseline={isH ? "auto" : "central"}
      fill="var(--editor-dimension)"
      font-size={fontSize}
      font-family="var(--font-body)"
      font-weight="600"
    >
      {dim.label}
    </text>
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
