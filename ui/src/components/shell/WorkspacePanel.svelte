<script>
  import { _ } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";

  let { side, label, width, open, ontoggle, heading, children } = $props();
  let buttonTitle = $derived($_(open ? "workspace.hidePanel" : "workspace.showPanel", { values: { panel: label } }));
</script>

<aside class="side-panel" class:collapsed={!open} class:right={side === "right"} aria-label={label}
  style:--panel-width={width + "px"}>
  <div class="panel-header">
    {#if open}
      {#if heading}{@render heading()}{:else}<span>{label}</span>{/if}
    {/if}
    <button class="panel-toggle" class:collapsed={!open} type="button" onclick={ontoggle}
      aria-label={buttonTitle} title={buttonTitle} aria-expanded={open}>
      <BlueprintIcon name={"panel." + side} />
      {#if !open}<span>{label}</span>{/if}
    </button>
  </div>
  {#if open}<div class="panel-body">{@render children()}</div>{/if}
</aside>

<style>
  .side-panel {
    flex: none;
    width: min(var(--panel-width), 32%);
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--oa-panel);
    color: var(--oa-text);
    border-right: 1px solid var(--oa-border);
  }
  .side-panel.right { border-right: 0; border-left: 1px solid var(--oa-border); }
  .side-panel.collapsed { width: 28px; }
  .panel-header {
    height: 26px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    padding: 0 3px 0 10px;
    background: var(--oa-frame);
    border-bottom: 1px solid var(--oa-border);
    font-size: var(--oa-size-label);
    font-weight: var(--oa-weight-semibold);
  }
  .panel-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    height: 22px;
    width: 22px;
    border-radius: var(--oa-radius-small);
    color: var(--oa-text-secondary);
  }
  .panel-toggle:hover { background: var(--oa-hover); color: var(--oa-text); }
  .collapsed .panel-header { height: 100%; padding: 0; border: 0; align-items: stretch; }
  .panel-toggle.collapsed {
    height: 100%; width: 100%; flex-direction: column; justify-content: flex-start;
    gap: 8px; padding-top: 6px; border-radius: 0;
  }
  .panel-toggle.collapsed span { writing-mode: vertical-rl; font-size: var(--oa-size-label); }
  .panel-body { min-height: 0; flex: 1; display: flex; flex-direction: column; overflow: auto; }

  @media (max-width: 760px) {
    .side-panel:not(.collapsed) {
      position: absolute;
      z-index: 12;
      inset: 0 auto 0 0;
      width: min(var(--panel-width), calc(100% - 56px));
      box-shadow: var(--oa-shadow-popup);
    }
    .side-panel.right:not(.collapsed) { inset: 0 0 0 auto; }
  }
</style>
