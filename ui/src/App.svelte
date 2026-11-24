<script>
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { _, isLoading } from "svelte-i18n";
  import TitleBar from "./components/shell/TitleBar.svelte";
  import Ribbon from "./components/shell/Ribbon.svelte";
  import StatusBar from "./components/shell/StatusBar.svelte";
  import AppMenu from "./components/shell/AppMenu.svelte";
  import Settings from "./components/shell/Settings.svelte";
  import Toast from "./components/shell/Toast.svelte";
  import WorkspaceTabs from "./components/shell/WorkspaceTabs.svelte";
  import WorkspacePanel from "./components/shell/WorkspacePanel.svelte";
  import ResizeHandle from "./components/shell/ResizeHandle.svelte";
  import KozijnEditor from "./components/editor/KozijnEditor.svelte";
  import VliesgevelEditor from "./components/editor/VliesgevelEditor.svelte";
  import { currentVliesgevel } from "./stores/vliesgevel.js";
  import PropertiesPanel from "./components/panels/PropertiesPanel.svelte";
  import ProjectOverview from "./components/project/ProjectOverview.svelte";
  import KozijnstaatView from "./components/project/KozijnstaatView.svelte";
  import ProductionListsView from "./components/project/ProductionListsView.svelte";
  import CalculationView from "./components/project/CalculationView.svelte";
  import DashboardView from "./components/project/DashboardView.svelte";
  import LayoutProtoView from "./components/project/LayoutProtoView.svelte";
  import Viewer3D from "./components/viewer3d/Viewer3D.svelte";
  import ProfileEditorView from "./components/profile-editor/ProfileEditorView.svelte";
  import QuotationView from "./components/project/QuotationView.svelte";
  import PlanningView from "./components/project/PlanningView.svelte";
  import EnergyView from "./components/project/EnergyView.svelte";
  import CircularityView from "./components/project/CircularityView.svelte";
  import PlausibilityView from "./components/project/PlausibilityView.svelte";
  import CertificationView from "./components/project/CertificationView.svelte";
  import BcfView from "./components/project/BcfView.svelte";
  import CutOptimizationView from "./components/project/CutOptimizationView.svelte";
  import IdsValidationView from "./components/project/IdsValidationView.svelte";
  import CombinationView from "./components/project/CombinationView.svelte";
  import ProcurementView from "./components/project/ProcurementView.svelte";
  import CncView from "./components/project/CncView.svelte";
  import AiAssistant from "./components/panels/AiAssistant.svelte";
  import { loadProject } from "./stores/project.js";
  import { loadProfiles } from "./stores/profiles.js";
  import { registerUndoRedoShortcuts } from "./stores/history.js";
  import { activeWorkspaceView, showAppMenu, showSettings } from "./stores/ui.js";
  import { getSetting, saveSetting } from "./lib/settings.js";
  import { isWeb, isPlatformBuild, getWebRuntimeMode } from "./lib/tauri.js";
  import { hostFileRef, isDirty } from "./stores/project.js";
  import { getOpenAecHost } from "./lib/platformHost.js";
  import { openHostFileById } from "./lib/project-actions.js";
  import { toast } from "./stores/toast.js";

  // OpenAEC's Frame profile uses a wrapping workspace strip and independent side panels.
  const views = [
    { id: "editor", icon: "frame.editor" },
    { id: "viewer3d", icon: "bim.workspace" },
    { id: "kozijnstaat", icon: "frame.schedule", component: KozijnstaatView },
    { id: "production", icon: "frame.production", component: ProductionListsView },
    { id: "calculation", icon: "frame.calculation", component: CalculationView },
    { id: "dashboard", icon: "field.dashboard", component: DashboardView },
    { id: "layoutProto", icon: "frame.layout", component: LayoutProtoView },
    { id: "profiles", icon: "profile.editor", component: ProfileEditorView },
    { id: "planning", icon: "planning.production", component: PlanningView },
    { id: "energy", icon: "frame.energy", component: EnergyView },
    { id: "circularity", icon: "frame.circularity", component: CircularityView },
    { id: "plausibility", icon: "frame.plausibility", component: PlausibilityView },
    { id: "quotation", icon: "frame.quotation", component: QuotationView },
    { id: "bcf", icon: "bcf.workspace", component: BcfView },
    { id: "optimization", icon: "planning.cut-list", component: CutOptimizationView },
    { id: "cnc", icon: "frame.cnc", component: CncView },
    { id: "procurement", icon: "frame.procurement", component: ProcurementView },
    { id: "combination", icon: "frame.combination", component: CombinationView },
    { id: "ids", icon: "validation.workspace", component: IdsValidationView },
    { id: "certification", icon: "frame.certification", component: CertificationView },
  ];
  $: selectedView = views.find(view => view.id === $activeWorkspaceView);
  $: isEditor = $activeWorkspaceView === "editor";
  $: hasPanels = isEditor || $activeWorkspaceView === "viewer3d";
  let viewportWidth = window.innerWidth;
  let compactPanel = null;
  $: compact = viewportWidth <= 760;
  $: if (!compact || !hasPanels) compactPanel = null;
  $: showLeft = compact ? compactPanel === "left" : leftOpen;
  $: showRight = compact ? compactPanel === "right" : rightOpen;

  let cleanupShortcuts;
  let hostStartupHandled = false;
  const webRuntimeMode = getWebRuntimeMode();
  let rightTab = "properties";
  let leftWidth = getSetting("left_panel_width") || 220;
  let rightWidth = getSetting("right_panel_width") || 290;
  let leftOpen = getSetting("left_panel_open") ?? true;
  let rightOpen = getSetting("right_panel_open") ?? true;

  function resizeLeft(delta) {
    leftWidth = Math.max(140, Math.min(400, leftWidth + delta));
    saveSetting("left_panel_width", leftWidth);
  }
  function resizeRight(delta) {
    rightWidth = Math.max(200, Math.min(500, rightWidth - delta));
    saveSetting("right_panel_width", rightWidth);
  }
  function togglePanel(side) {
    if (compact) {
      compactPanel = compactPanel === side ? null : side;
    } else if (side === "left") {
      leftOpen = !leftOpen;
      saveSetting("left_panel_open", leftOpen);
    } else {
      rightOpen = !rightOpen;
      saveSetting("right_panel_open", rightOpen);
    }
  }

  function handlePanelShortcut(event) {
    if (!hasPanels || $showAppMenu || $showSettings || event.altKey || event.shiftKey) return;
    if (!(event.ctrlKey || event.metaKey) || !["1", "2"].includes(event.key)) return;
    event.preventDefault();
    togglePanel(event.key === "1" ? "left" : "right");
  }

  async function openInitialHostProject() {
    if (hostStartupHandled) return;
    const host = getOpenAecHost();
    if (!host) return;
    hostStartupHandled = true;
    if (host.initialFileId) {
      if (get(isDirty)) {
        toast.error("Docs-project kon niet automatisch openen: sla eerst de huidige wijzigingen op");
        return;
      }
      await openHostFileById(host.initialFileId);
    }
  }

  onMount(async () => {
    await loadProject();
    await loadProfiles();
    window.addEventListener("openaec-host-ready", openInitialHostProject);
    await openInitialHostProject();
    cleanupShortcuts = registerUndoRedoShortcuts();
  });

  onDestroy(() => {
    window.removeEventListener("openaec-host-ready", openInitialHostProject);
    if (cleanupShortcuts) cleanupShortcuts();
  });
