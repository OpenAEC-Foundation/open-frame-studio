<script>
  import { _ } from "svelte-i18n";
  import { kozijnen } from "../../stores/project.js";
  import { selectKozijn, removeKozijn, currentKozijn } from "../../stores/kozijn.js";
  import { panelTypeSummary } from "../../lib/labels.js";

  function cellSummary(kozijn) {
    return panelTypeSummary($_, kozijn.cells);
  }
</script>

<div class="overview">
  <h3>{$_('project.kozijnen')}</h3>
  {#if $kozijnen.length === 0}
    <p class="empty">{$_('project.noKozijnen')}</p>
  {:else}
    <div class="list">
      {#each $kozijnen as kozijn}
        <div
          class="card"
          class:active={$currentKozijn?.id === kozijn.id}
          onclick={() => selectKozijn(kozijn.id)}
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
