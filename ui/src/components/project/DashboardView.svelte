<script>
  import { _ } from "svelte-i18n";
  import { kozijnen } from "../../stores/project.js";
  import { panelLabel } from "../../lib/labels.js";

  // Frontend-only analytics: aggregates directly from the kozijnen store, so it
  // shows real figures in every mode (no backend/wasm round-trip needed).

  function materialLabel(m) {
    if (!m) return "Onbekend";
    if (typeof m === "string") {
      return { aluminum: "Aluminium", pvc: "Kunststof", wood_aluminum: "Hout-aluminium" }[m] || m;
    }
    if (m.wood) {
      return { meranti: "Meranti", accoya: "Accoya", vuren: "Vuren", eiken: "Eiken" }[m.wood] || m.wood;
    }
    const key = Object.keys(m)[0];
    if (key === "wood") return materialLabel({ wood: m[key] });
    return { aluminum: "Aluminium", pvc: "Kunststof", woodAluminum: "Hout-aluminium" }[key] || key;
  }

  const round1 = (v) => Math.round((v || 0) * 10) / 10;
  const round2 = (v) => Math.round((v || 0) * 100) / 100;

  $: list = $kozijnen || [];
  $: kozijnCount = list.length;
  $: allCells = list.flatMap((k) => k.cells || []);
  $: cellCount = allCells.length;

  $: totalAreaM2 = round2(
    list.reduce((s, k) => s + ((k.frame?.outerWidth || 0) * (k.frame?.outerHeight || 0)) / 1e6, 0)
  );

  $: escapeCount = allCells.filter((c) => c.isEscape).length;

  const OPERABLE = new Set(["turn_tilt", "turn", "tilt", "sliding", "door", "top_hung", "bottom_hung", "lift_slide", "pivot"]);
  $: operableCount = allCells.filter((c) => OPERABLE.has(c.panelType)).length;

  $: ugValues = allCells.map((c) => c.glazing?.ugValue).filter((v) => typeof v === "number" && v > 0);
  $: avgUg = ugValues.length ? round2(ugValues.reduce((a, b) => a + b, 0) / ugValues.length) : 0;

  // Distributions
  function tally(items, keyFn) {
    const m = new Map();
    for (const it of items) {
      const k = keyFn(it);
      m.set(k, (m.get(k) || 0) + 1);
    }
    return [...m.entries()].map(([label, count]) => ({ label, count })).sort((a, b) => b.count - a.count);
  }

  $: byMaterial = tally(list, (k) => materialLabel(k.frame?.material));
  $: byPanelType = tally(allCells, (c) => panelLabel($_, c.panelType));

  $: maxMaterial = Math.max(1, ...byMaterial.map((d) => d.count));
  $: maxPanel = Math.max(1, ...byPanelType.map((d) => d.count));

  $: areaByMaterial = (() => {
    const m = new Map();
    for (const k of list) {
      const label = materialLabel(k.frame?.material);
      const a = ((k.frame?.outerWidth || 0) * (k.frame?.outerHeight || 0)) / 1e6;
      m.set(label, (m.get(label) || 0) + a);
    }
    return [...m.entries()].map(([label, area]) => ({ label, area: round2(area) })).sort((a, b) => b.area - a.area);
  })();
</script>

<div class="view">
  <div class="toolbar">
    <h2>Projectdashboard</h2>
    <span class="hint-inline">Live overzicht — afgeleid uit de kozijnen in het project</span>
  </div>

  {#if kozijnCount === 0}
    <p class="hint">Voeg kozijnen toe aan het project om analytics te tonen.</p>
  {:else}
    <div class="kpis">
      <div class="kpi"><span class="kpi-val">{kozijnCount}</span><span class="kpi-lbl">Kozijnen</span></div>
      <div class="kpi"><span class="kpi-val">{cellCount}</span><span class="kpi-lbl">Vakken</span></div>
      <div class="kpi"><span class="kpi-val">{totalAreaM2}</span><span class="kpi-lbl">Geveloppervlak (m²)</span></div>
      <div class="kpi"><span class="kpi-val">{operableCount}</span><span class="kpi-lbl">Beweegbare vakken</span></div>
      <div class="kpi"><span class="kpi-val">{escapeCount}</span><span class="kpi-lbl">Vluchtramen</span></div>
      <div class="kpi"><span class="kpi-val">{avgUg > 0 ? avgUg.toFixed(2) : "—"}</span><span class="kpi-lbl">Gem. Ug glas (W/m²K)</span></div>
    </div>

    <div class="charts">
      <div class="chart-card">
        <h3>Materiaalverdeling (kozijnen)</h3>
        {#each byMaterial as d}
          <div class="bar-row">
            <span class="bar-label">{d.label}</span>
            <div class="bar-track"><div class="bar-fill mat" style="width:{(d.count / maxMaterial) * 100}%"></div></div>
            <span class="bar-val">{d.count}</span>
          </div>
        {/each}
      </div>

      <div class="chart-card">
        <h3>Raamtype-verdeling (vakken)</h3>
        {#each byPanelType as d}
          <div class="bar-row">
            <span class="bar-label">{d.label}</span>
            <div class="bar-track"><div class="bar-fill pt" style="width:{(d.count / maxPanel) * 100}%"></div></div>
            <span class="bar-val">{d.count}</span>
          </div>
        {/each}
      </div>

      <div class="chart-card">
        <h3>Geveloppervlak per materiaal (m²)</h3>
        <table class="area-table">
          <tbody>
            {#each areaByMaterial as d}
              <tr><td>{d.label}</td><td class="num">{d.area.toFixed(2)} m²</td></tr>
            {/each}
            <tr class="total-row"><td>Totaal</td><td class="num">{totalAreaM2.toFixed(2)} m²</td></tr>
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: auto; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); flex-wrap: wrap; gap: var(--sp-2); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .hint-inline { font-size: 11px; color: var(--text-muted); font-style: italic; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: var(--sp-3); margin-bottom: var(--sp-4); }
  .kpi { padding: var(--sp-3); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); display: flex; flex-direction: column; gap: 2px; }
  .kpi-val { font-size: 24px; font-weight: 700; color: var(--amber); font-variant-numeric: tabular-nums; }
  .kpi-lbl { font-size: 10px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); }
  .charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: var(--sp-4); }
  .chart-card { padding: var(--sp-3); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); }
  .chart-card h3 { font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-secondary); margin: 0 0 var(--sp-3); }
  .bar-row { display: flex; align-items: center; gap: var(--sp-2); margin-bottom: var(--sp-2); font-size: 12px; }
  .bar-label { width: 110px; flex-shrink: 0; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bar-track { flex: 1; height: 14px; background: var(--bg-surface); border-radius: 3px; overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 3px; min-width: 2px; }
  .bar-fill.mat { background: #d97706; }
  .bar-fill.pt { background: #3b82f6; }
  .bar-val { width: 28px; text-align: right; font-variant-numeric: tabular-nums; color: var(--text-muted); flex-shrink: 0; }
  .area-table { width: 100%; border-collapse: collapse; font-size: 12px; }
  .area-table td { padding: var(--sp-1) var(--sp-2); border-bottom: 1px solid var(--border-color, #333); color: var(--text-primary); }
  .area-table td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .area-table .total-row td { font-weight: 700; border-bottom: none; border-top: 2px solid var(--border-color, #333); color: var(--amber); }
</style>
