<script>
  let { direction = "horizontal", onresize = null } = $props();

  let dragging = false;
  let startPos = 0;

  function onMouseDown(e) {
    dragging = true;
    startPos = direction === "horizontal" ? e.clientX : e.clientY;
    e.preventDefault();
  }

  function onMouseMove(e) {
    if (!dragging) return;
    const current = direction === "horizontal" ? e.clientX : e.clientY;
    const delta = current - startPos;
    startPos = current;
    onresize?.(delta);
  }

  function onMouseUp() {
    dragging = false;
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} />

<div
  class="resize-handle"
  class:horizontal={direction === "horizontal"}
  class:vertical={direction === "vertical"}
  class:active={dragging}
  onmousedown={onMouseDown}
  role="separator"
></div>

<style>
  .resize-handle {
    flex-shrink: 0;
    background: transparent;
    transition: background 0.15s;
    z-index: 5;
  }

  .resize-handle.horizontal {
    width: 4px;
    cursor: col-resize;
  }

  .resize-handle.vertical {
    height: 4px;
    cursor: row-resize;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background: var(--amber);
  }
</style>
