<script>
  import { get } from "svelte/store";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { zoom } from "../../stores/ui.js";
  import { invoke } from "../../lib/tauri.js";

  let { geometry, kozijn, onresize } = $props();

  const MIN_CELL_SIZE = 100; // minimum 100mm per cell
  const HIT_AREA = 20; // hit area width in SVG units (mm)

  // Drag state
  let dragging = null; // { axis: 'v'|'h', index, startPos, origSizes }

  function startDragV(index, e) {
    e.stopPropagation();
    e.preventDefault();
    const cols = kozijn.grid.columns.map((c) => c.size);
    dragging = {
      axis: "v",
      index,
      startClientX: e.clientX,
      origSizes: cols,
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function startDragH(index, e) {
    e.stopPropagation();
    e.preventDefault();
    const rows = kozijn.grid.rows.map((r) => r.size);
    dragging = {
      axis: "h",
      index,
      startClientY: e.clientY,
      origSizes: rows,
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function handleMouseMove(e) {
    if (!dragging) return;

    const currentZoom = get(zoom);

    if (dragging.axis === "v") {
      const deltaPx = e.clientX - dragging.startClientX;
      const deltaMm = deltaPx / currentZoom;

      const i = dragging.index;
      const leftOrig = dragging.origSizes[i];
      const rightOrig = dragging.origSizes[i + 1];

      let newLeft = leftOrig + deltaMm;
      let newRight = rightOrig - deltaMm;

      // Enforce minimum sizes
      if (newLeft < MIN_CELL_SIZE) {
        newLeft = MIN_CELL_SIZE;
        newRight = leftOrig + rightOrig - MIN_CELL_SIZE;
      }
      if (newRight < MIN_CELL_SIZE) {
        newRight = MIN_CELL_SIZE;
        newLeft = leftOrig + rightOrig - MIN_CELL_SIZE;
      }

      dragging.currentSizes = [...dragging.origSizes];
      dragging.currentSizes[i] = newLeft;
      dragging.currentSizes[i + 1] = newRight;
      dragging = dragging; // trigger reactivity
    } else {
      const deltaPx = e.clientY - dragging.startClientY;
      const deltaMm = deltaPx / currentZoom;

      const i = dragging.index;
      const topOrig = dragging.origSizes[i];
      const bottomOrig = dragging.origSizes[i + 1];

      let newTop = topOrig + deltaMm;
      let newBottom = bottomOrig - deltaMm;

      // Enforce minimum sizes
      if (newTop < MIN_CELL_SIZE) {
        newTop = MIN_CELL_SIZE;
        newBottom = topOrig + bottomOrig - MIN_CELL_SIZE;
      }
      if (newBottom < MIN_CELL_SIZE) {
        newBottom = MIN_CELL_SIZE;
        newTop = topOrig + bottomOrig - MIN_CELL_SIZE;
      }

      dragging.currentSizes = [...dragging.origSizes];
      dragging.currentSizes[i] = newTop;
      dragging.currentSizes[i + 1] = newBottom;
      dragging = dragging; // trigger reactivity
    }
  }

  function handleMouseUp() {
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    if (!dragging || !dragging.currentSizes) {
      dragging = null;
      return;
    }

    if (dragging.axis === "v") {
      onresize?.({
        columnSizes: dragging.currentSizes,
        rowSizes: kozijn.grid.rows.map((r) => r.size),
      });
    } else {
      onresize?.({
        columnSizes: kozijn.grid.columns.map((c) => c.size),
        rowSizes: dragging.currentSizes,
      });
    }

    dragging = null;
  }

  // Compute drag indicator position for vertical divider
  function dragVPos(dragging) {
    if (!dragging || dragging.axis !== "v" || !dragging.currentSizes) return null;
    const fw = kozijn.frame.frameWidth;
    const dw = fw; // divider width matches frame width
    let x = fw;
    for (let i = 0; i <= dragging.index; i++) {
      x += dragging.currentSizes[i];
      if (i < dragging.index) x += dw;
    }
    return x + dw / 2;
  }

  // Compute drag indicator position for horizontal divider
  function dragHPos(dragging) {
    if (!dragging || dragging.axis !== "h" || !dragging.currentSizes) return null;
    const fw = kozijn.frame.frameWidth;
    const dw = fw;
    let y = fw;
    for (let i = 0; i <= dragging.index; i++) {
      y += dragging.currentSizes[i];
      if (i < dragging.index) y += dw;
    }
    return y + dw / 2;
  }
</script>

<g class="grid-handles">
  <!-- Vertical divider handles -->
  {#each geometry.vDividers as div, i}
    {@const cx = div.x + div.width / 2}
    <rect
      x={cx - HIT_AREA / 2}
      y={div.y}
      width={HIT_AREA}
      height={div.height}
      fill="transparent"
      class="handle handle-v"
      pointer-events="all"
      onmousedown={(e) => startDragV(i, e)}
    />
  {/each}

  <!-- Horizontal divider handles -->
  {#each geometry.hDividers as div, i}
    {@const cy = div.y + div.height / 2}
    <rect
      x={div.x}
      y={cy - HIT_AREA / 2}
      width={div.width}
      height={HIT_AREA}
      fill="transparent"
      class="handle handle-h"
      pointer-events="all"
      onmousedown={(e) => startDragH(i, e)}
    />
  {/each}

  <!-- Drag indicator lines -->
  {#if dragging && dragging.axis === "v" && dragging.currentSizes}
    {@const xPos = dragVPos(dragging)}
    {#if xPos !== null}
      <line
        x1={xPos}
        y1={geometry.outerRect.y}
        x2={xPos}
        y2={geometry.outerRect.y + geometry.outerRect.height}
        stroke="#F59E0B"
        stroke-width="2"
        stroke-dasharray="6 4"
        pointer-events="none"
      />
    {/if}
  {/if}

  {#if dragging && dragging.axis === "h" && dragging.currentSizes}
    {@const yPos = dragHPos(dragging)}
    {#if yPos !== null}
      <line
        x1={geometry.outerRect.x}
        y1={yPos}
        x2={geometry.outerRect.x + geometry.outerRect.width}
        y2={yPos}
        stroke="#F59E0B"
        stroke-width="2"
        stroke-dasharray="6 4"
        pointer-events="none"
      />
    {/if}
  {/if}
</g>

<style>
  .handle-v {
    cursor: default;
  }

  .handle-h {
    cursor: default;
  }

  .handle:hover {
    fill: rgba(245, 158, 11, 0.15);
  }
</style>