</script>

<svelte:window bind:innerWidth={viewportWidth} onkeydown={handlePanelShortcut} />

{#if $isLoading}
  <div class="app-loading" role="status">Open Frame Studio…</div>
{:else}
<TitleBar />
<Ribbon />
{#if isWeb}
  <div class="web-mode-banner" role="status">
    {#if webRuntimeMode === "wasm"}
      {$_("workspace.webNotice")}
    {:else}
      {$_("workspace.previewNotice")}
    {/if}
  </div>
{/if}
{#if $hostFileRef?.readOnly}
  <div class="web-mode-banner" role="status">
    {$_("workspace.readOnlyNotice", { values: { name: $hostFileRef.name } })}
  </div>
{/if}
<AppMenu />
<Settings />

<WorkspaceTabs {views} />

<main class="workspace" role="tabpanel" id="workspace-panel" aria-labelledby={"tab-" + $activeWorkspaceView} tabindex="0">
  {#if hasPanels}
    <WorkspacePanel side="left" label={$_("workspace.explorer")} width={leftWidth} open={showLeft} ontoggle={() => togglePanel("left")}>
      <ProjectOverview />
    </WorkspacePanel>
    {#if showLeft && !compact}<ResizeHandle direction="horizontal" onresize={resizeLeft} />{/if}

    {#if !isEditor}
      <Viewer3D visible={true} />
    {:else if $currentVliesgevel}
      <VliesgevelEditor />
    {:else}
      <KozijnEditor />
    {/if}

    {#if showRight && !compact}<ResizeHandle direction="horizontal" onresize={resizeRight} />{/if}
    <WorkspacePanel side="right" label={$_("workspace.properties")} width={rightWidth} open={showRight} ontoggle={() => togglePanel("right")}>
      {#snippet heading()}
        {#if isEditor && !isPlatformBuild}
          <div class="panel-tabs">
            <button class:active={rightTab === "properties"} aria-pressed={rightTab === "properties"} onclick={() => rightTab = "properties"}>
              {$_("workspace.properties")}
            </button>
            <button class:active={rightTab === "ai"} aria-pressed={rightTab === "ai"} onclick={() => rightTab = "ai"}>AI</button>
          </div>
        {:else}
          <span>{$_("workspace.properties")}</span>
        {/if}
      {/snippet}
      {#if isEditor && rightTab === "ai" && !isPlatformBuild}
        <AiAssistant />
      {:else}
        <PropertiesPanel />
      {/if}
    </WorkspacePanel>
  {:else if selectedView?.component}
    <svelte:component this={selectedView.component} />
  {/if}
</main>

<StatusBar />
<Toast />
{/if}

<style>
  .app-loading {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--oa-text-secondary);
    background: var(--oa-frame);
  }

  .web-mode-banner {
    flex: none;
    padding: 3px var(--oa-space-md);
    border-bottom: 1px solid var(--oa-border);
    background: var(--oa-warning-tint);
    color: var(--oa-warning);
    font-size: var(--oa-size-label);
  }

  .workspace {
    position: relative;
    display: flex;
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: var(--oa-workspace);
  }

  .panel-tabs {
    display: flex;
    height: 100%;
    gap: 2px;
    margin-left: -8px;
  }

  .panel-tabs button {
    padding: 0 8px;
    color: var(--oa-text-secondary);
    border-bottom: 2px solid transparent;
    font-size: var(--oa-size-label);
    font-weight: var(--oa-weight-medium);
  }

  .panel-tabs button:hover { background: var(--oa-hover); }
  .panel-tabs button.active { color: var(--oa-accent-text); border-bottom-color: var(--oa-selected-bar); }
</style>
