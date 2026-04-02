<script>
  import { selectedCellIndex, updateCellType } from "../../stores/kozijn.js";

  export let geometry;
  export let kozijn;

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
    selectedCellIndex.set(cellIndex);
  }
</script>

<g>
  <!-- Frame members (outer border) -->
  {#each geometry.frameRects as rect}
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
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
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <rect
      x={cellRect.rect.x} y={cellRect.rect.y}
      width={cellRect.rect.width} height={cellRect.rect.height}
      fill={colors.fill}
      stroke={isSelected ? "var(--editor-selected)" : colors.stroke}
      stroke-width={isSelected ? 3 : 1}
      class="cell"
      on:click={(e) => handleCellClick(cellRect.cellIndex, e)}
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
  {#each geometry.vDividers as rect}
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
    />
  {/each}

  <!-- Horizontal dividers -->
  {#each geometry.hDividers as rect}
    <rect
      x={rect.x} y={rect.y}
      width={rect.width} height={rect.height}
      fill="var(--editor-frame)"
    />
  {/each}

  <!-- Dimension lines -->
  {#each geometry.dimensions as dim}
    <line
      x1={dim.x1} y1={dim.y1}
      x2={dim.x2} y2={dim.y2}
      stroke="var(--editor-dimension)"
      stroke-width="1"
    />
    <!-- Tick marks -->
    {#if dim.side === "bottom" || dim.side === "top"}
      <line x1={dim.x1} y1={dim.y1 - 4} x2={dim.x1} y2={dim.y1 + 4} stroke="var(--editor-dimension)" stroke-width="1"/>
      <line x1={dim.x2} y1={dim.y2 - 4} x2={dim.x2} y2={dim.y2 + 4} stroke="var(--editor-dimension)" stroke-width="1"/>
    {:else}
      <line x1={dim.x1 - 4} y1={dim.y1} x2={dim.x1 + 4} y2={dim.y1} stroke="var(--editor-dimension)" stroke-width="1"/>
      <line x1={dim.x2 - 4} y1={dim.y2} x2={dim.x2 + 4} y2={dim.y2} stroke="var(--editor-dimension)" stroke-width="1"/>
    {/if}
    <!-- Label -->
    <text
      x={(dim.x1 + dim.x2) / 2}
      y={(dim.y1 + dim.y2) / 2}
      text-anchor="middle"
      dominant-baseline="central"
      fill="var(--editor-dimension)"
      font-size="14"
      font-family="var(--font-body)"
    >
      {dim.label}
    </text>
  {/each}
</g>

<style>
  .cell {
    cursor: pointer;
    transition: stroke 0.15s, stroke-width 0.15s;
  }

  .cell:hover {
    stroke: var(--amber);
    stroke-width: 2;
  }
</style>
