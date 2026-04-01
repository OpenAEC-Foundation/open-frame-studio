<script>
  import { onMount, onDestroy } from "svelte";
  import TitleBar from "./components/shell/TitleBar.svelte";
  import Ribbon from "./components/shell/Ribbon.svelte";
  import StatusBar from "./components/shell/StatusBar.svelte";
  import Backstage from "./components/shell/Backstage.svelte";
  import KozijnEditor from "./components/editor/KozijnEditor.svelte";
  import PropertiesPanel from "./components/panels/PropertiesPanel.svelte";
  import ProjectOverview from "./components/project/ProjectOverview.svelte";
  import KozijnstaatView from "./components/project/KozijnstaatView.svelte";
  import ProductionListsView from "./components/project/ProductionListsView.svelte";
  import CalculationView from "./components/project/CalculationView.svelte";
  import Viewer3D from "./components/viewer3d/Viewer3D.svelte";
  import { loadProject } from "./stores/project.js";
  import { loadProfiles } from "./stores/profiles.js";
  import { registerUndoRedoShortcuts } from "./stores/history.js";
  import { activeRibbonTab } from "./stores/ui.js";

  let cleanupShortcuts;
  let workspaceView = "editor"; // "editor" | "kozijnstaat" | "viewer3d"

  onMount(async () => {
    await loadProject();
    await loadProfiles();
    cleanupShortcuts = registerUndoRedoShortcuts();
  });

  onDestroy(() => {
    if (cleanupShortcuts) cleanupShortcuts();
  });
</script>

<TitleBar />
<Ribbon />
<Backstage />

<div class="workspace-tabs">
  <button
    class="ws-tab"
    class:active={workspaceView === "editor"}
    on:click={() => (workspaceView = "editor")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="1"/>
      <line x1="12" y1="3" x2="12" y2="21"/>
      <line x1="3" y1="12" x2="21" y2="12"/>
    </svg>
    2D Editor
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "viewer3d"}
    on:click={() => (workspaceView = "viewer3d")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2L2 7l10 5 10-5-10-5z"/>
      <path d="M2 17l10 5 10-5"/>
      <path d="M2 12l10 5 10-5"/>
    </svg>
    3D Viewer
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "kozijnstaat"}
    on:click={() => (workspaceView = "kozijnstaat")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="1"/>
      <line x1="3" y1="9" x2="21" y2="9"/>
      <line x1="3" y1="15" x2="21" y2="15"/>
      <line x1="9" y1="3" x2="9" y2="21"/>
    </svg>
    Kozijnstaat
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "production"}
    on:click={() => (workspaceView = "production")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
      <rect x="9" y="3" width="6" height="4" rx="1"/>
      <line x1="9" y1="12" x2="15" y2="12"/>
      <line x1="9" y1="16" x2="15" y2="16"/>
    </svg>
    Productie
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "calculation"}
    on:click={() => (workspaceView = "calculation")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="2" width="16" height="20" rx="1"/>
      <line x1="8" y1="6" x2="16" y2="6"/>
      <line x1="8" y1="10" x2="16" y2="10"/>
      <line x1="8" y1="14" x2="12" y2="14"/>
      <line x1="8" y1="18" x2="12" y2="18"/>
    </svg>
    Calculatie
  </button>
</div>

<div class="workspace">
  {#if workspaceView === "editor"}
    <ProjectOverview />
    <KozijnEditor />
    <PropertiesPanel />
  {:else if workspaceView === "viewer3d"}
    <ProjectOverview />
    <Viewer3D visible={workspaceView === "viewer3d"} />
    <PropertiesPanel />
  {:else if workspaceView === "kozijnstaat"}
    <KozijnstaatView />
  {:else if workspaceView === "production"}
    <ProductionListsView />
  {:else if workspaceView === "calculation"}
    <CalculationView />
  {/if}
</div>

<StatusBar />

<style>
  .workspace-tabs {
    display: flex;
    background: var(--bg-surface-alt);
    border-bottom: var(--border-default);
    padding: 0 var(--sp-3);
    gap: 0;
  }

  .ws-tab {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-4);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .ws-tab:hover {
    color: var(--text-primary);
  }

  .ws-tab.active {
    color: var(--amber);
    border-bottom-color: var(--amber);
  }

  .workspace {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
</style>
