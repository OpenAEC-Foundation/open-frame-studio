<script>
  import { _ } from "svelte-i18n";
  import { activeWorkspaceView } from "../../stores/ui.js";
  import BlueprintIcon from "./BlueprintIcon.svelte";

  let { views } = $props();

  function navigate(event, index) {
    let next;
    switch (event.key) {
      case "ArrowLeft": next = (index + views.length - 1) % views.length; break;
      case "ArrowRight": next = (index + 1) % views.length; break;
      case "Home": next = 0; break;
      case "End": next = views.length - 1; break;
      default: return;
    }
    event.preventDefault();
    activeWorkspaceView.set(views[next].id);
    event.currentTarget.parentElement.children[next].focus();
  }
</script>

<div class="workspace-tabs" role="tablist" aria-label={$_("workspace.views")}>
  {#each views as view, index (view.id)}
    <button type="button" role="tab" class="ws-tab" class:active={$activeWorkspaceView === view.id}
      id={"tab-" + view.id} aria-controls="workspace-panel" aria-selected={$activeWorkspaceView === view.id}
      tabindex={$activeWorkspaceView === view.id ? 0 : -1}
      onclick={() => activeWorkspaceView.set(view.id)} onkeydown={(event) => navigate(event, index)}>
      <BlueprintIcon name={view.icon} />
      {$_("tabs." + view.id)}
    </button>
  {/each}
</div>

<style>
  .workspace-tabs {
    display: flex;
    flex: none;
    flex-wrap: wrap;
    gap: 0 2px;
    max-height: min(24vh, 150px);
    overflow-y: auto;
    padding: 0 var(--oa-space-sm);
    background: var(--oa-docbar);
    border-bottom: 1px solid var(--oa-border);
  }

  .ws-tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 32px;
    padding: 0 10px;
    border-bottom: 2px solid transparent;
    color: var(--oa-text-secondary);
    font-size: var(--oa-size-body);
    font-weight: var(--oa-weight-medium);
    white-space: nowrap;
  }

  .ws-tab:hover { background: var(--oa-hover); color: var(--oa-text); }
  .ws-tab.active {
    background: var(--oa-panel);
    color: var(--oa-text);
    border-bottom-color: var(--oa-selected-bar);
  }
  .ws-tab:focus-visible { outline-offset: -3px; }
</style>
