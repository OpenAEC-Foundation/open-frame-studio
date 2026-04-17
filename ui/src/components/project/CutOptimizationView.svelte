<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let result = null;
  let loading = false;
  let stockLength = 5800;

  onMount(optimize);

  async function optimize() {
    loading = true;
    try {
      result = await invoke("optimize_project_cut_list", { stockLengthMm: stockLength });
    } catch (e) {
      console.error("Zaagoptimalisatie mislukt:", e);
    }
    loading = false;
  }

  $: bars = result?.bars || [];
  $: totalBars = result?.totalBars || bars.length;
  $: wastePercent = result?.wastePercent || 0;
  $: totalWasteMm = result?.totalWasteMm || 0;

  function cutColor(i) {
    const colors = ["#3b82f6", "#f59e0b", "#22c55e", "#8b5cf6", "#06b6d4", "#ec4899", "#f97316", "#14b8a6"];
    return colors[i % colors.length];
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>Zaagoptimalisatie</h2>
    <div class="toolbar-actions">
      <label class="param">
        <span>Staaflengte</span>
        <input type="number" bind:value={stockLength} min="1000" max="12000" step="100" />
        <span>mm</span>
      </label>
      <button class="action-btn primary" onclick={optimize}>Optimaliseren</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Aantal staven</span>
      <span class="summary-value">{totalBars}</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Verschnitt</span>
      <span class="summary-value" class:waste-good={wastePercent < 10} class:waste-ok={wastePercent >= 10 && wastePercent < 20} class:waste-poor={wastePercent >= 20}>
        {wastePercent.toFixed(1)}%
      </span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Totaal afval</span>
      <span class="summary-value">{totalWasteMm.toFixed(0)} mm</span>
    </div>
  </div>

  {#if loading}
    <p class="hint">Optimaliseren...</p>
  {:else if bars.length === 0}
    <p class="hint">Geen zaaggegevens beschikbaar. Voeg kozijnen toe aan het project.</p>
  {:else}
    <div class="bars-container">
      {#each bars as bar, barIndex}
        <div class="bar-row">
          <span class="bar-label">Staaf {barIndex + 1}</span>
          <div class="bar-visual">
            {#each (bar.cuts || []) as cut, cutIndex}
              <div
                class="cut-block"
                style="width:{(cut.length / stockLength) * 100}%;background:{cutColor(cutIndex)}"
                title="{cut.mark || ''}: {cut.length}mm ({cut.profile || ''})"
              >
                <span class="cut-text">{cut.length}</span>
              </div>
            {/each}
            {#if bar.waste > 0}
              <div
                class="waste-block"
                style="width:{(bar.waste / stockLength) * 100}%"
                title="Verschnitt: {bar.waste}mm"
              >
                <span class="cut-text">{bar.waste}</span>
              </div>
            {/if}
          </div>
          <span class="bar-waste">{bar.waste || 0}mm afval</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); flex-wrap: wrap; gap: var(--sp-2); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-3); align-items: center; }
  .param { display: flex; align-items: center; gap: var(--sp-1); font-size: 11px; color: var(--text-muted); }
  .param input { width: 64px; padding: var(--sp-1) var(--sp-2); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; text-align: center; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-3); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .waste-good { color: #22c55e; }
  .waste-ok { color: #f59e0b; }
  .waste-poor { color: #ef4444; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }

  .bars-container { flex: 1; overflow: auto; display: flex; flex-direction: column; gap: var(--sp-2); }
  .bar-row { display: flex; align-items: center; gap: var(--sp-2); }
  .bar-label { font-size: 11px; font-weight: 600; color: var(--text-muted); min-width: 56px; }
  .bar-visual { flex: 1; display: flex; height: 28px; border-radius: 3px; overflow: hidden; background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); }
  .cut-block { display: flex; align-items: center; justify-content: center; overflow: hidden; }
  .waste-block { display: flex; align-items: center; justify-content: center; background: repeating-linear-gradient(45deg, transparent, transparent 3px, rgba(255,255,255,0.05) 3px, rgba(255,255,255,0.05) 6px); overflow: hidden; }
  .cut-text { font-size: 9px; font-weight: 600; color: #fff; text-shadow: 0 0 2px rgba(0,0,0,0.5); white-space: nowrap; }
  .bar-waste { font-size: 11px; color: var(--text-muted); min-width: 80px; text-align: right; }
</style>
