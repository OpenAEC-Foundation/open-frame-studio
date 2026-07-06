<script>
  import VliesgevelCanvas from "./VliesgevelCanvas.svelte";
  import { currentVliesgevel, currentVgGeometry, addMullion, addTransom, removeMullion, removeTransom } from "../../stores/vliesgevel.js";
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
    <div class="vg-toolbar">
      <button onclick={addMullion} title={$_('editor.vgAddMullionHint')}>{$_('editor.vgAddMullion')}</button>
      <button onclick={removeMullion} disabled={!$currentVliesgevel.mullions.length} title={$_('editor.vgRemoveMullionHint')}>{$_('editor.vgRemoveMullion')}</button>
      <span class="sep"></span>
      <button onclick={addTransom} title={$_('editor.vgAddTransomHint')}>{$_('editor.vgAddTransom')}</button>
      <button onclick={removeTransom} disabled={!$currentVliesgevel.transoms.length} title={$_('editor.vgRemoveTransomHint')}>{$_('editor.vgRemoveTransom')}</button>
    </div>
    <svg
      bind:this={svgEl}
      class="vg-svg"
      viewBox="{-100} {-100} {geom.overallWidth + 200} {geom.overallHeight + 200}"
      preserveAspectRatio="xMidYMid meet"
    >
      <g transform="translate({$editorPan.x / $zoom}, {$editorPan.y / $zoom}) scale({$zoom})">
        <VliesgevelCanvas />
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

  .vg-toolbar {
    position: absolute;
    top: var(--sp-3, 12px);
    left: var(--sp-3, 12px);
    z-index: 5;
    display: flex;
    gap: var(--sp-2, 8px);
    align-items: center;
  }

  .vg-toolbar button {
    padding: var(--sp-1, 4px) var(--sp-3, 12px);
    background: var(--bg-surface-alt, #2a2a38);
    color: var(--text-primary, #e8e8f0);
    border: 1px solid var(--border-color, #333);
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    font-weight: 600;
    cursor: default;
  }

  .vg-toolbar button:hover:not(:disabled) {
    border-color: var(--amber, #d97706);
  }

  .vg-toolbar button:disabled {
    opacity: 0.4;
  }

  .vg-toolbar .sep {
    width: 1px;
    height: 18px;
    background: var(--border-color, #333);
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
