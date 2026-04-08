<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";

  let estimates = [];
  let loading = false;

  onMount(loadEstimates);

  async function loadEstimates() {
    loading = true;
    try {
      estimates = await invoke("get_cost_estimate_project", { priceTableJson: null });
    } catch (e) {
      console.error("Calculatie laden mislukt:", e);
    }
    loading = false;
  }

  $: grandTotal = estimates.reduce((sum, e) => sum + e.totalCost, 0);
  $: totalLabor = estimates.reduce((sum, e) => sum + e.laborHours, 0);
</script>

<div class="calc-view">
  <div class="toolbar">
    <h2>{$_('calculation.title')}</h2>
    <button class="refresh-btn" onclick={loadEstimates}>{$_('calculation.recalculate')}</button>
  </div>

  {#if loading}
    <p class="hint">{$_('calculation.calculating')}</p>
  {:else if estimates.length === 0}
    <p class="hint">{$_('calculation.noKozijnen')}</p>
  {:else}
    <!-- Summary -->
    <div class="summary">
      <div class="summary-card">
        <span class="label">{$_('calculation.total')}</span>
        <span class="value">&euro; {grandTotal.toFixed(2)}</span>
      </div>
      <div class="summary-card">
        <span class="label">{$_('calculation.frames')}</span>
        <span class="value">{estimates.length}</span>
      </div>
      <div class="summary-card">
        <span class="label">{$_('calculation.hours')}</span>
        <span class="value">{totalLabor.toFixed(1)} {$_('calculation.hourUnit')}</span>
      </div>
    </div>

    <!-- Per kozijn -->
    <div class="table-container">
      <table>
        <colgroup>
          <col style="width:6%"><col style="width:8%"><col style="width:9%"><col style="width:9%">
          <col style="width:9%"><col style="width:9%"><col style="width:8%"><col style="width:9%">
          <col style="width:9%"><col style="width:7%"><col style="width:10%">
        </colgroup>
        <thead><tr>
          <th>{$_('calculation.nlSfb')}</th>
          <th>{$_('calculation.frame')}</th>
          <th class="num">{$_('calculation.material')}</th>
          <th class="num">{$_('calculation.glass')}</th>
          <th class="num">{$_('calculation.hardware')}</th>
          <th class="num">{$_('calculation.gasket')}</th>
          <th class="num">{$_('calculation.panels')}</th>
          <th class="num">{$_('calculation.surface')}</th>
          <th class="num">{$_('calculation.labor')}</th>
          <th class="num">{$_('calculation.hours')}</th>
          <th class="num">{$_('calculation.total')}</th>
        </tr></thead>
        <tbody>
          {#each estimates as est}
            <tr>
              <td><span class="sfb-badge">{est.nlSfb.code}</span></td>
              <td>—</td>
              <td class="num">&euro; {est.materialCost.toFixed(2)}</td>
              <td class="num">&euro; {est.glassCost.toFixed(2)}</td>
              <td class="num">&euro; {est.hardwareCost.toFixed(2)}</td>
              <td class="num">&euro; {est.gasketCost.toFixed(2)}</td>
              <td class="num">&euro; {est.panelCost.toFixed(2)}</td>
              <td class="num">&euro; {est.surfaceTreatmentCost.toFixed(2)}</td>
              <td class="num">&euro; {est.laborCost.toFixed(2)}</td>
              <td class="num">{est.laborHours.toFixed(1)}</td>
              <td class="num total">&euro; {est.totalCost.toFixed(2)}</td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <td colspan="2"><strong>{$_('calculation.totalProject')}</strong></td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.materialCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.glassCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.hardwareCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.gasketCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.panelCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.surfaceTreatmentCost, 0).toFixed(2)}</td>
            <td class="num">&euro; {estimates.reduce((s, e) => s + e.laborCost, 0).toFixed(2)}</td>
            <td class="num">{totalLabor.toFixed(1)}</td>
            <td class="num total">&euro; {grandTotal.toFixed(2)}</td>
          </tr>
        </tfoot>
      </table>
    </div>

    <!-- Line items detail -->
    <h3 class="section-title">{$_('calculation.costBreakdown')}</h3>
    <div class="table-container">
      <table>
        <thead><tr>
          <th>{$_('calculation.nlSfb')}</th><th>{$_('calculation.description')}</th><th>{$_('calculation.quantity')}</th>
          <th>{$_('calculation.unit')}</th><th>{$_('calculation.unitPrice')}</th><th>{$_('calculation.total')}</th>
        </tr></thead>
        <tbody>
          {#each estimates as est}
            {#each est.lineItems as item}
              <tr>
                <td><span class="sfb-badge">{item.nlSfbCode}</span></td>
                <td>{item.description}</td>
                <td class="num">{item.quantity.toFixed(2)}</td>
                <td>{item.unit}</td>
                <td class="num">&euro; {item.unitPrice.toFixed(2)}</td>
                <td class="num">&euro; {item.total.toFixed(2)}</td>
              </tr>
            {/each}
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .calc-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: var(--sp-4);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--sp-3);
  }

  .toolbar h2 {
    font-size: 16px;
    color: var(--text-primary);
    margin: 0;
  }

  .refresh-btn {
    padding: var(--sp-2) var(--sp-4);
    background: var(--amber);
    color: var(--bg-surface);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: default;
  }

  .summary {
    display: flex;
    gap: var(--sp-4);
    margin-bottom: var(--sp-4);
  }

  .summary-card {
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    padding: var(--sp-4);
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .summary-card .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    margin-bottom: var(--sp-1);
  }

  .summary-card .value {
    font-size: 20px;
    font-weight: 700;
    color: var(--amber);
  }

  .section-title {
    font-size: 14px;
    color: var(--text-primary);
    margin: var(--sp-4) 0 var(--sp-2);
  }

  .table-container {
    overflow: auto;
    margin-bottom: var(--sp-4);
  }

  table {
    width: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font-size: 12px;
  }

  thead th {
    position: sticky;
    top: 0;
    background: var(--bg-surface-alt);
    padding: var(--sp-2) var(--sp-3);
    text-align: left;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    border-bottom: 2px solid var(--border-color, #e5e7eb);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  tbody tr {
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  tbody tr:nth-child(even) {
    background: var(--bg-surface-alt);
  }

  tfoot tr {
    border-top: 2px solid var(--amber);
    font-weight: 700;
  }

  td {
    padding: var(--sp-2) var(--sp-3);
    color: var(--text-primary);
  }

  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  td.total {
    color: var(--amber);
    font-weight: 700;
  }

  .sfb-badge {
    font-size: 10px;
    font-weight: 700;
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color, #e5e7eb);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    font-variant-numeric: tabular-nums;
  }

  .hint {
    color: var(--text-muted);
    font-size: 13px;
    font-style: italic;
  }
</style>
