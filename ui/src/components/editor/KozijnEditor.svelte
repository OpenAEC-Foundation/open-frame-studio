<script>
  import { currentKozijn, currentGeometry, selectedCellIndex, updateGridSizes } from "../../stores/kozijn.js";
  import { zoom, editorPan } from "../../stores/ui.js";
  import KozijnCanvas from "./KozijnCanvas.svelte";
  import GridHandles from "./GridHandles.svelte";

  let containerEl;
  let isPanning = false;
  let panStart = { x: 0, y: 0 };
  let panOffset = { x: 0, y: 0 };

  // Auto zoom-to-fit when kozijn changes
  $: if ($currentGeometry && containerEl) {
    requestAnimationFrame(() => zoomToFit());
  }

  function zoomToFit() {
    if (!containerEl || !$currentGeometry) return;
    const rect = containerEl.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const outer = $currentGeometry.outerRect;
    // Add padding for dimension lines
    const contentW = outer.width + 120;
    const contentH = outer.height + 120;

    const scaleX = (rect.width - 60) / contentW;
    const scaleY = (rect.height - 60) / contentH;
    const newZoom = Math.min(scaleX, scaleY, 1.0);

    zoom.set(newZoom);

    // Center the kozijn
    const scaledW = contentW * newZoom;
    const scaledH = contentH * newZoom;
    editorPan.set({
      x: (rect.width - scaledW) / 2 + 20,
      y: (rect.height - scaledH) / 2 + 20,
    });
  }

  function handleWheel(e) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.03 : 0.03;
    zoom.update((z) => Math.max(0.05, Math.min(2.0, z + delta)));
  }

  function handleMouseDown(e) {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      panOffset = { ...$editorPan };
      e.preventDefault();
    }
  }

  function handleMouseMove(e) {
    if (isPanning) {
      editorPan.set({
        x: panOffset.x + (e.clientX - panStart.x),
        y: panOffset.y + (e.clientY - panStart.y),
      });
    }
  }

  function handleMouseUp() {
    isPanning = false;
  }

  async function handleGridResize(e) {
    const { columnSizes, rowSizes } = e.detail;
    await updateGridSizes(columnSizes, rowSizes);
  }

  // Export for use in Beeld tab
  export { zoomToFit };
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="editor"
  bind:this={containerEl}
  on:wheel={handleWheel}
  on:mousedown={handleMouseDown}
  on:mousemove={handleMouseMove}
  on:mouseup={handleMouseUp}
  on:mouseleave={handleMouseUp}
  role="application"
>
  {#if $currentKozijn && $currentGeometry}
    <svg class="canvas" xmlns="http://www.w3.org/2000/svg">
      <g transform="translate({$editorPan.x}, {$editorPan.y}) scale({$zoom})">
        <KozijnCanvas geometry={$currentGeometry} kozijn={$currentKozijn} />
        <GridHandles geometry={$currentGeometry} kozijn={$currentKozijn} on:resize={handleGridResize} />
      </g>
    </svg>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 64 64" fill="none" stroke="var(--scaffold-gray)" stroke-width="1.5">
          <rect x="8" y="8" width="48" height="48" rx="2"/>
          <line x1="32" y1="8" x2="32" y2="56"/>
          <line x1="8" y1="32" x2="56" y2="32"/>
        </svg>
      </div>
      <p>Maak een nieuw kozijn via de ribbon toolbar</p>
      <p class="hint">of druk op Ctrl+N</p>
    </div>
  {/if}
</div>

<style>
  .editor {
    flex: 1 1 0;
    min-width: 0;
    background: var(--editor-bg);
    overflow: hidden;
    position: relative;
    cursor: default;
    background-image:
      radial-gradient(var(--editor-grid) 1px, transparent 1px);
    background-size: 20px 20px;
  }

  .canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transform-origin: 0 0;
    overflow: visible;
  }

  .empty-state {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-3);
    color: var(--text-muted);
  }

  .empty-state p {
    font-size: 14px;
  }

  .hint {
    font-size: 12px !important;
    opacity: 0.6;
  }
</style>
