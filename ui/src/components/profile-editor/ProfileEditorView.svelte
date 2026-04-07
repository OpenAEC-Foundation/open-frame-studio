<script>
  import { _ } from "svelte-i18n";
  import ProfileLibrary from "./ProfileLibrary.svelte";
  import ProfileCanvas from "./ProfileCanvas.svelte";
  import ProfileParams from "./ProfileParams.svelte";
  import ProfileToolbar from "./ProfileToolbar.svelte";
  import { profileEditor, editorProfile } from "../../stores/profileEditor.js";

  let leftWidth = 260;

  // Start met leeg nieuw profiel
  if (!$editorProfile) {
    profileEditor.newProfile();
  }
</script>

<div class="profile-editor-view">
  <div class="pe-sidebar" style="width:{leftWidth}px">
    <ProfileLibrary />
  </div>

  <div class="pe-canvas-area">
    <ProfileToolbar />
    <ProfileCanvas />
  </div>

  <div class="pe-params">
    <ProfileParams />
  </div>
</div>

<style>
  .profile-editor-view {
    display: flex;
    flex: 1;
    overflow: hidden;
    background: var(--bg-surface);
  }

  .pe-sidebar {
    flex-shrink: 0;
    border-right: var(--border-default);
    overflow-y: auto;
    background: var(--bg-surface);
  }

  .pe-canvas-area {
    flex: 1;
    position: relative;
    overflow: hidden;
    background: var(--editor-bg, var(--bg-surface-alt));
  }

  .pe-params {
    width: 280px;
    flex-shrink: 0;
    border-left: var(--border-default);
    overflow-y: auto;
    background: var(--bg-surface);
  }
</style>
