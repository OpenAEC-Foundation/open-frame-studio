<script>
  import { _ } from "svelte-i18n";
  import {
    currentKozijn,
    selectedCellIndex,
    updateCellHardware,
    autoSelectHardware,
    updateSecurityClass,
  } from "../../stores/kozijn.js";

  let { visible = true } = $props();

  let collapsed = false;

  let SECURITY_CLASSES = $derived([
    { value: "none", label: $_('hardware.secNone') },
    { value: "RC1", label: $_('hardware.secRC1') },
    { value: "RC2", label: $_('hardware.secRC2') },
    { value: "RC3", label: $_('hardware.secRC3') },
    { value: "RC4", label: $_('hardware.secRC4') },
    { value: "RC5", label: $_('hardware.secRC5') },
    { value: "RC6", label: $_('hardware.secRC6') },
  ]);

  const OPERABLE_TYPES = [
    "turn_tilt", "turn", "tilt", "sliding", "door",
  ];

  let selectedCell = $derived(
    $currentKozijn && $selectedCellIndex !== null
      ? $currentKozijn.cells[$selectedCellIndex]
      : null
  );

  let hw = $derived(selectedCell?.hardwareSet || null);

  let isOperable = $derived(selectedCell
    ? OPERABLE_TYPES.includes(selectedCell.panelType)
    : false
  );

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
    <button class="collapse-header" onclick={toggleCollapsed}>
      <span class="collapse-icon" class:open={!collapsed}>&#9656;</span>
      <h3>{$_('hardware.title')}</h3>
      {#if hw?.autoSelected}
        <span class="auto-badge">AUTO</span>
      {/if}
    </button>

    {#if !collapsed}
      {#if selectedCell && isOperable}
        <!-- Security class -->
        <div class="section">
          <h3>{$_('hardware.security')}</h3>
          <div class="field">
            <label>{$_('hardware.resistanceClass')}</label>
            <select
              value={hw?.securityClass || "none"}
              onchange={onSecurityChange}
            >
              {#each SECURITY_CLASSES as sc}
                <option value={sc.value}>{sc.label}</option>
              {/each}
            </select>
          </div>
          <button class="auto-btn" onclick={onAutoSelect}>
            {$_('hardware.autoSelect')}
          </button>
        </div>

        {#if hw}
          <!-- Hinges -->
          {#if hw.hinges}
            <div class="section">
              <h3>{$_('hardware.hinges')}</h3>
              <div class="field">
                <label>{$_('hardware.type')}</label>
                <select
                  value={hw.hinges.hingeType}
                  onchange={(e) => onHwChange("hinges", "hingeType", e.target.value)}
                >
                  <option value="opleg">{$_('hardware.hingeOpleg')}</option>
                  <option value="inboor">{$_('hardware.hingeInboor')}</option>
                  <option value="verdekt">{$_('hardware.hingeVerdekt')}</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>{$_('hardware.count')}</label>
                  <input
                    type="number"
                    value={hw.hinges.count}
                    onchange={(e) => onHwChange("hinges", "count", parseInt(e.target.value))}
                    min="2"
                    max="5"
                  />
                </div>
                <div class="field">
                  <label>{$_('hardware.side')}</label>
                  <select
                    value={hw.hinges.side}
                    onchange={(e) => onHwChange("hinges", "side", e.target.value)}
                  >
                    <option value="left">{$_('hardware.left')}</option>
                    <option value="right">{$_('hardware.right')}</option>
                    <option value="top">{$_('hardware.top')}</option>
                    <option value="bottom">{$_('hardware.bottom')}</option>
                  </select>
                </div>
              </div>
              <div class="field">
                <label>{$_('hardware.loadCapacity')}</label>
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
              <h3>{$_('hardware.handle')}</h3>
              <div class="field">
                <label>{$_('hardware.type')}</label>
                <select
                  value={hw.handle.handleType}
                  onchange={(e) => onHwChange("handle", "handleType", e.target.value)}
                >
                  <option value="kruk">{$_('hardware.handleKruk')}</option>
                  <option value="knop">{$_('hardware.handleKnop')}</option>
                  <option value="t_greep">{$_('hardware.handleTGreep')}</option>
                  <option value="inlaat_greep">{$_('hardware.handleInlaat')}</option>
                  <option value="kruk_kruk">{$_('hardware.handleKrukKruk')}</option>
                  <option value="stangen_greep">{$_('hardware.handleStangen')}</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>{$_('hardware.side')}</label>
                  <select
                    value={hw.handle.side}
                    onchange={(e) => onHwChange("handle", "side", e.target.value)}
                  >
                    <option value="left">{$_('hardware.left')}</option>
                    <option value="right">{$_('hardware.right')}</option>
                    <option value="center">{$_('hardware.center')}</option>
                  </select>
                </div>
                <div class="field">
                  <label>{$_('hardware.height')}</label>
                  <input
                    type="number"
                    value={hw.handle.heightMm}
                    onchange={(e) => onHwChange("handle", "heightMm", parseFloat(e.target.value))}
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
                    onchange={(e) => onHwChange("handle", "lockable", e.target.checked)}
                  />
                  {$_('hardware.lockable')}
                </label>
              </div>
            </div>
          {/if}

          <!-- Locking -->
          {#if hw.locking}
            <div class="section">
              <h3>{$_('hardware.locking')}</h3>
              <div class="field">
                <label>{$_('hardware.type')}</label>
                <select
                  value={hw.locking.lockType}
                  onchange={(e) => onHwChange("locking", "lockType", e.target.value)}
                >
                  <option value="espagnolet">{$_('hardware.lockEspagnolet')}</option>
                  <option value="multi_point">{$_('hardware.lockMultiPoint')}</option>
                  <option value="cylinder_lock">{$_('hardware.lockCylinder')}</option>
                  <option value="sliding_lock">{$_('hardware.lockSliding')}</option>
                </select>
              </div>
              <div class="field-row">
                <div class="field">
                  <label>{$_('hardware.lockingPoints')}</label>
                  <input
                    type="number"
                    value={hw.locking.lockingPoints}
                    onchange={(e) => onHwChange("locking", "lockingPoints", parseInt(e.target.value))}
                    min="1"
                    max="12"
                  />
                </div>
                <div class="field">
                  <label>{$_('hardware.camType')}</label>
                  <select
                    value={hw.locking.camType}
                    onchange={(e) => onHwChange("locking", "camType", e.target.value)}
                  >
                    <option value="rol_nok">{$_('hardware.camRol')}</option>
                    <option value="paddenstoel_nok">{$_('hardware.camPaddenstoel')}</option>
                    <option value="haak_sluit_nok">{$_('hardware.camHaak')}</option>
                  </select>
                </div>
              </div>
              {#if hw.locking.cylinder && hw.locking.cylinder !== "none"}
                <div class="field">
                  <label>{$_('hardware.cylinder')}</label>
                  <select
                    value={hw.locking.cylinder}
                    onchange={(e) => onHwChange("locking", "cylinder", e.target.value)}
                  >
                    <option value="euro_profile">{$_('hardware.cylEuro')}</option>
                    <option value="skg1">{$_('hardware.cylSKG1')}</option>
                    <option value="skg2">{$_('hardware.cylSKG2')}</option>
                    <option value="skg3">{$_('hardware.cylSKG3')}</option>
                  </select>
                </div>
              {/if}
            </div>
          {/if}
        {:else}
          <div class="hint">
            <p>{$_('hardware.hintAutoSelect')}</p>
          </div>
        {/if}
      {:else if selectedCell && !isOperable}
        <div class="hint">
          <p>{$_('hardware.hintFixedNoHardware')}</p>
        </div>
      {:else}
        <div class="hint">
          <p>{$_('hardware.hintSelectCell')}</p>
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
    cursor: default;
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
    cursor: default;
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
