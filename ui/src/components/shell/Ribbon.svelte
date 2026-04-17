<script>
  import { activeRibbonTab, activeWorkspaceView, zoom, setTheme, theme, THEMES, showAppMenu } from "../../stores/ui.js";
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
  import { createVgFromTemplate } from "../../stores/vliesgevel.js";
  import { onMount } from "svelte";
  import { registerShortcuts } from "../../lib/shortcuts.js";
  import { fileNew, fileOpen, fileSave, fileSaveAs } from "../../lib/project-actions.js";
  import {
    exportIfc, exportDxf, exportKozijnstaat, exportWorkshop,
    exportGltf, exportProduction, sendToBlender,
    importDxfProfile, importCatalog,
    exportCncGcode, exportLabels, exportIfcWithLod,
    importIfcFile, compareIfcRoundtrip,
  } from "../../lib/export.js";
  import ShapeManager from "../panels/ShapeManager.svelte";
  import { _ } from "svelte-i18n";
  import { get } from "svelte/store";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let showShapeManager = false;
  let ifcLod = 200;

  const BUILTIN_SJABLONEN = [
    { id: "standaard-67-meranti", name: "Standaard 67mm Meranti", series: "67" },
    { id: "standaard-67-accoya", name: "Standaard 67mm Accoya", series: "67" },
    { id: "zwaar-78-meranti", name: "Zwaar 78mm Meranti", series: "78" },
    { id: "passief-90-meranti", name: "Passief 90mm Meranti", series: "90" },
  ];
  let customSjablonen = $state([]);
  let allSjablonen = $derived([...BUILTIN_SJABLONEN, ...customSjablonen]);
  let activeSjabloonId = "standaard-67-meranti";

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

  const tabs = [
    { id: "home", key: "ribbon.home" },
    { id: "insert", key: "ribbon.insert" },
    { id: "ifc", key: "ribbon.ifc" },
    { id: "production", key: "ribbon.production" },
    { id: "view", key: "ribbon.view" },
  ];

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
    await createKozijn(get(_)('ribbon.newKozijn'), "K01", 1200, 1500);
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

