<script>
  import { profileEditor, editorVertices, editorTool } from "../../stores/profileEditor.js";

  let { zoom = 4, screenToModel = () => ({ x: 0, y: 0 }) } = $props();

  function handleDblClick(edgeIndex, e) {
    e.stopPropagation();
    e.preventDefault();

    const verts = $editorVertices;
    const i = edgeIndex;
    const j = (i + 1) % verts.length;
    const p1 = verts[i];
    const p2 = verts[j];

    // Project mouse position onto the edge segment
    const mouse = screenToModel(e.clientX, e.clientY);
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const lenSq = dx * dx + dy * dy;

    let t = 0.5;
    if (lenSq > 0) {
      t = ((mouse.x - p1.x) * dx + (mouse.y - p1.y) * dy) / lenSq;
      t = Math.max(0.1, Math.min(0.9, t));
    }

    const nx = Math.round((p1.x + dx * t) * 10) / 10;
    const ny = Math.round((p1.y + dy * t) * 10) / 10;

    profileEditor.addVertex(i, nx, ny);
  }

  let edges = $derived.by(() => {
    const verts = $editorVertices;
    if (verts.length < 2) return [];
    return verts.map((v, i) => {
      const j = (i + 1) % verts.length;
      return {
        x1: v.x,
        y1: v.y,
        x2: verts[j].x,
        y2: verts[j].y,
        index: i,
      };
    });
  });
</script>

{#each edges as edge}
  <!-- Invisible wide hit area for double-click -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <line
    x1={edge.x1}
    y1={edge.y1}
    x2={edge.x2}
    y2={edge.y2}
    stroke="transparent"
    stroke-width={12 / zoom}
    style="cursor: crosshair"
    ondblclick={(e) => handleDblClick(edge.index, e)}
  />

  <!-- Midpoint indicator on hover (CSS-driven) -->
  <circle
    cx={(edge.x1 + edge.x2) / 2}
    cy={(edge.y1 + edge.y2) / 2}
    r={3 / zoom}
    fill="none"
    stroke="var(--text-muted, #888)"
    stroke-width={0.8 / zoom}
    stroke-dasharray="{2 / zoom} {1.5 / zoom}"
    opacity="0.3"
    style="pointer-events: none"
  />
{/each}
