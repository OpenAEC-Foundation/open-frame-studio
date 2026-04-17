<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let energyData = null;
  let loading = false;
  let maxUw = 1.65;

  onMount(loadEnergy);

  async function loadEnergy() {
    loading = true;
    try {
      energyData = await invoke("get_project_energy", { maxUw });
    } catch (e) {
      console.error("Energie laden mislukt:", e);
    }
    loading = false;
  }

  $: items = energyData?.items || [];
  $: avgUw = items.length > 0 ? items.reduce((s, i) => s + (i.uw || 0), 0) / items.length : 0;
  $: compliant = avgUw > 0 && avgUw <= maxUw;
  $: totalArea = items.reduce((s, i) => s + (i.area || 0), 0);
  $: totalLoss = items.reduce((s, i) => s + (i.transmissionLoss || 0), 0);
</script>

<div class="view">
  <div class="toolbar">
    <h2>Energieprestatie (BENG / Bouwbesluit)</h2>
    <div class="toolbar-actions">
      <label class="param">
        <span>Max Uw</span>
        <input type="number" bind:value={maxUw} min="0.5" max="5.0" step="0.05" />
        <span>W/m&sup2;K</span>
      </label>
      <button class="action-btn primary" onclick={loadEnergy}>Herberekenen</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Gem. Uw-waarde</span>
      <span class="summary-value" class:uw-good={compliant} class:uw-poor={!compliant && avgUw > 0}>
        {avgUw > 0 ? avgUw.toFixed(2) : "\u2014"} W/m&sup2;K
      </span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Totaal glasoppervlak</span>
      <span class="summary-value">{totalArea.toFixed(2)} m&sup2;</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Transmissieverlies</span>
      <span class="summary-value">{totalLoss.toFixed(1)} W/K</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Bouwbesluit</span>
      <span class="summary-value">
        {#if avgUw === 0}
          <span class="badge neutral">Geen data</span>
        {:else if compliant}
          <span class="badge pass">Voldoet</span>
        {:else}
          <span class="badge fail">Niet-conform</span>
        {/if}
      </span>
    </div>
  </div>

  {#if loading}
    <p class="hint">Laden...</p>
  {:else if items.length === 0}
    <p class="hint">Voeg kozijnen toe aan het project om de energieprestatie te berekenen.</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Merk</th>
            <th class="num-head">Uw (W/m&sup2;K)</th>
            <th class="num-head">g-waarde</th>
            <th class="num-head">Oppervlak (m&sup2;)</th>
            <th class="num-head">Transmissieverlies (W/K)</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each items as item}
            <tr>
              <td class="mark">{item.mark || "\u2014"}</td>
              <td class="num">
                <span class="uw-val" class:uw-good={item.uw <= maxUw} class:uw-poor={item.uw > maxUw}>
                  {(item.uw || 0).toFixed(2)}
                </span>
              </td>
              <td class="num">{(item.gValue || 0).toFixed(2)}</td>
              <td class="num">{(item.area || 0).toFixed(2)}</td>
              <td class="num">{(item.transmissionLoss || 0).toFixed(1)}</td>
              <td>
                {#if item.uw <= maxUw}
                  <span class="badge pass">OK</span>
                {:else}
                  <span class="badge fail">Te hoog</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); flex-wrap: wrap; gap: var(--sp-2); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-3); align-items: center; }
  .param { display: flex; align-items: center; gap: var(--sp-1); font-size: 11px; color: var(--text-muted); }
  .param input { width: 56px; padding: var(--sp-1) var(--sp-2); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; text-align: center; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-3); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .num-head { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.mark { font-weight: 700; color: var(--amber); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .uw-val { font-weight: 600; }
  .uw-good { color: #16a34a; }
  .uw-poor { color: #dc2626; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .badge.pass { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .badge.fail { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
  .badge.neutral { background: rgba(107, 114, 128, 0.15); color: #6b7280; }
</style>
