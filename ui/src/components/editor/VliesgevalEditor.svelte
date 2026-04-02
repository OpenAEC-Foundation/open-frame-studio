<script>
  import VliesgevalCanvas from "./VliesgevalCanvas.svelte";
  import { currentVliesgevel, currentVgGeometry } from "../../stores/vliesgevel.js";
  import { zoom, editorPan } from "../../stores/ui.js";
  import { _ } from "svelte-i18n";

  let svgEl;

  $: geom = $currentVgGeometry;

  // Auto zoom-to-fit when geometry changes
  $: if (geom && svgEl) {
    const padding = 120;
    const container = svgEl.parentElement;
    if (container) {
      const cw = container.clientWidth;
      const ch = container.clientHeight;
      const scaleX = (cw - padding * 2) / geom.overallWidth;
      const scaleY = (ch - padding * 2) / geom.overallHeight;
      const newZoom = Math.min(scaleX, scaleY, 1.0);
      zoom.set(newZoom);
      editorPan.set({
        x: (cw - geom.overallWidth * newZoom) / 2,
        y: (ch - geom.overallHeight * newZoom) / 2,
      });
    }
  }

  function handleWheel(e) {
    if (e.ctrlKey) {
      e.preventDefault();
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      zoom.update((z) => Math.max(0.05, Math.min(5, z * delta)));
    }
  }
</script>

<div class="vliesgevel-editor" onwheel={handleWheel}>
  {#if $currentVliesgevel && geom}
    <svg
      bind:this={svgEl}
      class="vg-svg"
      viewBox="{-100} {-100} {geom.overallWidth + 200} {geom.overallHeight + 200}"
      preserveAspectRatio="xMidYMid meet"
    >
      <g transform="translate({$editorPan.x / $zoom}, {$editorPan.y / $zoom}) scale({$zoom})">
        <VliesgevalCanvas />
      </g>
    </svg>
  {:else}
    <div class="placeholder">
      <p>{$_('editor.selectVliesgevel')}</p>
    </div>
  {/if}
</div>

<style>
  .vliesgevel-editor {
    flex: 1;
    overflow: hidden;
    background: var(--bg-canvas, #1a1a2e);
    position: relative;
  }

  .vg-svg {
    width: 100%;
    height: 100%;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
