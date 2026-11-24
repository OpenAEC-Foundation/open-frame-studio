<script>
  import { _ } from "svelte-i18n";
  import { finishDocsDialog } from "../../lib/docsDialog.js";

  // App keys this dialog by request so each new choice starts with its own defaults.
  export let options;
  let selected = options.files?.[0]?.id || "";
  let name = options.name || "";
  $: opening = options.mode === "open";

  function mountDialog(node) {
    node.showModal();
    node.querySelector("input, select")?.focus();
    return { destroy() { if (node.open) node.close(); } };
  }
  function submit() {
    const value = opening ? selected : name.trim();
    if (value) finishDocsDialog(value);
  }
</script>

<dialog use:mountDialog aria-labelledby="docs-dialog-title"
  oncancel={(event) => { event.preventDefault(); finishDocsDialog(); }}
  onkeydown={(event) => event.stopPropagation()}>
  <div>
    <h2 id="docs-dialog-title">Docs · {$_(opening ? "backstage.open" : "backstage.saveAs")}</h2>
    {#if opening}
      <label>{$_("titlebar.file")}
        <select size={Math.min(8, Math.max(2, options.files.length))} bind:value={selected}
          onkeydown={(event) => { if (event.key === "Enter") { event.preventDefault(); submit(); } }}>
          {#each options.files as file}
            <option value={file.id}>{file.name}</option>
          {/each}
        </select>
      </label>
    {:else}
      <label>{$_("props.name")}<input type="text" bind:value={name} required
        onkeydown={(event) => { if (event.key === "Enter") { event.preventDefault(); submit(); } }} /></label>
    {/if}
    <footer>
      <button type="button" onclick={() => finishDocsDialog()}>{$_("profileEditor.cancel")}</button>
      <button type="button" class="primary" onclick={submit} disabled={!(opening ? selected : name.trim())}>{$_(opening ? "backstage.open" : "dialog.save")}</button>
    </footer>
  </div>
</dialog>

<style>
  dialog { margin: auto; width: min(430px, calc(100% - 24px)); max-height: calc(100% - 24px); padding: 16px;
    border: 1px solid var(--border-color); border-radius: 8px; color: var(--text-primary); background: var(--bg-surface); }
  dialog::backdrop { background: var(--oa-overlay); }
  h2 { margin: 0 0 14px; font: 600 16px var(--font-heading); }
  label { display: grid; gap: 5px; font-size: 13px; }
  select, input, button { padding: 6px 8px; border: 1px solid var(--border-color); border-radius: 4px;
    background: var(--bg-app); color: var(--text-primary); font: inherit; }
  footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 12px; }
  .primary { color: var(--oa-text-on-accent); background: var(--oa-accent); }
</style>
