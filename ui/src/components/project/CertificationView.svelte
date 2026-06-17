<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { currentKozijn } from "../../stores/kozijn.js";
  import { get } from "svelte/store";
  import { toast } from "../../stores/toast.js";

  let ceChecks = [];
  let skhChecks = [];
  let perfClass = null;
  let loading = false;
  let dop = null;
  let dopLoading = false;

  onMount(runChecks);

  async function generateDop() {
    const k = get(currentKozijn);
    if (!k) return;
    dopLoading = true;
    try {
      dop = await invoke("generate_dop_for_kozijn", { id: k.id });
    } catch (e) {
      console.error("DoP genereren mislukt:", e);
      toast.error?.("Prestatieverklaring genereren mislukt");
    }
    dopLoading = false;
  }

  async function runChecks() {
    const k = get(currentKozijn);
    if (!k) return;
    loading = true;
    try {
      const result = await invoke("check_certification", { id: k.id });
      ceChecks = result?.ceMarking?.checks || [];
      skhChecks = result?.skhKomo?.checks || [];
      perfClass = result?.performanceClass || null;
    } catch (e) {
      console.error("Certificatie-check mislukt:", e);
    }
    loading = false;
  }

  $: allCePassed = ceChecks.length > 0 && ceChecks.every(c => c.passed);
  $: allSkhPassed = skhChecks.length > 0 && skhChecks.every(c => c.passed);
</script>

