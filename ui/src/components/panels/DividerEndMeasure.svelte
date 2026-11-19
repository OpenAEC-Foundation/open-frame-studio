<script>
  import { _, locale } from "svelte-i18n";
  let { end, vertical, angleDeg, radiusMm, receiverName = "" } = $props();
  let mm = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 }));
  let angle = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 2 }));
  let radius = $derived(end?.radiusMm ?? radiusMm);
  let offsets = $derived(end?.edgeOffsetsMm);
  let segments = $derived(end?.segments ?? []);
  let transitions = $derived(segments.filter((s, i) => i > 0 && s.kind !== segments[i - 1].kind));
  let joint = $derived(end?.tenon);
</script>

<span class="end-angle">{angle.format(end?.angleDeg ?? angleDeg)}°</span>
{#if radius != null}
  <small>{$_("production.copeRadius", { values: { r: mm.format(radius) } })}</small>
{/if}
{#each transitions as segment}
  {@const crossAxis = vertical ? 0 : 1}
  {@const distance = Math.abs(segment.from[crossAxis] - segments[0].from[crossAxis])}
  <small class="transition">{$_(segment.kind === "arc" ? "divider.arcFrom" : "divider.flatFrom",
    { values: { n: mm.format(distance), edge: $_(vertical ? "divider.fromLeft" : "divider.fromTop") } })}</small>
{/each}
{#if offsets && (radius != null || Math.max(...offsets) > 0.01)}
  <small class="edge-offsets" title={$_("divider.edgeReference")}>
    {$_(vertical ? "divider.leftRight" : "divider.topBottom")} {mm.format(offsets[0])} / {mm.format(offsets[1])} mm
  </small>
{/if}
{#if joint}
  <small class="tenon-length">{$_("joinery.tenonLength")} {joint.zFromInsideMm.length > 1 ? joint.zFromInsideMm.length + " × " : ""}{mm.format(joint.tenonLengthMm)} mm</small>
  <small class="mortise-depth">{$_("joinery.mortiseDepth")} {mm.format(joint.mortiseDepthMm)} mm</small>
  {#if receiverName}<small class="joint-host">{$_("joinery.receiver")} {receiverName}</small>{/if}
{:else if end?.tenonIssue}
  <small class="joint-issue">{$_("joinery.issue." + end.tenonIssue)}</small>
{/if}

<style>
  small { display: block; font-size: 0.88em; margin-top: 2px; white-space: nowrap; color: var(--text-secondary); }
  .transition, .joint-host, .joint-issue { white-space: normal; }
  .joint-issue { color: var(--amber); }
</style>
