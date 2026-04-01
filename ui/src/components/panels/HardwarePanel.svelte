<script>
  import {
    currentKozijn,
    selectedCellIndex,
    updateCellHardware,
    autoSelectHardware,
    updateSecurityClass,
  } from "../../stores/kozijn.js";

  export let visible = true;

  let collapsed = false;

  const SECURITY_CLASSES = [
    { value: "none", label: "Geen" },
    { value: "RC1", label: "RC1 (basis)" },
    { value: "RC2", label: "RC2 (standaard nieuwbouw)" },
    { value: "RC3", label: "RC3 (verhoogd)" },
    { value: "RC4", label: "RC4 (hoog)" },
    { value: "RC5", label: "RC5 (zeer hoog)" },
    { value: "RC6", label: "RC6 (maximaal)" },
  ];

  const OPERABLE_TYPES = [
    "turn_tilt", "turn", "tilt", "sliding", "door",
  ];

  $: selectedCell =
    $currentKozijn && $selectedCellIndex !== null
      ? $currentKozijn.cells[$selectedCellIndex]
      : null;

  $: hw = selectedCell?.hardwareSet || null;

  $: isOperable = selectedCell
    ? OPERABLE_TYPES.includes(selectedCell.panelType)
    : false;

  function toggleCollapsed() {
    collapsed = !collapsed;
  }

  async function onSecurityChange(e) {
    await updateSecurityClass($selectedCellIndex, e.target.value);
  }

  async function onAutoSelect() {
    await autoSelectHardware($selectedCellIndex);
  }

  async function onHwChange(field, subfield, value) {
    if (!hw) return;
    const updated = JSON.parse(JSON.stringify(hw));
    if (subfield) {
      if (!updated[field]) return;
      updated[field][subfield] = value;
    } else {
      updated[field] = value;
    }
    updated.autoSelected = false;
    await updateCellHardware($selectedCellIndex, updated);
  }
</script>