<div class="view">
  <div class="toolbar">
    <h2>Certificatie (CE / SKH / KOMO)</h2>
    <div class="toolbar-actions">
      <button class="action-btn" onclick={runChecks}>Controleren</button>
      <button class="action-btn primary" onclick={generateDop} disabled={!$currentKozijn}>
        {dopLoading ? "Genereren..." : "Genereer DoP"}
      </button>
    </div>
  </div>

  {#if !$currentKozijn}
    <p class="hint">Selecteer een kozijn om de certificatie-checks uit te voeren.</p>
  {:else if loading}
    <p class="hint">Controleren...</p>
  {:else}
    <div class="cert-section">
      <div class="section-header">
        <h3>CE EN 14351-1</h3>
        {#if ceChecks.length > 0}
          <span class="badge" class:pass={allCePassed} class:fail={!allCePassed}>
            {allCePassed ? "Voldoet" : "Aandachtspunten"}
          </span>
        {/if}
      </div>
      {#if ceChecks.length === 0}
        <p class="hint">Geen CE-controleresultaten beschikbaar.</p>
      {:else}
        <div class="checks-grid">
          {#each ceChecks as check}
            <div class="check-card" class:check-pass={check.passed} class:check-fail={!check.passed}>
              <div class="check-icon">{check.passed ? "\u2713" : "\u2717"}</div>
              <div class="check-content">
                <span class="check-req">{check.requirement || "\u2014"}</span>
                <span class="check-detail">Waarde: {check.value ?? "\u2014"} | Vereist: {check.expected ?? "\u2014"}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="cert-section">
      <div class="section-header">
        <h3>SKH / KOMO</h3>
        {#if skhChecks.length > 0}
          <span class="badge" class:pass={allSkhPassed} class:fail={!allSkhPassed}>
            {allSkhPassed ? "Voldoet" : "Aandachtspunten"}
          </span>
        {/if}
      </div>
      {#if skhChecks.length === 0}
        <p class="hint">Geen SKH/KOMO-controleresultaten beschikbaar.</p>
      {:else}
        <div class="checks-grid">
          {#each skhChecks as check}
            <div class="check-card" class:check-pass={check.passed} class:check-fail={!check.passed}>
              <div class="check-icon">{check.passed ? "\u2713" : "\u2717"}</div>
              <div class="check-content">
                <span class="check-req">{check.requirement || "\u2014"}</span>
                <span class="check-detail">Waarde: {check.value ?? "\u2014"} | Vereist: {check.expected ?? "\u2014"}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if perfClass}
      <div class="cert-section">
        <div class="section-header">
          <h3>Prestatieklassen (EN 14351-1)</h3>
        </div>
        <div class="perf-grid">
          <div class="perf-card"><span class="perf-lbl">Luchtdoorlatendheid</span><span class="perf-val">{perfClass.airPermeability || "—"}</span><span class="perf-std">EN 12207</span></div>
          <div class="perf-card"><span class="perf-lbl">Waterdichtheid</span><span class="perf-val">{perfClass.waterTightness || "—"}</span><span class="perf-std">EN 12208</span></div>
          <div class="perf-card"><span class="perf-lbl">Windweerstand</span><span class="perf-val">{perfClass.windResistance || "—"}</span><span class="perf-std">EN 12210</span></div>
          <div class="perf-card"><span class="perf-lbl">Warmtegeleiding</span><span class="perf-val">{perfClass.thermalTransmittance || "—"}</span><span class="perf-std">EN ISO 10077</span></div>
          <div class="perf-card"><span class="perf-lbl">Geluidsisolatie</span><span class="perf-val">{perfClass.soundInsulation || "—"}</span><span class="perf-std">EN ISO 717-1</span></div>
          <div class="perf-card"><span class="perf-lbl">Inbraakwerendheid</span><span class="perf-val">{perfClass.burglarResistance || "—"}</span><span class="perf-std">EN 1627</span></div>
        </div>
      </div>
    {/if}

    {#if dop && dop.dopNumber}
      <div class="cert-section dop-section">
        <div class="section-header">
          <h3>Prestatieverklaring (DoP) — EN 14351-1</h3>
          <span class="badge neutral">Concept</span>
        </div>
        <div class="dop-doc">
          <div class="dop-head">
            <div class="dop-no">{dop.dopNumber}</div>
            <div class="dop-product">{dop.productType}</div>
          </div>
          <div class="dop-meta">
            <div><span class="lbl">Kozijn</span><span>{dop.kozijnMark} — {dop.kozijnName}</span></div>
            <div><span class="lbl">Beoogd gebruik</span><span>{dop.intendedUse}</span></div>
            <div><span class="lbl">Geharmoniseerde norm</span><span>{dop.harmonisedStandard}</span></div>
            <div><span class="lbl">AVCP-systeem</span><span>{dop.avcpSystem}</span></div>
            <div><span class="lbl">Materiaal</span><span>{dop.material}</span></div>
            <div><span class="lbl">Afmeting (b×h)</span><span>{Math.round(dop.widthMm)} × {Math.round(dop.heightMm)} mm</span></div>
            <div><span class="lbl">Uw (gedeclareerd)</span><span>{dop.declaredUw.toFixed(2)} W/m²K</span></div>
            <div><span class="lbl">Aangemelde instantie</span><span class="placeholder">{dop.notifiedBody}</span></div>
            <div><span class="lbl">Fabrikant</span><span class="placeholder">{dop.manufacturer}</span></div>
          </div>

          <table class="dop-table">
            <thead><tr>
              <th>Essentiële eigenschap</th><th>Prestatie</th><th>Norm</th>
            </tr></thead>
            <tbody>
              {#each dop.characteristics as ch}
                <tr>
                  <td>{ch.characteristic}</td>
                  <td class:npd={ch.performance === "NPD"}>{ch.performance}</td>
                  <td class="std">{ch.standard}</td>
                </tr>
              {/each}
            </tbody>
          </table>

          <p class="dop-statement">{dop.conformityStatement}</p>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: auto; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-2); }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }

  .cert-section { margin-bottom: var(--sp-4); }
  .section-header { display: flex; align-items: center; gap: var(--sp-3); margin-bottom: var(--sp-2); }
  .section-header h3 { font-size: 13px; font-weight: 700; color: var(--amber); text-transform: uppercase; letter-spacing: 0.04em; margin: 0; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .badge.pass { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .badge.fail { background: rgba(239, 68, 68, 0.15); color: #ef4444; }

  .checks-grid { display: flex; flex-direction: column; gap: var(--sp-2); }
  .check-card { display: flex; align-items: center; gap: var(--sp-3); padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border-left: 3px solid transparent; }
  .check-card.check-pass { border-left-color: #22c55e; }
  .check-card.check-fail { border-left-color: #ef4444; }
  .check-icon { font-size: 14px; font-weight: 700; width: 20px; text-align: center; }
  .check-pass .check-icon { color: #22c55e; }
  .check-fail .check-icon { color: #ef4444; }
  .check-content { display: flex; flex-direction: column; gap: 1px; }
  .check-req { font-size: 12px; font-weight: 600; color: var(--text-primary); }
  .check-detail { font-size: 11px; color: var(--text-muted); }

  .badge.neutral { background: rgba(107, 114, 128, 0.18); color: #9aa0a8; }
  .action-btn:disabled { opacity: 0.5; }

  .perf-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: var(--sp-2); }
  .perf-card { display: flex; flex-direction: column; gap: 1px; padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border-left: 3px solid var(--amber); }
  .perf-lbl { font-size: 10px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted); }
  .perf-val { font-size: 14px; font-weight: 700; color: var(--text-primary); }
  .perf-std { font-size: 10px; color: var(--text-muted); }

  .dop-doc { background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); padding: var(--sp-4); }
  .dop-head { border-bottom: 2px solid var(--amber); padding-bottom: var(--sp-2); margin-bottom: var(--sp-3); }
  .dop-no { font-size: 11px; font-weight: 700; letter-spacing: 0.06em; color: var(--amber); text-transform: uppercase; }
  .dop-product { font-size: 16px; font-weight: 700; color: var(--text-primary); }
  .dop-meta { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: var(--sp-2) var(--sp-4); margin-bottom: var(--sp-3); }
  .dop-meta > div { display: flex; flex-direction: column; gap: 1px; font-size: 12px; }
  .dop-meta .lbl { font-size: 10px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); }
  .dop-meta span:not(.lbl) { color: var(--text-primary); }
  .dop-meta .placeholder { color: #d97706; font-style: italic; }
  .dop-table { width: 100%; border-collapse: collapse; font-size: 12px; margin-bottom: var(--sp-3); }
  .dop-table thead th { text-align: left; padding: var(--sp-2) var(--sp-3); font-size: 10px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .dop-table tbody td { padding: var(--sp-2) var(--sp-3); border-bottom: 1px solid var(--border-color, #333); color: var(--text-primary); }
  .dop-table td.std { color: var(--text-muted); font-size: 11px; }
  .dop-table td.npd { color: var(--text-muted); font-style: italic; }
  .dop-statement { font-size: 11px; color: var(--text-secondary); font-style: italic; line-height: 1.5; margin: 0; }
</style>