<div class="ribbon">
  <div class="ribbon-tabs">
    <button class="tab file-tab" onclick={() => showAppMenu.set(true)}>
      {$_('titlebar.file')}
    </button>
    {#each tabs as tab}
      <button
        class="tab"
        class:active={$activeRibbonTab === tab.id}
        onclick={() => activeRibbonTab.set(tab.id)}
      >
        {$_(tab.key)}
      </button>
    {/each}
  </div>

  <div class="ribbon-content">
    {#if $activeRibbonTab === "home"}
      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.template')}</span>
        <select class="sjabloon-select" bind:value={activeSjabloonId}>
          {#each allSjablonen as sj}
            <option value={sj.id}>{sj.name}</option>
          {/each}
        </select>
        <button class="ribbon-btn-sm" onclick={saveAsTemplate} title="Huidig kozijn opslaan als sjabloon">
          💾 Opslaan als sjabloon
        </button>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.newKozijn')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={handleNewKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="8" x2="12" y2="16"/>
              <line x1="8" y1="12" x2="16" y2="12"/>
            </svg>
            <span>{$_('ribbon.empty')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("single_turn_tilt")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <path d="M5 5 L19 12 L5 19" stroke-dasharray="2 2"/>
            </svg>
            <span>{$_('ribbon.turnTilt')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("double_turn_tilt")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
            </svg>
            <span>{$_('ribbon.double')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("sliding_door")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
              <path d="M14 10 L18 12 L14 14"/>
            </svg>
            <span>{$_('ribbon.slidingDoor')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("front_door")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="8" x2="21" y2="8"/>
              <circle cx="16" cy="14" r="1.5"/>
            </svg>
            <span>{$_('ribbon.frontDoor')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("klapraam")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="6" width="18" height="12" rx="1"/>
              <path d="M5 18L12 10L19 18" stroke-dasharray="2 2"/>
            </svg>
            <span>Klapraam</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("hefschuif")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="18" rx="1"/>
              <line x1="11" y1="3" x2="11" y2="21"/>
              <path d="M13 11L17 12L13 13"/>
              <path d="M7 11L3 12"/>
            </svg>
            <span>Hefschuif</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("pivot")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="5" y="3" width="14" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="2 2"/>
              <path d="M8 8L12 12L16 8"/>
            </svg>
            <span>Pivot</span>
          </button>
          <button class="ribbon-btn" onclick={() => handleTemplate("stolp")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
              <path d="M5 5L12 12L5 19" stroke-dasharray="2 2"/>
              <path d="M19 5L12 12L19 19" stroke-dasharray="2 2"/>
            </svg>
            <span>Stolp</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.edit')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={handleAddColumn} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="3 2"/>
            </svg>
            <span>{$_('ribbon.addColumn')}</span>
          </button>
          <button class="ribbon-btn" onclick={handleAddRow} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="12" x2="21" y2="12" stroke-dasharray="3 2"/>
            </svg>
            <span>{$_('ribbon.addRow')}</span>
          </button>
          <button class="ribbon-btn" onclick={handleDuplicate} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="6" y="6" width="14" height="14" rx="1"/>
              <rect x="4" y="4" width="14" height="14" rx="1"/>
            </svg>
            <span>{$_('ribbon.duplicate')}</span>
          </button>
          <button class="ribbon-btn" onclick={handleRemove} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/>
              <line x1="10" y1="11" x2="10" y2="17"/>
              <line x1="14" y1="11" x2="14" y2="17"/>
            </svg>
            <span>{$_('ribbon.remove')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.import')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn primary" onclick={() => showShapeManager = true}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="4" y="2" width="10" height="16" rx="1"/>
              <path d="M7 6h4m-4 3h4m-4 3h2"/>
              <circle cx="17" cy="17" r="5"/>
              <path d="M17 14v6m-3-3h6"/>
            </svg>
            <span>{$_('ribbon.newProfile')}</span>
          </button>
          <button class="ribbon-btn" onclick={importDxfProfile}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3v12m0 0l4-4m-4 4l-4-4"/>
              <rect x="4" y="18" width="16" height="3" rx="1"/>
            </svg>
            <span>{$_('ribbon.profileDxf')}</span>
          </button>
          <button class="ribbon-btn" onclick={importCatalog}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4 4h16v16H4z"/>
              <line x1="4" y1="9" x2="20" y2="9"/>
              <line x1="4" y1="14" x2="20" y2="14"/>
              <line x1="10" y1="4" x2="10" y2="20"/>
            </svg>
            <span>{$_('ribbon.catalog')}</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "insert"}
      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.subdivision')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={handleAddColumn} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="3 2"/>
            </svg>
            <span>{$_('ribbon.column')}</span>
          </button>
          <button class="ribbon-btn" onclick={handleAddRow} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="12" x2="21" y2="12" stroke-dasharray="3 2"/>
            </svg>
            <span>{$_('ribbon.row')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.panelType')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => quickSetPanel("fixed_glass")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#3B82F6" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><line x1="4" y1="4" x2="20" y2="20"/><line x1="20" y1="4" x2="4" y2="20"/></svg>
            <span>{$_('ribbon.fixedGlass')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => quickSetPanel("turn_tilt")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#3B82F6" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><path d="M6 6L18 12L6 18" stroke-dasharray="2 2"/></svg>
            <span>{$_('ribbon.turnTilt')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => quickSetPanel("sliding")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#34D399" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><path d="M13 10L17 12L13 14"/></svg>
            <span>{$_('panel.sliding')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => quickSetPanel("door")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#F97316" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><circle cx="15" cy="12" r="1.5"/></svg>
            <span>{$_('panel.door')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => quickSetPanel("panel")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#A8A29E" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1" fill="#E7E5E4"/></svg>
            <span>{$_('ribbon.panel')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.curtainWall')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => createVgFromTemplate("stick_system", 9000, 3600)}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="20" rx="0"/>
              <line x1="7" y1="2" x2="7" y2="22"/>
              <line x1="12" y1="2" x2="12" y2="22"/>
              <line x1="17" y1="2" x2="17" y2="22"/>
              <line x1="2" y1="11" x2="22" y2="11"/>
            </svg>
            <span>{$_('ribbon.stickSystem')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => createVgFromTemplate("unitized", 6000, 3600)}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="20" rx="0"/>
              <line x1="7" y1="2" x2="7" y2="22"/>
              <line x1="12" y1="2" x2="12" y2="22"/>
              <line x1="17" y1="2" x2="17" y2="22"/>
            </svg>
            <span>{$_('ribbon.unitized')}</span>
          </button>
          <!-- Winkelpui verwijderd — niet relevant voor kozijnsoftware -->
        </div>
      </div>

    {:else if $activeRibbonTab === "ifc"}
      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.exportKozijn')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn primary" onclick={() => exportIfcWithLod(ifcLod)} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3v12m0 0l-4-4m4 4l4-4"/>
              <path d="M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2"/>
            </svg>
            <span>IFC</span>
          </button>
          <button class="ribbon-btn" onclick={exportDxf} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3v12m0 0l-4-4m4 4l4-4"/>
              <path d="M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2"/>
            </svg>
            <span>DXF</span>
          </button>
        </div>
        <select class="sjabloon-select" style="min-width:100px" bind:value={ifcLod}>
          <option value={100}>LOD 100</option>
          <option value={200}>LOD 200</option>
          <option value={300}>LOD 300</option>
          <option value={350}>LOD 350</option>
          <option value={400}>LOD 400</option>
        </select>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">IFC Import</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={importIfcFile}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 15V3m0 0l-4 4m4-4l4 4"/>
              <path d="M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2"/>
            </svg>
            <span>IFC Import</span>
          </button>
          <button class="ribbon-btn" onclick={compareIfcRoundtrip}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17 1l4 4-4 4"/>
              <path d="M3 11V9a4 4 0 014-4h14"/>
              <path d="M7 23l-4-4 4-4"/>
              <path d="M21 13v2a4 4 0 01-4 4H3"/>
            </svg>
            <span>IFC Roundtrip</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.exportProject')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => exportKozijnstaat("pdf")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <line x1="8" y1="13" x2="16" y2="13"/>
              <line x1="8" y1="17" x2="13" y2="17"/>
            </svg>
            <span>{$_('ribbon.kozijnstaatPdf')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => exportKozijnstaat("xlsx")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <rect x="7" y="12" width="10" height="7" rx="1"/>
              <line x1="12" y1="12" x2="12" y2="19"/>
              <line x1="7" y1="15.5" x2="17" y2="15.5"/>
            </svg>
            <span>{$_('ribbon.kozijnstaatExcel')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.workshop')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={exportWorkshop} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <rect x="7" y="13" width="10" height="6" rx="1"/>
              <line x1="9" y1="16" x2="15" y2="16"/>
            </svg>
            <span>{$_('ribbon.drawingPdf')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.export3d')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={exportGltf} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z"/>
              <path d="M2 17l10 5 10-5"/>
              <path d="M2 12l10 5 10-5"/>
            </svg>
            <span>glTF/GLB</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.blender')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={sendToBlender} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="9"/>
              <path d="M8 12l3 3 5-5"/>
            </svg>
            <span>{$_('ribbon.toBlender')}</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "production"}
      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.prodExport')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => exportProduction("pdf")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            <span>PDF</span>
          </button>
          <button class="ribbon-btn" onclick={() => exportProduction("xlsx")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="9" x2="21" y2="9"/>
              <line x1="3" y1="15" x2="21" y2="15"/>
              <line x1="9" y1="3" x2="9" y2="21"/>
              <line x1="15" y1="3" x2="15" y2="21"/>
            </svg>
            <span>Excel</span>
          </button>
          <button class="ribbon-btn" onclick={() => exportProduction("csv")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="8" y1="13" x2="16" y2="13"/>
              <line x1="8" y1="17" x2="16" y2="17"/>
            </svg>
            <span>CSV</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">CNC / Labels</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={exportCncGcode} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="6" width="20" height="12" rx="2"/>
              <path d="M6 10h4v4H6z"/>
              <line x1="14" y1="10" x2="18" y2="10"/>
              <line x1="14" y1="14" x2="18" y2="14"/>
            </svg>
            <span>CNC G-code</span>
          </button>
          <button class="ribbon-btn" onclick={exportLabels}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="4" y="4" width="16" height="6" rx="1"/>
              <rect x="4" y="14" width="16" height="6" rx="1"/>
              <line x1="8" y1="7" x2="16" y2="7"/>
              <line x1="8" y1="17" x2="16" y2="17"/>
            </svg>
            <span>Labels PDF</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Planning / Optimalisatie</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("optimization")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="6" width="20" height="3" rx="1"/>
              <rect x="2" y="11" width="14" height="3" rx="1"/>
              <rect x="2" y="16" width="18" height="3" rx="1"/>
            </svg>
            <span>Zaagplan</span>
          </button>
          <button class="ribbon-btn" onclick={() => activeWorkspaceView.set("planning")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="16" rx="1"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
              <line x1="9" y1="4" x2="9" y2="20"/>
            </svg>
            <span>Planning</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "view"}
      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.undoGroup')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={undo} disabled={!$canUndo}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 10h10a5 5 0 015 5v2"/>
              <path d="M3 10l4-4m-4 4l4 4"/>
            </svg>
            <span>{$_('ribbon.undo')}</span>
          </button>
          <button class="ribbon-btn" onclick={redo} disabled={!$canRedo}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 10H11a5 5 0 00-5 5v2"/>
              <path d="M21 10l-4-4m4 4l-4 4"/>
            </svg>
            <span>{$_('ribbon.redo')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.zoom')}</span>
        <div class="group-buttons">
          <button class="ribbon-btn" onclick={() => zoom.update(z => Math.min(2.0, z + 0.1))}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>
              <line x1="8" y1="11" x2="14" y2="11"/><line x1="11" y1="8" x2="11" y2="14"/>
            </svg>
            <span>{$_('ribbon.zoomIn')}</span>
          </button>
          <button class="ribbon-btn" onclick={() => zoom.update(z => Math.max(0.05, z - 0.1))}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>
              <line x1="8" y1="11" x2="14" y2="11"/>
            </svg>
            <span>{$_('ribbon.zoomOut')}</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">{$_('ribbon.theme')}</span>
        <select class="sjabloon-select" value={$theme} onchange={(e) => setTheme(e.target.value)}>
          {#each THEMES as t}
            <option value={t}>{$_('settings.theme.' + t)}</option>
          {/each}
        </select>
      </div>

    {/if}
  </div>
</div>

<ShapeManager bind:visible={showShapeManager} onsaved={() => { showShapeManager = false; }} />

<style>
  .ribbon {
    background: var(--bg-ribbon);
    border-bottom: var(--border-featured);
  }

  .ribbon-tabs {
    display: flex;
    gap: 0;
    padding: 0 var(--sp-2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .tab {
    padding: var(--sp-2) var(--sp-4);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--scaffold-gray);
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: var(--text-on-dark);
  }

  .file-tab {
    background: var(--amber);
    color: var(--night-build) !important;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    font-weight: 700;
    margin-right: var(--sp-1);
  }

  .file-tab:hover {
    background: var(--warm-gold);
    color: var(--night-build) !important;
  }

  .tab.active {
    color: var(--amber);
    border-bottom-color: var(--amber);
  }

  .ribbon-content {
    display: flex;
    align-items: flex-start;
    padding: var(--sp-2) var(--sp-3);
    min-height: 72px;
    gap: var(--sp-3);
  }

  .ribbon-group {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .group-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--scaffold-gray);
    padding: 0 var(--sp-1);
  }

  .group-buttons {
    display: flex;
    gap: var(--sp-1);
  }

  .ribbon-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--sp-2) var(--sp-3);
    color: var(--text-on-dark);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 500;
    min-width: 56px;
    transition: background 0.15s;
  }

  .ribbon-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
  }

  .ribbon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .ribbon-btn.primary {
    color: var(--amber);
  }

  .ribbon-btn.primary:hover:not(:disabled) {
    background: rgba(217, 119, 6, 0.15);
  }

  .ribbon-divider {
    width: 1px;
    height: 56px;
    background: rgba(255, 255, 255, 0.1);
    align-self: center;
  }

  .lang-select {
    padding: var(--sp-2) var(--sp-3);
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: var(--radius-sm);
    color: var(--text-on-dark);
    font-size: 12px;
    cursor: default;
    margin-top: var(--sp-1);
  }
  .lang-select:focus {
    outline: none;
    border-color: var(--amber);
  }
  .lang-select option {
    background: var(--bg-ribbon);
    color: var(--text-on-dark);
  }

  .sjabloon-select {
    padding: var(--sp-2) var(--sp-3);
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: var(--radius-sm);
    color: var(--text-on-dark);
    font-size: 12px;
    font-weight: 600;
    min-width: 180px;
    cursor: default;
  }
  .sjabloon-select:focus {
    outline: none;
    border-color: var(--amber);
  }
  .sjabloon-select option {
    background: var(--bg-ribbon);
    color: var(--text-on-dark);
  }
</style>
