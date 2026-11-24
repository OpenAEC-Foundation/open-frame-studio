<script>
  import { getContext } from "svelte";
  import { _ } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";

  let { id, icon, children } = $props();
  const ribbon = getContext("frame-ribbon");
  let button = $state();
  let naturalWidth = $state(0);
  let left = $state(0);
  let top = $state(0);
  let compact = $derived(ribbon.compact(id));
  let open = $derived(compact && ribbon.open() === id);
  let label = $derived($_("ribbon." + id));

  $effect(() => { if (naturalWidth) ribbon.measure(id, naturalWidth); });

  function toggle() {
    const bounds = button.getBoundingClientRect();
    left = Math.max(4, Math.min(bounds.left, window.innerWidth - naturalWidth - 18));
    top = bounds.bottom + 5;
    ribbon.toggle(id);
  }
</script>

{#if ribbon.visible(id)}
  <div class="group" data-group={id} class:compact>
    {#if compact}
      <button class="trigger" bind:this={button} onclick={toggle} aria-expanded={open}
        aria-controls={"ribbon-group-" + id} title={label}>
        <BlueprintIcon name={icon} size={24} />
        <span>{label}</span>
        <svg width="9" height="5" viewBox="0 0 9 5" aria-hidden="true"><path d="m1 1 3.5 3L8 1" fill="none" stroke="currentColor"/></svg>
      </button>
    {/if}
    <div class="detail" id={"ribbon-group-" + id} class:concealed={compact && !open} class:popup={open}
      style:--popup-left={left + "px"} style:--popup-top={top + "px"} inert={compact && !open}>
      <div class="group-inner" bind:offsetWidth={naturalWidth}>
        <div class="commands">{@render children()}</div>
        <div class="caption">{label}</div>
      </div>
    </div>
  </div>
{/if}

<style>
  .group {
    flex: none;
    display: flex;
    border-inline-end: 1px solid var(--border-color);
    padding-inline: 4px;
  }
  .group:last-child { border-color: transparent; }
  .group-inner { display: flex; flex-direction: column; align-items: stretch; width: max-content; height: 100%; }
  .commands { flex: 1; display: flex; align-items: center; justify-content: center; gap: 4px; }
  .caption {
    text-align: center;
    white-space: nowrap;
    color: var(--text-secondary);
    font: 400 11px/16px var(--font-code);
    letter-spacing: .025em;
    text-transform: uppercase;
  }
  .trigger {
    display: flex;
    align-items: center;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
    width: 62px;
    min-height: 66px;
    padding: 2px;
    border-radius: 6px;
    color: var(--text-primary);
    font: 400 11px/13px var(--font-body);
  }
  .trigger span { max-width: 100%; overflow-wrap: anywhere; }
  .trigger:hover, .trigger[aria-expanded=true] { background: var(--oa-hover, var(--bg-surface-alt)); }
  .trigger:focus-visible { outline-offset: -2px; }
  .concealed { position: fixed; left: 0; top: 0; visibility: hidden; pointer-events: none; }
  .popup {
    position: fixed;
    z-index: 31;
    top: var(--popup-top);
    left: var(--popup-left);
    max-width: calc(100vw - 8px);
    max-height: calc(100vh - var(--popup-top) - 4px);
    overflow: auto;
    padding: 6px;
    background: var(--bg-ribbon);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    box-shadow: var(--shadow-md);
  }
</style>