{#if visible}
  <div class="hardware-panel">
    <button class="collapse-header" on:click={toggleCollapsed}>
      <span class="collapse-icon" class:open={!collapsed}>&#9656;</span>
      <h3>Hang &amp; Sluitwerk</h3>
      {#if hw?.autoSelected}
        <span class="auto-badge">AUTO</span>
      {/if}
    </button>

    {#if !collapsed}
      {#if selectedCell && isOperable}
        <!-- Security class -->
        <div class="section">
          <h3>Beveiliging</h3>
          <div class="field">
            <label>Weerstandsklasse</label>
            <select
              value={hw?.securityClass || "none"}
              on:change={onSecurityChange}
            >
              {#each SECURITY_CLASSES as sc}
                <option value={sc.value}>{sc.label}</option>
              {/each}
            </select>
          </div>
          <button class="auto-btn" on:click={onAutoSelect}>
            Automatisch selecteren
          </button>
        </div>

        {#if hw}
          <!-- Hinges -->
          {#if hw.hinges}
            <div class="section">
              <h3>Scharnieren</h3>
              <div class="field">
                <label>Type</label>
                <select
                  value={hw.hinges.hingeType}
                  on:change={(e) => onHwChange("hinges", "hingeType", e.target.value)}
                >
                  <option value="opleg">Opleg</option>
                  <option value="inboor">Inboor</option>
                  <option value="verdekt">Verdekt</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>Aantal</label>
                  <input
                    type="number"
                    value={hw.hinges.count}
                    on:change={(e) => onHwChange("hinges", "count", parseInt(e.target.value))}
                    min="2"
                    max="5"
                  />
                </div>
                <div class="field">
                  <label>Zijde</label>
                  <select
                    value={hw.hinges.side}
                    on:change={(e) => onHwChange("hinges", "side", e.target.value)}
                  >
                    <option value="left">Links</option>
                    <option value="right">Rechts</option>
                    <option value="top">Boven</option>
                    <option value="bottom">Onder</option>
                  </select>
                </div>
              </div>
              <div class="field">
                <label>Draagkracht (kg)</label>
                <input
                  type="number"
                  value={Math.round(hw.hinges.loadCapacityKg)}
                  disabled
                />
              </div>
            </div>
          {/if}

          <!-- Handle -->
          {#if hw.handle}
            <div class="section">
              <h3>Greep</h3>
              <div class="field">
                <label>Type</label>
                <select
                  value={hw.handle.handleType}
                  on:change={(e) => onHwChange("handle", "handleType", e.target.value)}
                >
                  <option value="kruk">Kruk</option>
                  <option value="knop">Knop</option>
                  <option value="t_greep">T-greep</option>
                  <option value="inlaat_greep">Inlaatgreep</option>
                  <option value="kruk_kruk">Kruk-kruk</option>
                  <option value="stangen_greep">Stangengreep</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>Zijde</label>
                  <select
                    value={hw.handle.side}
                    on:change={(e) => onHwChange("handle", "side", e.target.value)}
                  >
                    <option value="left">Links</option>
                    <option value="right">Rechts</option>
                    <option value="center">Midden</option>
                  </select>
                </div>
                <div class="field">
                  <label>Hoogte (mm)</label>
                  <input
                    type="number"
                    value={hw.handle.heightMm}
                    on:change={(e) => onHwChange("handle", "heightMm", parseFloat(e.target.value))}
                    min="500"
                    max="1500"
                    step="10"
                  />
                </div>
              </div>
              <div class="field">
                <label>
                  <input
                    type="checkbox"
                    checked={hw.handle.lockable}
                    on:change={(e) => onHwChange("handle", "lockable", e.target.checked)}
                  />
                  Afsluitbaar
                </label>
              </div>
            </div>
          {/if}

          <!-- Locking -->
          {#if hw.locking}
            <div class="section">
              <h3>Sluiting</h3>
              <div class="field">
                <label>Type</label>
                <select
                  value={hw.locking.lockType}
                  on:change={(e) => onHwChange("locking", "lockType", e.target.value)}
                >
                  <option value="espagnolet">Espagnolet</option>
                  <option value="multi_point">Meerpuntssluiting</option>
                  <option value="cylinder_lock">Cilinderslot</option>
                  <option value="sliding_lock">Schuifslot</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>Sluitpunten</label>
                  <input
                    type="number"
                    value={hw.locking.lockingPoints}
                    on:change={(e) => onHwChange("locking", "lockingPoints", parseInt(e.target.value))}
                    min="1"
                    max="12"
                  />
                </div>
                <div class="field">
                  <label>Noktype</label>
                  <select
                    value={hw.locking.camType}
                    on:change={(e) => onHwChange("locking", "camType", e.target.value)}
                  >
                    <option value="rol_nok">Rolnok</option>
                    <option value="paddenstoel_nok">Paddenstoelnok</option>
                    <option value="haak_sluit_nok">Haaksluitnok</option>
                  </select>
                </div>
              </div>
              {#if hw.locking.cylinder && hw.locking.cylinder !== "none"}
                <div class="field">
                  <label>Cilinder</label>
                  <select
                    value={hw.locking.cylinder}
                    on:change={(e) => onHwChange("locking", "cylinder", e.target.value)}
                  >
                    <option value="euro_profile">Euro profiel</option>
                    <option value="skg1">SKG*</option>
                    <option value="skg2">SKG**</option>
                    <option value="skg3">SKG***</option>
                  </select>
                </div>
              {/if}
            </div>
          {/if}
        {:else}
          <div class="hint">
            <p>Klik "Automatisch selecteren" om beslag te genereren</p>
          </div>
        {/if}
      {:else if selectedCell && !isOperable}
        <div class="hint">
          <p>Vast glas en panelen hebben geen hang &amp; sluitwerk</p>
        </div>
      {:else}
        <div class="hint">
          <p>Selecteer een cel om hang &amp; sluitwerk te configureren</p>
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .hardware-panel {
    margin-bottom: var(--sp-4);
  }

  .collapse-header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--sp-2) 0;
    margin-bottom: var(--sp-2);
    border-bottom: var(--border-default);
  }

  .collapse-header h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--amber);
    margin: 0;
  }

  .collapse-icon {
    display: inline-block;
    font-size: 10px;
    color: var(--text-muted);
    transition: transform 0.15s ease;
  }

  .collapse-icon.open {
    transform: rotate(90deg);
  }

  .auto-badge {
    font-size: 9px;
    font-weight: 700;
    background: var(--amber);
    color: var(--bg-surface);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    margin-left: auto;
  }

  .section {
    margin-bottom: var(--sp-4);
  }

  .section h3 {
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

  .field input[type="number"],
  .field input[type="text"],
  .field select {
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 13px;
  }

  .field input:focus,
  .field select:focus {
    outline: none;
    border-color: var(--amber);
    box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.2);
  }

  .field input[disabled] {
    opacity: 0.6;
  }

  .field input[type="checkbox"] {
    width: auto;
    margin-right: var(--sp-2);
    accent-color: var(--amber);
  }

  .field-row {
    display: flex;
    gap: var(--sp-2);
  }

  .field-row .field {
    flex: 1;
  }

  .auto-btn {
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    background: var(--amber);
    color: var(--bg-surface);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    margin-top: var(--sp-2);
  }

  .auto-btn:hover {
    filter: brightness(1.1);
  }

  .hint {
    color: var(--text-muted);
    font-size: 13px;
    font-style: italic;
    padding: var(--sp-2) 0;
  }
</style>
