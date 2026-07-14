<script>
  import { currentKozijn, currentGeometry, selectedCellIndex, updateGridSizes, addColumn, addRow, setKozijnLayout } from "../../stores/kozijn.js";
  import { zoom, editorPan } from "../../stores/ui.js";
  import { _ } from "svelte-i18n";
  import KozijnCanvas from "./KozijnCanvas.svelte";
  import GridHandles from "./GridHandles.svelte";
  import CellContextMenu from "./CellContextMenu.svelte";
  import { get } from "svelte/store";
  import { layoutToRects, splitLeafAt, ensureIds } from "../../lib/layout.js";

  // Leaf-hittest op de vrije indeling: welk vak ligt onder (mx,my) in model-mm?
  function layoutLeafAt(k, mx, my) {
    if (!k?.layout) return null;
    const fw = k.frame.frameWidth || 67;
    const inner = { x: fw, y: fw, width: (k.frame.outerWidth || 0) - 2 * fw, height: (k.frame.outerHeight || 0) - 2 * fw };
    const tree = ensureIds(k.layout);
    const geom = layoutToRects(tree, inner, fw);
    const leaf = geom.leaves.find((l) =>
      l.node.vulling?.type !== "buiten" &&
      mx >= l.rect.x && mx <= l.rect.x + l.rect.width &&
      my >= l.rect.y && my <= l.rect.y + l.rect.height);
    return leaf ? { tree, leaf } : null;
  }

  // Context menu state
  let contextMenu = $state({ visible: false, cellIndex: 0, x: 0, y: 0 });

  // Split preview state
  let splitPreview = $state(null); // { x, y1, y2 } for vertical or { y, x1, x2 } for horizontal

  let containerEl = $state(null);
  let isPanning = $state(false);
  let panStart = $state({ x: 0, y: 0 });
  let panOffset = $state({ x: 0, y: 0 });

  // Auto zoom-to-fit only when a *different* kozijn is loaded — NOT on every
  // geometry recompute. Otherwise routine edits (splits, resizes, cell changes)
  // retrigger this effect and reset zoom/pan, making the canvas visibly jump.
  let lastFitId = null;
  $effect(() => {
    const k = $currentKozijn;
    const geom = $currentGeometry;
    if (geom && containerEl && k && k.id !== lastFitId) {
      lastFitId = k.id;
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
        // Op een vrije-indeling-kozijn splitst Ctrl+klik alleen het vak onder de
        // cursor — clamp de preview-lijn dan ook op dat vak i.p.v. de hele dag.
        const hit = layoutLeafAt(k, mx, my);
        const b = hit ? hit.leaf.rect : { x: fw, y: fw, width: ow - 2 * fw, height: oh - 2 * fw };
        if (k.layout && !hit) {
          splitPreview = null;
        } else if (e.altKey) {
          // Horizontal split preview
          splitPreview = { type: "h", y: my, x1: b.x, x2: b.x + b.width };
        } else {
          // Vertical split preview
          splitPreview = { type: "v", x: mx, y1: b.y, y2: b.y + b.height };
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

  // Ctrl+click on canvas to add a column or row at click position
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

    if (k.layout) {
      // Vrije indeling: splits het vak onder de cursor op de klikpositie
      // (WH Okna-patroon) — de matrix-commando's zouden hier het onzichtbare
      // grid muteren zonder zichtbaar effect.
      const hit = layoutLeafAt(k, mx, my);
      if (!hit) return;
      const { tree, leaf } = hit;
      const ratio = horizontal
        ? (my - leaf.rect.y) / Math.max(1, leaf.rect.height)
        : (mx - leaf.rect.x) / Math.max(1, leaf.rect.width);
      setKozijnLayout(splitLeafAt(tree, leaf.node.id, horizontal ? "column" : "row", ratio));
    } else if (horizontal) {
      addRow(my);
    } else {
      addColumn(mx);
    }
    splitPreview = null;
  }

  // Double-click is intentionally a no-op. It previously inserted a grid
  // divider at the cursor — surprising when the user just meant to select a vak,
  // and in vrije-indeling mode it mutated the hidden grid (not the visible
  // split-tree), so "nothing" happened visibly while state/history churned.
  // Dividers are added deliberately via Ctrl+click / Ctrl+Alt+click, or via the
  // split buttons on a selected vak.
  function handleCanvasDblClick() {}

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
        <!-- GridHandles werkt op kozijn.grid en hoort niet op een vrije-indeling-
             kozijn: zijn hit-rects vangen de divider-drag van het canvas af en
             zijn resize leest het vestigiale 1×1-grid (NaN-gridcorruptie). -->
        {#if !$currentKozijn.layout}
          <GridHandles geometry={$currentGeometry} kozijn={$currentKozijn} onresize={handleGridResize} />
        {/if}

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
