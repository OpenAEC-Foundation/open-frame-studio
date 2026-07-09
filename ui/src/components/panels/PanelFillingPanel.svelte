<script>
  import {
    currentKozijn,
    selectedCellIndex,
    updateCellPanelFilling,
  } from "../../stores/kozijn.js";

  let { visible = true } = $props();

  const FILLING_TYPES = [
    { value: "sandwich", label: "Sandwichpaneel" },
    { value: "solid", label: "Massief paneel" },
    { value: "door_panel", label: "Deurpaneel" },
    { value: "ventilation", label: "Ventilatierooster" },
    { value: "blind", label: "Blind paneel" },
  ];
  const MOUNTINGS = [
    { value: "kalf", label: "Op kalf" },
    { value: "glass", label: "Op glas" },
  ];

  const DEFAULT_FILLING = {
    fillingType: "sandwich",
    material: "sandwich-pur",
    thicknessMm: 40,
    uValue: 0.6,
    edgeBanding: "",
    color: "RAL9010",
    manufacturer: "",
    ventilation: null,
    setbackMm: null,
  };

  let collapsed = $state(false);

  let selectedCell = $derived(
    $currentKozijn && $selectedCellIndex !== null
      ? $currentKozijn.cells[$selectedCellIndex]
      : null
  );

  // Vakvulling only applies to closed-panel and ventilation cells.
  let isFillable = $derived(
    !!selectedCell &&
      (selectedCell.panelType === "panel" ||
        selectedCell.panelType === "ventilation")
  );

  // Current filling merged with defaults so the inputs always have a value.
  let pf = $derived({ ...DEFAULT_FILLING, ...(selectedCell?.panelFilling || {}) });

  async function update(patch) {
    if ($selectedCellIndex === null) return;
    const next = { ...pf, ...patch };
    if (next.fillingType === "ventilation") {
      next.ventilation = next.ventilation || { mounting: "kalf", capacityDm3s: 10 };
    } else {
      next.ventilation = null;
    }
    await updateCellPanelFilling($selectedCellIndex, next);
  }

  function updateVent(patch) {
    const vent = {
      mounting: "kalf",
      capacityDm3s: 10,
      ...(pf.ventilation || {}),
      ...patch,
    };
    update({ ventilation: vent });
  }
</script>

{#if visible && isFillable}
  <div class="filling-panel">
    <button class="collapse-header" onclick={() => (collapsed = !collapsed)}>
      <span class="collapse-icon" class:open={!collapsed}>&#9656;</span>
      <h3>Vakvulling</h3>
    </button>

    {#if !collapsed}
      <div class="section">
        <div class="field">
          <label>Type</label>
          <select value={pf.fillingType} onchange={(e) => update({ fillingType: e.target.value })}>
            {#each FILLING_TYPES as t}
              <option value={t.value}>{t.label}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <label>Materiaal</label>
          <input value={pf.material} onchange={(e) => update({ material: e.target.value })} />
        </div>

        <div class="field-row">
          <div class="field">
            <label>Dikte (mm)</label>
            <input
              type="number"
              min="4"
              value={pf.thicknessMm}
              onchange={(e) => update({ thicknessMm: parseFloat(e.target.value) })}
            />
          </div>
          <div class="field">
            <label>U (W/m²K)</label>
            <input
              type="number"
              step="0.01"
              min="0"
              value={pf.uValue}
              onchange={(e) => update({ uValue: parseFloat(e.target.value) })}
            />
          </div>
          <div class="field">
            <label title="Inzetdiepte t.o.v. de binnenkant; leeg = automatisch gecentreerd">Inzet (mm)</label>
            <input
              type="number"
              step="1"
              placeholder="auto"
              value={pf.setbackMm ?? ""}
              onchange={(e) => update({ setbackMm: e.target.value === "" ? null : parseFloat(e.target.value) })}
            />
          </div>
        </div>

        <div class="field-row">
          <div class="field">
            <label>Kleur (RAL)</label>
            <input value={pf.color} onchange={(e) => update({ color: e.target.value })} />
          </div>
          <div class="field">
            <label>Randband</label>
            <input value={pf.edgeBanding} onchange={(e) => update({ edgeBanding: e.target.value })} />
          </div>
        </div>

        {#if pf.fillingType === "ventilation"}
          <div class="field-row">
            <div class="field">
              <label>Montage</label>
              <select
                value={pf.ventilation?.mounting || "kalf"}
                onchange={(e) => updateVent({ mounting: e.target.value })}
              >
                {#each MOUNTINGS as m}
                  <option value={m.value}>{m.label}</option>
                {/each}
              </select>
            </div>
            <div class="field">
              <label>Capaciteit (dm³/s)</label>
              <input
                type="number"
                min="0"
                value={pf.ventilation?.capacityDm3s ?? 10}
                onchange={(e) => updateVent({ capacityDm3s: parseFloat(e.target.value) })}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .filling-panel {
    border-top: var(--border-default);
  }
  .collapse-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    cursor: pointer;
  }
  .collapse-header h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-primary);
  }
  .collapse-icon {
    display: inline-block;
    transition: transform 0.15s;
    color: var(--text-muted);
  }
  .collapse-icon.open {
    transform: rotate(90deg);
  }
  .section {
    padding: 4px 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
  }
  .field label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }
  .field input,
  .field select {
    width: 100%;
    padding: 4px 6px;
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
  }
  .field-row {
    display: flex;
    gap: 8px;
  }
</style>
