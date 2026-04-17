<script>
  import { _ } from "svelte-i18n";
  import { invoke } from "../../lib/tauri.js";
  import {
    currentKozijn,
    selectedCellIndex,
    selectedMember,
    updateDimensions,
    updateCellType,
    updateFrameProfile,
    updateSillProfile,
    updateDividerProfile,
    updateFrameShape,
    updateGridSizes,
    updateFrameColors,
    calculateThermal,
    updateCellSashProfile,
  } from "../../stores/kozijn.js";
  import { allProfiles } from "../../stores/profiles.js";
  import { PANEL_TYPE_KEYS, panelLabel, memberLabel } from "../../lib/labels.js";
  import HardwarePanel from "./HardwarePanel.svelte";
  import JointPanel from "./JointPanel.svelte";
  import GlazingPanel from "./GlazingPanel.svelte";
  import ProfileSelector from "./ProfileSelector.svelte";
  import ProfileCrossSection from "./ProfileCrossSection.svelte";
  import { RAL_COLORS, ralToHex } from "../../lib/ral-colors.js";

  let editWidth = 0;
  let editHeight = 0;
  let thermalResult = null;

  $: if ($currentKozijn) {
    editWidth = $currentKozijn.frame.outerWidth;
    editHeight = $currentKozijn.frame.outerHeight;
    calculateThermal().then(r => thermalResult = r);
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

  $: panelTypes = Object.keys(PANEL_TYPE_KEYS).map(value => ({
    value,
    label: panelLabel($_, value),
  }));

  function handlePanelTypeChange(e) {
    if ($selectedCellIndex === null) return;
    updateCellType($selectedCellIndex, e.target.value, null);
  }

  function getMemberLabel(member) {
    if (!member) return "";
    const base = memberLabel($_, member.type);
    if (member.type === "divider_v" || member.type === "divider_h") {
      return `${base} ${member.index + 1}`;
    }
    return base;
  }

  function getMemberProfile(member) {
    if (!member || !$currentKozijn) return null;
    const frame = $currentKozijn.frame;
    if (member.type === "frame_top") return frame.topProfile || frame.profile;
    if (member.type === "frame_bottom") return frame.bottomProfile || frame.sillProfile || frame.profile;
    if (member.type === "frame_left") return frame.leftProfile || frame.profile;
    if (member.type === "frame_right") return frame.rightProfile || frame.profile;
    if (member.type === "divider_v") {
      const col = $currentKozijn.grid.columns[member.index];
      return col?.dividerProfile || frame.profile;
    }
    if (member.type === "divider_h") {
      const row = $currentKozijn.grid.rows[member.index];
      return row?.dividerProfile || frame.profile;
    }
    return frame.profile;
  }

  function getMemberProfileFilter(member) {
    if (!member) return "frame";
    if (member.type === "frame_bottom") return "sill";
    if (member.type === "divider_v" || member.type === "divider_h") return "divider";
    return "frame";
  }

  function getMemberProfileDefinition(member) {
    const ref = getMemberProfile(member);
    if (!ref) return null;
    return ($allProfiles || []).find(p => p.id === ref.id) || null;
  }

  function handleMemberProfileChange(detail) {
    const member = $selectedMember;
    if (!member) return;
    if (member.type.startsWith("frame_")) {
      // For now, frame members share the main frame profile
      updateFrameProfile(detail.id, detail.name, detail.width, detail.depth);
    } else if (member.type === "divider_v") {
      updateDividerProfile(member.index, true, detail.id, detail.name);
    } else if (member.type === "divider_h") {
      updateDividerProfile(member.index, false, detail.id, detail.name);
    }
  }
</script>

<div class="panel">
  {#if $currentKozijn}
    <div class="section">
      <h3>{$_('props.kozijn')}</h3>
      <div class="field">
        <label>{$_('props.name')}</label>
        <input type="text" value={$currentKozijn.name} disabled />
      </div>
      <div class="field">
        <label>{$_('props.mark')}</label>
        <input type="text" value={$currentKozijn.mark} disabled />
      </div>
      <div class="field-row">
        <div class="field">
          <label>{$_('props.width')}</label>
          <input
            type="number"
            bind:value={editWidth}
            onchange={handleDimensionChange}
            step="10"
            min="200"
            max="6000"
          />
        </div>
        <div class="field">
          <label>{$_('props.height')}</label>
          <input
            type="number"
            bind:value={editHeight}
            onchange={handleDimensionChange}
            step="10"
            min="200"
            max="4000"
          />
        </div>
      </div>
      <div class="field">
        <label>{$_('props.material')}</label>
        <div class="value">{$currentKozijn.frame.material?.wood || $_('props.wood')}</div>
      </div>
    </div>

    <div class="section">
      <h3>{$_('props.colors')}</h3>
      <div class="field">
        <label>{$_('props.inside')}</label>
        <div class="color-row">
          <span class="color-swatch" style="background: {ralToHex($currentKozijn.frame.colorInside)}"></span>
          <select
            value={$currentKozijn.frame.colorInside}
            onchange={(e) => updateFrameColors(e.target.value, $currentKozijn.frame.colorOutside)}
          >
            {#each RAL_COLORS as ral}
              <option value={ral.code}>{ral.code} — {ral.name}</option>
            {/each}
          </select>
        </div>
      </div>
      <div class="field">
        <label>{$_('props.outside')}</label>
        <div class="color-row">
          <span class="color-swatch" style="background: {ralToHex($currentKozijn.frame.colorOutside)}"></span>
          <select
            value={$currentKozijn.frame.colorOutside}
            onchange={(e) => updateFrameColors($currentKozijn.frame.colorInside, e.target.value)}
          >
            {#each RAL_COLORS as ral}
              <option value={ral.code}>{ral.code} — {ral.name}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>{$_('props.profiles')}</h3>
      <ProfileSelector
        label={$_('props.frameProfile')}
        filter="frame"
        value={$currentKozijn.frame.profile}
        onchange={(detail) => updateFrameProfile(detail.id, detail.name, detail.width, detail.depth)}
      />
      <ProfileSelector
        label={$_('props.sillProfile')}
        filter="sill"
        value={$currentKozijn.frame.sillProfile}
        onchange={(detail) => updateSillProfile(detail.id, detail.name)}
      />
    </div>

    <div class="section">
      <h3>{$_('props.shape')}</h3>
      <div class="field">
        <label>{$_('props.frameShape')}</label>
        <select
          value={$currentKozijn.frame.shape?.shapeType || "rectangular"}
          onchange={(e) => updateFrameShape(e.target.value, e.target.value === "arched" ? $currentKozijn.frame.outerWidth / 6 : null)}
        >
          <option value="rectangular">{$_('props.rectangular')}</option>
          <option value="arched">{$_('props.arched')}</option>
          <option value="round">{$_('props.round')}</option>
          <option value="trapezoid">{$_('props.trapezoid') || "Trapezium (schuine stijlen)"}</option>
          <option value="arched_trapezoid">{$_('props.archedTrapezoid') || "Boog + trapezium"}</option>
          <option value="elliptical">{$_('props.elliptical') || "Elliptisch"}</option>
          <option value="triangle">{$_('props.triangle') || "Driehoek"}</option>
          <option value="polygon">{$_('props.polygon') || "Polygonaal"}</option>
        </select>
      </div>
      {#if $currentKozijn.frame.shape?.shapeType === "arched" || $currentKozijn.frame.shape?.shapeType === "arched_trapezoid"}
        <div class="field">
          <label>{$_('props.archHeight')}</label>
          <input
            type="number"
            value={$currentKozijn.frame.shape.archHeight || Math.round($currentKozijn.frame.outerWidth / 6)}
            onchange={(e) => updateFrameShape("arched", parseFloat(e.target.value))}
            min="50"
            max={Math.round($currentKozijn.frame.outerHeight / 2)}
            step="10"
          />
        </div>
      {/if}
      {#if $currentKozijn.frame.shape?.shapeType === "trapezoid" || $currentKozijn.frame.shape?.shapeType === "arched_trapezoid"}
        {@const shape = $currentKozijn.frame.shape}
        {@const shapeType = shape.shapeType}
        <div class="field-row">
          <div class="field">
            <label>{$_('props.leftAngle') || "Hoek links (°)"}</label>
            <input
              type="number"
              value={shape.leftAngle || 75}
              onchange={(e) => {
                const angle = parseFloat(e.target.value);
                updateFrameShape(shapeType, shape.archHeight, shape.topWidth, angle, shape.rightAngle || 75);
              }}
              min="45" max="90" step="1"
            />
          </div>
          <div class="field">
            <label>{$_('props.rightAngle') || "Hoek rechts (°)"}</label>
            <input
              type="number"
              value={shape.rightAngle || 75}
              onchange={(e) => {
                const angle = parseFloat(e.target.value);
                updateFrameShape(shapeType, shape.archHeight, shape.topWidth, shape.leftAngle || 75, angle);
              }}
              min="45" max="90" step="1"
            />
          </div>
        </div>
        <div class="field">
          <label>{$_('props.topWidth') || "Breedte boven (mm)"}</label>
          <input
            type="number"
            value={shape.topWidth || Math.round($currentKozijn.frame.outerWidth * 0.6)}
            onchange={(e) => updateFrameShape(shapeType, shape.archHeight, parseFloat(e.target.value), shape.leftAngle, shape.rightAngle)}
            min="200"
            max={$currentKozijn.frame.outerWidth}
            step="10"
          />
        </div>
      {/if}
    </div>

    <div class="section">
      <h3>{$_('props.gridSizes')}</h3>
      <div class="field">
        <label>{$_('props.columns')}</label>
        {#each $currentKozijn.grid.columns as col, i}
          <div class="field-row" style="margin-bottom: 4px;">
            <span class="col-label">{i + 1}</span>
            <input
              type="number"
              value={Math.round(col.size)}
              onchange={(e) => {
                const sizes = $currentKozijn.grid.columns.map(c => c.size);
                sizes[i] = parseFloat(e.target.value) || sizes[i];
                updateGridSizes(sizes, $currentKozijn.grid.rows.map(r => r.size));
              }}
              min="100"
              step="10"
            />
          </div>
        {/each}
      </div>
      <div class="field">
        <label>{$_('props.rows')}</label>
        {#each $currentKozijn.grid.rows as row, i}
          <div class="field-row" style="margin-bottom: 4px;">
            <span class="col-label">{i + 1}</span>
            <input
              type="number"
              value={Math.round(row.size)}
              onchange={(e) => {
                const sizes = $currentKozijn.grid.rows.map(r => r.size);
                sizes[i] = parseFloat(e.target.value) || sizes[i];
                updateGridSizes($currentKozijn.grid.columns.map(c => c.size), sizes);
              }}
              min="100"
              step="10"
            />
          </div>
        {/each}
      </div>
    </div>

    {#if $selectedMember}
      <div class="section">
        <h3>{$_('member.title')}</h3>
        <div class="field">
          <label>{$_('member.type')}</label>
          <div class="value">{getMemberLabel($selectedMember)}</div>
        </div>
        <ProfileSelector
          label={$_('member.profile')}
          filter={getMemberProfileFilter($selectedMember)}
          value={getMemberProfile($selectedMember)}
          onchange={(detail) => handleMemberProfileChange(detail)}
        />
        {#if getMemberProfile($selectedMember)}
          {@const profileDef = getMemberProfileDefinition($selectedMember)}
          <div class="field-row">
            <div class="field">
              <label>{$_('props.width')}</label>
              <div class="value">{getMemberProfile($selectedMember)?.width || profileDef?.width || "—"}</div>
            </div>
            <div class="field">
              <label>{$_('member.depth')}</label>
              <div class="value">{getMemberProfile($selectedMember)?.depth || profileDef?.depth || "—"}</div>
            </div>
          </div>
          {#if profileDef?.sponning}
            <div class="field">
              <label>{$_('member.rebate')}</label>
              <div class="value">{profileDef.sponning.width}x{profileDef.sponning.depth}mm ({profileDef.sponning.position})</div>
            </div>
          {/if}
          {#if profileDef?.ufValue}
            <div class="field">
              <label>{$_('member.ufValue')}</label>
              <div class="value">{profileDef.ufValue} W/m²K</div>
            </div>
          {/if}
          {#if profileDef?.crossSection?.length > 2}
            <div class="field">
              <label>{$_('member.crossSection')}</label>
              <ProfileCrossSection
                crossSection={profileDef.crossSection}
                sponning={profileDef.sponning}
              />
            </div>
          {/if}
        {/if}
      </div>
    {:else if selectedCell}
      <div class="section">
        <h3>{$_('props.cell', { values: { index: $selectedCellIndex + 1 } })}</h3>
        <div class="field">
          <label>{$_('props.panelType')}</label>
          <select value={selectedCell.panelType} onchange={handlePanelTypeChange}>
            {#each panelTypes as pt}
              <option value={pt.value}>{pt.label}</option>
            {/each}
          </select>
        </div>

        {#if ["turn_tilt", "turn", "tilt"].includes(selectedCell.panelType)}
          <div class="field">
            <label>{$_('props.openingDirection')}</label>
            <select value={selectedCell.openingDirection || "left"}
              onchange={(e) => updateCellType($selectedCellIndex, selectedCell.panelType, e.target.value)}>
              <option value="left">{$_('props.left')}</option>
              <option value="right">{$_('props.right')}</option>
            </select>
          </div>
          <ProfileSelector
            label={$_('props.sashProfile')}
            filter="sash"
            value={selectedCell.sashProfile}
            onchange={(detail) => updateCellSashProfile($selectedCellIndex, detail.id, detail.name, detail.width, detail.depth)}
          />
          {#if selectedCell.sashProfile}
            <div class="field-row">
              <div class="field">
                <label>{$_('props.sashWidth')}</label>
                <div class="value">{selectedCell.sashWidth || 54}mm</div>
              </div>
              <div class="field">
                <label>{$_('props.sashDepth')}</label>
                <div class="value">{selectedCell.sashDepth || 67}mm</div>
              </div>
            </div>
          {/if}
        {/if}

        {#if selectedCell.panelType === "door"}
          <div class="field">
            <label>{$_('props.openingDirection')}</label>
            <select value={selectedCell.openingDirection || "inward"}
              onchange={(e) => updateCellType($selectedCellIndex, "door", e.target.value)}>
              <option value="inward">{$_('props.inward')}</option>
              <option value="outward">{$_('props.outward')}</option>
              <option value="left">{$_('props.left')}</option>
              <option value="right">{$_('props.right')}</option>
            </select>
          </div>
          <ProfileSelector
            label={$_('props.doorProfile')}
            filter="sash"
            value={selectedCell.sashProfile}
            onchange={(detail) => updateCellSashProfile($selectedCellIndex, detail.id, detail.name, detail.width, detail.depth)}
          />
        {/if}
      </div>
      <GlazingPanel />
      <HardwarePanel visible={true} />
      <JointPanel visible={true} />
    {:else}
      <div class="section hint">
        <p>{$_('props.hint')}</p>
      </div>
    {/if}

    {#if thermalResult}
      <div class="section">
        <h3>{$_('props.thermal')}</h3>
        <div class="field">
          <label>{$_('thermal.uwValue')}</label>
          <div class="value thermal-badge" class:thermal-a={thermalResult.rating === "A"} class:thermal-b={thermalResult.rating === "B"} class:thermal-c={thermalResult.rating === "C"} class:thermal-d={thermalResult.rating === "D"}>
            {thermalResult.uwValue} W/m²K
            <span class="rating">{thermalResult.rating}</span>
          </div>
        </div>
        <div class="field-row">
          <div class="field">
            <label>{$_('thermal.ufFrame')}</label>
            <div class="value">{thermalResult.ufValue}</div>
          </div>
          <div class="field">
            <label>{$_('thermal.ugGlass')}</label>
            <div class="value">{thermalResult.ugValue}</div>
          </div>
          <div class="field">
            <label>Ψg</label>
            <div class="value">{thermalResult.psiValue}</div>
          </div>
        </div>
        <div class="field-row">
          <div class="field">
            <label>{$_('thermal.glassPercent')}</label>
            <div class="value">{thermalResult.areaTotalM2 > 0 ? Math.round(thermalResult.areaGlassM2 / thermalResult.areaTotalM2 * 100) : 0}%</div>
          </div>
          <div class="field">
            <label>{$_('thermal.area')}</label>
            <div class="value">{thermalResult.areaTotalM2}</div>
          </div>
        </div>
      </div>
    {/if}

    <div class="section">
      <h3>{$_('props.grid')}</h3>
      <div class="field-row">
        <div class="field">
          <label>{$_('props.columnsCount')}</label>
          <div class="value">{$currentKozijn.grid.columns.length}</div>
        </div>
        <div class="field">
          <label>{$_('props.rowsCount')}</label>
          <div class="value">{$currentKozijn.grid.rows.length}</div>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>Randen</h3>
      {#each ["Links", "Rechts", "Boven", "Onder"] as eName, ei}
        {@const edge = ($currentKozijn.frame.edges || [])[ei]}
        <div class="field">
          <label>{eName}</label>
          <div class="edge-row">
            <select
              value={edge?.randsponningType || "haaks"}
              onchange={(e) => {
                invoke("update_edge_config", {
                  id: $currentKozijn.id,
                  edgeIndex: ei,
                  edgeJson: JSON.stringify({
                    ...(edge || {}),
                    randsponningType: e.target.value,
                    randsponningDepth: edge?.randsponningDepth || 5,
                    randsponningWidth: edge?.randsponningWidth || 46,
                    sealType: edge?.sealType || "compriband",
                    folieBinnen: edge?.folieBinnen || "geen",
                    folieBuiten: edge?.folieBuiten || "geen",
                    anchorSpacingMm: edge?.anchorSpacingMm || 500,
                  })
                }).then(k => currentKozijn.set(k));
              }}
            >
              <option value="haaks">Haaks</option>
              <option value="kalksponning">Kalksponning</option>
              <option value="renovatie">Renovatie</option>
              <option value="vlak">Vlak</option>
            </select>
            <select
              value={edge?.spouwlat?.width || 100}
              onchange={(e) => {
                const w = parseFloat(e.target.value);
                invoke("update_edge_config", {
                  id: $currentKozijn.id,
                  edgeIndex: ei,
                  edgeJson: JSON.stringify({
                    ...(edge || {}),
                    randsponningType: edge?.randsponningType || "haaks",
                    randsponningDepth: edge?.randsponningDepth || 5,
                    randsponningWidth: edge?.randsponningWidth || 46,
                    spouwlat: { width: w, height: 27, material: "vuren", glued: true, nailSpacingMm: 300 },
                    sealType: edge?.sealType || "compriband",
                    folieBinnen: edge?.folieBinnen || "geen",
                    folieBuiten: edge?.folieBuiten || "geen",
                    anchorSpacingMm: edge?.anchorSpacingMm || 500,
                  })
                }).then(k => currentKozijn.set(k));
              }}
            >
              <option value="100">SL 100</option>
              <option value="120">SL 120</option>
              <option value="140">SL 140</option>
              <option value="160">SL 160</option>
            </select>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty">
      <p>{$_('props.empty')}</p>
    </div>
  {/if}
</div>

<style>
  .panel {
    width: 100%;
    height: 100%;
    background: var(--bg-surface);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .section {
    padding: 10px 12px 6px;
    border-bottom: 1px solid var(--border-color, rgba(0,0,0,0.06));
  }

  .section h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    margin: 0 0 8px 0;
    padding: 0;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border: none;
  }

  .field {
    margin-bottom: 6px;
  }

  .field label {
    display: block;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 2px;
  }

  .field input, .field select {
    width: 100%;
    padding: 4px 6px;
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color, rgba(0,0,0,0.12));
    border-radius: 2px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-body);
    height: 24px;
  }

  .field input:focus, .field select:focus {
    outline: none;
    border-color: var(--amber);
  }

  .field input:disabled {
    opacity: 0.6;
    background: transparent;
    border-color: transparent;
  }

  .field-row {
    display: flex;
    gap: 6px;
    align-items: flex-start;
  }

  .field-row .field {
    flex: 1;
  }

  .col-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--text-muted);
    min-width: 14px;
    line-height: 24px;
  }

  .value {
    font-size: 12px;
    color: var(--text-primary);
    padding: 3px 0;
    line-height: 1.4;
  }

  .hint {
    color: var(--text-muted);
    font-size: 12px;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-muted);
    text-align: center;
    font-size: 12px;
  }

  .color-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .color-row select {
    flex: 1;
    padding: 4px 6px;
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color, rgba(0,0,0,0.12));
    border-radius: 2px;
    color: var(--text-primary);
    font-size: 11px;
    height: 24px;
  }

  .color-row select:focus {
    outline: none;
    border-color: var(--amber);
  }

  .color-swatch {
    display: inline-block;
    width: 18px;
    height: 18px;
    border-radius: 2px;
    border: 1px solid var(--border-color, rgba(0,0,0,0.15));
    flex-shrink: 0;
  }

  .thermal-badge {
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }

  .rating {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    font-size: 10px;
    font-weight: 700;
    color: white;
  }

  .thermal-a .rating { background: var(--success); }
  .thermal-b .rating { background: #84CC16; }
  .thermal-c .rating { background: var(--warning); }
  .thermal-d .rating { background: var(--error); }

  .edge-row {
    display: flex;
    gap: var(--sp-1);
  }

  .edge-row select {
    flex: 1;
    padding: 3px 4px;
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 11px;
  }

  .edge-row select:focus {
    outline: none;
    border-color: var(--amber);
  }
</style>
