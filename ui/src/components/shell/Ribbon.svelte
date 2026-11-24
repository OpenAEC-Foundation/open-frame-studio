<script>
  import { activeRibbonTab, activeWorkspaceView, zoom, setTheme, theme, THEMES } from "../../stores/ui.js";
  import { undo, redo, canUndo, canRedo } from "../../stores/history.js";
  import {
    createKozijn,
    createFromTemplate,
    currentKozijn,
    selectedCellIndex,
    addColumn,
    addRow,
    updateCellType,
    duplicateKozijn,
    removeKozijn,
  } from "../../stores/kozijn.js";
  import { createVgFromTemplate, createVliesgevel } from "../../stores/vliesgevel.js";
  import { onMount } from "svelte";
  import { registerShortcuts } from "../../lib/shortcuts.js";
  import { fileNew, fileOpen, fileSave, fileSaveAs } from "../../lib/project-actions.js";
  import {
    exportDxf, exportKozijnstaat, exportWorkshop,
    exportGltf, exportProduction, sendToBlender,
    importDxfProfile, importCatalog,
    exportLabels, exportIfcWithLod, exportIfcProjectWithLod, exportIfcLibrary, exportIfcPartsProject, exportManufacturerHandoff, restoreOwnIfcProject,
    importIfcFile, compareIfcRoundtrip, compareIfcFiles,
  } from "../../lib/export.js";
  import BlueprintIcon from "./BlueprintIcon.svelte";
  import RibbonLayout from "./RibbonLayout.svelte";
  import RibbonGroup from "./RibbonGroup.svelte";
  import ShapeManager from "../panels/ShapeManager.svelte";
  import { _ } from "svelte-i18n";
  import { get } from "svelte/store";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";
  import { project } from "../../stores/project.js";

  let showShapeManager = $state(false);
  let ifcLod = $state("300");

  const BUILTIN_SJABLONEN = [
    { id: "standaard-67-meranti", name: "Standaard 67mm Meranti", series: "67" },
    { id: "standaard-67-accoya", name: "Standaard 67mm Accoya", series: "67" },
    { id: "zwaar-78-meranti", name: "Zwaar 78mm Meranti", series: "78" },
    { id: "passief-90-meranti", name: "Passief 90mm Meranti", series: "90" },
    // Kunststof (PVC) — verstek-hoeken (gelast)
    { id: "kunststof-82-veka", name: "Kunststof 82mm VEKA Softline", series: "82" },
    { id: "kunststof-88-kommerling", name: "Kunststof 88mm Kömmerling", series: "88" },
    // Aluminium — verstek-hoeken
    { id: "aluminium-77-reynaers", name: "Reynaers CS 77 Functional", series: "77" },
    { id: "aluminium-75-schuco", name: "Aluminium 75mm Schüco AWS 75.SI+", series: "75" },
  ];
  let customSjablonen = $state([]);
  let allSjablonen = $derived([...BUILTIN_SJABLONEN, ...customSjablonen]);
  let activeSjabloonId = $state("standaard-67-meranti");
  let isCustomSjabloon = $derived(activeSjabloonId.startsWith("custom-"));

  // Load custom sjablonen from backend
  onMount(async () => {
    try {
      const all = await invoke("get_sjablonen", {});
      customSjablonen = all.filter(s => !BUILTIN_SJABLONEN.find(b => b.id === s.id));
    } catch (e) { /* ignore if command not available */ }
  });

  async function saveAsTemplate() {
    const k = get(currentKozijn);
    if (!k) return;
    const name = prompt("Sjabloonnaam:", `Sjabloon ${k.mark || "custom"}`);
    if (!name) return;
    const sjabloon = {
      id: `custom-${Date.now()}`,
      name,
      material: k.frame?.material || "wood",
      profileSeries: `${k.frame?.frameWidth || 67}`,
      frameWidth: k.frame?.frameWidth || 67,
      frameDepth: k.frame?.frameDepth || 114,
      sashWidth: k.cells?.[0]?.sashWidth || 67,
      sashDepth: k.cells?.[0]?.sashDepth || 80,
      defaultGlazing: { glassType: "HR++", thicknessMm: 24, ugValue: 1.0, spacerType: "warm_edge" },
      colorInside: k.frame?.colorInside || "RAL9010",
      colorOutside: k.frame?.colorOutside || "RAL9010",
    };
    try {
      await invoke("save_custom_sjabloon", { sjabloonJson: JSON.stringify(sjabloon) });
      customSjablonen = [...customSjablonen, sjabloon];
      toast.success(`Sjabloon "${name}" opgeslagen`);
    } catch (e) {
      toast.error(`Fout bij opslaan: ${e}`);
    }
  }

  async function deleteSjabloon() {
    if (!isCustomSjabloon) return;
    const sj = customSjablonen.find(s => s.id === activeSjabloonId);
    if (!confirm(get(_)('confirm.deleteSjabloon', { values: { name: sj?.name || activeSjabloonId } }))) return;
    try {
      await invoke("delete_custom_sjabloon", { sjabloonId: activeSjabloonId });
      customSjablonen = customSjablonen.filter(s => s.id !== activeSjabloonId);
      activeSjabloonId = BUILTIN_SJABLONEN[0].id;
      toast.success(get(_)('alert.sjabloonDeleted'));
    } catch (e) {
      toast.error(`${e}`);
    }
  }

  async function testBlenderConnection() {
    try {
      const ok = await invoke("check_blender_connection", {});
      if (ok === true) toast.success(get(_)('alert.blenderConnected'));
      else toast.warning(get(_)('alert.blenderNotConnected'));
    } catch (e) {
      toast.error(get(_)('alert.blenderError', { values: { error: e } }));
    }
  }

  const templateSizes = {
    single_turn_tilt: [900, 1400],
    double_turn_tilt: [1800, 1500],
    sliding_door: [3000, 2400],
    front_door: [1000, 2400],
    klapraam: [900, 600],
    hefschuif: [4000, 2400],
    pivot: [900, 1400],
    stolp: [1800, 1500],
  };

  async function handleNewKozijn() {
    await createKozijn(get(_)('ribbon.newKozijn'));
  }

  async function handleTemplate(template) {
    const [w, h] = templateSizes[template] || [1200, 1500];
    await createFromTemplate(template, w, h, activeSjabloonId);
  }

  function quickSetPanel(panelType) {
    if ($selectedCellIndex === null) return;
    const dir = panelType === "turn_tilt" ? "left" : panelType === "door" ? "inward" : null;
    updateCellType($selectedCellIndex, panelType, dir);
  }

  function handleAddColumn() {
    if (!$currentKozijn) return;
    addColumn(($currentKozijn.frame.outerWidth - 2 * $currentKozijn.frame.frameWidth) / 2);
  }

  function handleAddRow() {
    if (!$currentKozijn) return;
    addRow(($currentKozijn.frame.outerHeight - 2 * $currentKozijn.frame.frameWidth) / 2);
  }

  async function handleDuplicate() {
    if (!$currentKozijn) return;
    await duplicateKozijn("K" + String(Date.now()).slice(-3));
  }

  async function handleRemove() {
    if (!$currentKozijn) return;
    if (confirm(get(_)('confirm.deleteKozijn', { values: { mark: $currentKozijn.mark } }))) {
      await removeKozijn($currentKozijn.id);
    }
  }

  onMount(() => {
    registerShortcuts({
      onDuplicate: handleDuplicate,
      onNew: fileNew,
      onOpen: fileOpen,
      onSave: fileSave,
      onSaveAs: fileSaveAs,
    });
  });
