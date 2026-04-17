<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let checks = [];
  let loading = false;

  const defaultReqs = [
    { property: "Uw", operator: "<=", value: 1.65, description: "Max. Uw-waarde conform Bouwbesluit" },
    { property: "frameWidth", operator: ">=", value: 50, description: "Min. profielbreedte (mm)" },
    { property: "glazingThickness", operator: ">=", value: 20, description: "Min. glasdikte (mm)" },
  ];

  onMount(validate);

  async function validate() {
    loading = true;
    try {
      const result = await invoke("validate_project_ids", {
        requirementsJson: JSON.stringify(defaultReqs),
      });
      checks = result?.checks || [];
    } catch (e) {
      console.error("IDS validatie mislukt:", e);
    }
    loading = false;
  }

  $: passCount = checks.filter(c => c.pass).length;
  $: failCount = checks.filter(c => !c.pass).length;
</script>

<div class="view">
  <div class="toolbar">
    <h2>IDS Validatie (Information Delivery Specification)</h2>
    <div class="toolbar-actions">
      <button class="action-btn primary" onclick={validate}>Valideren</button>
    </div>
  </div>

  <div class="summary">
    <div class="summary-card">
      <span class="summary-label">Geslaagd</span>
      <span class="summary-value pass-val">{passCount}</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Niet geslaagd</span>
      <span class="summary-value fail-val">{failCount}</span>
    </div>
    <div class="summary-card">
      <span class="summary-label">Totaal checks</span>
      <span class="summary-value">{checks.length}</span>
    </div>
  </div>

  {#if loading}
    <p class="hint">Valideren...</p>
  {:else if checks.length === 0}
    <p class="hint">Geen validatieresultaten. Voeg kozijnen toe aan het project en voer de validatie opnieuw uit.</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Eis</th>
            <th>Kozijn</th>
            <th class="num-head">Waarde</th>
            <th class="num-head">Vereist</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each checks as check}
            <tr class:row-fail={!check.pass}>
              <td>{check.description || check.requirement || "\u2014"}</td>
              <td class="mark">{check.kozijnMark || "\u2014"}</td>
              <td class="num">{check.actualValue ?? "\u2014"}</td>
              <td class="num">{check.operator || ""} {check.expectedValue ?? "\u2014"}</td>
              <td>
                {#if check.pass}
                  <span class="badge pass">OK</span>
                {:else}
                  <span class="badge fail">Niet-conform</span>
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
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-2); }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .summary { display: flex; gap: var(--sp-3); margin-bottom: var(--sp-3); }
  .summary-card { flex: 1; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .summary-label { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); margin-bottom: var(--sp-1); }
  .summary-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .pass-val { color: #22c55e; }
  .fail-val { color: #ef4444; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  .num-head { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  tbody tr.row-fail { background: rgba(239, 68, 68, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.mark { font-weight: 700; color: var(--amber); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .badge.pass { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .badge.fail { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
</style>
