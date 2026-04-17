<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let plan = null;
  let loading = false;
  let hoursPerDay = 8;
  let workers = 2;

  onMount(loadPlan);

  async function loadPlan() {
    loading = true;
    try {
      plan = await invoke("get_production_plan", { hoursPerDay, workers });
    } catch (e) {
      console.error("Productieplanning laden mislukt:", e);
    }
    loading = false;
  }

  $: jobs = plan?.jobs || [];
  $: totalHours = plan?.totalHours || jobs.reduce((s, j) => s + (j.totalHours || 0), 0);
  $: estimatedDays = plan?.estimatedDays || (totalHours > 0 ? Math.ceil(totalHours / (hoursPerDay * workers)) : 0);
  $: deliveryDate = plan?.deliveryDate || "";

  function phasePercent(phase) {
    if (!phase) return 0;
    return Math.round((phase.completed || 0) / Math.max(phase.total || 1, 1) * 100);
  }

  function phaseColor(name) {
    const colors = { zaag: "#3b82f6", freesmachine: "#8b5cf6", assemblage: "#f59e0b", beglazing: "#06b6d4", afwerking: "#22c55e" };
    return colors[name] || "#6b7280";
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>Productieplan</h2>
    <div class="toolbar-actions">
      <label class="param">
        <span>Uren/dag</span>
        <input type="number" bind:value={hoursPerDay} min="1" max="24" />
      </label>
      <label class="param">
        <span>Medewerkers</span>
        <input type="number" bind:value={workers} min="1" max="20" />
      </label>
      <button class="action-btn primary" onclick={loadPlan}>Herberekenen</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Totaal uren</span>
      <span class="summary-value">{totalHours.toFixed(1)} u</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Geschatte werkdagen</span>
      <span class="summary-value">{estimatedDays} dagen</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Leveringsdatum</span>
      <span class="summary-value">{deliveryDate || "\u2014"}</span>
    </div>
  </div>

  {#if loading}
    <p class="hint">Laden...</p>
  {:else if jobs.length === 0}
    <p class="hint">Nog geen kozijnen in het project. Maak kozijnen aan om een productieplanning te genereren.</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Merk</th>
            <th>Naam</th>
            <th>Fasen</th>
            <th class="num-head">Uren</th>
          </tr>
        </thead>
        <tbody>
          {#each jobs as job}
            <tr>
              <td class="mark">{job.mark || "\u2014"}</td>
              <td>{job.name || "\u2014"}</td>
              <td class="phases">
                {#if job.phases}
                  <div class="phase-bars">
                    {#each Object.entries(job.phases) as [name, phase]}
                      <div class="phase-bar" title="{name}: {phasePercent(phase)}%">
                        <div class="phase-fill" style="width:{phasePercent(phase)}%;background:{phaseColor(name)}"></div>
                        <span class="phase-name">{name}</span>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <span class="hint-inline">Niet gepland</span>
                {/if}
              </td>
              <td class="num">{(job.totalHours || 0).toFixed(1)}</td>
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
  .param input { width: 48px; padding: var(--sp-1) var(--sp-2); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; text-align: center; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-3); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .hint-inline { color: var(--text-muted); font-size: 11px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .num-head { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.mark { font-weight: 700; color: var(--amber); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .phases { min-width: 200px; }
  .phase-bars { display: flex; flex-direction: column; gap: 2px; }
  .phase-bar { position: relative; height: 14px; background: rgba(255,255,255,0.05); border-radius: 2px; overflow: hidden; }
  .phase-fill { height: 100%; border-radius: 2px; transition: width 0.3s; }
  .phase-name { position: absolute; left: 4px; top: 0; font-size: 9px; line-height: 14px; color: #fff; font-weight: 600; text-shadow: 0 0 2px rgba(0,0,0,0.5); }
</style>
