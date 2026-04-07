<script>
  import { currentKozijn, currentGeometry, selectedCellIndex, updateGridSizes, addColumn, addRow } from "../../stores/kozijn.js";
  import { zoom, editorPan } from "../../stores/ui.js";
  import { _ } from "svelte-i18n";
  import KozijnCanvas from "./KozijnCanvas.svelte";
  import GridHandles from "./GridHandles.svelte";
  import CellContextMenu from "./CellContextMenu.svelte";
  import { get } from "svelte/store";

  // Context menu state
  let contextMenu = $state({ visible: false, cellIndex: 0, x: 0, y: 0 });

  // Split preview state
  let splitPreview = $state(null); // { x, y1, y2 } for vertical or { y, x1, x2 } for horizontal

  let containerEl = $state(null);
  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0 });
  let panOffset = $state({ x: 0, y: 0 });

  // Auto zoom-to-fit when kozijn changes
  $effect(() => {
    if ($currentGeometry && containerEl) {
      requestAnimationFrame(() => zoomToFit());
    }
  });

  function zoomToFit() {
    if (!containerEl || !$currentGeometry) return;
    const rect = containerEl.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const outer = $currentGeometry.outerRect;
    // Add padding for dimension lines (3 levels of offset + labels)
    const contentW = outer.width + 160;
    const contentH = outer.height + 160;

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
    const rect = containerEl.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const oldZoom = $zoom;
    const delta = e.deltaY > 0 ? -0.03 : 0.03;
    const newZoom = Math.max(0.05, Math.min(2.0, oldZoom + delta));

    // Adjust pan so the point under the cursor stays fixed
    const scale = newZoom / oldZoom;
    editorPan.update((pan) => ({
      x: mouseX - (mouseX - pan.x) * scale,
      y: mouseY - (mouseY - pan.y) * scale,
    }));

    zoom.set(newZoom);
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

    // Show split preview when Ctrl is held
    const k = get(currentKozijn);
    const geom = get(currentGeometry);
    if (k && geom && containerEl && e.ctrlKey) {
      const rect = containerEl.getBoundingClientRect();
      const z = get(zoom);
      const pan = get(editorPan);
      const mx = (e.clientX - rect.left - pan.x) / z;
      const my = (e.clientY - rect.top - pan.y) / z;
      const fw = k.frame.frameWidth;
      const ow = k.frame.outerWidth;
      const oh = k.frame.outerHeight;

      if (mx > fw && mx < ow - fw && my > fw && my < oh - fw) {
        if (e.altKey) {
          // Horizontal split preview
          splitPreview = { type: "h", y: my, x1: fw, x2: ow - fw };
        } else {
          // Vertical split preview
          splitPreview = { type: "v", x: mx, y1: fw, y2: oh - fw };
        }
      } else {
        splitPreview = null;
      }
    } else if (!e.ctrlKey) {
      splitPreview = null;
    }
  }

  function handleMouseUp() {
    isPanning = false;
  }

  async function handleGridResize({ columnSizes, rowSizes }) {
    await updateGridSizes(columnSizes, rowSizes);
  }

  function handleCellContextMenu(cellIndex, screenX, screenY) {
    contextMenu = { visible: true, cellIndex, x: screenX, y: screenY };
  }

  function closeContextMenu() {
    contextMenu = { ...contextMenu, visible: false };
  }

  // Ctrl+click or double-click on canvas to add a column or row at click position
  function handleCanvasClick(e) {
    if (e.ctrlKey && !e.altKey) {
      // Ctrl+Click = add vertical column
      addDividerAtMouse(e, false);
    } else if (e.ctrlKey && e.altKey) {
      // Ctrl+Alt+Click = add horizontal row
      addDividerAtMouse(e, true);
    }
  }

  function addDividerAtMouse(e, horizontal) {
    const k = get(currentKozijn);
    if (!k || !containerEl) return;

    const rect = containerEl.getBoundingClientRect();
    const z = get(zoom);
    const pan = get(editorPan);
    const mx = (e.clientX - rect.left - pan.x) / z;
    const my = (e.clientY - rect.top - pan.y) / z;
    const fw = k.frame.frameWidth;
    const ow = k.frame.outerWidth;
    const oh = k.frame.outerHeight;

    if (mx < fw || mx > ow - fw || my < fw || my > oh - fw) return;

    if (horizontal) {
      addRow(my);
    } else {
      addColumn(mx);
    }
    splitPreview = null;
  }

  // Double-click on canvas to add a column or row at click position
  function handleCanvasDblClick(e) {
    addDividerAtMouse(e, e.altKey);
  }

  // Export for use in Beeld tab
  export { zoomToFit };
</script>

<div
  class="editor"
  bind:this={containerEl}
  onwheel={handleWheel}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseUp}
  role="application"
>
  {#if $currentKozijn && $currentGeometry}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <svg class="canvas" xmlns="http://www.w3.org/2000/svg" ondblclick={handleCanvasDblClick} onclick={handleCanvasClick}>
      <g transform="translate({$editorPan.x}, {$editorPan.y}) scale({$zoom})">
        <KozijnCanvas geometry={$currentGeometry} kozijn={$currentKozijn} zoom={$zoom} oncellcontextmenu={handleCellContextMenu} />
        <GridHandles geometry={$currentGeometry} kozijn={$currentKozijn} onresize={handleGridResize} />

        <!-- Split preview line (Ctrl+hover) -->
        {#if splitPreview}
          {#if splitPreview.type === "v"}
            <line
              x1={splitPreview.x} y1={splitPreview.y1}
              x2={splitPreview.x} y2={splitPreview.y2}
              stroke="var(--amber, #D97706)" stroke-width={2 / $zoom}
              stroke-dasharray="{6 / $zoom} {4 / $zoom}"
              opacity="0.7" pointer-events="none"
            />
            <text x={splitPreview.x} y={splitPreview.y1 - 4 / $zoom}
              text-anchor="middle" fill="var(--amber)" font-size={10 / $zoom}
              font-family="var(--font-body)" pointer-events="none">
              {Math.round(splitPreview.x)}mm
            </text>
          {:else}
            <line
              x1={splitPreview.x1} y1={splitPreview.y}
              x2={splitPreview.x2} y2={splitPreview.y}
              stroke="var(--amber, #D97706)" stroke-width={2 / $zoom}
              stroke-dasharray="{6 / $zoom} {4 / $zoom}"
              opacity="0.7" pointer-events="none"
            />
            <text x={splitPreview.x2 + 4 / $zoom} y={splitPreview.y}
              text-anchor="start" dominant-baseline="middle"
              fill="var(--amber)" font-size={10 / $zoom}
              font-family="var(--font-body)" pointer-events="none">
              {Math.round(splitPreview.y)}mm
            </text>
          {/if}
        {/if}
      </g>
    </svg>
    <CellContextMenu
      visible={contextMenu.visible}
      cellIndex={contextMenu.cellIndex}
      screenX={contextMenu.x}
      screenY={contextMenu.y}
      onclose={closeContextMenu}
    />
  {:else}
    <div class="empty-state">
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 64 64" fill="none" stroke="var(--scaffold-gray)" stroke-width="1.5">
          <rect x="8" y="8" width="48" height="48" rx="2"/>
          <line x1="32" y1="8" x2="32" y2="56"/>
          <line x1="8" y1="32" x2="56" y2="32"/>
        </svg>
      </div>
      <p>{$_('editor.newKozijnHint')}</p>
      <p class="hint">{$_('editor.shortcutHint')}</p>
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
