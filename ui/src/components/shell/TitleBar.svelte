<script>
  import { _ } from "svelte-i18n";
  import { showSettings } from "../../stores/ui.js";
  import { undo, redo, canUndo, canRedo } from "../../stores/history.js";
  import { confirmUnsavedChanges } from "../../lib/project-actions.js";
  import { isDirty } from "../../stores/project.js";

  let appWindow = null;
  let maximized = false;
  let platform = "windows";
  const isTauri = typeof window !== "undefined" && window.__TAURI_INTERNALS__;

  if (isTauri) {
    import("@tauri-apps/api/window").then(async (mod) => {
      appWindow = mod.getCurrentWindow();
      maximized = await appWindow.isMaximized();
      appWindow.onResized(async () => {
        maximized = await appWindow.isMaximized();
      });
    });
    import("@tauri-apps/api/core").then(async (mod) => {
      try { platform = await mod.invoke("get_platform"); } catch {}
    });
  }

  async function toggleMaximize() {
    if (!appWindow) return;
    await appWindow.toggleMaximize();
    maximized = await appWindow.isMaximized();
  }

  function minimize() { appWindow?.minimize(); }
  async function close() {
    if (!(await confirmUnsavedChanges())) return;
    if (isTauri) {
      try {
        const { exit } = await import("@tauri-apps/plugin-process");
        exit(0);
      } catch (e1) {
        console.warn("exit() failed:", e1);
        try {
          await appWindow?.destroy();
        } catch (e2) {
          console.warn("destroy() failed:", e2);
          try {
            await appWindow?.close();
          } catch {
            window.close();
          }
        }
      }
    }
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left">
    <img class="app-icon" src="/icon.png" alt="Open Frame Studio" />
    <button class="settings-btn" onclick={() => showSettings.set(true)}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
      </svg>
    </button>
    <div class="titlebar-sep"></div>
    <button class="titlebar-action" onclick={undo} disabled={!$canUndo}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3 5.5h6.5a4 4 0 0 1 0 8H6"/>
        <path d="M6 2.5L3 5.5l3 3"/>
      </svg>
    </button>
    <button class="titlebar-action" onclick={redo} disabled={!$canRedo}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
        <path d="M13 5.5H6.5a4 4 0 0 0 0 8H10"/>
        <path d="M10 2.5l3 3-3 3"/>
      </svg>
    </button>
  </div>

  <div class="titlebar-center" data-tauri-drag-region>
    <span class="subtitle">{$isDirty ? '* ' : ''}{$_('app.name')} <span class="version">v0.1.3</span></span>
  </div>

  {#if isTauri && platform !== "linux"}
  <div class="titlebar-controls">
    <button class="ctrl-btn" onclick={(e) => { e.preventDefault(); minimize(e); }}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 5.5h10" stroke="currentColor" stroke-width="1"/></svg>
    </button>
    <button class="ctrl-btn" onclick={(e) => { e.preventDefault(); toggleMaximize(e); }}>
      {#if maximized}
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M3 3v-2h8.5v8.5h-2" stroke="currentColor" stroke-width="1" fill="none"/>
          <path d="M0.5 3h8.5v8.5h-8.5z" stroke="currentColor" stroke-width="1" fill="var(--bg-titlebar, #1a1a1a)"/>
        </svg>
      {:else}
        <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1.5 1.5h9v9h-9z" stroke="currentColor" stroke-width="1" fill="none"/></svg>
      {/if}
    </button>
    <button class="ctrl-btn close" onclick={(e) => { e.preventDefault(); close(e); }}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
    </button>
  </div>
  {/if}
</div>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    background: var(--bg-titlebar);
    color: var(--text-on-dark);
    padding: 0 var(--sp-2);
    -webkit-app-region: drag;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    -webkit-app-region: no-drag;
  }

  .app-icon {
    width: 18px;
    height: 18px;
    margin-left: var(--sp-2);
  }

  .settings-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: var(--scaffold-gray);
    border-radius: var(--radius-sm);
    transition: background 0.15s, color 0.15s;
  }

  .settings-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-on-dark);
  }

  .titlebar-sep {
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.15);
  }

  .titlebar-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: var(--scaffold-gray);
    border-radius: var(--radius-sm);
    transition: background 0.15s, color 0.15s;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar-action:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-on-dark);
  }

  .titlebar-action:disabled {
    opacity: 0.3;
  }

  .file-btn {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: var(--sp-1) var(--sp-3);
    color: var(--text-on-dark);
    font-size: 13px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }

  .file-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .title {
    font-family: var(--font-heading);
    font-size: 13px;
    font-weight: 700;
    color: var(--amber);
  }

  .titlebar-center {
    flex: 1;
    text-align: center;
    -webkit-app-region: drag;
  }

  .subtitle {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-on-dark);
  }

  .version {
    font-size: 9px;
    opacity: 0.5;
    vertical-align: baseline;
    margin-left: 2px;
    letter-spacing: 0.5px;
  }

  .titlebar-controls {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .ctrl-btn {
    display: flex;
    align-items: center;
    user-select: none;
    -webkit-user-select: none;
    justify-content: center;
    width: 36px;
    height: 36px;
    color: var(--scaffold-gray);
    transition: background 0.15s, color 0.15s;
  }

  .ctrl-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-on-dark);
  }

  .ctrl-btn.close:hover {
    background: var(--error);
    color: white;
  }
</style>
