<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let estimates = [];
  let kozijnList = [];
  let loading = false;

  // ── Pricing config ────────────────────────────────────────
  const DEFAULT_PRICING = {
    discountPercentage: 0,
    btwPercentage: 21,
    btwVerlegd: false,
    transportCost: 0,
    montageCostPerHour: 45,
    montageHours: 0,
  };
  let pricing = { ...DEFAULT_PRICING };
  let pricingOpen = false;
  let savingPricing = false;

  // ── Per-kozijn detail ─────────────────────────────────────
  let expandedIndex = -1;
  let detailEstimate = null;
  let detailLoading = false;

  onMount(() => {
    loadEstimates();
    loadPricing();
  });

  async function loadEstimates() {
    loading = true;
    expandedIndex = -1;
    detailEstimate = null;
    try {
      const [ests, koz] = await Promise.all([
        invoke("get_cost_estimate_project", { priceTableJson: null }),
        invoke("get_all_kozijnen", {}),
      ]);
      estimates = ests || [];
      kozijnList = koz || [];
    } catch (e) {
      console.error("Calculatie laden mislukt:", e);
    }
    loading = false;
  }

  async function loadPricing() {
    try {
      const cfg = await invoke("get_pricing_config", {});
      if (cfg) pricing = { ...DEFAULT_PRICING, ...cfg };
    } catch (e) {
      console.error("Prijsconfiguratie laden mislukt:", e);
    }
  }

  async function savePricing() {
    const config = {
      discountPercentage: Number(pricing.discountPercentage) || 0,
      btwPercentage: Number(pricing.btwPercentage) || 0,
      btwVerlegd: !!pricing.btwVerlegd,
      transportCost: Number(pricing.transportCost) || 0,
      montageCostPerHour: Number(pricing.montageCostPerHour) || 0,
      montageHours: Number(pricing.montageHours) || 0,
    };
    savingPricing = true;
    try {
      await invoke("update_pricing_config", { configJson: JSON.stringify(config) });
      pricing = config;
      toast.success($_("calculation.pricingSaved"));
      await loadEstimates();
    } catch (e) {
      toast.error($_("calculation.pricingSaveFailed") + ": " + e);
    }
    savingPricing = false;
  }

  async function toggleDetail(i) {
    if (expandedIndex === i) {
      expandedIndex = -1;
      detailEstimate = null;
      return;
    }
    expandedIndex = i;
    detailEstimate = null;
    const kozijn = kozijnList[i];
    if (!kozijn?.id) return;
    detailLoading = true;
    try {
      const est = await invoke("get_cost_estimate", { id: kozijn.id, priceTableJson: null });
      if (expandedIndex === i) detailEstimate = est;
    } catch (e) {
      console.error("Kostendetail laden mislukt:", e);
      toast.error($_("calculation.detailFailed") + ": " + e);
      if (expandedIndex === i) expandedIndex = -1;
    }
    detailLoading = false;
  }

  $: grandTotal = estimates.reduce((sum, e) => sum + e.totalCost, 0);
  $: totalLabor = estimates.reduce((sum, e) => sum + e.laborHours, 0);

  // Mirror of ofs-core PricingConfig::calculate() for a live preview.
  $: pricePreview = (() => {
    const discount = grandTotal * (Number(pricing.discountPercentage) || 0) / 100;
    const montage = (Number(pricing.montageCostPerHour) || 0) * (Number(pricing.montageHours) || 0);
    const transport = Number(pricing.transportCost) || 0;
    const afterExtras = grandTotal - discount + transport + montage;
    const btw = pricing.btwVerlegd ? 0 : afterExtras * (Number(pricing.btwPercentage) || 0) / 100;
    return { discount, montage, transport, afterExtras, btw, total: afterExtras + btw };
  })();
</script>

