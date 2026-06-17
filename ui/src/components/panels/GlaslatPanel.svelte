<script>
  import {
    currentKozijn,
    selectedCellIndex,
    updateCellGlaslat,
  } from "../../stores/kozijn.js";

  let { visible = true } = $props();

  const POSITIONS = [
    { value: "binnen", label: "Binnen" },
    { value: "buiten", label: "Buiten" },
  ];

  const DEFAULT_GLASLAT = {
    position: "binnen",
    material: "meranti",
    widthMm: 15,
    heightMm: 17,
    mitered: true,
  };

  let collapsed = $state(false);

  let selectedCell = $derived(
    $currentKozijn && $selectedCellIndex !== null
      ? $currentKozijn.cells[$selectedCellIndex]
      : null
  );

  // Glaslatten apply to glazed cells (not closed panels / ventilation grilles).
  let isGlazed = $derived(
    !!selectedCell &&
      selectedCell.panelType !== "panel" &&
      selectedCell.panelType !== "ventilation"
  );

  let gl = $derived({ ...DEFAULT_GLASLAT, ...(selectedCell?.glaslat || {}) });

  // KVT minimum: glaslat height ≥ 17 mm.
  let heightOk = $derived((gl.heightMm ?? 17) >= 17);

  async function update(patch) {
    if ($selectedCellIndex === null) return;
    await updateCellGlaslat($selectedCellIndex, { ...gl, ...patch });
  }
</script>

{#if visible && isGlazed}
  <div class="glaslat-panel">
    <button class="collapse-header" onclick={() => (collapsed = !collapsed)}>
      <span class="collapse-icon" class:open={!collapsed}>&#9656;</span>
      <h3>Glaslat</h3>
    </button>

    {#if !collapsed}
      <div class="section">
        <div class="field-row">
          <div class="field">
            <label>Positie</label>
            <select value={gl.position} onchange={(e) => update({ position: e.target.value })}>
              {#each POSITIONS as p}
                <option value={p.value}>{p.label}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label>Materiaal</label>
            <input value={gl.material} onchange={(e) => update({ material: e.target.value })} />
          </div>
        </div>

        <div class="field-row">
          <div class="field">
            <label>Breedte (mm)</label>
            <input
              type="number"
              min="10"
              value={gl.widthMm}
              onchange={(e) => update({ widthMm: parseFloat(e.target.value) })}
            />
          </div>
          <div class="field">
            <label>Hoogte (mm)</label>
            <input
              type="number"
              min="10"
              value={gl.heightMm}
              onchange={(e) => update({ heightMm: parseFloat(e.target.value) })}
            />
          </div>
        </div>

        <label class="check">
          <input
            type="checkbox"
            checked={gl.mitered}
            onchange={(e) => update({ mitered: e.target.checked })}
          />
          Verstek (45°) hoeken
        </label>

        {#if !heightOk}
          <p class="warn">⚠ Hoogte &lt; 17mm (KVT-minimum)</p>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .glaslat-panel {
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
  .check {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-primary);
  }
  .warn {
    margin: 0;
    font-size: 11px;
    color: var(--amber, #d97706);
  }
</style>