</script>

<RibbonLayout>
  {#if $activeRibbonTab === "home"}
    <RibbonGroup id="template" icon="file.save">
      <div class="template-tools">
        <select class="sjabloon-select" aria-label={$_('ribbon.template')} bind:value={activeSjabloonId}>
          {#each allSjablonen as sj}
            <option value={sj.id}>{sj.name}</option>
          {/each}
        </select>
        <div class="template-actions">
          <button class="ribbon-btn-sm" onclick={saveAsTemplate} disabled={!$currentKozijn}>
            <BlueprintIcon name="file.save" /> {$_('ribbon.saveTemplate')}
          </button>
          <button class="ribbon-btn-sm" onclick={deleteSjabloon} disabled={!isCustomSjabloon} title={$_('ribbon.deleteSjabloon')} aria-label={$_('ribbon.deleteSjabloon')}>
            <BlueprintIcon name="edit.delete" />
          </button>
        </div>
      </div>
    </RibbonGroup>

    <RibbonGroup id="newKozijn" icon="frame.new">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={handleNewKozijn}>
          <BlueprintIcon name="frame.new" size={24} />
          <span>{$_('ribbon.empty')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("single_turn_tilt")}>
          <BlueprintIcon name="frame.new-turn-tilt" size={24} />
          <span>{$_('ribbon.turnTilt')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("double_turn_tilt")}>
          <BlueprintIcon name="frame.new-double" size={24} />
          <span>{$_('ribbon.double')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("sliding_door")}>
          <BlueprintIcon name="frame.new-sliding" size={24} />
          <span>{$_('ribbon.slidingDoor')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("front_door")}>
          <BlueprintIcon name="frame.new-door" size={24} />
          <span>{$_('ribbon.frontDoor')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("klapraam")}>
          <BlueprintIcon name="frame.new-top-hung" size={24} />
          <span>Klapraam</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("hefschuif")}>
          <BlueprintIcon name="frame.new-lift-slide" size={24} />
          <span>Hefschuif</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("pivot")}>
          <BlueprintIcon name="frame.new-pivot" size={24} />
          <span>Pivot</span>
        </button>
        <button class="ribbon-btn" onclick={() => handleTemplate("stolp")}>
          <BlueprintIcon name="frame.new-french" size={24} />
          <span>Stolp</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="edit" icon="frame.column-add">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={handleAddColumn} disabled={!$currentKozijn}>
          <BlueprintIcon name="frame.column-add" size={24} />
          <span>{$_('ribbon.addColumn')}</span>
        </button>
        <button class="ribbon-btn" onclick={handleAddRow} disabled={!$currentKozijn}>
          <BlueprintIcon name="frame.row-add" size={24} />
          <span>{$_('ribbon.addRow')}</span>
        </button>
        <button class="ribbon-btn" onclick={handleDuplicate} disabled={!$currentKozijn}>
          <BlueprintIcon name="edit.duplicate" size={24} />
          <span>{$_('ribbon.duplicate')}</span>
        </button>
        <button class="ribbon-btn" onclick={handleRemove} disabled={!$currentKozijn}>
          <BlueprintIcon name="edit.delete" size={24} />
          <span>{$_('ribbon.remove')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="import" icon="profile.new">
      <div class="group-buttons">
        <button class="ribbon-btn primary" onclick={() => showShapeManager = true}>
          <BlueprintIcon name="profile.new" size={24} />
          <span>{$_('ribbon.newProfile')}</span>
        </button>
        <button class="ribbon-btn" onclick={importDxfProfile}>
          <BlueprintIcon name="file.import" size={24} />
          <span>{$_('ribbon.profileDxf')}</span>
        </button>
        <button class="ribbon-btn" onclick={importCatalog}>
          <BlueprintIcon name="file.import" size={24} />
          <span>{$_('ribbon.catalog')}</span>
        </button>
      </div>
    </RibbonGroup>

  {:else if $activeRibbonTab === "insert"}
    <RibbonGroup id="subdivision" icon="frame.column-add">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={handleAddColumn} disabled={!$currentKozijn}>
          <BlueprintIcon name="frame.column-add" size={24} />
          <span>{$_('ribbon.column')}</span>
        </button>
        <button class="ribbon-btn" onclick={handleAddRow} disabled={!$currentKozijn}>
          <BlueprintIcon name="frame.row-add" size={24} />
          <span>{$_('ribbon.row')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="panelType" icon="frame.cell-fixed">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => quickSetPanel("fixed_glass")} disabled={$selectedCellIndex === null}>
          <BlueprintIcon name="frame.cell-fixed" size={24} />
          <span>{$_('ribbon.fixedGlass')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => quickSetPanel("turn_tilt")} disabled={$selectedCellIndex === null}>
          <BlueprintIcon name="frame.cell-turn-tilt" size={24} />
          <span>{$_('ribbon.turnTilt')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => quickSetPanel("sliding")} disabled={$selectedCellIndex === null}>
          <BlueprintIcon name="frame.cell-sliding" size={24} />
          <span>{$_('panel.sliding')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => quickSetPanel("door")} disabled={$selectedCellIndex === null}>
          <BlueprintIcon name="frame.cell-door" size={24} />
          <span>{$_('panel.door')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => quickSetPanel("panel")} disabled={$selectedCellIndex === null}>
          <BlueprintIcon name="frame.cell-panel" size={24} />
          <span>{$_('ribbon.panel')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="curtainWall" icon="curtain.new">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => createVliesgevel()}>
          <BlueprintIcon name="curtain.new" size={24} />
          <span>{$_('ribbon.empty')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => createVgFromTemplate("stick_system", 9000, 3600)}>
          <BlueprintIcon name="curtain.stick" size={24} />
          <span>{$_('ribbon.stickSystem')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => createVgFromTemplate("unitized", 6000, 3600)}>
          <BlueprintIcon name="curtain.unitized" size={24} />
          <span>{$_('ribbon.unitized')}</span>
        </button>
      </div>
    </RibbonGroup>

  {:else if $activeRibbonTab === "ifc"}
    <RibbonGroup id="exportKozijn" icon="ifc.export">
      <div class="group-buttons">
        <button class="ribbon-btn primary" onclick={() => exportIfcWithLod(ifcLod)} disabled={!$currentKozijn}>
          <BlueprintIcon name="ifc.export" size={24} />
          <span>IFC</span>
        </button>
        <button class="ribbon-btn" onclick={() => exportIfcProjectWithLod(ifcLod)} disabled={!$project?.kozijnen?.length}>
          <BlueprintIcon name="ifc.export" size={24} />
          <span>Project IFC</span>
        </button>
        <button class="ribbon-btn" onclick={exportIfcLibrary} disabled={!$project?.kozijnen?.length} title="Herbruikbare IFC4 kozijn typen">
          <BlueprintIcon name="ifc.export" size={24} />
          <span>Typebieb IFC</span>
        </button>
        <button class="ribbon-btn" onclick={exportIfcPartsProject} disabled={!$project?.kozijnen?.length} title="Kozijnsamenstellingen met losse profielen en ruiten; nog geen productie-IFC">
          <BlueprintIcon name="ifc.export" size={24} />
          <span>Onderdelen IFC</span>
        </button>
        <button class="ribbon-btn" onclick={exportManufacturerHandoff} disabled={!$project?.kozijnen?.length} title="IFC, JSON en CSV met gekoppelde onderdeel-ID's en indicatieve lijsten">
          <BlueprintIcon name="file.export" size={24} />
          <span>Producentenoverdracht</span>
        </button>
        <button class="ribbon-btn" onclick={exportDxf} disabled={!$currentKozijn}>
          <BlueprintIcon name="file.export" size={24} />
          <span>DXF</span>
        </button>
      </div>
      <select class="sjabloon-select" aria-label="IFC LOD" bind:value={ifcLod}>
        <option value="200">LOD 200</option>
        <option value="300">LOD 300</option>
        <option value="400" disabled>LOD 400 (in ontwikkeling)</option>
      </select>
    </RibbonGroup>

    <RibbonGroup id="ifcImport" icon="ifc.import">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={importIfcFile}>
          <BlueprintIcon name="ifc.import" size={24} />
          <span>IFC Inspectie</span>
        </button>
        <button class="ribbon-btn" onclick={restoreOwnIfcProject} disabled={!!$project?.kozijnen?.length} title="Herstel een bewerkbaar basisproject uit een eigen OFS-onderdelen-IFC; begin in een leeg project">
          <BlueprintIcon name="ifc.import" size={24} />
          <span>IFC Basis herstellen</span>
        </button>
        <button class="ribbon-btn" onclick={compareIfcRoundtrip}>
          <BlueprintIcon name="ifc.roundtrip-compare" size={24} />
          <span>IFC Roundtrip</span>
        </button>
        <button class="ribbon-btn" onclick={compareIfcFiles}>
          <BlueprintIcon name="ifc.compare-files" size={24} />
          <span>{$_('ribbon.compareIfc')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="validation" icon="validation.workspace">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("ids")}>
          <BlueprintIcon name="validation.workspace" size={24} />
          <span>{$_('tabs.ids')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("certification")}>
          <BlueprintIcon name="frame.certification" size={24} />
          <span>{$_('tabs.certification')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="exportProject" icon="file.export">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => exportKozijnstaat("pdf")}>
          <BlueprintIcon name="file.export" size={24} />
          <span>{$_('ribbon.kozijnstaatPdf')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => exportKozijnstaat("xlsx")}>
          <BlueprintIcon name="file.export" size={24} />
          <span>{$_('ribbon.kozijnstaatExcel')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="workshop" icon="file.export">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={exportWorkshop} disabled={!$currentKozijn}>
          <BlueprintIcon name="file.export" size={24} />
          <span>{$_('ribbon.drawingPdf')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="export3d" icon="file.export">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={exportGltf} disabled={!$currentKozijn}>
          <BlueprintIcon name="file.export" size={24} />
          <span>glTF/GLB</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="blender" icon="integration.send-model">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={sendToBlender} disabled={!$currentKozijn}>
          <BlueprintIcon name="integration.send-model" size={24} />
          <span>{$_('ribbon.toBlender')}</span>
        </button>
        <button class="ribbon-btn" onclick={testBlenderConnection}>
          <BlueprintIcon name="integration.test-connection" size={24} />
          <span>{$_('ribbon.testBlender')}</span>
        </button>
      </div>
    </RibbonGroup>

  {:else if $activeRibbonTab === "production"}
    <RibbonGroup id="prodExport" icon="file.export">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => exportProduction("pdf")}>
          <BlueprintIcon name="file.export" size={24} />
          <span>PDF</span>
        </button>
        <button class="ribbon-btn" onclick={() => exportProduction("xlsx")}>
          <BlueprintIcon name="file.export" size={24} />
          <span>Excel</span>
        </button>
        <button class="ribbon-btn" onclick={() => exportProduction("csv")}>
          <BlueprintIcon name="file.export" size={24} />
          <span>CSV</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="cncLabels" icon="frame.cnc">
      <div class="group-buttons">
        <button class="ribbon-btn" disabled title="G-code is niet vrijgegeven zonder gevalideerde producent- en machineconfiguratie">
          <BlueprintIcon name="frame.cnc" size={24} />
          <span>G-code niet vrijgegeven</span>
        </button>
        <button class="ribbon-btn" onclick={exportLabels}>
          <BlueprintIcon name="file.export" size={24} />
          <span>Labels PDF</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="productionPlanning" icon="planning.cut-list">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("optimization")}>
          <BlueprintIcon name="planning.cut-list" size={24} />
          <span>Zaagplan</span>
        </button>
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("planning")}>
          <BlueprintIcon name="planning.production" size={24} />
          <span>Planning</span>
        </button>
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("cnc")}>
          <BlueprintIcon name="frame.cnc" size={24} />
          <span>{$_('tabs.cnc')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("procurement")}>
          <BlueprintIcon name="frame.procurement" size={24} />
          <span>{$_('tabs.procurement')}</span>
        </button>
      </div>
    </RibbonGroup>

  {:else if $activeRibbonTab === "view"}
    <RibbonGroup id="undoGroup" icon="edit.undo">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={undo} disabled={!$canUndo}>
          <BlueprintIcon name="edit.undo" size={24} />
          <span>{$_('ribbon.undo')}</span>
        </button>
        <button class="ribbon-btn" onclick={redo} disabled={!$canRedo}>
          <BlueprintIcon name="edit.redo" size={24} />
          <span>{$_('ribbon.redo')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="zoom" icon="view.zoom-in">
      <div class="group-buttons">
        <button class="ribbon-btn" onclick={() => zoom.update(z => Math.min(2.0, z + 0.1))}>
          <BlueprintIcon name="view.zoom-in" size={24} />
          <span>{$_('ribbon.zoomIn')}</span>
        </button>
        <button class="ribbon-btn" onclick={() => zoom.update(z => Math.max(0.05, z - 0.1))}>
          <BlueprintIcon name="view.zoom-out" size={24} />
          <span>{$_('ribbon.zoomOut')}</span>
        </button>
      </div>
    </RibbonGroup>

    <RibbonGroup id="theme" icon="ui.settings">
      <select aria-label={$_('ribbon.theme')} class="sjabloon-select" value={$theme} onchange={(e) => setTheme(e.target.value)}>
        {#each THEMES as t}
          <option value={t}>{$_('settings.theme.' + t)}</option>
        {/each}
      </select>
    </RibbonGroup>

  {/if}
</RibbonLayout>

<ShapeManager bind:visible={showShapeManager} onsaved={() => { showShapeManager = false; }} />

<style>
  .group-buttons { display: flex; align-items: stretch; }
  .ribbon-btn {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    gap: 3px;
    min-width: 56px;
    max-width: 112px;
    min-height: 56px;
    padding: 4px 5px;
    border-radius: 6px;
    color: var(--text-primary);
    font: 400 11px/13px var(--font-body);
    overflow-wrap: anywhere;
  }
  .ribbon-btn :global(svg) { flex: none; }
  .ribbon-btn:hover:not(:disabled), .ribbon-btn-sm:hover:not(:disabled) { background: var(--oa-hover, var(--bg-surface-alt)); }
  .ribbon-btn:active:not(:disabled), .ribbon-btn-sm:active:not(:disabled) { background: var(--oa-pressed, var(--bg-app)); }
  .ribbon-btn:focus-visible, .ribbon-btn-sm:focus-visible { outline-offset: -2px; }
  button:disabled { color: var(--oa-text-disabled, var(--text-muted)); }
  button:disabled :global(svg) { --oa-accent-text: currentColor; opacity: .55; }
  .template-tools { display: grid; gap: 2px; width: 174px; }
  .template-actions { display: flex; align-items: center; justify-content: space-between; }
  .ribbon-btn-sm { display: inline-flex; align-items: center; justify-content: center; gap: 4px; min-height: 22px; padding: 0 4px; border-radius: 3px; font-size: 11px; font-weight: 400; }
  .sjabloon-select { min-width: 0; max-width: 176px; height: 26px; padding: 0 4px; border: 1px solid var(--border-color); border-radius: 6px; color: var(--text-primary); background: var(--bg-surface); font-size: 12px; }
</style>
