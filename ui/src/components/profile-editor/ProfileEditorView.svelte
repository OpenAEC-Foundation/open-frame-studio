<script>
  import { _ } from "svelte-i18n";
  import ProfileLibrary from "./ProfileLibrary.svelte";
  import ProfileCanvas from "./ProfileCanvas.svelte";
  import ProfileParams from "./ProfileParams.svelte";
  import WorkspacePanel from "../shell/WorkspacePanel.svelte";
  import ProfileToolbar from "./ProfileToolbar.svelte";
  import { profileEditor, editorProfile } from "../../stores/profileEditor.js";

  let viewportWidth = window.innerWidth;
  let openPanel = null;
  let showLibrary = true;
  let showProperties = true;
  $: compact = viewportWidth <= 760;
  $: if (!compact) openPanel = null;

  // Start met leeg nieuw profiel
  if (!$editorProfile) {
    profileEditor.newProfile();
  }
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<div class="profile-editor-view">
  <WorkspacePanel side="left" width={260} label={$_("profileEditor.library")} open={compact ? openPanel === "left" : showLibrary}
    ontoggle={() => { if (compact) openPanel = openPanel === "left" ? null : "left"; else showLibrary = !showLibrary; }}>
    <ProfileLibrary />
  </WorkspacePanel>

  <div class="pe-canvas-area">
    <ProfileToolbar />
    <ProfileCanvas />
  </div>

  <WorkspacePanel side="right" width={280} label={$_("workspace.properties")} open={compact ? openPanel === "right" : showProperties}
    ontoggle={() => { if (compact) openPanel = openPanel === "right" ? null : "right"; else showProperties = !showProperties; }}>
    <ProfileParams />
  </WorkspacePanel>
</div>

<style>
  .profile-editor-view {
    display: flex;
    flex: 1;
    min-width: 0;
    position: relative;
    overflow: hidden;
    background: var(--bg-surface);
  }

  .pe-canvas-area {
    flex: 1;
    position: relative;
    overflow: hidden;
    background: var(--editor-bg, var(--bg-surface-alt));
  }

</style>
