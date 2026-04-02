<script>
  /**
   * DimensionOverlay — renders dimension lines in screen-space coordinates.
   * Font-size and stroke-width are constant screen pixels, always readable.
   */
  let { dimensions = [], zoom = 0.35, panX = 0, panY = 0 } = $props();

  // Convert model coordinate to screen coordinate
  function toScreen(modelX, modelY) {
    return {
      x: modelX * zoom + panX,
      y: modelY * zoom + panY,
    };
  }

  const FONT_SIZE = 12;       // px — always readable
  const STROKE_WIDTH = 1;     // px — always visible
  const TICK_SIZE = 6;        // px — visible tick marks
  const LABEL_OFFSET = -4;    // px — offset from line for labels
</script>

<svg class="dimension-overlay">
  {#each dimensions as dim}
    {@const p1 = toScreen(dim.x1, dim.y1)}
    {@const p2 = toScreen(dim.x2, dim.y2)}
    {@const isHorizontal = dim.side === "bottom" || dim.side === "top"}
    {@const midX = (p1.x + p2.x) / 2}
    {@const midY = (p1.y + p2.y) / 2}

    <!-- Dimension line -->
    <line
      x1={p1.x} y1={p1.y}
      x2={p2.x} y2={p2.y}
      stroke="var(--editor-dimension)"
      stroke-width={STROKE_WIDTH}
    />

    <!-- Tick marks -->
    {#if isHorizontal}
      <line x1={p1.x} y1={p1.y - TICK_SIZE} x2={p1.x} y2={p1.y + TICK_SIZE} stroke="var(--editor-dimension)" stroke-width={STROKE_WIDTH}/>
      <line x1={p2.x} y1={p2.y - TICK_SIZE} x2={p2.x} y2={p2.y + TICK_SIZE} stroke="var(--editor-dimension)" stroke-width={STROKE_WIDTH}/>
    {:else}
      <line x1={p1.x - TICK_SIZE} y1={p1.y} x2={p1.x + TICK_SIZE} y2={p1.y} stroke="var(--editor-dimension)" stroke-width={STROKE_WIDTH}/>
      <line x1={p2.x - TICK_SIZE} y1={p2.y} x2={p2.x + TICK_SIZE} y2={p2.y} stroke="var(--editor-dimension)" stroke-width={STROKE_WIDTH}/>
    {/if}

    <!-- Label with background for readability -->
    {#if isHorizontal}
      <rect
        x={midX - dim.label.length * 3.5 - 3}
        y={midY + LABEL_OFFSET - FONT_SIZE + 2}
        width={dim.label.length * 7 + 6}
        height={FONT_SIZE + 4}
        fill="var(--editor-bg, #1a1a2e)"
        rx="2"
        opacity="0.85"
      />
    {:else}
      <rect
        x={midX + LABEL_OFFSET + 2}
        y={midY - FONT_SIZE / 2 - 2}
        width={dim.label.length * 7 + 6}
        height={FONT_SIZE + 4}
        fill="var(--editor-bg, #1a1a2e)"
        rx="2"
        opacity="0.85"
      />
    {/if}

    <text
      x={isHorizontal ? midX : midX + LABEL_OFFSET + 5}
      y={isHorizontal ? midY + LABEL_OFFSET : midY}
      text-anchor={isHorizontal ? "middle" : "start"}
      dominant-baseline={isHorizontal ? "auto" : "central"}
      fill="var(--editor-dimension)"
      font-size="{FONT_SIZE}px"
      font-family="var(--font-body)"
      font-weight="600"
    >
      {dim.label}
    </text>
  {/each}
</svg>

<style>
  .dimension-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
    z-index: 10;
  }
</style>
