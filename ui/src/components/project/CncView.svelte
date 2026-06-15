<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";
  import { exportCncGcode } from "../../lib/export.js";

  let kozijnen = [];
  let selectedId = "";
  let parts = [];
  let loading = false;

  // Operation variant -> i18n label + tooling + badge class.
  // Rust enum CncOperation serializes externally tagged with camelCase variants
  // and snake_case fields: { crossCut: { position_mm, angle_deg } }, etc.
  const OP_META = {
    crossCut: { labelKey: "cnc.op.crossCut", toolKey: "cnc.tool.saw" },
    mortise: { labelKey: "cnc.op.mortise", toolKey: "cnc.tool.router" },
    tenon: { labelKey: "cnc.op.tenon", toolKey: "cnc.tool.tenoner" },
    drill: { labelKey: "cnc.op.drill", toolKey: "cnc.tool.drill" },
    groove: { labelKey: "cnc.op.groove", toolKey: "cnc.tool.grooveCutter" },
  };

  onMount(load);

  async function load() {
    loading = true;
    try {
      kozijnen = (await invoke("get_all_kozijnen")) || [];
      if (!kozijnen.some((k) => k.id === selectedId)) {
        selectedId = kozijnen[0]?.id || "";
      }
      await loadParts();
    } catch (e) {
      console.error("Kozijnen laden mislukt:", e);
      toast.error($_("cnc.loadError") + ": " + e);
    }
    loading = false;
  }

  async function loadParts() {
    if (!selectedId) {
      parts = [];
      return;
    }
    loading = true;
    try {
      parts = (await invoke("get_cnc_parts", { id: selectedId })) || [];
    } catch (e) {
      console.error("CNC-onderdelen laden mislukt:", e);
      toast.error($_("cnc.loadError") + ": " + e);
      parts = [];
    }
    loading = false;
  }

  async function handleExportGcode() {
    try {
      await exportCncGcode();
    } catch (e) {
      console.error("G-code export mislukt:", e);
      toast.error($_("cnc.exportError") + ": " + e);
    }
  }

  function fmt(n) {
    if (n == null || Number.isNaN(n)) return "—";
    return Number.isInteger(n) ? String(n) : n.toFixed(1);
  }

  function describeOp(op) {
    const kind = Object.keys(op || {})[0] || "";
    const d = (op && op[kind]) || {};
    const meta = OP_META[kind] || null;
    let position = "—";
    let dims = "—";
    switch (kind) {
      case "crossCut":
        position = fmt(d.position_mm);
        dims = fmt(d.angle_deg) + "°";
        break;
      case "mortise":
        position = `x ${fmt(d.x)}, y ${fmt(d.y)}`;
        dims = `${fmt(d.width)} × ${fmt(d.length)} × ${fmt(d.depth)}`;
        break;
      case "tenon":
        position = `x ${fmt(d.x)}, y ${fmt(d.y)}`;
        dims = `${fmt(d.width)} × ${fmt(d.length)}`;
        break;
      case "drill":
        position = `x ${fmt(d.x)}, y ${fmt(d.y)}`;
        dims = `⌀${fmt(d.diameter)} × ${fmt(d.depth)}`;
        break;
      case "groove":
        position = `${fmt(d.start_x)} → ${fmt(d.end_x)}`;
        dims = `${fmt(d.width)} × ${fmt(d.depth)}`;
        break;
    }
    return { kind, meta, position, dims };
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>{$_("cnc.title")}</h2>
    <div class="toolbar-actions">
      <select
        class="kozijn-select"
        aria-label={$_("cnc.selectKozijn")}
        bind:value={selectedId}
        onchange={loadParts}
        disabled={kozijnen.length === 0}
      >
        {#each kozijnen as k (k.id)}
          <option value={k.id}>{k.mark} — {k.name}</option>
        {/each}
      </select>
      <button class="action-btn" onclick={load}>{$_("cnc.refresh")}</button>
      <button class="action-btn primary" onclick={handleExportGcode}>{$_("cnc.exportGcode")}</button>
    </div>
  </div>

  {#if loading}
    <p class="hint">{$_("cnc.loading")}</p>
  {:else if kozijnen.length === 0}
    <p class="hint">{$_("cnc.noKozijnen")}</p>
  {:else if parts.length === 0}
    <p class="hint">{$_("cnc.noParts")}</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th class="idx">#</th>
            <th>{$_("cnc.col.operation")}</th>
            <th>{$_("cnc.col.position")}</th>
            <th>{$_("cnc.col.dimensions")}</th>
            <th>{$_("cnc.col.tooling")}</th>
          </tr>
        </thead>
        <tbody>
          {#each parts as part}
            <tr class="part-row">
              <td colspan="5">
                <span class="piece-id">{part.pieceId}</span>
                <span class="part-detail">{part.profileName}</span>
                <span class="part-detail">{$_("cnc.part.material")}: {part.material}</span>
                <span class="part-detail">{$_("cnc.part.length")}: {fmt(part.grossLengthMm)} mm</span>
                <span class="op-count">{part.operations?.length || 0} {$_("cnc.part.operations")}</span>
              </td>
            </tr>
            {#each part.operations || [] as op, i}
              {@const info = describeOp(op)}
              <tr>
                <td class="num idx">{i + 1}</td>
                <td>
                  <span class="op-badge {info.kind}">
                    {info.meta ? $_(info.meta.labelKey) : info.kind}
                  </span>
                </td>
                <td class="num">{info.position}</td>
                <td class="num">{info.dims}</td>
                <td>{info.meta ? $_(info.meta.toolKey) : "—"}</td>
              </tr>
            {/each}
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
  .toolbar-actions { display: flex; gap: var(--sp-2); align-items: center; }
  .kozijn-select { padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; max-width: 240px; }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .table-container { flex: 1; overflow: auto; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  th.idx { width: 36px; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.num { font-variant-numeric: tabular-nums; }
  td.idx { color: var(--text-muted); }
  .part-row td { background: var(--bg-surface-alt); padding: var(--sp-2) var(--sp-3); }
  .part-row:hover { background: var(--bg-surface-alt); }
  .piece-id { font-weight: 700; color: var(--amber); margin-right: var(--sp-3); }
  .part-detail { color: var(--text-muted); margin-right: var(--sp-3); }
  .op-count { float: right; padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; background: rgba(217, 119, 6, 0.15); color: var(--amber); }
  .op-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; background: #333; color: #ccc; }
  .op-badge.crossCut { background: #3b82f6; color: #fff; }
  .op-badge.mortise { background: #8b5cf6; color: #fff; }
  .op-badge.tenon { background: #22c55e; color: #111; }
  .op-badge.drill { background: #f59e0b; color: #111; }
  .op-badge.groove { background: #14b8a6; color: #111; }
</style>
