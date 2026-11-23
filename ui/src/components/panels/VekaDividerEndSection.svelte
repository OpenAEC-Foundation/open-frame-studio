<script>
  import { _, locale } from "svelte-i18n";
  import { dividerToolReference } from "../../lib/dividerMachining.js";

  let { end } = $props();
  let path = $derived(dividerToolReference(end));
  let mm = $derived(new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 2 }));
</script>

{#if path}
  <details class="end-tool">
    <summary>{$_("divider.toolReference")}</summary>
    <p>VEKA 102.310 → 101.290</p>
    <svg viewBox="-8 -33 95 43" role="img" aria-label={$_("divider.vekaMdReference")}>
      <path class="baseline" d="M 0 0 H 82 M 0 0 V -26"/>
      <path class="cut" d={path}/>
      {#each [0, 19, 43.5, 61.5, 77.5, 82] as x}
        <path class="tick" d={`M ${x} 0 v 1.5`}/>
        <text x={x} y={x === 77.5 ? 8 : 5} text-anchor="middle">{mm.format(x)}</text>
      {/each}
      {#each [15, 26] as y}
        <path class="tick" d={`M 0 ${-y} h -1.5`}/>
        <text x="-2.2" y={-y + 1} text-anchor="end">{y}</text>
      {/each}
    </svg>
    <p>{$_("divider.vekaMdReference")}</p>
    <small>VEKA · SOFTLINE 82 MD · 5.12 · 06/2012</small>
  </details>
{/if}

<style>
  .end-tool { margin-block: 0.3rem; font-size: 0.85em; }
  summary { cursor: pointer; color: var(--text-secondary); }
  p { margin: 0.3rem 0; white-space: normal; }
  svg { display: block; width: 100%; min-width: 9rem; max-width: 21rem; height: auto; overflow: visible; }
  svg path { fill: none; stroke: currentColor; }
  .baseline { stroke-width: 0.24; stroke-dasharray: 1.25 0.8; }
  .cut { stroke-width: 0.56; stroke-linejoin: round; }
  .tick { stroke-width: 0.22; }
  text { font-size: 2.8px; fill: currentColor; }
  small { color: var(--text-secondary); }
</style>
