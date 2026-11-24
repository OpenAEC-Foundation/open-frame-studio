<script>
  import { _, locale, locales } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";
  import { showSettings, theme, setTheme, THEMES } from "../../stores/ui.js";
  import { APP_VERSION } from "../../lib/version.js";
  import { isPlatformBuild } from "../../lib/tauri.js";
  import { openProjectLink } from "../../lib/shell-actions.js";

  const pages = [
    { id: "general", label: "settings.cat.general", icon: "ui.settings" },
    { id: "appearance", label: "shell.appearance", icon: "ui.settings" },
    { id: "about", label: "settings.cat.about", icon: "ui.about" },
  ];
  let activePage = pages[0];
  let dialog;
  let drag = null;

  function openModal(node) {
    node.showModal();
    return { destroy() { node.close(); } };
  }

  function startDrag(event) {
    if (event.button !== 0 || event.target.closest("button")) return;
    const rect = dialog.getBoundingClientRect();
    drag = { x: event.clientX - rect.left, y: event.clientY - rect.top };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function moveDialog(event) {
    if (!drag) return;
    const left = Math.max(0, Math.min(window.innerWidth - dialog.offsetWidth, event.clientX - drag.x));
    const top = Math.max(0, Math.min(window.innerHeight - dialog.offsetHeight, event.clientY - drag.y));
    dialog.style.margin = "0";
    dialog.style.left = left + "px";
    dialog.style.top = top + "px";
  }

  function draggableTitle(node) {
    const release = () => { drag = null; };
    node.addEventListener("pointerdown", startDrag);
    node.addEventListener("pointermove", moveDialog);
    node.addEventListener("lostpointercapture", release);
    return { destroy() {
      node.removeEventListener("pointerdown", startDrag);
      node.removeEventListener("pointermove", moveDialog);
      node.removeEventListener("lostpointercapture", release);
      release();
    } };
  }

  function settingsShortcut(event) {
    if (event.key === "," && (event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey) {
      event.preventDefault();
      showSettings.set(true);
    }
  }
</script>

<svelte:window onkeydown={settingsShortcut} />
{#if $showSettings}
  <dialog bind:this={dialog} use:openModal aria-labelledby="settings-heading" oncancel={() => showSettings.set(false)} onkeydown={(event) => event.stopPropagation()}>
    <header use:draggableTitle>
      <h1 id="settings-heading">{$_('settings.title')}</h1>
      <button class="close" title={$_('settings.close')} aria-label={$_('settings.close')} onclick={() => showSettings.set(false)}>
        <BlueprintIcon name="ui.close" />
      </button>
    </header>
    <div class="body">
      <nav aria-label={$_('settings.title')}>
        {#each pages as page}
          <button aria-current={page === activePage ? 'page' : undefined} onclick={() => activePage = page}>
            <BlueprintIcon name={page.icon} />{$_(page.label)}
          </button>
        {/each}
      </nav>
      <section aria-labelledby="settings-page-heading">
        <h2 id="settings-page-heading">{$_(activePage.label)}</h2>
        {#if activePage.id === "general"}
          <label>
            <span>{$_('settings.language')}</span>
            <select value={$locale} onchange={(event) => locale.set(event.currentTarget.value)}>
              {#each $locales as language}
                <option value={language} lang={language}>{$_('lang.' + language)}</option>
              {/each}
            </select>
          </label>
        {:else if activePage.id === "appearance"}
          <label>
            <span>{$_('settings.theme')}</span>
            <select value={$theme} onchange={(event) => setTheme(event.currentTarget.value)}>
              {#each THEMES as name}<option value={name}>{$_('settings.theme.' + name)}</option>{/each}
            </select>
          </label>
        {:else}
          <div class="about">
            <img src={import.meta.env.BASE_URL + "icon.png"} alt="" width="40" height="40" />
            <div><strong>{$_('app.name')}</strong><p>v{APP_VERSION}</p><p>{$_('app.org')}</p></div>
          </div>
          {#if !isPlatformBuild}<button class="repository" onclick={() => openProjectLink()}>GitHub</button>{/if}
        {/if}
      </section>
    </div>
    <footer>
      <button class="primary" onclick={() => showSettings.set(false)}>{$_('settings.close')}</button>
    </footer>
  </dialog>
{/if}

<style>
  dialog {
    position: fixed;
    inset: 0;
    width: min(560px, calc(100% - 24px));
    max-height: calc(100% - 24px);
    margin: auto;
    padding: 0;
    border: 1px solid var(--border-color);
    border-radius: 10px;
    overflow: auto;
    background: var(--oa-raised);
    color: var(--text-primary);
    box-shadow: var(--oa-shadow-dialog);
  }
  dialog::backdrop { background: var(--oa-overlay); }
  header { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px 8px 16px; border-bottom: 1px solid var(--border-color); touch-action: none; }
  h1, h2 { font: 600 16px var(--font-heading); }
  .close { display: grid; place-items: center; height: 28px; width: 28px; border-radius: 4px; }
  .body { display: grid; grid-template-columns: 158px minmax(0, 1fr); min-height: min(246px, 42vh); }
  nav { padding: 10px 6px; background: var(--bg-surface-alt); border-right: 1px solid var(--border-color); }
  nav button { width: 100%; display: flex; align-items: center; gap: 8px; margin-bottom: 2px; padding: 8px 10px; border-inline-start: 2px solid transparent; border-radius: 4px; text-align: left; font-size: 12px; font-weight: 400; }
  nav button:hover, .close:hover { background: var(--oa-hover); }
  nav button[aria-current] { background: var(--oa-selected); color: var(--text-accent); border-inline-start-color: var(--text-accent); font-weight: 600; }
  section { min-width: 0; padding: 20px; }
  h2 { margin-bottom: 16px; }
  label { display: grid; gap: 6px; color: var(--text-secondary); font-size: 12px; }
  select { width: 100%; max-width: 240px; height: 32px; padding: 0 8px; border: 1px solid var(--border-color); border-radius: 6px; color: var(--text-primary); background: var(--bg-app); font-size: 12px; }
  .about { display: flex; align-items: center; gap: 12px; }
  .about strong { font: 600 14px var(--font-heading); }
  .about p { font-size: 12px; color: var(--text-secondary); }
  .repository { margin-top: 16px; font-size: 12px; color: var(--text-accent); text-decoration: underline; }
  footer { display: flex; justify-content: end; padding: 10px 14px; border-top: 1px solid var(--border-color); }
  .primary { padding: 6px 14px; font-size: 12px; font-weight: 600; background: var(--oa-accent); color: var(--oa-text-on-accent); border-radius: 6px; }
  :global([data-oa-theme^="high-contrast"]) nav button[aria-current] { outline: 2px solid var(--text-accent); outline-offset: -2px; }
  @media (max-width: 460px) { .body { grid-template-columns: 112px minmax(0, 1fr); } nav button { padding-inline: 4px; gap: 4px; } section { padding: 14px; } }
</style>
