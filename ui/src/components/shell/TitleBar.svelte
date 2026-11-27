<script>
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";
  import { showSettings, showFeedback } from "../../stores/ui.js";
  import { undo, redo, canUndo, canRedo } from "../../stores/history.js";
  import { isDirty, project, projectPath } from "../../stores/project.js";
  import { fileSave } from "../../lib/project-actions.js";
  import { closeApplication } from "../../lib/shell-actions.js";
  import { isTauri, isPlatformBuild } from "../../lib/tauri.js";

  let appWindow;
  let maximized = false;
  let platform = "windows";
  $: documentName = $projectPath?.split(/[\\/]/).pop() || $project?.projectInfo?.name;

  onMount(() => {
    if (!isTauri) return;
    let disposed = false;
    let unlisten;
    async function connect() {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const { invoke } = await import("@tauri-apps/api/core");
      appWindow = getCurrentWindow();
      try { platform = await invoke("get_platform"); } catch {}
      maximized = await appWindow.isMaximized();
      unlisten = await appWindow.onResized(async () => {
        maximized = await appWindow.isMaximized();
      });
      if (disposed) unlisten();
    }
    connect();
    return () => { disposed = true; unlisten?.(); };
  });
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="quick">
    <img class="app-icon" src={import.meta.env.BASE_URL + "icon.png"} alt="" />
    <span class="divider"></span>
    <button class="action" title={$_('backstage.save') + ' (Ctrl+S)'} aria-label={$_('backstage.save')} onclick={fileSave}>
      <BlueprintIcon name="file.save" />
    </button>
    <button class="action" title={$_('profileEditor.undo') + ' (Ctrl+Z)'} aria-label={$_('profileEditor.undo')} onclick={undo} disabled={!$canUndo}>
      <BlueprintIcon name="edit.undo" />
    </button>
    <button class="action" title={$_('ribbon.redo') + ' (Ctrl+Y)'} aria-label={$_('ribbon.redo')} onclick={redo} disabled={!$canRedo}>
      <BlueprintIcon name="edit.redo" />
    </button>
  </div>

  <div class="caption" data-tauri-drag-region>
    {#if documentName}<span class="document" title={documentName} data-tauri-drag-region>{documentName}</span>{/if}
    {#if $isDirty}<span class="dirty" role="img" aria-label={$_('shell.unsaved')} title={$_('shell.unsaved')}></span>{/if}
    {#if documentName}<span class="divider"></span>{/if}
    <strong data-tauri-drag-region>{$_('app.name')}</strong>
  </div>

  <div class="utilities">
    {#if !isPlatformBuild}
      <button class="action feedback" title={$_('feedback.title')} onclick={() => showFeedback.set(true)}>
        <BlueprintIcon name="feedback.github-issue" /><span>{$_('shell.feedback')}</span>
      </button>
    {/if}
    <button class="action" title={$_('settings.title') + ' (Ctrl+,)'} aria-label={$_('settings.title')} onclick={() => showSettings.set(true)}>
      <BlueprintIcon name="ui.settings" />
    </button>
    {#if isTauri && platform !== "linux"}
      <span class="divider"></span>
      <button class="ctrl" title={$_('shell.minimize')} aria-label={$_('shell.minimize')} onclick={() => appWindow?.minimize()}>
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M1 6h10" stroke="currentColor"/></svg>
      </button>
      <button class="ctrl" title={$_(maximized ? 'shell.restore' : 'shell.maximize')} aria-label={$_(maximized ? 'shell.restore' : 'shell.maximize')} onclick={() => appWindow?.toggleMaximize()}>
        <svg width="12" height="12" viewBox="0 0 12 12" stroke="currentColor" fill="none" aria-hidden="true">
          {#if maximized}<path d="M3 3V1.5h7.5V9H9M1.5 3.5h7v7h-7z"/>{:else}<rect x="1.5" y="1.5" width="9" height="9"/>{/if}
        </svg>
      </button>
      <button class="ctrl close" title={$_('shell.exit')} aria-label={$_('shell.exit')} onclick={closeApplication}>
        <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M2 2l8 8m0-8l-8 8" stroke="currentColor"/></svg>
      </button>
    {/if}
  </div>
</header>

<style>
  .titlebar {
    flex: 0 0 var(--oa-shell-titlebar);
    height: var(--oa-shell-titlebar);
    display: grid;
    grid-template-columns: minmax(max-content, 1fr) minmax(0, auto) minmax(max-content, 1fr);
    align-items: center;
    gap: 8px;
    padding-inline: 8px 4px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-titlebar);
    color: var(--text-primary);
    user-select: none;
  }
  .quick, .utilities, .caption { display: flex; align-items: center; min-width: 0; }
  .quick, .utilities { gap: 2px; width: fit-content; }
  .utilities { justify-self: end; }
  .app-icon { width: 20px; height: 20px; object-fit: contain; margin-inline: 2px; }
  .divider { width: 1px; height: 14px; flex: none; margin-inline: 4px; background: var(--border-color); }
  .caption { justify-content: center; gap: 6px; white-space: nowrap; overflow: hidden; }
  .caption strong { font: 600 13px var(--font-heading); }
  .document { font: 500 12px var(--font-body); overflow: hidden; text-overflow: ellipsis; color: var(--text-secondary); }
  .dirty { flex: none; width: 7px; height: 7px; border-radius: 50%; background: var(--text-accent); }
  .action, .ctrl { display: inline-flex; align-items: center; justify-content: center; color: var(--text-secondary); }
  .action { width: 28px; height: 28px; border-radius: 4px; }
  .feedback { width: auto; gap: 5px; padding-inline: 6px; font-size: 12px; font-weight: 400; }
  .ctrl { width: 36px; height: 36px; }
  .action:hover:not(:disabled), .ctrl:hover { color: var(--text-primary); background: var(--oa-hover); }
  .action:disabled { opacity: .4; }
  .ctrl.close:hover { background: var(--error); color: white; }
  @media (max-width: 700px) {
    .feedback { width: 28px; }
    .feedback span, .caption strong, .caption .divider { display: none; }
  }
</style>
