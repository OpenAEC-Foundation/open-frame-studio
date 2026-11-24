<script>
  import { setContext, tick } from "svelte";
  import { _ } from "svelte-i18n";
  import { activeRibbonTab, showAppMenu } from "../../stores/ui.js";
  import { getSetting, saveSetting } from "../../lib/settings.js";

  let { children } = $props();
  const tabs = {
    home: ["template", "newKozijn", "edit", "import"],
    insert: ["subdivision", "panelType", "curtainWall"],
    ifc: ["exportKozijn", "ifcImport", "validation", "exportProject", "workshop", "export3d", "blender"],
    production: ["prodExport", "cncLabels", "productionPlanning"],
    view: ["undoGroup", "zoom", "theme"],
  };
  const defaults = { minimized: false, hiddenTabs: [], hiddenGroups: [] };
  let preferences = $state({ ...defaults, ...getSetting("ribbon_layout") });
  let visibleTabs = $derived(Object.keys(tabs).filter(id => !preferences.hiddenTabs.includes(id)));
  let widths = $state({});
  let bandWidth = $state(0);
  let floating = $state(false);
  let openGroup = $state(null);
  let root = $state();
  let dialog = $state();
  let menu = $state(null);
  let expanded = $derived(!preferences.minimized || floating);
  let collapsed = $derived.by(() => {
    const groups = (tabs[$activeRibbonTab] || []).filter(id => !preferences.hiddenGroups.includes(id));
    // Each group has 8 px of padding and a 1 px separator. Reserve space for the pin.
    let total = groups.reduce((sum, id) => sum + (widths[id] || 62) + 9, 0) + 30;
    const result = [];
    for (let i = groups.length - 1; i >= 0 && total > bandWidth; i--) {
      const id = groups[i];
      if (widths[id] > 62) {
        total -= widths[id] - 62;
        result.push(id);
      }
    }
    return result;
  });

  $effect(() => {
    if (!visibleTabs.includes($activeRibbonTab) && visibleTabs.length) activeRibbonTab.set(visibleTabs[0]);
  });
  setContext("frame-ribbon", {
    compact: id => collapsed.includes(id),
    visible: id => !preferences.hiddenGroups.includes(id),
    open: () => openGroup,
    toggle: id => { openGroup = openGroup === id ? null : id; },
    measure: (id, width) => { widths[id] = width; },
  });

  function update(change) {
    preferences = { ...preferences, ...change };
    saveSetting("ribbon_layout", $state.snapshot(preferences));
  }

  function dismiss() {
    openGroup = null;
    floating = false;
    menu = null;
  }

  function minimize() {
    update({ minimized: !preferences.minimized });
    dismiss();
  }

  function selectTab(id) {
    activeRibbonTab.set(id);
    openGroup = null;
    menu = null;
    if (preferences.minimized) floating = true;
  }

  function keydown(event) {
    if (dialog?.open) return;
    if (event.ctrlKey && event.key === "F1") {
      event.preventDefault();
      minimize();
    } else if (event.key === "Escape" && (openGroup || floating || menu)) {
      const focus = openGroup ? root.querySelector('[data-group="' + openGroup + '"] .trigger') : root.querySelector(".tab.active");
      if (openGroup) openGroup = null;
      else dismiss();
      focus?.focus();
    }
  }

  function ribbonKeydown(event) {
    // Tab belongs to the ribbon while its controls have focus. The editor uses
    // document-level Tab and Escape shortcuts for selecting frame cells.
    if (event.key === "Tab") event.stopPropagation();
    if (event.key === "Escape" && (openGroup || floating || menu)) {
      event.stopPropagation();
      keydown(event);
    }
  }

  function outside(event) {
    if (!root?.contains(event.target)) dismiss();
    else if (!event.target.closest(".menu")) {
      menu = null;
      if (event.target.closest("[data-group]")?.dataset.group !== openGroup) openGroup = null;
    }
  }

  function command(event) {
    if (event.target.closest(".ribbon-btn, .ribbon-btn-sm") || event.type === "change") {
      if (floating) root.querySelector(".tab.active")?.focus();
      else if (openGroup) root.querySelector('[data-group="' + openGroup + '"] .trigger')?.focus();
      dismiss();
    }
  }

  async function contextMenu(event) {
    event.preventDefault();
    menu = { x: Math.max(4, Math.min(event.clientX, window.innerWidth - 210)), y: Math.max(4, Math.min(event.clientY, window.innerHeight - 82)) };
    await tick();
    root.querySelector(".menu button")?.focus();
  }

  function customize() {
    dismiss();
    dialog.showModal();
  }

  function toggleVisible(list, id) {
    const previous = preferences[list];
    update({ [list]: previous.includes(id) ? previous.filter(value => value !== id) : [...previous, id] });
  }
</script>

<svelte:window onpointerdown={outside} onkeydown={keydown} onresize={() => { openGroup = null; menu = null; }} />

