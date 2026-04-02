<script>
  import {
    currentKozijn,
    selectedCellIndex,
    updateDimensions,
    updateCellType,
    updateFrameProfile,
    updateSillProfile,
    updateFrameShape,
  } from "../../stores/kozijn.js";
  import HardwarePanel from "./HardwarePanel.svelte";
  import ProfileSelector from "./ProfileSelector.svelte";

  let editWidth = 0;
  let editHeight = 0;

  $: if ($currentKozijn) {
    editWidth = $currentKozijn.frame.outerWidth;
    editHeight = $currentKozijn.frame.outerHeight;
  }

  $: selectedCell =
    $currentKozijn && $selectedCellIndex !== null
      ? $currentKozijn.cells[$selectedCellIndex]
      : null;

  function handleDimensionChange() {
    if (editWidth > 0 && editHeight > 0) {
      updateDimensions(editWidth, editHeight);
    }
  }

  const panelTypes = [
    { value: "fixed_glass", label: "Vast glas" },
    { value: "turn_tilt", label: "Draaikiepraam" },
    { value: "turn", label: "Draairaam" },
    { value: "tilt", label: "Kiepraam" },
    { value: "sliding", label: "Schuifraam" },
    { value: "door", label: "Deur" },
    { value: "panel", label: "Paneel" },
    { value: "ventilation", label: "Ventilatie" },
  ];

  function handlePanelTypeChange(e) {
    if ($selectedCellIndex === null) return;
    updateCellType($selectedCellIndex, e.target.value, null);
  }
</script>

<div class="panel">
  {#if $currentKozijn}
    <div class="section">
      <h3>Kozijn</h3>
      <div class="field">
        <label>Naam</label>
        <input type="text" value={$currentKozijn.name} disabled />
      </div>
      <div class="field">
        <label>Merk</label>
        <input type="text" value={$currentKozijn.mark} disabled />
      </div>
      <div class="field-row">
        <div class="field">
          <label>Breedte (mm)</label>
          <input
            type="number"
            bind:value={editWidth}
            on:change={handleDimensionChange}
            step="10"
            min="200"
            max="6000"
          />
        </div>
        <div class="field">
          <label>Hoogte (mm)</label>
          <input
            type="number"
            bind:value={editHeight}
            on:change={handleDimensionChange}
            step="10"
            min="200"
            max="4000"
          />
        </div>
      </div>
      <div class="field">
        <label>Materiaal</label>
        <div class="value">{$currentKozijn.frame.material?.wood || "Hout"}</div>
      </div>
    </div>

    <div class="section">
      <h3>Profielen</h3>
      <ProfileSelector
        label="Kozijnprofiel"
        filter="frame"
        value={$currentKozijn.frame.profile}
        on:change={(e) => updateFrameProfile(e.detail.id, e.detail.name)}
      />
      <ProfileSelector
        label="Dorpelprofiel"
        filter="sill"
        value={$currentKozijn.frame.sillProfile}
        on:change={(e) => updateSillProfile(e.detail.id, e.detail.name)}
      />
    </div>

    <div class="section">
      <h3>Vorm</h3>
      <div class="field">
        <label>Kozijnvorm</label>
        <select
          value={$currentKozijn.frame.shape?.shapeType || "rectangular"}
          on:change={(e) => updateFrameShape(e.target.value, e.target.value === "arched" ? $currentKozijn.frame.outerWidth / 6 : null)}
        >
          <option value="rectangular">Rechthoekig</option>
          <option value="arched">Getoogd (segmentboog)</option>
          <option value="round">Rond</option>
        </select>
      </div>
      {#if $currentKozijn.frame.shape?.shapeType === "arched"}
        <div class="field">
          <label>Booghoogte (mm)</label>
          <input
            type="number"
            value={$currentKozijn.frame.shape.archHeight || Math.round($currentKozijn.frame.outerWidth / 6)}
            on:change={(e) => updateFrameShape("arched", parseFloat(e.target.value))}
            min="50"
            max={Math.round($currentKozijn.frame.outerHeight / 2)}
            step="10"
          />
        </div>
      {/if}
    </div>

    {#if selectedCell}
      <div class="section">
        <h3>Cel {$selectedCellIndex + 1}</h3>
        <div class="field">
          <label>Paneel type</label>
          <select value={selectedCell.panelType} on:change={handlePanelTypeChange}>
            {#each panelTypes as pt}
              <option value={pt.value}>{pt.label}</option>
            {/each}
          </select>
        </div>
        <div class="field">
          <label>Beglazing</label>
          <div class="value">{selectedCell.glazing.glassType} ({selectedCell.glazing.thicknessMm}mm)</div>
        </div>
        <div class="field">
          <label>Ug-waarde</label>
          <div class="value">{selectedCell.glazing.ugValue} W/m²K</div>
        </div>
      </div>
      <HardwarePanel visible={true} />
    {:else}
      <div class="section hint">
        <p>Klik op een cel in het kozijn om de eigenschappen te bewerken</p>
      </div>
    {/if}

    <div class="section">
      <h3>Grid</h3>
      <div class="field-row">
        <div class="field">
          <label>Kolommen</label>
          <div class="value">{$currentKozijn.grid.columns.length}</div>
        </div>
        <div class="field">
          <label>Rijen</label>
          <div class="value">{$currentKozijn.grid.rows.length}</div>
        </div>
      </div>
    </div>
  {:else}
    <div class="empty">
      <p>Selecteer of maak een kozijn</p>
    </div>
  {/if}
</div>

<style>
  .panel {
    width: 280px;
    flex-shrink: 0;
    background: var(--bg-surface);
    border-left: var(--border-default);
    overflow-y: auto;
    padding: var(--sp-4);
  }

  .section {
    margin-bottom: var(--sp-6);
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

  .field input, .field select {
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 13px;
  }

  .field input:focus, .field select:focus {
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

  .hint {
    color: var(--text-muted);
    font-size: 13px;
    font-style: italic;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-muted);
    text-align: center;
  }
</style>
