<script>
  import { _ } from "svelte-i18n";
  import { cubeSurfaces, projectViewCube } from "./viewCubeGeometry.js";

  let { orientation, onselect, onorbit, onhome, orthographic = false, ontoggleprojection } = $props();
  const groups = ["faces", "edges", "corners"].map((name, index) => ({
    name, entries: cubeSurfaces.filter(entry => entry.labels.length === index + 1),
  }));
  let visible = $derived(orientation ? projectViewCube(orientation) : []);
  let selected = $derived(visible.find(surface => surface.aligned)?.key || "");
  let drag;
  let dragging = $state(false);

  function start(event) {
    if (event.button !== 0 || drag) return;
    const key = event.target.closest("[data-view]")?.dataset.view;
    if (!key) return;
    drag = { id: event.pointerId, key, x: event.clientX, y: event.clientY };
    event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function move(event) {
    if (!drag || event.pointerId !== drag.id) return;
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    if (!dragging && Math.hypot(dx, dy) < 4) return;
    dragging = true;
    drag.x = event.clientX;
    drag.y = event.clientY;
    onorbit(dx, dy);
  }

  function end(event, cancelled = false) {
    if (!drag || event.pointerId !== drag.id) return;
    const key = !cancelled && !dragging && drag.key;
    drag = undefined;
    dragging = false;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    if (key) onselect(key.split(",").map(Number));
  }

  function choose(event) {
    if (event.currentTarget.value) onselect(event.currentTarget.value.split(",").map(Number));
  }
</script>

<div class="view-cube" role="group" aria-label={$_("viewer3d.cube.title")}>
  <!-- The native selector below exposes every surface, including the hidden side, to the keyboard. -->
  <svg class="solid" class:dragging viewBox="0 0 128 116" aria-hidden="true"
    onpointerdown={start} onpointermove={move} onpointerup={end}
    onpointercancel={event => end(event, true)} onlostpointercapture={event => end(event, true)}>
    {#each visible as surface (surface.key)}
      <g>
        <title>{surface.labels.map(label => $_("viewer3d.cube." + label)).join(" · ")}</title>
        <polygon data-view={surface.key} points={surface.points} class:aligned={surface.aligned}
          class:side={surface.direction[0] !== 0} class:cap={surface.direction[1] !== 0}
          class:bevel={surface.labels.length > 1} />
        {#if surface.textTransform}
          <text transform={surface.textTransform} text-anchor="middle" dominant-baseline="central">
            {$_("viewer3d.cube." + surface.labels[0])}
          </text>
        {/if}
      </g>
    {/each}
  </svg>
  <div class="controls">
    <button type="button" aria-label={$_("viewer3d.cube.home")} title={$_("viewer3d.cube.home")} onclick={onhome}>
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m2.5 9 7.5-6.5L17.5 9M5 7.5V17h10V7.5M8.5 17v-5h3v5" /></svg>
    </button>
    <label class="picker">
      <span aria-hidden="true">{$_("viewer3d.cube.views")}</span>
      <svg viewBox="0 0 10 8" aria-hidden="true"><path d="m1.5 2 3.5 3.5L8.5 2" /></svg>
      <select value={selected} onchange={choose} aria-label={$_("viewer3d.cube.views")} title={$_("viewer3d.cube.views")}>
        <option value="" disabled>{$_("viewer3d.cube.views")}</option>
        {#each groups as group (group.name)}
          <optgroup label={$_("viewer3d.cube." + group.name)}>
            {#each group.entries as entry (entry.key)}
              <option value={entry.key}>{entry.labels.map(label => $_("viewer3d.cube." + label)).join(" · ")}</option>
            {/each}
          </optgroup>
        {/each}
      </select>
    </label>
    <button type="button" aria-pressed={orthographic} aria-label={$_("viewer3d.cube.orthographic")}
      title={$_(orthographic ? "viewer3d.cube.toPerspective" : "viewer3d.cube.toOrthographic")} onclick={ontoggleprojection}>
      <svg viewBox="0 0 20 20" aria-hidden="true">
        {#if orthographic}
          <path d="M3 7h10v10H3z M7 3h10v10M3 7l4-4m6 4 4-4m-4 14 4-4" />
        {:else}
          <path d="M2 6h11v12H2z M9 3h8v8M2 6l7-3m4 3 4-3m-4 15 4-7" />
        {/if}
      </svg>
    </button>
  </div>
</div>

<style>
  .view-cube {
    position: absolute; top: 4px; right: 8px; width: 140px;
    max-width: calc(100% - 16px); pointer-events: none; user-select: none;
  }
  .solid { display: block; width: 100%; height: 116px; touch-action: none; overflow: visible; }
  .solid polygon {
    fill: #b4c1cf; stroke: #283240; stroke-width: .85; stroke-linejoin: round;
    pointer-events: auto; cursor: pointer;
  }
  .solid .side { fill: #9eafc1; }
  .solid .cap { fill: #dde6ee; }
  .solid .bevel { fill: #8192a6; }
  .solid .aligned { fill: #edb874; }
  .solid g:hover polygon { fill: #ffd199; }
  .solid.dragging polygon { cursor: grabbing; }
  .solid text {
    fill: #17212c; font: 600 11px var(--font-body, sans-serif); pointer-events: none;
  }
  .controls {
    display: flex; width: max-content; max-width: 100%; align-items: stretch; margin: 0 auto;
    border: 1px solid #566172; border-radius: 5px; overflow: hidden;
    background: #242a35; color: #e3e9ef; pointer-events: auto;
  }
  button, .picker {
    display: flex; align-items: center; justify-content: center; color: inherit;
    font: 500 11px var(--font-body, sans-serif); background: transparent;
    border: 0; height: 25px; cursor: pointer;
  }
  button { width: 26px; padding: 3px 4px; box-sizing: border-box; }
  button svg { width: 16px; height: 16px; }
  .controls svg { fill: none; stroke: currentColor; stroke-width: 1.45; stroke-linejoin: round; stroke-linecap: round; }
  button[aria-pressed="true"] { color: #ffd199; background: #4e4031; }
  button:hover, .picker:hover { background: #394454; }
  button:focus-visible, .picker:focus-within { outline: 2px solid #edb874; outline-offset: -2px; }
  .picker { position: relative; gap: 3px; padding: 0 5px; border-inline: 1px solid #495362; }
  .picker svg { width: 8px; height: 8px; }
  select { position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer; }
</style>
