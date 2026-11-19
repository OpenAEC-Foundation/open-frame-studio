<script>
  import { get } from "svelte/store";
  import { selectDividerPiece } from "../../stores/kozijn.js";
  import { contourPath } from "../../lib/contour2d.js";
  import { dividerPieceLabel } from "../../lib/labels.js";
  import { gridRowHeights } from "../../lib/layout.js";
  import { _ } from "svelte-i18n";
  import { zoom } from "../../stores/ui.js";

  let { geometry, kozijn, onresize } = $props();

  const MIN_CELL_SIZE = 100; // minimum 100mm per cell
  const HIT_AREA = 20; // hit area width in SVG units (mm)

  // Drag state
  let dragging = $state(null); // { axis: 'v'|'h', index, startPos, origSizes }

  function startDragV(index, e, piece = null) {
    if (e.ctrlKey) return;
    e.stopPropagation();
    e.preventDefault();
    const cols = kozijn.grid.columns.map((c) => c.size);
    dragging = {
      axis: "v", piece,
      index,
      startClientX: e.clientX,
      origSizes: cols,
    };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function startDragH(index, e, piece = null) {
    if (e.ctrlKey) return;
    e.stopPropagation();
    e.preventDefault();
    const fromDay = !!piece;
    const rows = gridRowHeights(kozijn, fromDay ? geometry : null);
    dragging = {
      axis: "h", piece, fromDay,
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
      if (!dragging.currentSizes && Math.abs(deltaPx) < 2) return;
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
      if (!dragging.currentSizes && Math.abs(deltaPx) < 2) return;
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
      if (dragging?.piece) selectDividerPiece(dragging.piece);
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
        rowsFromDay: dragging.fromDay,
      });
    }

    dragging = null;
  }

  // Compute drag indicator position for vertical divider
  function dragVPos(dragging) {
    if (!dragging || dragging.axis !== "v" || !dragging.currentSizes) return null;
    const i = dragging.index, bar = geometry.vDividers[i];
    return bar.x + bar.width / 2 + dragging.currentSizes[i] - dragging.origSizes[i];
  }

  // Compute drag indicator position for horizontal divider
  function dragHPos(dragging) {
    if (!dragging || dragging.axis !== "h" || !dragging.currentSizes) return null;
    const i = dragging.index, bar = geometry.hDividers[i];
    return bar.y + bar.height / 2 + dragging.currentSizes[i] - dragging.origSizes[i];
  }
</script>

<g class="grid-handles">
  {#if geometry.curvedFrame && geometry.curvedDividerTimber}
    {#each geometry.curvedDividerTimber as piece}
      {@const label = dividerPieceLabel($_, piece, geometry.curvedDividerTimber)}
      <path d={contourPath(piece.outline)} fill="transparent" class="handle divider-piece-hit"
        class:handle-v={piece.vertical} class:handle-h={!piece.vertical}
        role="button" tabindex="0" aria-label={label}
        onmousedown={(e) => piece.vertical ? startDragV(piece.sourceIndex, e, piece) : startDragH(piece.sourceIndex, e, piece)}
        onkeydown={(e) => e.key === "Enter" && selectDividerPiece(piece)} />
    {/each}
  {:else}
  <!-- Vertical divider handles -->
  {#each geometry.vDividers as div, i}
    {@const cx = div.x + div.width / 2}
    <rect
      x={cx - HIT_AREA / 2}
      y={geometry.curvedFrame ? geometry.outerRect.y : div.y}
      width={HIT_AREA}
      height={geometry.curvedFrame ? geometry.outerRect.height : div.height}
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
  {/if}

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
  .divider-piece-hit.handle-v { cursor: col-resize; }
  .divider-piece-hit.handle-h { cursor: row-resize; }

  .handle-h {
    cursor: default;
  }

  .handle:hover {
    fill: rgba(245, 158, 11, 0.15);
  }
</style>
