<script>
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";

  let data = $state(null);
  let loading = $state(false);

  onMount(loadCircularity);

  async function loadCircularity() {
    loading = true;
    try {
      data = await invoke("get_project_circularity");
    } catch (e) {
      console.error("Circulariteit laden mislukt:", e);
    }
    loading = false;
  }

  let categories = $derived(data?.categories || []);
  let passports = $derived(data?.kozijnPassports || []);
  let totalMass = $derived(data?.totalMassKg || 0);
  let totalCo2 = $derived(data?.totalCo2Kg || 0);
  let score = $derived(data?.circularityScore || 0);
  let renewablePct = $derived(data?.renewablePct || 0);

  function scoreClass(v) {
    if (v >= 70) return "good";
    if (v >= 50) return "medium";
    return "poor";
  }
  const fmt = (v, d = 1) => (v || 0).toFixed(d);
</script>

<div class="view">
  <div class="toolbar">
    <h2>Circulariteit &amp; materiaalpaspoort</h2>
    <div class="toolbar-actions">
      <span class="hint-inline">Indicatieve generieke factoren (NMD/EPD-stijl)</span>
      <button class="action-btn primary" onclick={loadCircularity}>Herberekenen</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Circulariteitsscore</span>
      <span class="summary-value score {scoreClass(score)}">{fmt(score)}%</span>
      <span class="summary-sub">herbruikbaar / recyclebaar</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Totale massa</span>
      <span class="summary-value">{fmt(totalMass, 1)} kg</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Ingebedde CO&#8322; (A1&#8211;A3)</span>
      <span class="summary-value">{fmt(totalCo2, 1)} kg</span>
      <span class="summary-sub">CO&#8322;-eq, indicatief</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Hernieuwbaar (biobased)</span>
      <span class="summary-value">{fmt(renewablePct)}%</span>
      <span class="summary-sub">van totale massa</span>
    </div>
  </div>

  {#if loading}
    <p class="hint">Laden...</p>
  {:else if categories.length === 0}
    <p class="hint">Voeg kozijnen toe aan het project om het materiaalpaspoort te berekenen.</p>
  {:else}
    <div class="table-container">
      <h3 class="section-title">Materiaalpaspoort (projectbreed)</h3>
      <table>
        <thead>
          <tr>
            <th>Materiaalcategorie</th>
            <th class="num-head">Massa (kg)</th>
            <th class="num-head">Recyclebaar (kg)</th>
            <th class="num-head">Recyclebaarheid</th>
            <th class="num-head">CO&#8322; (kg)</th>
            <th>Hernieuwbaar</th>
          </tr>
        </thead>
        <tbody>
          {#each categories as cat}
            <tr>
              <td class="mark">{cat.category}</td>
              <td class="num">{fmt(cat.massKg, 1)}</td>
              <td class="num">{fmt(cat.recyclableKg, 1)}</td>
              <td class="num">
                <div class="bar-cell">
                  <div class="bar-track"><div class="bar-fill {scoreClass(cat.recyclabilityPct)}" style="width:{Math.min(100, cat.recyclabilityPct)}%"></div></div>
                  <span>{fmt(cat.recyclabilityPct)}%</span>
                </div>
              </td>
              <td class="num">{fmt(cat.co2Kg, 1)}</td>
              <td>
                {#if cat.renewable}
                  <span class="badge pass">Ja</span>
                {:else}
                  <span class="badge neutral">Nee</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <td class="mark">Totaal</td>
            <td class="num">{fmt(totalMass, 1)}</td>
            <td class="num">{fmt(data?.totalRecyclableKg, 1)}</td>
            <td class="num">{fmt(score)}%</td>
            <td class="num">{fmt(totalCo2, 1)}</td>
            <td></td>
          </tr>
        </tfoot>
      </table>

      {#if passports.length > 1}
        <h3 class="section-title">Per kozijn</h3>
        <table>
          <thead>
            <tr>
              <th>Merk</th>
              <th>Naam</th>
              <th class="num-head">Massa (kg)</th>
              <th class="num-head">Score</th>
              <th class="num-head">CO&#8322; (kg)</th>
              <th class="num-head">Biobased</th>
            </tr>
          </thead>
          <tbody>
            {#each passports as p}
              <tr>
                <td class="mark">{p.kozijnMark || "—"}</td>
                <td>{p.kozijnName || "—"}</td>
                <td class="num">{fmt(p.totalMassKg, 1)}</td>
                <td class="num">
                  <span class="score-tag {scoreClass(p.circularityScore)}">{fmt(p.circularityScore)}%</span>
                </td>
                <td class="num">{fmt(p.totalCo2Kg, 1)}</td>
                <td class="num">{fmt(p.renewablePct)}%</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); flex-wrap: wrap; gap: var(--sp-2); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-3); align-items: center; }
  .hint-inline { font-size: 11px; color: var(--text-muted); font-style: italic; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-3); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); display: flex; flex-direction: column; }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .summary-value.score.good { color: #16a34a; }
  .summary-value.score.medium { color: #d97706; }
  .summary-value.score.poor { color: #dc2626; }
  .summary-sub { font-size: 10px; color: var(--text-muted); margin-top: 2px; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  .section-title { font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-secondary); margin: var(--sp-3) 0 var(--sp-2); }
  table { width: 100%; border-collapse: collapse; font-size: 12px; margin-bottom: var(--sp-2); }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .num-head { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.mark { font-weight: 700; color: var(--amber); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  tfoot td { padding: var(--sp-2) var(--sp-3); font-weight: 700; border-top: 2px solid var(--border-color, #333); color: var(--text-primary); }
  .bar-cell { display: flex; align-items: center; gap: var(--sp-2); justify-content: flex-end; }
  .bar-track { width: 56px; height: 6px; background: var(--bg-surface); border-radius: 3px; overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 3px; }
  .bar-fill.good { background: #16a34a; }
  .bar-fill.medium { background: #d97706; }
  .bar-fill.poor { background: #dc2626; }
  .score-tag { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 700; }
  .score-tag.good { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .score-tag.medium { background: rgba(217, 119, 6, 0.15); color: #d97706; }
  .score-tag.poor { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .badge.pass { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .badge.neutral { background: rgba(107, 114, 128, 0.15); color: #6b7280; }
</style>
