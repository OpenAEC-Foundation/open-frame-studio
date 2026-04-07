<script>
  import { onMount, onDestroy } from "svelte";
  import { _, isLoading } from "svelte-i18n";
  import TitleBar from "./components/shell/TitleBar.svelte";
  import Ribbon from "./components/shell/Ribbon.svelte";
  import StatusBar from "./components/shell/StatusBar.svelte";
  import AppMenu from "./components/shell/AppMenu.svelte";
  import Settings from "./components/shell/Settings.svelte";
  import Toast from "./components/shell/Toast.svelte";
  import ResizeHandle from "./components/shell/ResizeHandle.svelte";
  import KozijnEditor from "./components/editor/KozijnEditor.svelte";
  import VliesgevalEditor from "./components/editor/VliesgevalEditor.svelte";
  import { currentVliesgevel } from "./stores/vliesgevel.js";
  import PropertiesPanel from "./components/panels/PropertiesPanel.svelte";
  import ProjectOverview from "./components/project/ProjectOverview.svelte";
  import KozijnstaatView from "./components/project/KozijnstaatView.svelte";
  import ProductionListsView from "./components/project/ProductionListsView.svelte";
  import CalculationView from "./components/project/CalculationView.svelte";
  import Viewer3D from "./components/viewer3d/Viewer3D.svelte";
  import ProfileEditorView from "./components/profile-editor/ProfileEditorView.svelte";
  import AiAssistant from "./components/panels/AiAssistant.svelte";
  import { loadProject } from "./stores/project.js";
  import { loadProfiles } from "./stores/profiles.js";
  import { registerUndoRedoShortcuts } from "./stores/history.js";
  import { activeRibbonTab } from "./stores/ui.js";
  import { getSetting, saveSetting } from "./lib/settings.js";

  let cleanupShortcuts;
  let workspaceView = "editor";
  let rightTab = "properties";
  let leftWidth = getSetting("left_panel_width") || 220;
  let rightWidth = getSetting("right_panel_width") || 290;
  let leftOpen = getSetting("left_panel_open") ?? true;
  let rightOpen = getSetting("right_panel_open") ?? true;
  let leftWidthBefore = leftWidth;
  let rightWidthBefore = rightWidth;

  function resizeLeft(delta) {
    leftWidth = Math.max(140, Math.min(400, leftWidth + delta));
    leftWidthBefore = leftWidth;
    saveSetting("left_panel_width", leftWidth);
  }
  function resizeRight(delta) {
    rightWidth = Math.max(200, Math.min(500, rightWidth - delta));
    rightWidthBefore = rightWidth;
    saveSetting("right_panel_width", rightWidth);
  }
  function toggleLeft() {
    leftOpen = !leftOpen;
    leftWidth = leftOpen ? leftWidthBefore : 0;
    saveSetting("left_panel_open", leftOpen);
  }
  function toggleRight() {
    rightOpen = !rightOpen;
    rightWidth = rightOpen ? rightWidthBefore : 0;
    saveSetting("right_panel_open", rightOpen);
  }

  onMount(async () => {
    await loadProject();
    await loadProfiles();
    cleanupShortcuts = registerUndoRedoShortcuts();
  });

  onDestroy(() => {
    if (cleanupShortcuts) cleanupShortcuts();
  });
</script>

