<script>
  import { _ } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";
  import { showAppMenu, showSettings } from "../../stores/ui.js";
  import { project, projectPath, isDirty, kozijnen } from "../../stores/project.js";
  import { isTauri, isPlatformBuild } from "../../lib/tauri.js";
  import { fileNew, fileOpen, fileSave, fileSaveAs } from "../../lib/project-actions.js";
  import { openProjectLink, closeApplication } from "../../lib/shell-actions.js";
  import { APP_VERSION } from "../../lib/version.js";

  const fileActions = [
    { label: "backstage.newProject", icon: "file.new", shortcut: "Ctrl+N", run: fileNew },
    { label: "backstage.open", icon: "file.open", shortcut: "Ctrl+O", run: fileOpen },
    { label: "backstage.save", icon: "file.save", shortcut: "Ctrl+S", run: fileSave },
    { label: "backstage.saveAs", icon: "file.save-as", shortcut: "Ctrl+Shift+S", run: fileSaveAs },
  ];

  let pending = false;
  async function run(action) {
    if (pending) return;
    pending = true;
    try {
      if (await action()) showAppMenu.set(false);
    } finally {
      pending = false;
    }
  }

  function openSettings() {
    showAppMenu.set(false);
    showSettings.set(true);
  }

  function focusMenu(node) {
    const previous = document.activeElement;
    node.querySelector("button").focus();
    return { destroy() { if (previous?.isConnected) previous.focus(); } };
  }

  function handleKeydown(event) {
    event.stopPropagation();
    if (event.key === "Escape") {
      showAppMenu.set(false);
    } else if (event.key === "Tab") {
      const buttons = event.currentTarget.querySelectorAll("button:not(:disabled)");
      const first = buttons[0];
      const last = buttons[buttons.length - 1];
      if (event.shiftKey && event.target === first || !event.shiftKey && event.target === last) {
        event.preventDefault();
        (event.shiftKey ? last : first).focus();
      }
    } else if (event.ctrlKey || event.metaKey) {
      const key = event.key.toLowerCase();
      if (key === ",") {
        event.preventDefault();
        openSettings();
        return;
      }
      const action = { n: fileNew, o: fileOpen, s: event.shiftKey ? fileSaveAs : fileSave }[key];
      if (action) {
        event.preventDefault();
        run(action);
      }
    }
  }
</script>

{#if $showAppMenu}
  <div class="backstage" role="dialog" tabindex="-1" aria-label={$_('titlebar.file')} onkeydown={handleKeydown} use:focusMenu>
    <nav aria-label={$_('titlebar.file')}>
      <button class="back" onclick={() => showAppMenu.set(false)}>
        <BlueprintIcon name="nav.back" />{$_('backstage.back')}
      </button>
      <div class="group">
        <h2>{$_('backstage.project')}</h2>
        {#each fileActions as action}
          <button class="nav-item" disabled={pending} onclick={() => run(action.run)}>
            <BlueprintIcon name={action.icon} /><span>{$_(action.label)}</span><kbd>{action.shortcut}</kbd>
          </button>
        {/each}
      </div>
      {#if !isPlatformBuild}
        <div class="group">
          <h2>{$_('backstage.help')}</h2>
          <button class="nav-item" onclick={() => openProjectLink('/wiki')}><BlueprintIcon name="ui.help" />{$_('backstage.documentation')}</button>
          <button class="nav-item" onclick={() => openProjectLink('/issues')}><BlueprintIcon name="feedback.github-issue" />{$_('backstage.reportIssue')}</button>
          <button class="nav-item" onclick={() => openProjectLink()}><BlueprintIcon name="ui.about" />GitHub</button>
        </div>
      {/if}
      <div class="group tools">
        <button class="nav-item" onclick={openSettings}><BlueprintIcon name="ui.settings" />{$_('settings.title')}</button>
        {#if isTauri}<button class="nav-item" onclick={closeApplication}><BlueprintIcon name="ui.close" />{$_('shell.exit')}</button>{/if}
      </div>
      <small>v{APP_VERSION}</small>
    </nav>

    <div class="content">
      <h1>{$_('titlebar.file')}</h1>
      <div class="tasks">
        {#each fileActions.slice(0, 2) as action}
          <button class="tile" disabled={pending} onclick={() => run(action.run)}>
            <BlueprintIcon name={action.icon} size={24} />
            <strong>{$_(action.label)}</strong><kbd>{action.shortcut}</kbd>
          </button>
        {/each}
      </div>
      {#if $project}
        <section>
          <h2>{$_('shell.currentProject')}</h2>
          <div class="document">
            <strong>{$project.projectInfo?.name || $projectPath?.split(/[\\/]/).pop() || $_('backstage.project')}</strong>
            {#if $projectPath}<span class="path">{$projectPath}</span>{/if}
            <span>{$_('status.kozijnen', { values: { count: $kozijnen.length } })}{#if $isDirty} · {$_('shell.unsaved')}{/if}</span>
          </div>
        </section>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backstage {
    position: fixed;
    inset: var(--oa-shell-titlebar) 0 var(--oa-shell-statusbar);
    display: grid;
    grid-template-columns: min(260px, 34%) minmax(0, 1fr);
    z-index: 50;
    background: var(--bg-app);
    color: var(--text-primary);
  }
  nav { min-width: 0; overflow: auto; display: flex; flex-direction: column; padding: 12px 10px; background: var(--bg-surface); border-right: 1px solid var(--border-color); }
  .back, .nav-item { display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 4px; text-align: left; color: var(--text-primary); font: 400 13px var(--font-body); }
  .back { margin-bottom: 12px; align-self: flex-start; }
  .group { display: grid; gap: 2px; padding: 10px 0; border-top: 1px solid var(--border-color); }
  .group h2 { margin: 0 10px 4px; font: 600 11px var(--font-body); color: var(--text-secondary); }
  .tools { margin-top: auto; }
  .nav-item:hover, .back:hover { background: var(--oa-hover); }
  .nav-item kbd { margin-left: auto; }
  kbd, small, .document span { color: var(--text-secondary); }
  kbd, small { font: 11px var(--font-code); }
  small { padding: 5px 10px; }
  .content { min-width: 0; padding: clamp(20px, 4vw, 56px); overflow: auto; }
  h1 { font: 600 22px var(--font-heading); margin-bottom: 20px; }
  .tasks { display: flex; gap: 12px; flex-wrap: wrap; }
  .tile { display: grid; grid-template-columns: 24px 1fr; align-items: center; gap: 4px 12px; min-width: 160px; padding: 16px; border: 1px solid var(--border-color); border-radius: 8px; background: var(--bg-surface); text-align: left; }
  .tile :global(svg) { grid-row: span 2; }
  .tile strong, .document strong { font-size: 14px; font-weight: 600; }
  .tile kbd { grid-column: 2; }
  .tile:hover { border-color: var(--text-accent); }
  section { margin-top: 32px; max-width: 640px; }
  section h2 { margin-bottom: 10px; font: 600 16px var(--font-heading); }
  .document { display: grid; gap: 3px; padding: 12px 0; border-block: 1px solid var(--border-color); }
  .document span { font-size: 12px; overflow-wrap: anywhere; }
  .path { user-select: text; }
  @media (max-width: 680px) { .nav-item kbd { display: none; } }
  @media (max-width: 420px) { .backstage { grid-template-columns: 135px minmax(0, 1fr); } .tile { min-width: 0; width: 100%; } .content { padding: 16px; } }
</style>
