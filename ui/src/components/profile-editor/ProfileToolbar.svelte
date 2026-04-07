<script>
  import { _ } from "svelte-i18n";
  import { profileEditor, editorTool, editorSnap, editorSelectedVertex, editorIsDirty } from "../../stores/profileEditor.js";

  function handleKeyDown(e) {
    if (e.key === "Delete" || e.key === "Backspace") {
      if ($editorSelectedVertex >= 0) {
        profileEditor.removeVertex($editorSelectedVertex);
      }
    }
    if (e.ctrlKey && e.key === "z") {
      e.preventDefault();
      profileEditor.undo();
    }
    if (e.ctrlKey && e.key === "y") {
      e.preventDefault();
      profileEditor.redo();
    }
  }

  // Global keyboard shortcuts for profile editor
  import { onMount, onDestroy } from "svelte";
  let cleanup;
  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    cleanup = () => window.removeEventListener("keydown", handleKeyDown);
  });
  onDestroy(() => cleanup?.());
</script>

<div class="profile-toolbar">
  <div class="tool-group">
    <button
      class="tool-btn"
      class:active={$editorTool === "select"}
      onclick={() => profileEditor.setTool("select")}
      title={$_("profileEditor.toolSelect") || "Selecteren (V)"}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/>
      </svg>
    </button>
    <button
      class="tool-btn"
      class:active={$editorTool === "delete"}
      onclick={() => profileEditor.setTool("delete")}
      title={$_("profileEditor.toolDelete") || "Punt verwijderen"}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="8" y1="12" x2="16" y2="12"/>
      </svg>
    </button>
  </div>

  <div class="separator"></div>

  <div class="tool-group">
    <button class="tool-btn" onclick={() => profileEditor.mirrorH()}
      title={$_("profileEditor.mirrorH") || "Spiegel horizontaal"}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="3" x2="12" y2="21" stroke-dasharray="3 2"/>
        <polyline points="7 8 3 12 7 16"/>
        <polyline points="17 8 21 12 17 16"/>
      </svg>
    </button>
    <button class="tool-btn" onclick={() => profileEditor.mirrorV()}
      title={$_("profileEditor.mirrorV") || "Spiegel verticaal"}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="3" y1="12" x2="21" y2="12" stroke-dasharray="3 2"/>
        <polyline points="8 7 12 3 16 7"/>
        <polyline points="8 17 12 21 16 17"/>
      </svg>
    </button>
  </div>

  <div class="separator"></div>

  <div class="tool-group">
    <button
      class="tool-btn"
      class:active={$editorSnap}
      onclick={() => profileEditor.toggleSnap()}
      title={$_("profileEditor.snapGrid") || "Snap aan raster"}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="7"/>
        <rect x="14" y="3" width="7" height="7"/>
        <rect x="3" y="14" width="7" height="7"/>
        <rect x="14" y="14" width="7" height="7"/>
      </svg>
    </button>
  </div>

  <div class="separator"></div>

  <div class="tool-group">
    <button class="tool-btn" onclick={() => profileEditor.undo()}
      title={$_("profileEditor.undo") || "Ongedaan maken (Ctrl+Z)"}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="1 4 1 10 7 10"/>
        <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
      </svg>
    </button>
    <button class="tool-btn" onclick={() => profileEditor.redo()}
      title={$_("profileEditor.redo") || "Opnieuw (Ctrl+Y)"}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10"/>
        <path d="M20.49 15a9 9 0 1 1-2.13-9.36L23 10"/>
      </svg>
    </button>
  </div>

  <div class="spacer"></div>

  <div class="hint">
    {#if $editorTool === "select"}
      {$_("profileEditor.hintSelect") || "Klik punt om te selecteren. Sleep om te verplaatsen. Dubbelklik lijn voor nieuw punt."}
    {:else if $editorTool === "delete"}
      {$_("profileEditor.hintDelete") || "Klik op een punt om het te verwijderen."}
    {/if}
  </div>
</div>

<style>
  .profile-toolbar {
    position: absolute;
    top: 8px;
    left: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-surface);
    border: var(--border-default);
    border-radius: var(--radius-md);
    padding: 4px 8px;
    box-shadow: var(--shadow-md);
    z-index: 10;
  }

  .tool-group {
    display: flex;
    gap: 2px;
  }

  .tool-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.15s;
  }

  .tool-btn:hover {
    background: var(--bg-surface-alt);
    color: var(--text-primary);
  }

  .tool-btn.active {
    background: var(--amber);
    color: #fff;
  }

  .separator {
    width: 1px;
    height: 20px;
    background: var(--border-color);
    margin: 0 4px;
  }

  .spacer {
    flex: 1;
  }

  .hint {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
  }
</style>
