<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let quotations = [];
  let loading = false;

  onMount(loadQuotations);

  async function loadQuotations() {
    loading = true;
    try {
      quotations = await invoke("get_quotations", {}) || [];
    } catch (e) {
      console.error("Offertes laden mislukt:", e);
    }
    loading = false;
  }

  async function createQuotation() {
    try {
      await invoke("create_quotation", { kozijnMarks: [], totalInclBtw: 0 });
      await loadQuotations();
      toast.success("Offerte aangemaakt");
    } catch (e) {
      toast.error("Fout bij aanmaken offerte: " + e);
    }
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>Offertes</h2>
    <div class="toolbar-actions">
      <button class="action-btn" onclick={loadQuotations}>Vernieuwen</button>
      <button class="action-btn primary" onclick={createQuotation}>+ Nieuwe offerte</button>
    </div>
  </div>
  {#if loading}
    <p class="hint">Laden...</p>
  {:else if quotations.length === 0}
    <p class="hint">Nog geen offertes. Klik op "+ Nieuwe offerte" om te beginnen.</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Versie</th>
            <th>Status</th>
            <th>Datum</th>
            <th>Totaal incl. BTW</th>
            <th>Wijziging</th>
          </tr>
        </thead>
        <tbody>
          {#each quotations as q}
            <tr>
              <td>v{q.version || 1}</td>
              <td><span class="status-badge {q.status || 'draft'}">{q.status || "concept"}</span></td>
              <td>{q.createdAt?.slice(0, 10) || "\u2014"}</td>
              <td class="num">&euro; {(q.totalInclBtw || 0).toFixed(2)}</td>
              <td>{q.changeDescription || "\u2014"}</td>
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
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .status-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; }
  .status-badge.draft { background: #333; color: #888; }
  .status-badge.sent { background: #3b82f6; color: #fff; }
  .status-badge.accepted { background: #22c55e; color: #111; }
  .status-badge.rejected { background: #ef4444; color: #fff; }
</style>
