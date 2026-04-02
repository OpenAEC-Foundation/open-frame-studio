<script>
  import { showBackstage } from "../../stores/ui.js";

  let appWindow = null;
  let maximized = false;
  const isTauri = typeof window !== "undefined" && window.__TAURI_INTERNALS__;

  if (isTauri) {
    import("@tauri-apps/api/window").then((mod) => {
      appWindow = mod.getCurrentWindow();
    });
  }

  async function toggleMaximize() {
    if (!appWindow) return;
    await appWindow.toggleMaximize();
    maximized = await appWindow.isMaximized();
  }

  function minimize() { appWindow?.minimize(); }
  async function close() {
    if (isTauri) {
      try {
        const { exit } = await import("@tauri-apps/plugin-process");
        await exit(0);
      } catch {
        // Fallback: try window close, then force via API
        try {
          await appWindow?.close();
        } catch {
          window.close();
        }
      }
    }
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left">
    <button class="file-btn" on:click={() => showBackstage.set(true)}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <rect x="1" y="2" width="14" height="2" rx="0.5" />
        <rect x="1" y="7" width="14" height="2" rx="0.5" />
        <rect x="1" y="12" width="14" height="2" rx="0.5" />
      </svg>
      Bestand
    </button>
    <span class="title">Open Frame Studio</span>
  </div>

  <div class="titlebar-center" data-tauri-drag-region>
    <span class="subtitle">OpenAEC Foundation</span>
  </div>

  <div class="titlebar-controls">
    <button class="ctrl-btn" on:click={minimize}>
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1" y="5.5" width="10" height="1" fill="currentColor"/></svg>
    </button>
    <button class="ctrl-btn" on:click={toggleMaximize}>
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1" fill="none"/></svg>
    </button>
    <button class="ctrl-btn close" on:click={close}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
    </button>
  </div>
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
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    -webkit-app-region: no-drag;
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
    font-size: 11px;
    color: var(--scaffold-gray);
  }

  .titlebar-controls {
    display: flex;
    gap: 2px;
    -webkit-app-region: no-drag;
  }

  .ctrl-btn {
    display: flex;
    align-items: center;
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
