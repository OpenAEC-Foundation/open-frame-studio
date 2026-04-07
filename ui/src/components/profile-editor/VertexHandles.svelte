<script>
  import { profileEditor, editorVertices, editorSelectedVertex, editorTool, editorSnap } from "../../stores/profileEditor.js";

  let { zoom = 4, screenToModel = () => ({ x: 0, y: 0 }) } = $props();

  let dragging = $state(null);

  function handleMouseDown(index, e) {
    e.stopPropagation();
    e.preventDefault();

    const tool = $editorTool;
    if (tool === "delete") {
      profileEditor.removeVertex(index);
      return;
    }

    profileEditor.selectVertex(index);
    profileEditor.beginDrag();

    const startX = e.clientX;
    const startY = e.clientY;
    const origVert = { ...$editorVertices[index] };
    const snap = $editorSnap;

    dragging = { index, startX, startY, origVert };

    function onMove(me) {
      const dx = (me.clientX - startX) / zoom;
      const dy = (me.clientY - startY) / zoom;
      let nx = origVert.x + dx;
      let ny = origVert.y + dy;

      if (snap) {
        nx = Math.round(nx);
        ny = Math.round(ny);
      }

      profileEditor.moveVertex(index, nx, ny);
    }

    function onUp() {
      dragging = null;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

{#each $editorVertices as vert, i}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <circle
    cx={vert.x}
    cy={vert.y}
    r={5 / zoom}
    fill={$editorSelectedVertex === i ? "var(--amber, #D97706)" : "var(--bg-surface, #fff)"}
    stroke={$editorSelectedVertex === i ? "var(--amber, #D97706)" : "var(--text-muted, #888)"}
    stroke-width={1.5 / zoom}
    style="cursor: {$editorTool === 'delete' ? 'not-allowed' : 'grab'}"
    onmousedown={(e) => handleMouseDown(i, e)}
  />

  <!-- Vertex coordinate label (only for selected) -->
  {#if $editorSelectedVertex === i}
    <text
      x={vert.x}
      y={vert.y - 8 / zoom}
      text-anchor="middle"
      fill="var(--amber, #D97706)"
      font-size={10 / zoom}
      font-family="JetBrains Mono, monospace"
    >
      {Math.round(vert.x * 10) / 10}, {Math.round(vert.y * 10) / 10}
    </text>
  {/if}
{/each}
