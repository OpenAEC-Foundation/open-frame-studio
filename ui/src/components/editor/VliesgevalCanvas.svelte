<script>
  import { currentVgGeometry, selectedPanelIndex, currentVliesgevel, updatePanel } from "../../stores/vliesgevel.js";

  $: geom = $currentVgGeometry;
  $: vg = $currentVliesgevel;

  const PANEL_COLORS = {
    Glass: { fill: "rgba(135, 206, 250, 0.3)", stroke: "#87CEEB" },
    OpaquePanel: { fill: "rgba(160, 160, 160, 0.5)", stroke: "#808080" },
    SpandrelPanel: { fill: "rgba(100, 100, 100, 0.5)", stroke: "#666" },
    Ventilation: { fill: "rgba(144, 238, 144, 0.3)", stroke: "#32CD32" },
    Door: { fill: "rgba(210, 180, 140, 0.3)", stroke: "#D2B48C" },
    OpenableWindow: { fill: "rgba(135, 206, 250, 0.5)", stroke: "#4169E1" },
  };

  function panelColor(panelType) {
    return PANEL_COLORS[panelType] || PANEL_COLORS.Glass;
  }

  function handlePanelClick(col, row) {
    const nc = vg?.mullions?.length + 1 || 1;
    const idx = row * nc + col;
    selectedPanelIndex.set(idx);
  }
</script>

{#if geom}
  <g class="vliesgevel-canvas">
    <!-- Panels -->
    {#each geom.panelRects as panel}
      {@const colors = panelColor(panel.panelType)}
      {@const nc = (vg?.mullions?.length || 0) + 1}
      {@const idx = panel.row * nc + panel.col}
      <rect
        x={panel.x}
        y={panel.y}
        width={panel.width}
        height={panel.height}
        fill={colors.fill}
        stroke={$selectedPanelIndex === idx ? "var(--amber, #D97706)" : colors.stroke}
        stroke-width={$selectedPanelIndex === idx ? 2 : 0.5}
        class="panel"
        on:click={() => handlePanelClick(panel.col, panel.row)}
      />
      <!-- Glass cross pattern for glass panels -->
      {#if panel.panelType === "Glass" || panel.panelType === "OpenableWindow"}
        <line
          x1={panel.x} y1={panel.y}
          x2={panel.x + panel.width} y2={panel.y + panel.height}
          stroke={colors.stroke} stroke-width="0.3" opacity="0.4"
        />
        <line
          x1={panel.x + panel.width} y1={panel.y}
          x2={panel.x} y2={panel.y + panel.height}
          stroke={colors.stroke} stroke-width="0.3" opacity="0.4"
        />
      {/if}
      <!-- Panel type label -->
      <text
        x={panel.x + panel.width / 2}
        y={panel.y + panel.height / 2}
        text-anchor="middle"
        dominant-baseline="central"
        font-size="10"
        fill="var(--text-muted, #999)"
        pointer-events="none"
      >
        {panel.panelType === "Glass" ? "G" : panel.panelType === "Door" ? "D" : panel.panelType === "OpenableWindow" ? "R" : panel.panelType === "SpandrelPanel" ? "B" : panel.panelType === "OpaquePanel" ? "P" : "V"}
      </text>
    {/each}

    <!-- Mullions (vertical members) -->
    {#each geom.mullionRects as rect}
      <rect
        x={rect.x} y={rect.y}
        width={rect.width} height={rect.height}
        fill="var(--text-primary, #333)"
        opacity="0.8"
      />
    {/each}

    <!-- Transoms (horizontal members) -->
    {#each geom.transomRects as rect}
      <rect
        x={rect.x} y={rect.y}
        width={rect.width} height={rect.height}
        fill="var(--text-primary, #333)"
        opacity="0.7"
      />
    {/each}

    <!-- Outer border -->
    <rect
      x="0" y="0"
      width={geom.overallWidth}
      height={geom.overallHeight}
      fill="none"
      stroke="var(--text-primary, #333)"
      stroke-width="1"
    />

    <!-- Dimensions -->
    {#each geom.dimensions as dim}
      <line
        x1={dim.x1} y1={dim.y1}
        x2={dim.x2} y2={dim.y2}
        stroke="var(--text-muted, #999)"
        stroke-width="0.5"
        stroke-dasharray="4 2"
      />
      <text
        x={(dim.x1 + dim.x2) / 2}
        y={(dim.y1 + dim.y2) / 2 - 3}
        text-anchor="middle"
        font-size="8"
        fill="var(--text-muted, #999)"
      >
        {dim.label}
      </text>
    {/each}
  </g>
{/if}

<style>
  .panel {
    cursor: pointer;
  }
  .panel:hover {
    filter: brightness(1.1);
  }
</style>
