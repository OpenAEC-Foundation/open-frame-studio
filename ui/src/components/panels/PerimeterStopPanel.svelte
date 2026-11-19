<script>
  import { _, locale } from "svelte-i18n";
  import { untrack } from "svelte";
  import { toast } from "../../stores/toast.js";
  import { updatePerimeterStop } from "../../stores/kozijn.js";

  let { frame, geometry, supported } = $props();
  const saved = untrack(() => frame.perimeterStop);
  let name = $state(saved?.profileName ?? "");
  let projection = $state(saved?.projectionMm);
  let thickness = $state(saved?.thicknessMm);
  let busy = $state(false);
  let size = $derived(geometry?.outerBounds);
  let mm = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 }));
  async function save(profile) {
    busy = true;
    try { await updatePerimeterStop(profile); }
    catch (error) { toast.error(String(error)); }
    finally { busy = false; }
  }
</script>

<details open={!!saved}>
  <summary>{$_("system.perimeter")}</summary>
  {#if supported}
    <form onsubmit={(e) => { e.preventDefault(); save({ profileName: name.trim(), projectionMm: projection, thicknessMm: thickness }); }}>
      <label>{$_("system.profile")}<input type="text" bind:value={name} required /></label>
      <label>{$_("system.projection")}<input type="number" bind:value={projection} step="any" required /></label>
      <label>{$_("system.thickness")}<input type="number" bind:value={thickness} step="any" max={frame.frameDepth} required /></label>
      <button disabled={busy || !name.trim() || !(projection > 0 && thickness > 0 && thickness <= frame.frameDepth)}>{$_("system.apply")}</button>
    </form>
  {/if}
  {#if size}
    <p>{$_("system.outerSize")} <strong>{mm.format(size.width)} × {mm.format(size.height)} mm</strong></p>
  {:else if !supported || saved}
    <p>{$_("system.perimeterUnavailable")}</p>
  {/if}
  {#if saved}<button type="button" disabled={busy} onclick={() => save(null)}>{$_("system.remove")}</button>{/if}
</details>

<style>
  details { padding: var(--sp-3) 0; border-bottom: var(--border-default); }
  summary { cursor: pointer; color: var(--amber); font-size: 12px; font-weight: 600; }
  form { display: grid; gap: var(--sp-2); margin-top: var(--sp-2); }
  label, p { font-size: 12px; color: var(--text-secondary); }
  label { display: grid; gap: 3px; }
  input { border: var(--border-default); border-radius: var(--radius-sm); background: var(--bg-surface-alt); color: var(--text-primary); padding: var(--sp-2); width: 100%; }
  button { padding: var(--sp-1) var(--sp-2); }
  strong { color: var(--text-primary); }
</style>
