<script>
  import { _ } from "svelte-i18n";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let proposals = [];
  let orders = [];
  let loading = false;
  let generated = false;
  let ordersGenerated = false;
  let mode = "proposals"; // "proposals" | "orders"

  $: grandTotal = proposals.reduce((sum, p) => sum + (p.totalExclBtw || 0), 0);
  $: ordersGrandTotal = orders.reduce((sum, o) => sum + (o.totalInclBtw || 0), 0);

  async function generateProposals() {
    loading = true;
    try {
      proposals = (await invoke("generate_purchase_proposals", {})) || [];
      generated = true;
      if (proposals.length > 0) {
        toast.success($_("procurement.generated"));
      }
    } catch (e) {
      console.error("Inkoopvoorstel genereren mislukt:", e);
      toast.error($_("procurement.generateError") + ": " + e);
    }
    loading = false;
  }

  async function generateOrders() {
    loading = true;
    try {
      orders = (await invoke("generate_purchase_orders", {})) || [];
      ordersGenerated = true;
      if (orders.length > 0) {
        toast.success($_("procurement.generated"));
      }
    } catch (e) {
      console.error("Bestellingen genereren mislukt:", e);
      toast.error($_("procurement.generateError") + ": " + e);
    }
    loading = false;
  }

  function runGenerate() {
    if (mode === "orders") generateOrders();
    else generateProposals();
  }

  function fmtQty(n) {
    const v = n || 0;
    return Number.isInteger(v) ? String(v) : v.toFixed(2);
  }

  function fmtEur(n) {
    return "€ " + (n || 0).toFixed(2);
  }

  function categoryClass(category) {
    const c = (category || "").toLowerCase();
    return ["hout", "glas", "beslag", "rubber"].includes(c) ? c : "other";
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>{$_("procurement.title")}</h2>
    <div class="toolbar-actions">
      <div class="mode-toggle">
        <button class="mode-btn" class:active={mode === "proposals"} onclick={() => (mode = "proposals")}>
          {$_("procurement.proposals")}
        </button>
        <button class="mode-btn" class:active={mode === "orders"} onclick={() => (mode = "orders")}>
          {$_("procurement.orders")}
        </button>
      </div>
      <button class="action-btn primary" onclick={runGenerate} disabled={loading}>
        {mode === "orders" ? $_("procurement.generateOrders") : $_("procurement.generate")}
      </button>
    </div>
  </div>

  {#if loading}
    <p class="hint">{$_("procurement.loading")}</p>
  {:else if mode === "proposals"}
    {#if !generated}
    <p class="hint">{$_("procurement.emptyHint")}</p>
    {:else if proposals.length === 0}
    <p class="hint">{$_("procurement.noResults")}</p>
    {:else}
    <div class="table-container">
      {#each proposals as proposal (proposal.id)}
        <div class="group">
          <div class="group-header">
            <span class="cat-badge {categoryClass(proposal.category)}">{proposal.category}</span>
            <span class="group-total">
              {$_("procurement.totalExclBtw")}: <strong>{fmtEur(proposal.totalExclBtw)}</strong>
            </span>
          </div>
          <table>
            <thead>
              <tr>
                <th>{$_("procurement.colDescription")}</th>
                <th>{$_("procurement.colArticle")}</th>
                <th class="num">{$_("procurement.colQuantity")}</th>
                <th>{$_("procurement.colUnit")}</th>
                <th class="num">{$_("procurement.colUnitPrice")}</th>
                <th class="num">{$_("procurement.colTotal")}</th>
                <th>{$_("procurement.colKozijnen")}</th>
              </tr>
            </thead>
            <tbody>
              {#each proposal.items as item}
                <tr>
                  <td>{item.description}</td>
                  <td>{item.articleNumber || "—"}</td>
                  <td class="num">{fmtQty(item.quantity)}</td>
                  <td>{item.unit}</td>
                  <td class="num">{fmtEur(item.unitPrice)}</td>
                  <td class="num">{fmtEur(item.total)}</td>
                  <td class="marks">{(item.kozijnMarks || []).join(", ") || "—"}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/each}

      <div class="grand-total">
        <span>{$_("procurement.grandTotal")}</span>
        <strong>{fmtEur(grandTotal)}</strong>
      </div>
    </div>
    {/if}
  {:else}
    {#if !ordersGenerated}
    <p class="hint">{$_("procurement.ordersEmptyHint")}</p>
    {:else if orders.length === 0}
    <p class="hint">{$_("procurement.noResults")}</p>
    {:else}
    <div class="table-container">
      <p class="hint export-note">{$_("procurement.orderExportHint")}</p>
      {#each orders as order (order.orderNumber)}
        <div class="order-doc">
          <div class="order-head">
            <div class="order-id">
              <span class="order-no">{order.orderNumber}</span>
              <span class="cat-badge {categoryClass(order.category)}">{order.category}</span>
            </div>
            <span class="order-supplier">{order.supplierName}</span>
          </div>
          <div class="order-meta">
            <span>{$_("procurement.project")}: {order.projectName || "—"}{order.projectNumber ? ` (${order.projectNumber})` : ""}</span>
            <span>{$_("procurement.deliveryAddress")}: {order.deliveryAddress || "—"}</span>
          </div>
          <table>
            <thead>
              <tr>
                <th>{$_("procurement.colDescription")}</th>
                <th>{$_("procurement.colArticle")}</th>
                <th class="num">{$_("procurement.colQuantity")}</th>
                <th>{$_("procurement.colUnit")}</th>
                <th class="num">{$_("procurement.colUnitPrice")}</th>
                <th class="num">{$_("procurement.colTotal")}</th>
                <th>{$_("procurement.colKozijnen")}</th>
              </tr>
            </thead>
            <tbody>
              {#each order.lines as line}
                <tr>
                  <td>{line.description}</td>
                  <td>{line.articleNumber || "—"}</td>
                  <td class="num">{fmtQty(line.quantity)}</td>
                  <td>{line.unit}</td>
                  <td class="num">{fmtEur(line.unitPrice)}</td>
                  <td class="num">{fmtEur(line.total)}</td>
                  <td class="marks">{(line.kozijnMarks || []).join(", ") || "—"}</td>
                </tr>
              {/each}
            </tbody>
          </table>
          <div class="order-totals">
            <div><span>{$_("procurement.subtotal")}</span><span>{fmtEur(order.subtotalExclBtw)}</span></div>
            <div><span>{$_("procurement.btw")} {order.btwPct}%</span><span>{fmtEur(order.btwAmount)}</span></div>
            <div class="grand"><span>{$_("procurement.totalInclBtw")}</span><span>{fmtEur(order.totalInclBtw)}</span></div>
          </div>
        </div>
      {/each}
      <div class="grand-total">
        <span>{$_("procurement.totalInclBtw")}</span>
        <strong>{fmtEur(ordersGrandTotal)}</strong>
      </div>
    </div>
    {/if}
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-2); }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .action-btn:disabled { opacity: 0.6; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  .group { margin-bottom: var(--sp-4); }
  .group-header { display: flex; justify-content: space-between; align-items: center; padding: var(--sp-2) 0; }
  .group-total { font-size: 12px; color: var(--text-muted); }
  .group-total strong { color: var(--text-primary); font-variant-numeric: tabular-nums; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  thead th.num { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  td.marks { color: var(--text-muted); }
  .cat-badge { padding: 2px 10px; border-radius: 10px; font-size: 11px; font-weight: 700; text-transform: uppercase; }
  .cat-badge.hout { background: #92400e; color: #fde68a; }
  .cat-badge.glas { background: #3b82f6; color: #fff; }
  .cat-badge.beslag { background: #6b7280; color: #fff; }
  .cat-badge.rubber { background: #1f2937; color: #d1d5db; }
  .cat-badge.other { background: #333; color: #888; }
  .grand-total { display: flex; justify-content: flex-end; align-items: center; gap: var(--sp-3); padding: var(--sp-3); border-top: 2px solid var(--border-color, #333); font-size: 13px; color: var(--text-primary); }
  .grand-total strong { font-size: 14px; color: var(--amber); font-variant-numeric: tabular-nums; }

  .mode-toggle { display: flex; border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); overflow: hidden; }
  .mode-btn { padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); color: var(--text-muted); border: none; font-size: 12px; font-weight: 600; cursor: default; }
  .mode-btn.active { background: var(--amber); color: var(--bg-surface); }
  .export-note { margin: 0 0 var(--sp-3); }
  .order-doc { border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); padding: var(--sp-3); margin-bottom: var(--sp-4); }
  .order-head { display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid var(--amber); padding-bottom: var(--sp-2); margin-bottom: var(--sp-2); }
  .order-id { display: flex; align-items: center; gap: var(--sp-2); }
  .order-no { font-size: 13px; font-weight: 700; letter-spacing: 0.04em; color: var(--amber); }
  .order-supplier { font-size: 12px; color: #d97706; font-style: italic; }
  .order-meta { display: flex; flex-wrap: wrap; gap: var(--sp-1) var(--sp-4); font-size: 11px; color: var(--text-muted); margin-bottom: var(--sp-2); }
  .order-totals { display: flex; flex-direction: column; align-items: flex-end; gap: 2px; margin-top: var(--sp-2); font-size: 12px; }
  .order-totals > div { display: flex; gap: var(--sp-4); min-width: 220px; justify-content: space-between; color: var(--text-secondary); }
  .order-totals .grand { font-weight: 700; color: var(--text-primary); border-top: 1px solid var(--border-color, #333); padding-top: 2px; margin-top: 2px; }
  .order-totals .grand span:last-child { color: var(--amber); }
</style>
