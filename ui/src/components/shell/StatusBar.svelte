<script>
  import { currentGeometry, currentKozijn, calculateThermal } from "../../stores/kozijn.js";
  import { zoom } from "../../stores/ui.js";
  import { kozijnen } from "../../stores/project.js";
  import { APP_VERSION } from "../../lib/version.js";
  import { _ } from "svelte-i18n";

  let thermalResult = $state(null);
  $effect(() => {
    let current = true;
    thermalResult = null;
    if ($currentKozijn) {
      calculateThermal().then(result => { if (current) thermalResult = result; });
    }
    return () => { current = false; };
  });

  let system = $derived(["pvc", "aluminum"].includes($currentKozijn?.frame.material));
  let missingHardware = $derived(system
    ? ($currentGeometry?.cellRects ?? []).filter(c => c.hardwareLeaves?.some(l => l.issue)).length
    : ($currentKozijn?.cells ?? []).filter(c =>
        !["fixed_glass", "panel", "ventilation"].includes(c.panelType) && !c.hardwareSet && !c.hardware?.length
      ).length);
  let material = $derived($currentKozijn?.frame?.material?.wood
    || ($currentKozijn?.frame?.material === "aluminum" ? $_('status.aluminum')
    : $currentKozijn?.frame?.material === "pvc" ? $_('status.pvc')
    : $currentKozijn?.frame?.material === "wood_aluminum" ? $_('status.woodAlu')
    : $_('status.wood')));
</script>

<footer class="statusbar">
  <div class="selection">
    {#if $currentKozijn}
      <span class="mark">{$currentKozijn.mark}</span>
      <span class="name" title={$currentKozijn.name}>{$currentKozijn.name}</span>
      <span class="size">{$currentKozijn.frame.outerWidth} × {$currentKozijn.frame.outerHeight} mm</span>
      <span class="detail">{material}</span>
      <span class="detail">{$_('status.cells', { values: { count: $currentKozijn.cells.length } })}</span>
    {:else}
      <span class="name">{$_('status.noKozijn')}</span>
    {/if}
  </div>

  {#if missingHardware > 0}
    <span class="warning" title={$_(system ? 'system.fittingIssues' : 'status.missingHardware', { values: { count: missingHardware } })}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M12 9v4m0 4h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"/>
      </svg>
      <span>{$_(system ? 'system.fittingIssues' : 'status.missingHardware', { values: { count: missingHardware } })}</span>
    </span>
  {/if}

  <div class="metrics">
    {#if thermalResult}
      <span class="thermal" class:good={thermalResult.rating === "A" || thermalResult.rating === "B"} class:warn={thermalResult.rating === "C" || thermalResult.rating === "D"}>
        Uw {thermalResult.uwValue}
      </span>
    {/if}
    <span class="count">{$_('status.kozijnen', { values: { count: $kozijnen.length } })}</span>
    <span>{$_('status.zoom', { values: { value: Math.round($zoom * 100) } })}</span>
    <span class="version">v{APP_VERSION}</span>
  </div>
</footer>

<style>
  .statusbar {
    flex: 0 0 var(--oa-shell-statusbar);
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 16px;
    height: var(--oa-shell-statusbar);
    padding: 0 12px;
    border-top: 1px solid var(--border-color);
    background: var(--bg-statusbar);
    color: var(--oa-statusbar-text);
    font: 400 12px var(--font-body);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .selection, .metrics, .warning { display: flex; align-items: center; gap: 10px; }
  .selection { flex: 1 1 auto; min-width: 0; overflow: hidden; }
  .mark { flex: none; padding: 1px 5px; border-radius: 4px; background: var(--oa-selected); color: var(--text-accent); font-size: 11px; font-weight: 600; }
  .name, .warning span { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .name { color: var(--text-primary); }
  .metrics { flex: none; margin-left: auto; }
  .metrics > span + span { border-left: 1px solid var(--border-color); padding-left: 10px; }
  .version, .size { font: 11px var(--font-code); }
  .warning { min-width: 0; gap: 4px; color: var(--warning); }
  .warning svg { flex: none; }
  .good { color: var(--success); }
  .warn { color: var(--warning); }
  @media (max-width: 1000px) { .detail { display: none; } }
  @media (max-width: 760px) { .size, .thermal, .count { display: none; } }
  @media (max-width: 500px) { .version, .warning span { display: none; } .statusbar { gap: 6px; padding-inline: 8px; } }
</style>
