<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";
  import { currentKozijn } from "../../stores/kozijn.js";
  import { kozijnen } from "../../stores/project.js";

  // CouplingType enum values as serialized by the backend (snake_case serde)
  const COUPLING_TYPES = ["side_to_side", "top_to_bottom", "corner"];

  let combinations = $state([]);
  let loading = $state(false);
  let showCreate = $state(false);
  let newName = $state("");
  let newMark = $state("");

  // Coupling form (one combination at a time, like the create form)
  let couplingFormCombo = $state(null);
  let cplA = $state("");
  let cplB = $state("");
  let cplType = $state("side_to_side");
  let cplWidth = $state(100);

  onMount(loadCombinations);

  async function loadCombinations() {
    loading = true;
    try {
      combinations = (await invoke("get_combinations", {})) || [];
    } catch (e) {
      console.error("Combinaties laden mislukt:", e);
    }
    loading = false;
  }

  async function createCombination() {
    if (!newName.trim()) {
      toast.error($_("combination.nameRequired"));
      return;
    }
    try {
      await invoke("create_combination", {
        name: newName.trim(),
        mark: newMark.trim(),
      });
      newName = "";
      newMark = "";
      showCreate = false;
      await loadCombinations();
      toast.success($_("combination.created"));
    } catch (e) {
      toast.error($_("combination.createError") + ": " + e);
    }
  }

  function nextOffsetX(combo) {
    let maxX = 0;
    for (const m of combo.members || []) {
      const k = $kozijnen.find((k) => k.id === m.kozijnId);
      const w = k?.frame?.outerWidth || 0;
      maxX = Math.max(maxX, (m.offsetX || 0) + w);
    }
    return maxX;
  }

  async function addCurrentKozijn(combo) {
    const k = $currentKozijn;
    if (!k) {
      toast.error($_("combination.noCurrentKozijn"));
      return;
    }
    try {
      await invoke("add_to_combination", {
        combinationId: combo.id,
        kozijnId: k.id,
        offsetX: nextOffsetX(combo),
        offsetY: 0,
      });
      await loadCombinations();
      toast.success($_("combination.added"));
    } catch (e) {
      toast.error($_("combination.addError") + ": " + e);
    }
  }

  async function removeCombination(combo) {
    try {
      await invoke("remove_combination", { combinationId: combo.id });
      await loadCombinations();
      toast.success($_("combination.removed"));
    } catch (e) {
      toast.error($_("combination.removeError") + ": " + e);
    }
  }

  function kozijnLabel(id) {
    const k = $kozijnen.find((k) => k.id === id);
    if (k) return `${k.mark} — ${k.name}`;
    return id ? id.slice(0, 8) : "—";
  }

  function toggleCouplingForm(combo) {
    if (couplingFormCombo === combo.id) {
      couplingFormCombo = null;
      return;
    }
    const members = combo.members || [];
    cplA = members[0]?.kozijnId || "";
    cplB = members[1]?.kozijnId || "";
    cplType = "side_to_side";
    cplWidth = 100;
    couplingFormCombo = combo.id;
  }

  async function addCoupling(combo) {
    if (!cplA || !cplB || cplA === cplB) {
      toast.error($_("combination.couplingMembersInvalid"));
      return;
    }
    try {
      await invoke("add_coupling", {
        combinationId: combo.id,
        memberAId: cplA,
        memberBId: cplB,
        couplingType: cplType,
        couplingWidth: Number(cplWidth) || 0,
      });
      couplingFormCombo = null;
      await loadCombinations();
      toast.success($_("combination.couplingAdded"));
    } catch (e) {
      toast.error($_("combination.couplingAddError") + ": " + e);
    }
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>{$_("combination.title")}</h2>
    <div class="toolbar-actions">
      <button class="action-btn" onclick={loadCombinations}>{$_("combination.refresh")}</button>
      <button class="action-btn primary" onclick={() => (showCreate = !showCreate)}>{$_("combination.new")}</button>
    </div>
  </div>

  {#if showCreate}
    <div class="create-form">
      <input
        type="text"
        bind:value={newName}
        placeholder={$_("combination.namePlaceholder")}
      />
      <input
        type="text"
        class="mark-input"
        bind:value={newMark}
        placeholder={$_("combination.markPlaceholder")}
      />
      <button class="action-btn primary" onclick={createCombination}>{$_("combination.create")}</button>
      <button class="action-btn" onclick={() => (showCreate = false)}>{$_("combination.cancel")}</button>
    </div>
  {/if}

  {#if loading}
    <p class="hint">{$_("combination.loading")}</p>
  {:else if combinations.length === 0}
    <p class="hint">{$_("combination.empty")}</p>
  {:else}
    <div class="list-container">
      {#each combinations as combo (combo.id)}
        <div class="combo-card">
          <div class="combo-header">
            <div class="combo-title">
              <span class="combo-name">{combo.name}</span>
              {#if combo.mark}
                <span class="mark-badge">{combo.mark}</span>
              {/if}
              <span class="count">{(combo.members || []).length} &times; {$_("combination.kozijn")}</span>
            </div>
            <div class="combo-actions">
              <button class="action-btn" onclick={() => addCurrentKozijn(combo)}>{$_("combination.addCurrent")}</button>
              <button class="action-btn danger" onclick={() => removeCombination(combo)}>{$_("combination.remove")}</button>
            </div>
          </div>

          <div class="section-label">{$_("combination.members")}</div>
          {#if (combo.members || []).length === 0}
            <p class="hint">{$_("combination.noMembers")}</p>
          {:else}
            <table>
              <thead>
                <tr>
                  <th>{$_("combination.kozijn")}</th>
                  <th class="num-col">{$_("combination.offsetX")}</th>
                  <th class="num-col">{$_("combination.offsetY")}</th>
                </tr>
              </thead>
              <tbody>
                {#each combo.members as m}
                  <tr>
                    <td>{kozijnLabel(m.kozijnId)}</td>
                    <td class="num">{Math.round(m.offsetX || 0)}</td>
                    <td class="num">{Math.round(m.offsetY || 0)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}

          <div class="section-label">{$_("combination.couplings")}</div>
          {#if (combo.couplings || []).length === 0}
            <p class="hint">{$_("combination.noCouplings")}</p>
          {:else}
            <table>
              <thead>
                <tr>
                  <th>{$_("combination.couplingTypeLabel")}</th>
                  <th>{$_("combination.memberA")}</th>
                  <th>{$_("combination.memberB")}</th>
                  <th class="num-col">{$_("combination.width")}</th>
                </tr>
              </thead>
              <tbody>
                {#each combo.couplings as c}
                  <tr>
                    <td>
                      <span class="coupling-badge {c.couplingType}">
                        {$_(`combination.couplingType.${c.couplingType}`)}
                      </span>
                    </td>
                    <td>{kozijnLabel(c.memberAId)}</td>
                    <td>{kozijnLabel(c.memberBId)}</td>
                    <td class="num">{Math.round(c.couplingWidth || 0)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}

          {#if (combo.members || []).length >= 2}
            {#if couplingFormCombo === combo.id}
              <div class="coupling-form">
                <select bind:value={cplA} title={$_("combination.memberA")}>
                  {#each combo.members as m}
                    <option value={m.kozijnId}>{kozijnLabel(m.kozijnId)}</option>
                  {/each}
                </select>
                <select bind:value={cplB} title={$_("combination.memberB")}>
                  {#each combo.members as m}
                    <option value={m.kozijnId}>{kozijnLabel(m.kozijnId)}</option>
                  {/each}
                </select>
                <select bind:value={cplType} title={$_("combination.couplingTypeLabel")}>
                  {#each COUPLING_TYPES as t}
                    <option value={t}>{$_(`combination.couplingType.${t}`)}</option>
                  {/each}
                </select>
                <input
                  type="number"
                  class="width-input"
                  bind:value={cplWidth}
                  min="0"
                  step="1"
                  title={$_("combination.width")}
                />
                <button class="action-btn primary" onclick={() => addCoupling(combo)}>{$_("combination.create")}</button>
                <button class="action-btn" onclick={() => (couplingFormCombo = null)}>{$_("combination.cancel")}</button>
              </div>
            {:else}
              <button class="action-btn coupling-add-btn" onclick={() => toggleCouplingForm(combo)}>
                {$_("combination.addCoupling")}
              </button>
            {/if}
          {/if}
        </div>
      {/each}
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
  .action-btn.danger { color: #ef4444; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }
  .create-form { display: flex; gap: var(--sp-2); align-items: center; margin-bottom: var(--sp-3); padding: var(--sp-3); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); }
  .create-form input { flex: 1; padding: var(--sp-2); background: var(--bg-surface); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; }
  .create-form input.mark-input { flex: 0 0 140px; }
  .list-container { flex: 1; overflow: auto; display: flex; flex-direction: column; gap: var(--sp-3); }
  .combo-card { border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); padding: var(--sp-3); background: var(--bg-surface-alt); }
  .combo-header { display: flex; justify-content: space-between; align-items: center; gap: var(--sp-2); margin-bottom: var(--sp-2); }
  .combo-title { display: flex; align-items: center; gap: var(--sp-2); }
  .combo-name { font-size: 14px; font-weight: 700; color: var(--text-primary); }
  .mark-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; background: var(--amber); color: var(--bg-surface); }
  .count { font-size: 11px; color: var(--text-muted); }
  .combo-actions { display: flex; gap: var(--sp-2); }
  .section-label { font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); margin: var(--sp-2) 0; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  thead th { background: var(--bg-surface); padding: var(--sp-2) var(--sp-3); text-align: left; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-muted); border-bottom: 2px solid var(--border-color, #333); }
  thead th.num-col { text-align: right; }
  tbody tr { border-bottom: 1px solid var(--border-color, #333); }
  tbody tr:hover { background: rgba(217, 119, 6, 0.04); }
  td { padding: var(--sp-2) var(--sp-3); color: var(--text-primary); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .coupling-form { display: flex; gap: var(--sp-2); align-items: center; margin-top: var(--sp-2); }
  .coupling-form select { flex: 1; min-width: 0; padding: var(--sp-2); background: var(--bg-surface); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; }
  .coupling-form input.width-input { flex: 0 0 90px; padding: var(--sp-2); background: var(--bg-surface); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; }
  .coupling-add-btn { margin-top: var(--sp-2); }
  .coupling-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; background: #333; color: #ccc; }
  .coupling-badge.side_to_side { background: #3b82f6; color: #fff; }
  .coupling-badge.top_to_bottom { background: #22c55e; color: #111; }
  .coupling-badge.corner { background: #a855f7; color: #fff; }
</style>