<div role="toolbar" tabindex="-1" class="ribbon" aria-label={$_("ribbon.name")} bind:this={root} oncontextmenu={contextMenu}
  onkeydown={ribbonKeydown}>
  <div class="ribbon-tabs">
    <button class="tab file-tab" onclick={() => { dismiss(); showAppMenu.set(true); }}>{$_("titlebar.file")}</button>
    {#each visibleTabs as id (id)}
      <button class="tab" class:active={$activeRibbonTab === id} aria-pressed={$activeRibbonTab === id}
        onclick={() => selectTab(id)} ondblclick={minimize}>{$_("ribbon." + id)}</button>
    {/each}
  </div>
  {#if expanded}
    <!-- Commands handle pointer and keyboard activation on their own buttons. -->
    <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
    <div class="band" class:floating onclick={command} onchange={command} bind:clientWidth={bandWidth}>
      {@render children()}
    </div>
  {/if}
  <button class="pin" class:down={preferences.minimized} aria-expanded={expanded} onclick={minimize}
    title={$_(preferences.minimized ? "ribbon.pin" : "ribbon.minimize") + " (Ctrl+F1)"}
    aria-label={$_(preferences.minimized ? "ribbon.pin" : "ribbon.minimize")}>
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true"><path d="m3 10 5-5 5 5" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
  </button>
  {#if menu}
    <div class="menu" style:left={menu.x + "px"} style:top={menu.y + "px"}>
      <button onclick={minimize}>{$_(preferences.minimized ? "ribbon.pin" : "ribbon.minimize")}</button>
      <button onclick={customize}>{$_("ribbon.customize")}…</button>
    </div>
  {/if}
</div>

<dialog bind:this={dialog} aria-labelledby="ribbon-options-title" onkeydown={event => {
  // The editor has document-level selection shortcuts, including Tab.
  event.stopPropagation();
}}>
  <div class="dialog-head"><h2 id="ribbon-options-title">{$_("ribbon.customize")}</h2>
    <button class="close" aria-label={$_("ribbon.close")} onclick={() => dialog.close()}>×</button>
  </div>
  <div class="options">
    <label><input type="checkbox" checked={preferences.minimized} onchange={minimize}/> {$_("ribbon.minimize")}</label>
    {#each Object.entries(tabs) as [id, groups]}
      <div class="tab-options">
        <label><input type="checkbox" checked={visibleTabs.includes(id)} disabled={visibleTabs.includes(id) && visibleTabs.length === 1}
          onchange={() => toggleVisible("hiddenTabs", id)}/> {$_("ribbon." + id)}</label>
        <div class="group-options">
          {#each groups as group}
            <label><input type="checkbox" checked={!preferences.hiddenGroups.includes(group)} disabled={!visibleTabs.includes(id)}
              onchange={() => toggleVisible("hiddenGroups", group)}/> {$_("ribbon." + group)}</label>
          {/each}
        </div>
      </div>
    {/each}
  </div>
  <div class="dialog-actions">
    <button onclick={() => update(defaults)}>{$_("ribbon.reset")}</button>
    <button onclick={() => dialog.close()}>{$_("ribbon.close")}</button>
  </div>
</dialog>

<style>
  .ribbon { position: relative; flex: none; min-width: 0; background: var(--bg-ribbon); }
  .ribbon-tabs { display: flex; align-items: center; height: 28px; padding: 0 30px 0 8px; gap: 2px; border-bottom: 1px solid var(--border-color); overflow-x: auto; scrollbar-width: none; }
  .tab { flex: none; height: 100%; padding: 0 12px; border-bottom: 2px solid transparent; color: var(--text-secondary); font-size: 12px; font-weight: 500; }
  .tab:hover, .pin:hover, .menu button:hover { background: var(--oa-hover, var(--bg-surface-alt)); color: var(--text-primary); }
  .tab.active { border-color: var(--oa-accent, var(--amber)); color: var(--text-primary); font-weight: 600; }
  .tab.file-tab { height: 22px; margin-right: 4px; background: var(--oa-accent, var(--amber)); color: var(--oa-text-on-accent, #1c1917); border: 0; border-radius: 6px; font-weight: 600; }
  .tab.file-tab:hover { background: var(--oa-accent-hover, var(--warm-gold)); }
  .band { display: flex; min-height: 72px; padding: 5px 28px 5px 2px; border-bottom: 1px solid var(--border-color); background: var(--bg-ribbon); overflow-x: auto; }
  .band.floating { position: absolute; inset: 100% 0 auto; z-index: 25; box-shadow: var(--shadow-md); }
  .pin { position: absolute; bottom: 6px; right: 3px; z-index: 1; width: 21px; height: 20px; display: grid; place-items: center; border-radius: 3px; color: var(--text-secondary); background: var(--bg-ribbon); }
  .pin.down { bottom: 4px; }
  .pin.down svg { transform: rotate(180deg); }
  .ribbon button:focus-visible { outline-offset: -2px; }
  .menu, dialog { color: var(--text-primary); background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 10px; box-shadow: var(--shadow-lg); }
  .menu { position: fixed; z-index: 35; width: 200px; padding: 4px; display: grid; }
  .menu button { text-align: left; padding: 7px 12px; border-radius: 4px; font-size: 12px; }
  dialog { margin: auto; width: min(440px, calc(100vw - 24px)); padding: 0; }
  dialog::backdrop { background: #0006; }
  .dialog-head, .dialog-actions { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 16px; }
  .dialog-head { border-bottom: 1px solid var(--border-color); }
  .dialog-head h2 { font-size: 16px; }
  .close { font-size: 22px; width: 28px; }
  .options { padding: 12px 16px; max-height: 60vh; overflow-y: auto; }
  .options label { display: flex; align-items: center; gap: 7px; font-size: 12px; }
  .options input { accent-color: var(--oa-accent, var(--amber)); }
  .tab-options { margin-top: 12px; }
  .tab-options > label { font-weight: 600; }
  .group-options { display: grid; padding-left: 20px; gap: 3px; }
  .dialog-actions { border-top: 1px solid var(--border-color); }
  .dialog-actions button { padding: 5px 12px; border: 1px solid var(--border-color); border-radius: 6px; }
  @media (max-width: 600px) { .tab { padding-inline: 8px; } }
</style>
