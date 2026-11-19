<script>
  import { _, locale } from "svelte-i18n";
  let { entries = [] } = $props();
  let mm = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 }));
  let count = $derived(entries.reduce((sum, e) => sum + e.tenon.zFromInsideMm.length, 0));
</script>

{#if count}
  <details class="received-mortises">
    <summary>{$_("joinery.received", { values: { n: count } })}</summary>
    <p>{$_("production.frameOrigin")}</p>
    {#each entries as entry}
      {@const t = entry.tenon}
      <div class="mortise-entry">
        <strong>{entry.label} · {entry.endLabel}</strong>
        <div>{$_("joinery.shoulderCenter")} X {mm.format(t.shoulderCenter[0])} · Y {mm.format(t.shoulderCenter[1])} mm</div>
        <div>{$_("joinery.mortiseDepth")} {mm.format(t.mortiseDepthMm)} mm</div>
        <div>{$_("joinery.width")} {mm.format(t.tip
          ? Math.hypot(t.tip[1][0] - t.tip[0][0], t.tip[1][1] - t.tip[0][1])
          : t.crossRangeMm[1] - t.crossRangeMm[0])} mm</div>
        <div>{$_("joinery.bottomClearance")} {mm.format(t.bottomClearanceMm)} mm</div>
        <div>{$_("joinery.depthBands")} {t.zFromInsideMm.map(([near, far]) => mm.format(near) + "–" + mm.format(far)).join(" / ")} mm</div>
      </div>
    {/each}
  </details>
{/if}

<style>
  details { margin-top: 4px; font-size: 11px; line-height: 1.5; }
  summary { cursor: pointer; color: var(--text-primary); }
  p { margin: 4px 0; color: var(--text-secondary); max-width: 32em; white-space: normal; }
  .mortise-entry { padding: 5px 0; }
  .mortise-entry + .mortise-entry { border-top: 1px solid var(--border-color); }
</style>
