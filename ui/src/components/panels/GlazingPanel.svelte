<script>
  import { _ } from "svelte-i18n";
  import { currentKozijn, selectedCellIndex } from "../../stores/kozijn.js";
  import { invoke } from "../../lib/tauri.js";
  import { get } from "svelte/store";

  const GLASS_PRESETS = [
    { type: "Enkel", thickness: 6, ug: 5.8, panes: [{ thickness_mm: 6, pane_type: "float" }], spacer: "aluminium" },
    { type: "Dubbel", thickness: 20, ug: 2.8, panes: [{ thickness_mm: 4, pane_type: "float" }, { thickness_mm: 4, pane_type: "float" }], spacer: "aluminium" },
    { type: "HR++", thickness: 24, ug: 1.0, panes: [{ thickness_mm: 4, pane_type: "float" }, { thickness_mm: 4, pane_type: "low-e" }], spacer: "warm-edge" },
    { type: "HR+++", thickness: 30, ug: 0.7, panes: [{ thickness_mm: 4, pane_type: "float" }, { thickness_mm: 4, pane_type: "low-e" }, { thickness_mm: 4, pane_type: "low-e" }], spacer: "super-spacer" },
    { type: "Triple", thickness: 36, ug: 0.5, panes: [{ thickness_mm: 4, pane_type: "low-e" }, { thickness_mm: 4, pane_type: "float" }, { thickness_mm: 4, pane_type: "low-e" }], spacer: "super-spacer" },
  ];

  const PANE_TYPES = ["float", "gehard", "gelamineerd", "low-e"];
  $: SPACER_TYPES = [
    { value: "aluminium", label: $_('glazing.spacerAluminium') },
    { value: "warm-edge", label: $_('glazing.spacerWarmEdge') },
    { value: "super-spacer", label: $_('glazing.spacerSuperSpacer') },
  ];

  $: cell = $currentKozijn && $selectedCellIndex !== null
    ? $currentKozijn.cells[$selectedCellIndex]
    : null;

  $: glazing = cell?.glazing || null;

  async function applyPreset(preset) {
    if ($selectedCellIndex === null || !$currentKozijn) return;
    const newGlazing = {
      glassType: preset.type,
      thicknessMm: preset.thickness,
      ugValue: preset.ug,
      panes: preset.panes,
      spacerType: preset.spacer,
    };
    try {
      await invoke("update_cell_glazing", {
        id: $currentKozijn.id,
        cellIndex: $selectedCellIndex,
        glazingJson: JSON.stringify(newGlazing),
      });
      // Refresh
      const k = await invoke("get_kozijn", { id: $currentKozijn.id });
      currentKozijn.set(k);
    } catch (e) {
      console.error("Glazing update fout:", e);
    }
  }

  async function updateGlazingField(field, value) {
    if ($selectedCellIndex === null || !$currentKozijn || !glazing) return;
    const updated = { ...glazing, [field]: value };
    try {
      await invoke("update_cell_glazing", {
        id: $currentKozijn.id,
        cellIndex: $selectedCellIndex,
        glazingJson: JSON.stringify(updated),
      });
      const k = await invoke("get_kozijn", { id: $currentKozijn.id });
      currentKozijn.set(k);
    } catch (e) {
      console.error("Glazing update fout:", e);
    }
  }
</script>

{#if glazing}
  <div class="glazing-panel">
    <h3>{$_('glazing.title')}</h3>

    <div class="field">
      <label>{$_('glazing.glassType')}</label>
      <select value={glazing.glassType} onchange={(e) => applyPreset(GLASS_PRESETS.find(p => p.type === e.target.value) || GLASS_PRESETS[2])}>
        {#each GLASS_PRESETS as preset}
          <option value={preset.type}>{preset.type}</option>
        {/each}
      </select>
    </div>

    <div class="field-row">
      <div class="field">
        <label>{$_('glazing.thickness')}</label>
        <div class="value">{glazing.thicknessMm}</div>
      </div>
      <div class="field">
        <label>{$_('glazing.ugValue')}</label>
        <div class="value ug-badge" class:ug-good={glazing.ugValue < 1.3} class:ug-ok={glazing.ugValue >= 1.3 && glazing.ugValue < 2.0} class:ug-poor={glazing.ugValue >= 2.0}>
          {glazing.ugValue} W/m²K
        </div>
      </div>
    </div>

    <div class="field">
      <label>{$_('glazing.composition')}</label>
      <div class="pane-stack">
        {#each (glazing.panes || []) as pane, i}
          {#if i > 0}
            <div class="spacer-row">
              <span class="spacer-line"></span>
              <span class="spacer-label">{glazing.spacerType || "warm-edge"}</span>
              <span class="spacer-line"></span>
            </div>
          {/if}
          <div class="pane-row">
            <span class="pane-thickness">{pane.thickness_mm}mm</span>
            <select class="pane-type-select" value={pane.pane_type} disabled>
              {#each PANE_TYPES as pt}
                <option value={pt}>{pt}</option>
              {/each}
            </select>
          </div>
        {/each}
        {#if !glazing.panes || glazing.panes.length === 0}
          <div class="pane-row">
            <span class="pane-thickness">{glazing.thicknessMm}mm</span>
            <span class="pane-type">{glazing.glassType}</span>
          </div>
        {/if}
      </div>
    </div>

    <div class="field">
      <label>{$_('glazing.spacer')}</label>
      <select value={glazing.spacerType || "warm-edge"} onchange={(e) => updateGlazingField("spacerType", e.target.value)}>
        {#each SPACER_TYPES as st}
          <option value={st.value}>{st.label}</option>
        {/each}
      </select>
    </div>
  </div>
{/if}

<style>
  .glazing-panel {
    margin-bottom: var(--sp-4);
  }

  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--amber);
    margin-bottom: var(--sp-3);
    padding-bottom: var(--sp-2);
    border-bottom: var(--border-default);
  }

  .field {
    margin-bottom: var(--sp-3);
  }

  .field label {
    display: block;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--sp-1);
  }

  .field select {
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
  }

  .field select:focus {
    outline: none;
    border-color: var(--amber);
    box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.2);
  }

  .field-row {
    display: flex;
    gap: var(--sp-2);
  }

  .field-row .field {
    flex: 1;
  }

  .value {
    font-size: 13px;
    color: var(--text-primary);
    padding: var(--sp-2) 0;
  }

  .ug-badge {
    font-weight: 600;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    display: inline-block;
  }

  .ug-good { color: #16A34A; background: rgba(22, 163, 74, 0.1); }
  .ug-ok { color: #F59E0B; background: rgba(245, 158, 11, 0.1); }
  .ug-poor { color: #DC2626; background: rgba(220, 38, 38, 0.1); }

  .pane-stack {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-2);
    background: var(--bg-surface-alt);
    border-radius: var(--radius-sm);
    border: var(--border-default);
  }

  .pane-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 2px 4px;
    background: rgba(147, 197, 253, 0.15);
    border-radius: 2px;
  }

  .pane-thickness {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-primary);
    min-width: 32px;
  }

  .pane-type-select, .pane-type {
    font-size: 10px;
    color: var(--text-muted);
    background: transparent;
    border: none;
    padding: 0;
  }

  .spacer-row {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0 4px;
  }

  .spacer-line {
    flex: 1;
    height: 1px;
    background: var(--scaffold-gray);
  }

  .spacer-label {
    font-size: 9px;
    color: var(--scaffold-gray);
    text-transform: uppercase;
    white-space: nowrap;
  }
</style>
