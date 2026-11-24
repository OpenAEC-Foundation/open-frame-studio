<script>
  import { _ } from "svelte-i18n";
  import BlueprintIcon from "../shell/BlueprintIcon.svelte";
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

{#snippet projectItem(mark, name, width, height, summary, active, select, remove)}
  <div class="item">
    <button class="card" class:active aria-pressed={active} onclick={select}>
      <span class="card-header">
        <span class="mark">{mark}</span>
        <span class="name">{name}</span>
      </span>
      <span class="card-info">
        <span>{width} × {height} mm</span>
        <span class="cells">{summary}</span>
      </span>
    </button>
    <button class="delete-btn" onclick={remove} title={$_("project.delete") + " · " + mark}
      aria-label={$_("project.delete") + " · " + mark}>
      <BlueprintIcon name="edit.delete" />
    </button>
  </div>
{/snippet}

<div class="overview">
  <h3>{$_('project.kozijnen')}</h3>
  {#if !$kozijnen || $kozijnen.length === 0}
    <p class="empty">{$_('project.noKozijnen')}</p>
  {:else}
    <div class="list">
      {#each $kozijnen as kozijn (kozijn.id)}
        {@render projectItem(kozijn.mark, kozijn.name, kozijn.frame.outerWidth, kozijn.frame.outerHeight,
          cellSummary(kozijn), !$currentVliesgevel && $currentKozijn?.id === kozijn.id,
          () => openKozijn(kozijn.id), () => removeKozijn(kozijn.id))}
      {/each}
    </div>
  {/if}

  {#if $vliesgevels && $vliesgevels.length > 0}
    <h3 class="section-title">{$_('vliesgevel.title')}</h3>
    <div class="list">
      {#each $vliesgevels as vg (vg.id)}
        {@render projectItem(vg.mark, vg.name, Math.round(vg.overallWidth), Math.round(vg.overallHeight),
          vg.panels.length + " " + $_("vliesgevel.panels"), $currentVliesgevel?.id === vg.id,
          () => selectVliesgevel(vg.id), () => removeVliesgevel(vg.id))}
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

  .item { position: relative; }

  .card {
    text-align: left;
    padding: 6px 28px 6px 10px;
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
    background: var(--oa-selected);
    border-left-color: var(--oa-selected-bar);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 2px;
  }

  .mark {
    background: var(--oa-frame);
    color: var(--oa-text);
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
    position: absolute;
    top: 6px;
    right: 4px;
    opacity: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .item:hover .delete-btn, .item:focus-within .delete-btn { opacity: 1; }

  .delete-btn:hover {
    background: var(--error);
    color: white;
  }

  .card-info {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-muted);
    gap: 4px;
    flex-wrap: wrap;
    font-weight: 400;
    padding-left: 1px;
  }

  .cells {
    color: var(--text-secondary);
  }
</style>
