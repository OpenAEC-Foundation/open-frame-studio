<script>
  import { _, locale } from "svelte-i18n";
  import { archJointLabel } from "../../lib/dividerJoinery.js";

  let { joint, mate = "", received = false } = $props();
  let mm = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 }));
  let deg = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 2 }));
</script>

<div class="arch-end">
  <strong>{archJointLabel($_, joint.corner, joint.kind)}</strong>
  <span>{$_("archJoint.shoulderAngle")} {deg.format(received ? joint.receiverAngleDeg : joint.sourceAngleDeg)}°</span>
  {#if joint.tenon}
    {@const tenon = joint.tenon}
    <span>{$_(received ? "joinery.mortiseDepth" : "joinery.tenonLength")} {tenon.zFromInsideMm.length} × {mm.format(received ? tenon.mortiseDepthMm : tenon.tenonLengthMm)} mm</span>
    {#if mate}<span>{$_(received ? "archJoint.tenonOn" : "joinery.receiver")} {mate}</span>{/if}
  {:else if joint.issue}
    <span>{$_("archJoint.requested")} {mm.format(joint.requestedLengthMm)} mm</span>
    <span class="joint-issue">{$_("joinery.issue." + joint.issue)}</span>
  {/if}
  <details>
    <summary>{$_("archJoint.shoulderFace")}</summary>
    {#each joint.shoulderFace as point, i}
      <span>{$_(i === 0 ? "archJoint.outer" : "archJoint.day")} X {mm.format(point[0])} · Y {mm.format(point[1])} mm</span>
    {/each}
    {#if joint.tenon}
      <span>{$_(received ? "joinery.tenonLength" : "joinery.mortiseDepth")} {mm.format(received ? joint.tenon.tenonLengthMm : joint.tenon.mortiseDepthMm)} mm</span>
      <span>{$_("joinery.bottomClearance")} {mm.format(joint.tenon.bottomClearanceMm)} mm</span>
    {/if}
  </details>
</div>

<style>
  .arch-end { display: grid; gap: 2px; font-size: 0.9em; line-height: 1.45; }
  .arch-end span { display: block; }
  summary { cursor: pointer; }
  details { color: var(--text-secondary); }
  .joint-issue { color: var(--amber); }
</style>