{#if $isLoading}
  <div style="display:flex;align-items:center;justify-content:center;height:100vh;background:var(--bg-surface);color:var(--text-muted);">Loading...</div>
{:else}
<TitleBar />
<Ribbon />
<AppMenu />
<Settings />

<div class="workspace-tabs">
  <button
    class="ws-tab"
    class:active={workspaceView === "editor"}
    onclick={() => (workspaceView = "editor")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="1"/>
      <line x1="12" y1="3" x2="12" y2="21"/>
      <line x1="3" y1="12" x2="21" y2="12"/>
    </svg>
    {$_('tabs.editor')}
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "viewer3d"}
    onclick={() => (workspaceView = "viewer3d")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2L2 7l10 5 10-5-10-5z"/>
      <path d="M2 17l10 5 10-5"/>
      <path d="M2 12l10 5 10-5"/>
    </svg>
    {$_('tabs.viewer3d')}
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "kozijnstaat"}
    onclick={() => (workspaceView = "kozijnstaat")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="3" width="18" height="18" rx="1"/>
      <line x1="3" y1="9" x2="21" y2="9"/>
      <line x1="3" y1="15" x2="21" y2="15"/>
      <line x1="9" y1="3" x2="9" y2="21"/>
    </svg>
    {$_('tabs.kozijnstaat')}
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "production"}
    onclick={() => (workspaceView = "production")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2"/>
      <rect x="9" y="3" width="6" height="4" rx="1"/>
      <line x1="9" y1="12" x2="15" y2="12"/>
      <line x1="9" y1="16" x2="15" y2="16"/>
    </svg>
    {$_('tabs.production')}
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "calculation"}
    onclick={() => (workspaceView = "calculation")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="4" y="2" width="16" height="20" rx="1"/>
      <line x1="8" y1="6" x2="16" y2="6"/>
      <line x1="8" y1="10" x2="16" y2="10"/>
      <line x1="8" y1="14" x2="12" y2="14"/>
      <line x1="8" y1="18" x2="12" y2="18"/>
    </svg>
    {$_('tabs.calculation')}
  </button>
  <button
    class="ws-tab"
    class:active={workspaceView === "profiles"}
    onclick={() => (workspaceView = "profiles")}
  >
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M4 4h4v16H4z"/>
      <path d="M12 4h2v16h-2z"/>
      <path d="M18 4h2v16h-2z"/>
    </svg>
    {$_('tabs.profiles')}
  </button>
</div>

<div class="workspace">
  {#if workspaceView === "editor"}
    {#if leftOpen}
      <div class="side-panel" style="width:{leftWidth}px">
        <div class="panel-header">
          <span>{$_('project.kozijnen')}</span>
          <button class="collapse-btn" onclick={toggleLeft}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
          </button>
        </div>
        <div class="panel-body"><ProjectOverview /></div>
      </div>
      <ResizeHandle direction="horizontal" onresize={resizeLeft} />
    {:else}
      <div class="collapsed-tab left" onclick={toggleLeft}>
        <span>{$_('project.kozijnen')}</span>
      </div>
    {/if}
    {#if $currentVliesgevel}
      <VliesgevalEditor />
    {:else}
      <KozijnEditor />
    {/if}
    {#if rightOpen}
      <ResizeHandle direction="horizontal" onresize={resizeRight} />
      <div class="side-panel right" style="width:{rightWidth}px">
        <div class="panel-header tabbed">
          <div class="panel-tabs">
            <button class="panel-tab" class:active={rightTab === "properties"} onclick={() => rightTab = "properties"}>
              {$_('props.kozijn')}
            </button>
            <button class="panel-tab" class:active={rightTab === "ai"} onclick={() => rightTab = "ai"}>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M12 2a5 5 0 015 5v1h1a3 3 0 013 3v1a3 3 0 01-3 3h-1v4a3 3 0 01-3 3H10a3 3 0 01-3-3v-4H6a3 3 0 01-3-3v-1a3 3 0 013-3h1V7a5 5 0 015-5z"/>
              </svg>
              AI
            </button>
          </div>
          <button class="collapse-btn" onclick={toggleRight}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </button>
        </div>
        <div class="panel-body">
          {#if rightTab === "properties"}
            <PropertiesPanel />
          {:else}
            <AiAssistant />
          {/if}
        </div>
      </div>
    {:else}
      <div class="collapsed-tab right" onclick={toggleRight}>
        <span>{$_('props.kozijn')}</span>
      </div>
    {/if}
  {:else if workspaceView === "viewer3d"}
    {#if leftOpen}
      <div class="side-panel" style="width:{leftWidth}px">
        <div class="panel-header">
          <span>{$_('project.kozijnen')}</span>
          <button class="collapse-btn" onclick={toggleLeft}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
          </button>
        </div>
        <div class="panel-body"><ProjectOverview /></div>
      </div>
      <ResizeHandle direction="horizontal" onresize={resizeLeft} />
    {:else}
      <div class="collapsed-tab left" onclick={toggleLeft}>
        <span>{$_('project.kozijnen')}</span>
      </div>
    {/if}
    <Viewer3D visible={workspaceView === "viewer3d"} />
    {#if rightOpen}
      <ResizeHandle direction="horizontal" onresize={resizeRight} />
      <div class="side-panel right" style="width:{rightWidth}px">
        <div class="panel-header">
          <span>{$_('props.kozijn')}</span>
          <button class="collapse-btn" onclick={toggleRight}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </button>
        </div>
        <div class="panel-body"><PropertiesPanel /></div>
      </div>
    {:else}
      <div class="collapsed-tab right" onclick={toggleRight}>
        <span>{$_('props.kozijn')}</span>
      </div>
    {/if}
  {:else if workspaceView === "kozijnstaat"}
    <KozijnstaatView />
  {:else if workspaceView === "production"}
    <ProductionListsView />
  {:else if workspaceView === "calculation"}
    <CalculationView />
  {:else if workspaceView === "profiles"}
    <ProfileEditorView />
  {/if}
</div>

<StatusBar />
<Toast />
{/if}

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

  .side-panel {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-color, rgba(0,0,0,0.1));
    overflow: hidden;
  }

  .side-panel.right {
    border-right: none;
    border-left: 1px solid var(--border-color, rgba(0,0,0,0.1));
  }

  .collapsed-tab {
    width: 24px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface-alt);
    border-right: 1px solid var(--border-color, rgba(0,0,0,0.08));
    user-select: none;
    transition: background 0.12s;
  }

  .collapsed-tab.right {
    border-right: none;
    border-left: 1px solid var(--border-color, rgba(0,0,0,0.08));
  }

  .collapsed-tab:hover {
    background: var(--bg-surface);
  }

  .collapsed-tab span {
    writing-mode: vertical-lr;
    transform: rotate(180deg);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .collapsed-tab:hover span {
    color: var(--text-primary);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 26px;
    padding: 0 8px;
    background: var(--bg-surface-alt);
    border-bottom: 1px solid var(--border-color, rgba(0,0,0,0.08));
    flex-shrink: 0;
    user-select: none;
  }

  .panel-header span {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .collapse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 2px;
    flex-shrink: 0;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.12s;
  }

  .panel-header:hover .collapse-btn {
    opacity: 1;
  }

  .collapse-btn:hover {
    background: rgba(0, 0, 0, 0.08);
    color: var(--text-primary);
  }

  .panel-header.tabbed {
    padding: 0 4px 0 0;
  }

  .panel-tabs {
    display: flex;
    gap: 0;
    height: 100%;
  }

  .panel-tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 10px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
    height: 100%;
  }

  .panel-tab:hover {
    color: var(--text-primary);
  }

  .panel-tab.active {
    color: var(--amber);
    border-bottom-color: var(--amber);
  }

  .panel-body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
