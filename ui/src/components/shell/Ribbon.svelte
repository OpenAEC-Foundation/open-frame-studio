<script>
  import { activeRibbonTab, zoom, toggleTheme, theme } from "../../stores/ui.js";
  import { undo, redo, canUndo, canRedo } from "../../stores/history.js";
  import {
    createKozijn,
    createFromTemplate,
    currentKozijn,
    selectedCellIndex,
    addColumn,
    addRow,
    updateCellType,
  } from "../../stores/kozijn.js";
  import { invoke, isTauri } from "../../lib/tauri.js";

  const tabs = [
    { id: "home", label: "Home" },
    { id: "insert", label: "Invoegen" },
    { id: "ifc", label: "IFC / Export" },
    { id: "production", label: "Productie" },
    { id: "view", label: "Beeld" },
  ];

  // Template sizes including melkmeisje
  const templateSizes = {
    single_turn_tilt: [900, 1400],
    double_turn_tilt: [1800, 1500],
    sliding_door: [3000, 2400],
    front_door: [1000, 2400],
    melkmeisje: [1800, 2400],
    melkmeisje_bovenlicht: [1800, 2600],
  };

  async function handleNewKozijn() {
    await createKozijn("Nieuw kozijn", "K01", 1200, 1500);
  }

  async function handleTemplate(template) {
    const [w, h] = templateSizes[template] || [1200, 1500];
    await createFromTemplate(template, w, h);
  }

  async function handleExportIfc() {
    if (!$currentKozijn) return;
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", `${$currentKozijn.mark}.ifc`) };
    const path = await save({
      filters: [{ name: "IFC", extensions: ["ifc"] }],
      defaultPath: `${$currentKozijn.mark}.ifc`,
    });
    if (path) {
      try {
        await invoke("export_ifc", { id: $currentKozijn.id, outputPath: path });
        alert("IFC export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  async function handleExportDxf() {
    if (!$currentKozijn) return;
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", `${$currentKozijn.mark}.dxf`) };
    const path = await save({
      filters: [{ name: "DXF", extensions: ["dxf"] }],
      defaultPath: `${$currentKozijn.mark}.dxf`,
    });
    if (path) {
      try {
        await invoke("export_dxf", { id: $currentKozijn.id, outputPath: path });
        alert("DXF export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  async function handleExportKozijnstaat(format) {
    const ext = format === "xlsx" ? "xlsx" : "pdf";
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", `kozijnstaat.${ext}`) };
    const path = await save({
      filters: [{ name: format.toUpperCase(), extensions: [ext] }],
      defaultPath: `kozijnstaat.${ext}`,
    });
    if (path) {
      try {
        await invoke("export_kozijnstaat", { outputPath: path, format });
        alert("Kozijnstaat export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  async function handleExportWorkshop() {
    if (!$currentKozijn) return;
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", `${$currentKozijn.mark}_werkplaats.pdf`) };
    const path = await save({
      filters: [{ name: "PDF", extensions: ["pdf"] }],
      defaultPath: `${$currentKozijn.mark}_werkplaats.pdf`,
    });
    if (path) {
      try {
        await invoke("export_workshop_drawing", { id: $currentKozijn.id, outputPath: path });
        alert("Werkplaatstekening export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  async function handleExportGltf() {
    if (!$currentKozijn) return;
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", `${$currentKozijn.mark}.glb`) };
    const path = await save({
      filters: [{ name: "glTF Binary", extensions: ["glb"] }],
      defaultPath: `${$currentKozijn.mark}.glb`,
    });
    if (path) {
      try {
        await invoke("export_gltf", { id: $currentKozijn.id, outputPath: path });
        alert("glTF export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  async function handleSendToBlender() {
    if (!$currentKozijn) return;
    try {
      const result = await invoke("send_to_blender", { id: $currentKozijn.id });
      alert("Verzonden naar Blender: " + result);
    } catch (e) {
      alert("Blender fout: " + e);
    }
  }

  function quickSetPanel(panelType) {
    if ($selectedCellIndex === null) return;
    const dir = panelType === "turn_tilt" ? "left" : panelType === "door" ? "inward" : null;
    updateCellType($selectedCellIndex, panelType, dir);
  }

  function handleAddColumn() {
    if (!$currentKozijn) return;
    const innerW = $currentKozijn.frame.outerWidth - 2 * $currentKozijn.frame.frameWidth;
    addColumn(innerW / 2);
  }

  async function handleExportProduction(format) {
    const extMap = { pdf: "pdf", xlsx: "xlsx", csv: "csv" };
    const ext = extMap[format] || "pdf";
    const defaultName = format === "csv" ? "productiestaten" : `productiestaten.${ext}`;
    const { save } = isTauri
      ? await import("@tauri-apps/plugin-dialog")
      : { save: async () => prompt("Bestandspad:", defaultName) };
    const path = await save({
      filters: [{ name: format.toUpperCase(), extensions: [ext] }],
      defaultPath: defaultName,
    });
    if (path) {
      try {
        await invoke("export_production_lists", { outputPath: path, format });
        alert("Productiestaten export succesvol: " + path);
      } catch (e) {
        alert("Export fout: " + e);
      }
    }
  }

  function handleAddRow() {
    if (!$currentKozijn) return;
    const innerH = $currentKozijn.frame.outerHeight - 2 * $currentKozijn.frame.frameWidth;
    addRow(innerH / 2);
  }
</script>

<div class="ribbon">
  <div class="ribbon-tabs">
    {#each tabs as tab}
      <button
        class="tab"
        class:active={$activeRibbonTab === tab.id}
        on:click={() => activeRibbonTab.set(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="ribbon-content">
    {#if $activeRibbonTab === "home"}
      <div class="ribbon-group">
        <span class="group-label">Nieuw kozijn</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleNewKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="8" x2="12" y2="16"/>
              <line x1="8" y1="12" x2="16" y2="12"/>
            </svg>
            <span>Leeg</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("single_turn_tilt")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <path d="M5 5 L19 12 L5 19" stroke-dasharray="2 2"/>
            </svg>
            <span>Draaikiep</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("double_turn_tilt")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
            </svg>
            <span>Dubbel</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("sliding_door")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21"/>
              <path d="M14 10 L18 12 L14 14"/>
            </svg>
            <span>Schuifpui</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("front_door")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="8" x2="21" y2="8"/>
              <circle cx="16" cy="14" r="1.5"/>
            </svg>
            <span>Voordeur</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("melkmeisje")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="9" y1="3" x2="9" y2="21"/>
              <line x1="15" y1="3" x2="15" y2="21"/>
              <line x1="3" y1="10" x2="9" y2="10"/>
              <line x1="15" y1="10" x2="21" y2="10"/>
              <circle cx="13" cy="14" r="1"/>
            </svg>
            <span>Melkmeisje</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleTemplate("melkmeisje_bovenlicht")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="7" x2="21" y2="7"/>
              <line x1="9" y1="7" x2="9" y2="21"/>
              <line x1="15" y1="7" x2="15" y2="21"/>
              <line x1="3" y1="13" x2="9" y2="13"/>
              <line x1="15" y1="13" x2="21" y2="13"/>
              <circle cx="13" cy="16" r="1"/>
            </svg>
            <span>Melkmeisje+</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Bewerken</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleAddColumn} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="3 2"/>
            </svg>
            <span>+ Kolom</span>
          </button>
          <button class="ribbon-btn" on:click={handleAddRow} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="12" x2="21" y2="12" stroke-dasharray="3 2"/>
            </svg>
            <span>+ Rij</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "insert"}
      <div class="ribbon-group">
        <span class="group-label">Onderverdeling</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleAddColumn} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="3 2"/>
            </svg>
            <span>Kolom</span>
          </button>
          <button class="ribbon-btn" on:click={handleAddRow} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="12" x2="21" y2="12" stroke-dasharray="3 2"/>
            </svg>
            <span>Rij</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Paneel type</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={() => quickSetPanel("fixed_glass")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#3B82F6" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><line x1="4" y1="4" x2="20" y2="20"/><line x1="20" y1="4" x2="4" y2="20"/></svg>
            <span>Vast glas</span>
          </button>
          <button class="ribbon-btn" on:click={() => quickSetPanel("turn_tilt")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#3B82F6" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><path d="M6 6L18 12L6 18" stroke-dasharray="2 2"/></svg>
            <span>Draaikiep</span>
          </button>
          <button class="ribbon-btn" on:click={() => quickSetPanel("sliding")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#34D399" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><path d="M13 10L17 12L13 14"/></svg>
            <span>Schuif</span>
          </button>
          <button class="ribbon-btn" on:click={() => quickSetPanel("door")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#F97316" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1"/><circle cx="15" cy="12" r="1.5"/></svg>
            <span>Deur</span>
          </button>
          <button class="ribbon-btn" on:click={() => quickSetPanel("panel")} disabled={$selectedCellIndex === null}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#A8A29E" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="1" fill="#E7E5E4"/></svg>
            <span>Paneel</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "ifc"}
      <div class="ribbon-group">
        <span class="group-label">Kozijn exporteren</span>
        <div class="group-buttons">
          <button class="ribbon-btn primary" on:click={handleExportIfc} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3v12m0 0l-4-4m4 4l4-4"/>
              <path d="M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2"/>
            </svg>
            <span>IFC</span>
          </button>
          <button class="ribbon-btn" on:click={handleExportDxf} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 3v12m0 0l-4-4m4 4l4-4"/>
              <path d="M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2"/>
            </svg>
            <span>DXF</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Project exporteren</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={() => handleExportKozijnstaat("pdf")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <line x1="8" y1="13" x2="16" y2="13"/>
              <line x1="8" y1="17" x2="13" y2="17"/>
            </svg>
            <span>Kozijnstaat PDF</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleExportKozijnstaat("xlsx")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <rect x="7" y="12" width="10" height="7" rx="1"/>
              <line x1="12" y1="12" x2="12" y2="19"/>
              <line x1="7" y1="15.5" x2="17" y2="15.5"/>
            </svg>
            <span>Kozijnstaat Excel</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Werkplaats</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleExportWorkshop} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M6 2h8l6 6v12a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2z"/>
              <path d="M14 2v6h6"/>
              <rect x="7" y="13" width="10" height="6" rx="1"/>
              <line x1="9" y1="16" x2="15" y2="16"/>
            </svg>
            <span>Tekening PDF</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">3D Export</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleExportGltf} disabled={!$currentKozijn}>
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
        <span class="group-label">Blender / Bonsai</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={handleSendToBlender} disabled={!$currentKozijn}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="9"/>
              <path d="M8 12l3 3 5-5"/>
            </svg>
            <span>Naar Blender</span>
          </button>
        </div>
      </div>

    {:else if $activeRibbonTab === "production"}
      <div class="ribbon-group">
        <span class="group-label">Productiestaten exporteren</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={() => handleExportProduction("pdf")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            <span>PDF</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleExportProduction("xlsx")}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="1"/>
              <line x1="3" y1="9" x2="21" y2="9"/>
              <line x1="3" y1="15" x2="21" y2="15"/>
              <line x1="9" y1="3" x2="9" y2="21"/>
              <line x1="15" y1="3" x2="15" y2="21"/>
            </svg>
            <span>Excel</span>
          </button>
          <button class="ribbon-btn" on:click={() => handleExportProduction("csv")}>
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

    {:else if $activeRibbonTab === "view"}
      <div class="ribbon-group">
        <span class="group-label">Ongedaan maken</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={undo} disabled={!$canUndo}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 10h10a5 5 0 015 5v2"/>
              <path d="M3 10l4-4m-4 4l4 4"/>
            </svg>
            <span>Ongedaan</span>
          </button>
          <button class="ribbon-btn" on:click={redo} disabled={!$canRedo}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 10H11a5 5 0 00-5 5v2"/>
              <path d="M21 10l-4-4m4 4l-4 4"/>
            </svg>
            <span>Opnieuw</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Zoom</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={() => zoom.update(z => Math.min(2.0, z + 0.1))}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>
              <line x1="8" y1="11" x2="14" y2="11"/><line x1="11" y1="8" x2="11" y2="14"/>
            </svg>
            <span>Inzoomen</span>
          </button>
          <button class="ribbon-btn" on:click={() => zoom.update(z => Math.max(0.05, z - 0.1))}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>
              <line x1="8" y1="11" x2="14" y2="11"/>
            </svg>
            <span>Uitzoomen</span>
          </button>
        </div>
      </div>

      <div class="ribbon-divider"></div>

      <div class="ribbon-group">
        <span class="group-label">Thema</span>
        <div class="group-buttons">
          <button class="ribbon-btn" on:click={toggleTheme}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              {#if $theme === "light"}
                <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
              {:else}
                <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
              {/if}
            </svg>
            <span>{$theme === "light" ? "Donker" : "Licht"}</span>
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

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
</style>
