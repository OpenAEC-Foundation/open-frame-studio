<script>
  import { profileEditor, editorVertices, editorBounds, editorSelectedVertex, editorSnap } from "../../stores/profileEditor.js";
  import VertexHandles from "./VertexHandles.svelte";
  import EdgeSegments from "./EdgeSegments.svelte";

  let svgEl;
  let containerEl;
  let zoom = $state(4);
  let pan = $state({ x: 200, y: 200 });
  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0 });
  let panOffset = $state({ x: 0, y: 0 });
  let containerRect = $state({ width: 800, height: 600 });

  $effect(() => {
    if (containerEl) {
      const r = containerEl.getBoundingClientRect();
      containerRect = { width: r.width, height: r.height };
    }
  });

  function handleWheel(e) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.3 : 0.3;
    const oldZoom = zoom;
    const newZoom = Math.max(0.5, Math.min(20, zoom + delta));
    const rect = containerEl.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    const scale = newZoom / oldZoom;
    pan = {
      x: mx - (mx - pan.x) * scale,
      y: my - (my - pan.y) * scale,
    };
    zoom = newZoom;
  }

  function handleMouseDown(e) {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      e.preventDefault();
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      panOffset = { ...pan };
    }
  }

  function handleMouseMove(e) {
    if (isPanning) {
      pan = {
        x: panOffset.x + (e.clientX - panStart.x),
        y: panOffset.y + (e.clientY - panStart.y),
      };
    }
  }

  function handleMouseUp() {
    isPanning = false;
  }

  function handleCanvasClick(e) {
    if (e.target === svgEl || e.target.classList.contains("grid-bg")) {
      profileEditor.deselectAll();
    }
  }

  function screenToModel(clientX, clientY) {
    const rect = containerEl.getBoundingClientRect();
    return {
      x: (clientX - rect.left - pan.x) / zoom,
      y: (clientY - rect.top - pan.y) / zoom,
    };
  }

  function zoomToFit() {
    if (!containerEl || $editorVertices.length === 0) return;
    const b = $editorBounds;
    const rect = containerEl.getBoundingClientRect();
    const padding = 80;
    const availW = rect.width - padding * 2;
    const availH = rect.height - padding * 2;
    const w = b.width || 100;
    const h = b.height || 100;
    const fitZoom = Math.min(availW / w, availH / h, 15);
    zoom = fitZoom;
    pan = {
      x: rect.width / 2 - (b.minX + w / 2) * fitZoom,
      y: rect.height / 2 - (b.minY + h / 2) * fitZoom,
    };
  }

  // Auto zoom-to-fit on first load
  let hasAutoFit = false;
  $effect(() => {
    if ($editorVertices.length > 0 && containerEl && !hasAutoFit) {
      requestAnimationFrame(() => {
        zoomToFit();
        hasAutoFit = true;
      });
    }
  });

  // Grid rendering helpers
  function getGridLines() {
    if (!containerEl) return { minor: [], major: [] };
    const rect = containerEl.getBoundingClientRect();
    const minor = [];
    const major = [];

    const startX = Math.floor(-pan.x / zoom);
    const endX = Math.ceil((rect.width - pan.x) / zoom);
    const startY = Math.floor(-pan.y / zoom);
    const endY = Math.ceil((rect.height - pan.y) / zoom);

    const step = zoom > 2 ? 1 : zoom > 0.8 ? 5 : 10;
    const majorStep = step * 10;

    for (let x = Math.floor(startX / step) * step; x <= endX; x += step) {
      if (x % majorStep === 0) major.push({ x1: x, y1: startY, x2: x, y2: endY });
      else minor.push({ x1: x, y1: startY, x2: x, y2: endY });
    }
    for (let y = Math.floor(startY / step) * step; y <= endY; y += step) {
      if (y % majorStep === 0) major.push({ x1: startX, y1: y, x2: endX, y2: y });
      else minor.push({ x1: startX, y1: y, x2: endX, y2: y });
    }

    return { minor, major };
  }

  let gridLines = $derived.by(() => {
    // Re-derive when zoom or pan changes
    void zoom;
    void pan;
    void containerRect;
    return getGridLines();
  });

  // Polygon path string
  let polyPoints = $derived(
    $editorVertices.map((v) => `${v.x},${v.y}`).join(" ")
  );

  // Sponning overlay
  let sponningStore = $derived.by(() => {
    let s;
    profileEditor.subscribe(v => s = v)();
    return s.sponning;
  });

  // Dimension labels
  let dims = $derived.by(() => {
    const b = $editorBounds;
    const invZ = 1 / zoom;
    return {
      width: { x: b.minX + b.width / 2, y: b.minY - 8 * invZ, label: `${Math.round(b.width * 10) / 10}`, fontSize: 11 * invZ },
      height: { x: b.maxX + 8 * invZ, y: b.minY + b.height / 2, label: `${Math.round(b.height * 10) / 10}`, fontSize: 11 * invZ },
    };
  });
