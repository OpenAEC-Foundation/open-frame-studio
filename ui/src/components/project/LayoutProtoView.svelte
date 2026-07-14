<script>
  import { onMount } from "svelte";
  import {
    layoutToRects, vullingLabel, splitLeaf, setVulling, countLeaves,
    melkmeisje1, melkmeisje2, deurMetBovenlicht, freeGrid, gridToLayout,
    findNode, setSplitChildSizes, mergeAt, patchVulling, ensureIds,
    RAAM_OPENTYPES, DEUR_SOORTEN,
  } from "../../lib/layout.js";
  import { currentKozijn, setKozijnLayout } from "../../stores/kozijn.js";

  // Proof-of-concept for the free subdivision model (recursive split tree).
  // Fully local — pure JS/SVG, no backend/wasm.

  // Draw on the REAL kozijn (outer size + wood width) so "Toepassen" is 1:1
  // with the main editor — the fixed 1800×2100/90mm sandbox only applies when
  // no kozijn is selected. Tree child sizes are weights, so designing on the
  // wrong canvas silently rescaled every vak on apply.
  let OUTER = $derived({
    w: $currentKozijn?.frame?.outerWidth || 1800,
    h: $currentKozijn?.frame?.outerHeight || 2100,
  });
  let FRAME = $derived($currentKozijn?.frame?.frameWidth || 90);
  let DIVIDER = $derived($currentKozijn?.frame?.frameWidth || 90);

  let tree = $state(melkmeisje1());
  let selectedId = $state(null);

  // If the selected kozijn already has a layout, start from it (ensureIds is
  // defensive: old .ofs files / stale wasm bundles may strip the node ids).
  onMount(() => { if ($currentKozijn?.layout) tree = ensureIds($currentKozijn.layout); });
  function applyToKozijn() { if ($currentKozijn) setKozijnLayout(tree); }
  function loadFromKozijn() {
    const k = $currentKozijn;
    if (!k) return;
    tree = k.layout ? ensureIds(k.layout) : gridToLayout(k); // matrix → split tree when no layout yet
    selectedId = null;
  }

  let inner = $derived({ x: FRAME, y: FRAME, width: OUTER.w - 2 * FRAME, height: OUTER.h - 2 * FRAME });
  let geom = $derived(layoutToRects(tree, inner, DIVIDER));
  let vakken = $derived(countLeaves(tree));
  let selectedLeaf = $derived(geom.leaves.find((l) => l.node.id === selectedId)?.node || null);

  const FILL = {
    glas: "rgba(136,204,221,0.30)",
    raam: "rgba(136,204,221,0.22)",
    deur: "#b98b5e",
    paneel: "#c9c2b2",
    rooster: "#9aa0a8",
  };

  function setTemplate(fn) { tree = fn(); selectedId = null; }
  function doSplit(dir) { if (selectedId) tree = splitLeaf(tree, selectedId, dir); }
  function mergeSelected() { if (selectedId) tree = mergeAt(tree, selectedId); }

  // ── Drag-to-resize dividers ────────────────────────────────────
  let svgEl = $state(null);
  let drag = $state(null);

  function startDrag(d, e) {
    const split = findNode(tree, d.splitId);
    if (!split || split.kind !== "split") return;
    const r = svgEl?.getBoundingClientRect();
    const scale = r ? Math.min(r.width / OUTER.w, r.height / OUTER.h) : 1;
    drag = {
      splitId: d.splitId, childIndex: d.childIndex, axis: d.direction,
      avail: d.avail, sizeSum: d.sizeSum, scale,
      startX: e.clientX, startY: e.clientY,
      origI: split.children[d.childIndex].size,
      origNext: split.children[d.childIndex + 1].size,
    };
    e.preventDefault();
  }
  function onPointerMove(e) {
    if (!drag) return;
    const deltaPx = drag.axis === "v" ? e.clientX - drag.startX : e.clientY - drag.startY;
    const deltaMm = deltaPx / (drag.scale || 1);
    let deltaSize = deltaMm * drag.sizeSum / Math.max(1, drag.avail);
    const min = 0.05 * drag.sizeSum;
    deltaSize = Math.max(-(drag.origI - min), Math.min(drag.origNext - min, deltaSize));
    tree = setSplitChildSizes(tree, drag.splitId, drag.childIndex, drag.origI + deltaSize, drag.origNext - deltaSize);
  }
  function onPointerUp() { drag = null; }
  function changeVulling(e) {
    if (!selectedLeaf) return;
    const val = e.target.value;
    // Carry compatible extras across a type switch — resetting glaslat/
    // scharnier/draairichting/beslag on every switch loses the user's input.
    const prev = selectedLeaf.vulling || {};
    const glazed = (t) => t === "glas" || t === "raam";
    const operable = (t) => t === "raam" || t === "deur";
    const keep = (t) => ({
      ...(glazed(t) && prev.glaslat != null ? { glaslat: prev.glaslat } : {}),
      ...(operable(t) && prev.hingeSide != null ? { hingeSide: prev.hingeSide } : {}),
      ...(operable(t) && prev.opensOutward != null ? { opensOutward: prev.opensOutward } : {}),
      ...(operable(t) && prev.hardware != null ? { hardware: prev.hardware } : {}),
    });
    let v;
    if (val.startsWith("raam:")) v = { ...keep("raam"), type: "raam", openType: val.slice(5) };
    else if (val.startsWith("deur:")) v = { ...keep("deur"), type: "deur", doorKind: val.slice(5) };
    else v = { ...keep(val), type: val };
    tree = setVulling(tree, selectedLeaf.id, v);
  }

  function patch(p) { if (selectedLeaf) tree = patchVulling(tree, selectedLeaf.id, p); }
  const isGlazed = (v) => v && (v.type === "glas" || v.type === "raam");
  const isOperable = (v) => v && (v.type === "raam" || v.type === "deur");

  // Centre of a rect for label placement
  const cx = (r) => r.x + r.width / 2;
  const cy = (r) => r.y + r.height / 2;

  // EN 12519-style opening triangle (open V), apex on the handle side.
  function openingTriangle(l) {
    const { x, y, width: w, height: h } = l.rect;
    const left = (l.node.vulling.hingeSide || "links") === "links";
    return left
      ? `M${x + w * 0.14},${y + h * 0.14} L${x + w * 0.86},${y + h / 2} L${x + w * 0.14},${y + h * 0.86}`
      : `M${x + w * 0.86},${y + h * 0.14} L${x + w * 0.14},${y + h / 2} L${x + w * 0.86},${y + h * 0.86}`;
  }
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="view">
  <div class="toolbar">
    <h2>Vrije indeling (proto) — vakvullingen &amp; splitsboom</h2>
    <span class="hint-inline">{vakken} vak(ken)</span>
  </div>

  <div class="body">
    <div class="controls">
      <div class="group">
        <span class="g-label">Kozijn</span>
        {#if $currentKozijn}
          <button onclick={applyToKozijn}>Toepassen op {$currentKozijn.mark}</button>
          <button onclick={loadFromKozijn}>Laad van kozijn (matrix→boom)</button>
        {:else}
          <span class="hint">Selecteer een kozijn om de indeling toe te passen.</span>
        {/if}
      </div>

      <div class="group">
        <span class="g-label">Sjablonen</span>
        <button onclick={() => setTemplate(melkmeisje1)}>Melkmeisje (1-zijdig)</button>
        <button onclick={() => setTemplate(melkmeisje2)}>Melkmeisje (2-zijdig)</button>
        <button onclick={() => setTemplate(deurMetBovenlicht)}>Deur + bovenlicht</button>
        <button onclick={() => setTemplate(() => freeGrid(9, 4))}>Vrij raster 9×4</button>
        <button onclick={() => setTemplate(() => freeGrid(3, 2))}>Raster 3×2</button>
      </div>

      <div class="group">
        <span class="g-label">Geselecteerd vak</span>
        {#if selectedLeaf}
          <button onclick={() => doSplit("row")}>Splits ⟷ (tussenstijl)</button>
          <button onclick={() => doSplit("column")}>Splits ↕ (tussendorpel)</button>
          <button onclick={mergeSelected}>Samenvoegen (splitsing opheffen)</button>
          <label class="vsel">
            <span>Vulling</span>
            <select onchange={changeVulling} value={selectedLeaf.vulling.type === "raam" ? `raam:${selectedLeaf.vulling.openType}` : selectedLeaf.vulling.type === "deur" ? `deur:${selectedLeaf.vulling.doorKind}` : selectedLeaf.vulling.type}>
              <option value="glas">Vast glas</option>
              <optgroup label="Raam">
                {#each Object.entries(RAAM_OPENTYPES) as [k, lbl]}
                  <option value={`raam:${k}`}>{lbl}</option>
                {/each}
              </optgroup>
              <optgroup label="Deur">
                {#each Object.entries(DEUR_SOORTEN) as [k, lbl]}
                  <option value={`deur:${k}`}>{lbl}</option>
                {/each}
              </optgroup>
              <option value="paneel">Paneel</option>
              <option value="rooster">Ventilatie</option>
              <option value="buiten">Buiten kozijn (trapt)</option>
            </select>
          </label>
          {#if isGlazed(selectedLeaf.vulling)}
            <label class="vsel"><span>Glaslat</span>
              <select value={selectedLeaf.vulling.glaslat || "geen"} onchange={(e) => patch({ glaslat: e.target.value === "geen" ? null : e.target.value })}>
                <option value="geen">Geen</option>
                <option value="binnen">Binnen</option>
                <option value="buiten">Buiten</option>
              </select>
            </label>
          {/if}
          {#if isOperable(selectedLeaf.vulling)}
            <label class="vsel"><span>Scharnierzijde</span>
              <select value={selectedLeaf.vulling.hingeSide || "links"} onchange={(e) => patch({ hingeSide: e.target.value })}>
                <option value="links">Links</option>
                <option value="rechts">Rechts</option>
              </select>
            </label>
            <label class="vsel"><span>Draairichting</span>
              <select value={selectedLeaf.vulling.opensOutward ? "buiten" : "binnen"} onchange={(e) => patch({ opensOutward: e.target.value === "buiten" })}>
                <option value="binnen">Binnendraaiend</option>
                <option value="buiten">Buitendraaiend</option>
              </select>
            </label>
            <label class="vcheck"><input type="checkbox" checked={!!selectedLeaf.vulling.hardware} onchange={(e) => patch({ hardware: e.target.checked })} /> Hang- en sluitwerk</label>
          {/if}
        {:else}
          <span class="hint">Klik een vak in de tekening om te splitsen of een vulling te kiezen.</span>
        {/if}
      </div>
    </div>

    <div class="canvas-wrap">
      <svg viewBox="0 0 {OUTER.w} {OUTER.h}" class="frame-svg" bind:this={svgEl}>
        <defs>
          <!-- Metselwerk (borstwering) — running bond, mm user-space. -->
          <pattern id="proto-brick" width="300" height="150" patternUnits="userSpaceOnUse">
            <rect width="300" height="150" fill="#9c876d" />
            <g stroke="#cabfa8" stroke-width="10" stroke-linecap="square">
              <line x1="0" y1="0" x2="300" y2="0" />
              <line x1="0" y1="75" x2="300" y2="75" />
              <line x1="0" y1="0" x2="0" y2="75" />
              <line x1="150" y1="75" x2="150" y2="150" />
            </g>
          </pattern>
        </defs>
        <!-- wall / outside the kozijn -->
        <rect x="0" y="0" width={OUTER.w} height={OUTER.h} fill="#2a2a38" />

        <!-- frame mass: each real vak expanded by the frame width; 'buiten' = no frame,
             so the kozijn outline steps (melkmeisje). -->
        {#each geom.leaves as l (l.node.id + "-f")}
          {#if l.node.vulling.type !== "buiten"}
            <rect x={l.rect.x - FRAME} y={l.rect.y - FRAME} width={l.rect.width + 2 * FRAME} height={l.rect.height + 2 * FRAME} fill="#6b5b3e" />
          {/if}
        {/each}

        <!-- vak fills + details -->
        {#each geom.leaves as l (l.node.id)}
          <g class="vak" role="button" tabindex="0"
             onclick={() => (selectedId = l.node.id)} onkeydown={(e) => e.key === "Enter" && (selectedId = l.node.id)}>
            {#if l.node.vulling.type === "buiten"}
              <rect x={l.rect.x} y={l.rect.y} width={l.rect.width} height={l.rect.height}
                    fill="url(#proto-brick)" stroke="#3a3a52" stroke-width="3" />
              <text x={cx(l.rect)} y={cy(l.rect)} text-anchor="middle" dominant-baseline="middle"
                    font-size="54" fill="#f5efe3" font-weight="700">Borstwering</text>
            {:else}
              <rect x={l.rect.x} y={l.rect.y} width={l.rect.width} height={l.rect.height}
                    fill={FILL[l.node.vulling.type] || "#888"} stroke="#3a3a52" stroke-width="3" />
              {#if l.node.vulling.type === "raam" || l.node.vulling.type === "deur"}
                <rect x={l.rect.x + 28} y={l.rect.y + 28} width={Math.max(0, l.rect.width - 56)} height={Math.max(0, l.rect.height - 56)}
                      fill="none" stroke="#caa06a" stroke-width="20" />
                <path d={openingTriangle(l)} fill="none" stroke="#caa06a" stroke-width="5" opacity="0.75" />
              {/if}
              {#if l.node.vulling.glaslat}
                <rect x={l.rect.x + 55} y={l.rect.y + 55} width={Math.max(0, l.rect.width - 110)} height={Math.max(0, l.rect.height - 110)}
                      fill="none" stroke="#9ec5d4" stroke-width="6" stroke-dasharray="16 12" />
              {/if}
              <text x={cx(l.rect)} y={cy(l.rect)} text-anchor="middle" dominant-baseline="middle"
                    font-size="74" fill="#e8e8f0">{vullingLabel(l.node.vulling)}</text>
            {/if}
            {#if l.node.id === selectedId}
              <rect x={l.rect.x + 4} y={l.rect.y + 4} width={Math.max(0, l.rect.width - 8)} height={Math.max(0, l.rect.height - 8)}
                    fill="none" stroke="#D97706" stroke-width="12" />
            {/if}
          </g>
        {/each}

        <!-- drag handles on the dividers (tussenstijl/-dorpel) -->
        {#each geom.dividers as d, i (i)}
          <rect x={d.rect.x} y={d.rect.y} width={d.rect.width} height={d.rect.height}
                class="divider-handle {d.direction}" fill="transparent"
                onpointerdown={(e) => startDrag(d, e)} role="separator" aria-label="versleep deellijn" tabindex="-1" />
        {/each}
      </svg>
    </div>
  </div>
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .hint-inline { font-size: 11px; color: var(--text-muted); }
  .body { flex: 1; display: flex; gap: var(--sp-4); overflow: hidden; }
  .controls { width: 280px; flex-shrink: 0; display: flex; flex-direction: column; gap: var(--sp-4); overflow: auto; }
  .group { display: flex; flex-direction: column; gap: var(--sp-2); }
  .g-label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-muted); }
  .group button { padding: var(--sp-2) var(--sp-3); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; text-align: left; }
  .group button:hover { border-color: var(--amber); }
  .vsel { display: flex; flex-direction: column; gap: 2px; font-size: 11px; color: var(--text-muted); }
  .vsel select { padding: var(--sp-1) var(--sp-2); background: var(--bg-surface-alt); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; }
  .vcheck { display: flex; align-items: center; gap: var(--sp-2); font-size: 12px; color: var(--text-primary); }
  .hint { color: var(--text-muted); font-size: 12px; font-style: italic; }
  .canvas-wrap { flex: 1; display: flex; align-items: center; justify-content: center; background: var(--bg-app); border-radius: var(--radius-sm); overflow: hidden; }
  .frame-svg { max-height: 100%; max-width: 100%; height: 100%; }
  .vak { cursor: default; }
  .vak:hover rect:first-child { fill-opacity: 0.5; }
  .divider-handle.v { cursor: col-resize; }
  .divider-handle.h { cursor: row-resize; }
  .divider-handle:hover { fill: rgba(217, 119, 6, 0.25) !important; }
</style>
