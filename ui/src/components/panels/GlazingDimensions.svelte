<script>
  import { _, locale } from "svelte-i18n";
  import { circleDiameter } from "../../lib/contour2d.js";
  let { fit, timber, paired = false, leaf = null } = $props();
  let number = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 }));
  function size(rect, contour) {
    const diameter = circleDiameter(contour);
    return diameter != null ? `Ø ${number.format(diameter)}` : `${number.format(rect.width)} × ${number.format(rect.height)}`;
  }
  let arc = $derived(fit.contours?.glass.segments.find(edge => edge.kind === "arc"));
  let sash = $derived(fit.contours?.sashOuter);
  let side = $derived(fit.contours?.sashLeaf ?? leaf);
  let system = $derived(fit.systemSeals);
  let overlap = $derived(system ? system.sashOverlapMm : fit.contours?.sashOverlap);
  let clearance = $derived(system ? system.sashClearanceMm : fit.contours?.sashClearance);
  let minBearing = $derived(fit.beadPosition === "buiten" ? 15 : 13);
  let requiredDepth = $derived(fit.glassThickness + 2 * fit.sealThickness + minBearing);
  let minGlass = $derived(fit.minGlassThickness);
  let maxGlass = $derived(fit.maxGlassThickness);
  let outsideRange = $derived((minGlass != null && fit.glassThickness + 1e-6 < minGlass) ||
    (maxGlass != null && fit.glassThickness > maxGlass + 1e-6));
  let thicknessRange = $derived.by(() => {
    if (minGlass == null) return maxGlass == null ? null : `≤ ${number.format(maxGlass)}`;
    if (maxGlass == null) return `≥ ${number.format(minGlass)}`;
    return `${number.format(minGlass)}–${number.format(maxGlass)}`;
  });
</script>

<div class="detail">
  {#if sash || (system && fit.sashOuter)}
    <h3>{$_(side === "left" ? "sashFit.left" : side === "right" ? "sashFit.right" : "sashFit.title")}</h3>
    <dl class="sash-fit">
      <dt>{$_("sashFit.outerSize")}</dt><dd>{size(fit.sashOuter, sash)} mm</dd>
      {#if overlap != null}
        <dt>{$_("sashFit.overlap")}</dt><dd>{number.format(overlap)} mm</dd>
      {/if}
      {#if clearance != null}
        <dt>{$_("sashFit.clearance")}</dt><dd>{number.format(clearance)} mm</dd>
      {/if}
      {#if fit.contours?.sashSeatDepth != null}
        <dt>{$_("sashFit.seatDepth")}</dt><dd>{number.format(fit.contours.sashSeatDepth)} mm</dd>
      {/if}
      {#if system?.hinged}
        <dt>{$_("system.sealLevels")}</dt><dd>{system.sealingLevels ?? $_("system.unknown")}</dd>
      {/if}
      {#if system?.meetingGapMm != null}
        <dt>{$_("system.meetingGap")}</dt><dd>{number.format(system.meetingGapMm)} mm</dd>
      {/if}
    </dl>
  {/if}
  <h3>{$_(paired ? "glazing.fitPair" : "glazing.fit")}</h3>
  <dl>
    <dt>{$_("glazing.daySize")}</dt><dd>{size(fit.day, fit.contours?.day)} mm</dd>
    <dt>{$_("glazing.rebateSize")}</dt><dd>{size(fit.rebate, fit.contours?.rebate)} mm</dd>
    <dt>{$_("glazing.cutSize")}</dt><dd>{size(fit.glass, fit.contours?.glass)} mm</dd>
    {#if thicknessRange}
      <dt>{$_("glazing.profileThicknessRange")}</dt>
      <dd>{thicknessRange} mm</dd>
    {/if}
    {#if arc && circleDiameter(fit.contours.glass) == null}
      <dt>{$_("glazing.glassArcRadius")}</dt><dd>{number.format(arc.radius)} mm</dd>
    {/if}
    <dt>{$_("glazing.edgeClearance")}</dt><dd>{number.format(fit.contours?.edgeClearance ?? (fit.glass.x - fit.rebate.x))} mm</dd>
  </dl>
  {#if outsideRange}
    <p>{$_("glazing.outsideProfileRange", { values: { thickness: number.format(fit.glassThickness) } })}</p>
  {/if}
  {#if timber && fit.rebateDepth != null && requiredDepth > fit.rebateDepth + 0.001}
    <p>{$_("glazing.depthConflict", { values: { available: number.format(fit.rebateDepth), required: number.format(requiredDepth) } })}</p>
  {/if}
</div>

<style>
  .detail { padding: 10px 12px; border-top: var(--border-default); }
  h3 { margin: 0 0 6px; font-size: 12px; color: var(--text-primary); }
  dl { display: grid; grid-template-columns: 1fr auto; gap: 3px 8px; margin: 0; font-size: 11px; }
  .sash-fit { margin-bottom: 12px; }
  dt { color: var(--text-secondary); }
  dd { margin: 0; text-align: right; font-variant-numeric: tabular-nums; }
  p { margin: 8px 0 0; color: var(--amber); font-size: 12px; }
</style>
