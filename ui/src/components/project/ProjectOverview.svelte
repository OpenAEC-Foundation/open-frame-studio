<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { kozijnen } from "../../stores/project.js";
  import { selectKozijn, removeKozijn, currentKozijn } from "../../stores/kozijn.js";
  import {
    vliesgevels,
    loadVliesgevels,
    selectVliesgevel,
    removeVliesgevel,
    currentVliesgevel,
    clearVliesgevelSelection,
  } from "../../stores/vliesgevel.js";
  import { panelTypeSummary } from "../../lib/labels.js";
  import { vullingLabel } from "../../lib/layout.js";

  onMount(loadVliesgevels);

  // Bij een vrije indeling beschrijft de boom de vakken; de matrix-cellen zijn
  // dan vestigiaal (die zeiden "1x Vast glas" bij een melkmeisje met deur).
  function layoutCounts(node, acc = {}) {
    if (!node) return acc;
    if (node.kind === "leaf") {
      if (node.vulling?.type !== "buiten") {
        const lbl = vullingLabel(node.vulling);
        acc[lbl] = (acc[lbl] || 0) + 1;
      }
      return acc;
    }
    for (const c of node.children || []) layoutCounts(c.node, acc);
    return acc;
  }

  function cellSummary(kozijn) {
    if (kozijn.layout) {
      return Object.entries(layoutCounts(kozijn.layout))
        .map(([lbl, n]) => `${n}x ${lbl}`)
        .join(", ");
    }
    return panelTypeSummary($_, kozijn.cells);
  }

  function openKozijn(id) {
    // Clear the vliesgevel selection so the editor swaps back to the kozijn editor.
    clearVliesgevelSelection();
    selectKozijn(id);
  }
</script>

<div class="overview">
  <h3>{$_('project.kozijnen')}</h3>
  {#if !$kozijnen || $kozijnen.length === 0}
    <p class="empty">{$_('project.noKozijnen')}</p>
  {:else}
    <div class="list">
      {#each $kozijnen as kozijn}
        <div
          class="card"
          class:active={!$currentVliesgevel && $currentKozijn?.id === kozijn.id}
          onclick={() => openKozijn(kozijn.id)}
          role="button"
          tabindex="0"
        >
          <div class="card-header">
            <span class="mark">{kozijn.mark}</span>
            <span class="name">{kozijn.name}</span>
            <button
              class="delete-btn"
              onclick={(e) => { e.stopPropagation(); removeKozijn(kozijn.id); }}
              title={$_('project.delete')}
            >x</button>
          </div>
          <div class="card-info">
            <span>{kozijn.frame.outerWidth} x {kozijn.frame.outerHeight} mm</span>
            <span class="cells">{cellSummary(kozijn)}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if $vliesgevels && $vliesgevels.length > 0}
    <h3 class="section-title">{$_('vliesgevel.title')}</h3>
    <div class="list">
      {#each $vliesgevels as vg (vg.id)}
        <div
          class="card"
          class:active={$currentVliesgevel?.id === vg.id}
          onclick={() => selectVliesgevel(vg.id)}
          role="button"
          tabindex="0"
        >
          <div class="card-header">
            <span class="mark">{vg.mark}</span>
            <span class="name">{vg.name}</span>
            <button
              class="delete-btn"
              onclick={(e) => { e.stopPropagation(); removeVliesgevel(vg.id); }}
              title={$_('project.delete')}
            >x</button>
          </div>
          <div class="card-info">
            <span>{Math.round(vg.overallWidth)} x {Math.round(vg.overallHeight)} mm</span>
            <span class="cells">{vg.panels.length} {$_('vliesgevel.panels')}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .overview {
    width: 100%;
    height: 100%;
    background: var(--bg-surface);
    overflow-y: auto;
    padding: 8px 0;
  }

  h3 {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: 0;
    padding: 4px 10px 6px;
  }

  .section-title {
    margin-top: 12px;
    border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
    padding-top: 10px;
  }

  .empty {
    color: var(--text-muted);
    font-size: 12px;
    text-align: center;
    padding: 24px 10px;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .card {
    text-align: left;
    padding: 6px 10px;
    background: transparent;
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
  }

  .card:hover {
    background: var(--bg-surface-alt);
  }

  .card.active {
    background: var(--bg-surface-alt);
    border-left-color: var(--amber);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 2px;
  }

  .mark {
    background: var(--amber);
    color: var(--night-build);
    padding: 0 5px;
    border-radius: 2px;
    font-size: 10px;
    font-weight: 700;
    line-height: 16px;
  }

  .name {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .delete-btn {
    width: 18px;
    height: 18px;
    font-size: 11px;
    color: var(--text-muted);
    border-radius: 2px;
    display: none;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .card:hover .delete-btn {
    display: flex;
  }

  .delete-btn:hover {
    background: var(--error);
    color: white;
  }

  .card-info {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-muted);
    padding-left: 1px;
  }

  .cells {
    color: var(--text-secondary);
  }
</style>
