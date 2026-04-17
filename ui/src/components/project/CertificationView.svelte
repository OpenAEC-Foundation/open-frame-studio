<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { currentKozijn } from "../../stores/kozijn.js";
  import { get } from "svelte/store";
  import { toast } from "../../stores/toast.js";

  let ceChecks = [];
  let skhChecks = [];
  let loading = false;

  onMount(runChecks);

  async function runChecks() {
    const k = get(currentKozijn);
    if (!k) return;
    loading = true;
    try {
      const result = await invoke("check_certification", { id: k.id });
      ceChecks = result?.ceChecks || [];
      skhChecks = result?.skhChecks || [];
    } catch (e) {
      console.error("Certificatie-check mislukt:", e);
    }
    loading = false;
  }

  $: allCePassed = ceChecks.length > 0 && ceChecks.every(c => c.pass);
  $: allSkhPassed = skhChecks.length > 0 && skhChecks.every(c => c.pass);
</script>

<div class="view">
  <div class="toolbar">
    <h2>Certificatie (CE / SKH / KOMO)</h2>
    <div class="toolbar-actions">
      <button class="action-btn primary" onclick={runChecks}>Controleren</button>
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
            <div class="check-card" class:check-pass={check.pass} class:check-fail={!check.pass}>
              <div class="check-icon">{check.pass ? "\u2713" : "\u2717"}</div>
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
            <div class="check-card" class:check-pass={check.pass} class:check-fail={!check.pass}>
              <div class="check-icon">{check.pass ? "\u2713" : "\u2717"}</div>
              <div class="check-content">
                <span class="check-req">{check.requirement || "\u2014"}</span>
                <span class="check-detail">Waarde: {check.value ?? "\u2014"} | Vereist: {check.expected ?? "\u2014"}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
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
</style>
