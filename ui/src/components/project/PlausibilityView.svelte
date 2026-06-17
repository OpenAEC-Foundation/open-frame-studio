<script>
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";

  let data = $state(null);
  let loading = $state(false);
  let windKnm2 = $state(1.0); // design wind pressure in kN/m²

  onMount(load);

  async function load() {
    loading = true;
    try {
      data = await invoke("get_project_plausibility", { windPressurePa: windKnm2 * 1000 });
    } catch (e) {
      console.error("Plausibiliteit laden mislukt:", e);
    }
    loading = false;
  }

  let results = $derived(data?.kozijnResults || []);
  let limitRatio = $derived(data?.limitRatio || 200);
  let overallPass = $derived(data?.overallPass ?? true);
  let worstProject = $derived(
    results.length
      ? Math.min(...results.map((r) => r.worstRatio || Infinity))
      : Infinity
  );

  const fmtRatio = (r) => (r && isFinite(r) && r < 90000 ? `L/${Math.round(r)}` : "L/∞");
</script>

<div class="view">
  <div class="toolbar">
    <h2>Plausibiliteit — windbelasting (indicatief)</h2>
    <div class="toolbar-actions">
      <label class="param">
        <span>Winddruk</span>
        <input type="number" bind:value={windKnm2} min="0.2" max="3.0" step="0.1" />
        <span>kN/m&sup2;</span>
      </label>
      <button class="action-btn primary" onclick={load}>Herberekenen</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Beoordeling</span>
      <span class="summary-value">
        {#if results.length === 0}
          <span class="badge neutral">Geen data</span>
        {:else if overallPass}
          <span class="badge pass">Plausibel</span>
        {:else}
          <span class="badge fail">Aandachtspunt</span>
        {/if}
      </span>
      <span class="summary-sub">grens {fmtRatio(limitRatio)} (EN 12210)</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Maatgevende doorbuiging</span>
      <span class="summary-value">{fmtRatio(worstProject)}</span>
      <span class="summary-sub">kleinste L/δ in project</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Winddruk</span>
      <span class="summary-value">{windKnm2.toFixed(1)} kN/m&sup2;</span>
      <span class="summary-sub">aangenomen ontwerpwaarde</span>
    </div>
  </div>

  <p class="disclaimer">
    Indicatieve serviceability-controle (simpel ligger-model, massieve doorsnede, conservatieve
    E-waarden). Géén vervanging voor een constructieberekening per Eurocode (NEN-EN 1991-1-4).
  </p>

  {#if loading}
    <p class="hint">Laden...</p>
  {:else if results.length === 0}
    <p class="hint">Voeg kozijnen toe aan het project om de windbelasting-plausibiliteit te bepalen.</p>
  {:else}
    <div class="results">
      {#each results as kz}
        <div class="kozijn-block">
          <div class="kozijn-head">
            <span class="km">{kz.kozijnMark}</span>
            <span class="kn">{kz.kozijnName}</span>
            <span class="worst">maatgevend: {fmtRatio(kz.worstRatio)}</span>
            <span class="badge" class:pass={kz.pass} class:fail={!kz.pass}>
              {kz.pass ? "Plausibel" : "Aandachtspunt"}
            </span>
          </div>
          <table>
            <thead><tr>
              <th>Lid</th><th>Richting</th><th class="num">Overspanning</th><th class="num">Belast. breedte</th>
              <th class="num">Lijnlast</th><th class="num">Doorbuiging</th><th class="num">L/δ</th><th>Status</th>
            </tr></thead>
            <tbody>
              {#each kz.members as m}
                <tr>
                  <td class="mark">{m.member}</td>
                  <td>{m.orientation}</td>
                  <td class="num">{Math.round(m.spanMm)} mm</td>
                  <td class="num">{Math.round(m.tributaryMm)} mm</td>
                  <td class="num">{(m.lineLoadNPerMm || 0).toFixed(2)} N/mm</td>
                  <td class="num">{(m.deflectionMm || 0).toFixed(2)} mm</td>
                  <td class="num" class:ratio-good={m.pass} class:ratio-bad={!m.pass}>{fmtRatio(m.deflectionRatio)}</td>
                  <td>
                    {#if m.pass}
                      <span class="badge pass">OK</span>
                    {:else}
                      <span class="badge fail">Te slap</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: auto; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); flex-wrap: wrap; gap: var(--sp-2); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-3); align-items: center; }
  .param { display: flex; align-items: center; gap: var(--sp-1); font-size: 11px; color: var(--text-muted); }
  .param input { width: 56px; padding: var(--sp-1) var(--sp-2); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; text-align: center; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-2); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); display: flex; flex-direction: column; }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .summary-sub { font-size: 10px; color: var(--text-muted); margin-top: 2px; }
  .disclaimer { font-size: 11px; color: var(--text-muted); font-style: italic; line-height: 1.5; margin: 0 0 var(--sp-3); }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .results { display: flex; flex-direction: column; gap: var(--sp-4); }
  .kozijn-block { border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); overflow: hidden; }
  .kozijn-head { display: flex; align-items: center; gap: var(--sp-3); padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); }
  .km { font-weight: 700; color: var(--amber); }
  .kn { color: var(--text-secondary); font-size: 12px; }
  .worst { margin-left: auto; font-size: 11px; color: var(--text-muted); font-variant-numeric: tabular-nums; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:last-child { border-bottom: none; }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.mark { font-weight: 600; }
  .ratio-good { color: #16a34a; font-weight: 600; }
  .ratio-bad { color: #dc2626; font-weight: 600; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .badge.pass { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .badge.fail { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
  .badge.neutral { background: rgba(107, 114, 128, 0.15); color: #6b7280; }
</style>