</script>

<div
  class="canvas-container"
  bind:this={containerEl}
  onwheel={handleWheel}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <svg
    bind:this={svgEl}
    width="100%"
    height="100%"
    onclick={handleCanvasClick}
    style="cursor: {isPanning ? 'grabbing' : 'default'}"
  >
    <g transform="translate({pan.x}, {pan.y}) scale({zoom})">
      <!-- Grid -->
      {#each gridLines.minor as l}
        <line x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
          stroke="var(--border-color, #333)" stroke-width={0.5 / zoom} opacity="0.15" />
      {/each}
      {#each gridLines.major as l}
        <line x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
          stroke="var(--border-color, #555)" stroke-width={0.8 / zoom} opacity="0.3" />
      {/each}

      <!-- Origin axes -->
      <line x1="-2000" y1="0" x2="2000" y2="0"
        stroke="var(--text-muted)" stroke-width={0.8 / zoom} opacity="0.4" stroke-dasharray="{4 / zoom} {3 / zoom}" />
      <line x1="0" y1="-2000" x2="0" y2="2000"
        stroke="var(--text-muted)" stroke-width={0.8 / zoom} opacity="0.4" stroke-dasharray="{4 / zoom} {3 / zoom}" />

      <!-- Profile polygon -->
      {#if $editorVertices.length >= 3}
        <polygon
          points={polyPoints}
          fill="var(--bg-surface-alt, #2a2a32)"
          fill-opacity="0.6"
          stroke="var(--amber, #D97706)"
          stroke-width={2 / zoom}
          stroke-linejoin="miter"
        />

        <!-- Sponning zone indicator -->
        {#if sponningStore.position === "buiten"}
          {@const b = $editorBounds}
          <rect
            x={sponningStore.width}
            y={b.maxY - sponningStore.depth}
            width={b.width - sponningStore.width * 2}
            height={sponningStore.depth}
            fill="var(--info, #2563EB)"
            fill-opacity="0.12"
            stroke="var(--info, #2563EB)"
            stroke-width={1 / zoom}
            stroke-dasharray="{3 / zoom} {2 / zoom}"
          />
        {/if}
      {/if}

      <!-- Dimension labels -->
      {#if $editorVertices.length >= 3}
        <!-- Width -->
        {@const b = $editorBounds}
        {@const invZ = 1 / zoom}
        <line x1={b.minX} y1={b.minY - 5 * invZ} x2={b.maxX} y2={b.minY - 5 * invZ}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <line x1={b.minX} y1={b.minY - 3 * invZ} x2={b.minX} y2={b.minY - 7 * invZ}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <line x1={b.maxX} y1={b.minY - 3 * invZ} x2={b.maxX} y2={b.minY - 7 * invZ}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <text x={dims.width.x} y={dims.width.y}
          text-anchor="middle" fill="var(--text-primary)"
          font-size={dims.width.fontSize} font-family="Inter, sans-serif">
          {dims.width.label}mm
        </text>

        <!-- Height -->
        <line x1={b.maxX + 5 * invZ} y1={b.minY} x2={b.maxX + 5 * invZ} y2={b.maxY}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <line x1={b.maxX + 3 * invZ} y1={b.minY} x2={b.maxX + 7 * invZ} y2={b.minY}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <line x1={b.maxX + 3 * invZ} y1={b.maxY} x2={b.maxX + 7 * invZ} y2={b.maxY}
          stroke="var(--text-muted)" stroke-width={0.8 * invZ} />
        <text x={b.maxX + 10 * invZ} y={dims.height.y}
          text-anchor="start" dominant-baseline="middle" fill="var(--text-primary)"
          font-size={dims.height.fontSize} font-family="Inter, sans-serif">
          {dims.height.label}mm
        </text>
      {/if}

      <!-- Edge hit areas (for adding points) -->
      <EdgeSegments {zoom} {screenToModel} />

      <!-- Vertex handles -->
      <VertexHandles {zoom} {screenToModel} />
    </g>
  </svg>

  <!-- Zoom indicator -->
  <div class="zoom-indicator">
    <button class="zoom-btn" onclick={zoomToFit} title="Zoom passend">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/>
      </svg>
    </button>
    <span>{Math.round(zoom * 100)}%</span>
  </div>
</div>

<style>
  .canvas-container {
    width: 100%;
    height: 100%;
    overflow: hidden;
    position: relative;
  }

  svg {
    display: block;
  }

  .zoom-indicator {
    position: absolute;
    bottom: 10px;
    right: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--bg-surface);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font-size: 11px;
    color: var(--text-muted);
    box-shadow: var(--shadow-sm);
  }

  .zoom-btn {
    display: flex;
    align-items: center;
    padding: 2px;
    border-radius: 2px;
    color: var(--text-muted);
    transition: color 0.15s;
  }
  .zoom-btn:hover {
    color: var(--amber);
  }
</style>