<div class="calc-view">
  <div class="toolbar">
    <h2>{$_('calculation.title')}</h2>
    <button class="refresh-btn" onclick={loadEstimates}>{$_('calculation.recalculate')}</button>
  </div>

  <!-- Pricing configuration -->
  <div class="pricing-section">
    <button class="pricing-header" onclick={() => (pricingOpen = !pricingOpen)} aria-expanded={pricingOpen}>
      <span class="chevron" class:open={pricingOpen}>&#9656;</span>
      {$_('calculation.pricingConfig')}
    </button>
    {#if pricingOpen}
      <div class="pricing-body">
        <div class="pricing-grid">
          <label class="field">
            <span>{$_('calculation.discountPct')}</span>
            <input type="number" min="0" max="100" step="0.5" bind:value={pricing.discountPercentage} />
          </label>
          <label class="field">
            <span>{$_('calculation.btwPct')}</span>
            <input type="number" min="0" max="100" step="0.5" bind:value={pricing.btwPercentage} disabled={pricing.btwVerlegd} />
          </label>
          <label class="field">
            <span>{$_('calculation.transportCost')}</span>
            <input type="number" min="0" step="1" bind:value={pricing.transportCost} />
          </label>
          <label class="field">
            <span>{$_('calculation.montageRate')}</span>
            <input type="number" min="0" step="1" bind:value={pricing.montageCostPerHour} />
          </label>
          <label class="field">
            <span>{$_('calculation.montageHours')}</span>
            <input type="number" min="0" step="0.5" bind:value={pricing.montageHours} />
          </label>
          <label class="field check">
            <input type="checkbox" bind:checked={pricing.btwVerlegd} />
            <span>{$_('calculation.btwVerlegd')}</span>
          </label>
        </div>

        <div class="pricing-preview">
          <div class="preview-row"><span>{$_('calculation.subtotal')}</span><span class="num">&euro; {grandTotal.toFixed(2)}</span></div>
          <div class="preview-row"><span>{$_('calculation.discount')}</span><span class="num">&minus; &euro; {pricePreview.discount.toFixed(2)}</span></div>
          <div class="preview-row"><span>{$_('calculation.transport')}</span><span class="num">&euro; {pricePreview.transport.toFixed(2)}</span></div>
          <div class="preview-row"><span>{$_('calculation.montage')}</span><span class="num">&euro; {pricePreview.montage.toFixed(2)}</span></div>
          <div class="preview-row"><span>{$_('calculation.btwAmount')} {pricing.btwVerlegd ? `(${$_('calculation.btwVerlegd').toLowerCase()})` : ''}</span><span class="num">&euro; {pricePreview.btw.toFixed(2)}</span></div>
          <div class="preview-row preview-total"><span>{$_('calculation.totalInclBtw')}</span><span class="num">&euro; {pricePreview.total.toFixed(2)}</span></div>
        </div>

        <div class="pricing-actions">
          <button class="save-btn" onclick={savePricing} disabled={savingPricing}>{$_('calculation.savePricing')}</button>
        </div>
      </div>
    {/if}
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
        <span class="label">{$_('calculation.kozijnen')}</span>
        <span class="value">{estimates.length}</span>
      </div>
      <div class="summary-card">
        <span class="label">{$_('calculation.hours')}</span>
        <span class="value">{totalLabor.toFixed(1)} {$_('calculation.hourUnit')}</span>
      </div>
      <div class="summary-card">
        <span class="label">{$_('calculation.totalInclBtw')}</span>
        <span class="value">&euro; {pricePreview.total.toFixed(2)}</span>
      </div>
    </div>

    <!-- Per kozijn -->
    <div class="table-container">
      <table>
        <colgroup>
          <col style="width:4%"><col style="width:7%"><col style="width:10%"><col style="width:9%">
          <col style="width:9%"><col style="width:9%"><col style="width:9%"><col style="width:8%">
          <col style="width:9%"><col style="width:8%"><col style="width:8%"><col style="width:10%">
        </colgroup>
        <thead><tr>
          <th></th>
          <th>{$_('calculation.nlsfb')}</th>
          <th>{$_('calculation.kozijn')}</th>
          <th class="num">{$_('calculation.material')}</th>
          <th class="num">{$_('calculation.glass')}</th>
          <th class="num">{$_('calculation.hardware')}</th>
          <th class="num">{$_('calculation.rubber')}</th>
          <th class="num">{$_('calculation.panels')}</th>
          <th class="num">{$_('calculation.surface')}</th>
          <th class="num">{$_('calculation.labor')}</th>
          <th class="num">{$_('calculation.hours')}</th>
          <th class="num">{$_('calculation.total')}</th>
        </tr></thead>
        <tbody>
          {#each estimates as est, i}
            <tr class:expanded={expandedIndex === i}>
              <td class="expand-cell">
                <button
                  class="expand-btn"
                  onclick={() => toggleDetail(i)}
                  aria-expanded={expandedIndex === i}
                  title={$_('calculation.showDetail')}
                >{expandedIndex === i ? "▾" : "▸"}</button>
              </td>
              <td><span class="sfb-badge">{est.nlSfb.code}</span></td>
              <td>{kozijnList[i]?.mark || "—"}</td>
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
            {#if expandedIndex === i}
              <tr class="detail-row">
                <td colspan="12">
                  {#if detailLoading}
                    <p class="hint">{$_('calculation.loadingDetail')}</p>
                  {:else if detailEstimate}
                    <div class="detail-block">
                      <h4>{$_('calculation.lineItems')} &mdash; {kozijnList[i]?.mark || kozijnList[i]?.name || ""}</h4>
                      <table class="detail-table">
                        <thead><tr>
                          <th>{$_('calculation.nlsfb')}</th>
                          <th>{$_('calculation.description')}</th>
                          <th class="num">{$_('calculation.quantity')}</th>
                          <th>{$_('calculation.unit')}</th>
                          <th class="num">{$_('calculation.unitPrice')}</th>
                          <th class="num">{$_('calculation.total')}</th>
                        </tr></thead>
                        <tbody>
                          {#each detailEstimate.lineItems || [] as item}
                            <tr>
                              <td><span class="sfb-badge">{item.nlSfbCode}</span></td>
                              <td>{item.description}</td>
                              <td class="num">{item.quantity.toFixed(2)}</td>
                              <td>{item.unit}</td>
                              <td class="num">&euro; {item.unitPrice.toFixed(2)}</td>
                              <td class="num">&euro; {item.total.toFixed(2)}</td>
                            </tr>
                          {/each}
                        </tbody>
                        <tfoot>
                          <tr>
                            <td colspan="5"><strong>{$_('calculation.total')}</strong></td>
                            <td class="num total">&euro; {(detailEstimate.totalCost || 0).toFixed(2)}</td>
                          </tr>
                        </tfoot>
                      </table>
                    </div>
                  {/if}
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <td colspan="3"><strong>{$_('calculation.totalProject')}</strong></td>
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
          <th>{$_('calculation.nlsfb')}</th><th>{$_('calculation.costBreakdown')}</th><th class="num">{$_('calculation.quantity')}</th>
          <th>{$_('calculation.unit')}</th><th class="num">{$_('calculation.unitPrice')}</th><th class="num">{$_('calculation.total')}</th>
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
    overflow: auto;
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

  /* ── Pricing configuration ─────────────────────────────── */

  .pricing-section {
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    margin-bottom: var(--sp-4);
  }

  .pricing-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-3) var(--sp-4);
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    text-align: left;
    cursor: default;
  }

  .chevron {
    display: inline-block;
    transition: transform 0.15s ease;
    color: var(--text-muted);
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .pricing-body {
    padding: 0 var(--sp-4) var(--sp-4);
  }

  .pricing-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(140px, 1fr));
    gap: var(--sp-3);
    margin-bottom: var(--sp-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .field span {
    font-size: 11px;
    color: var(--text-muted);
  }

  .field input[type="number"] {
    padding: var(--sp-1) var(--sp-2);
    background: var(--bg-surface);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .field input[type="number"]:disabled {
    opacity: 0.5;
  }

  .field.check {
    flex-direction: row;
    align-items: center;
    gap: var(--sp-2);
    align-self: end;
    padding-bottom: var(--sp-1);
  }

  .field.check span {
    font-size: 12px;
    color: var(--text-primary);
  }

  .pricing-preview {
    border-top: 1px solid var(--border-color, #e5e7eb);
    padding-top: var(--sp-3);
    margin-bottom: var(--sp-3);
    max-width: 360px;
  }

  .preview-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-primary);
    padding: 2px 0;
  }

  .preview-row .num {
    font-variant-numeric: tabular-nums;
  }

  .preview-total {
    border-top: 1px solid var(--amber);
    margin-top: var(--sp-1);
    padding-top: var(--sp-1);
    font-weight: 700;
    color: var(--amber);
  }

  .pricing-actions {
    display: flex;
    justify-content: flex-end;
  }

  .save-btn {
    padding: var(--sp-2) var(--sp-4);
    background: var(--amber);
    color: var(--bg-surface);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: default;
  }

  .save-btn:disabled {
    opacity: 0.6;
  }

  /* ── Summary ───────────────────────────────────────────── */

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

  tbody tr.expanded {
    background: rgba(217, 119, 6, 0.06);
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

  /* ── Expandable detail row ─────────────────────────────── */

  .expand-cell {
    padding: var(--sp-1) var(--sp-2);
    text-align: center;
  }

  .expand-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 12px;
    padding: 2px 6px;
    cursor: default;
  }

  .expand-btn:hover {
    color: var(--amber);
  }

  tbody tr.detail-row {
    background: var(--bg-surface-alt);
  }

  .detail-block h4 {
    font-size: 12px;
    color: var(--text-primary);
    margin: var(--sp-1) 0 var(--sp-2);
  }

  .detail-table {
    width: 100%;
    table-layout: auto;
    border: 1px solid var(--border-color, #e5e7eb);
  }

  .detail-table thead th {
    position: static;
  }

  .detail-table tfoot tr {
    border-top: 1px solid var(--amber);
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
