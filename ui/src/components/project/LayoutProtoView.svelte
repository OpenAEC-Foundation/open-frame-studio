<script>
  import {
    layoutToRects, vullingLabel, splitLeaf, setVulling, countLeaves,
    melkmeisje1, melkmeisje2, deurMetBovenlicht, freeGrid,
    RAAM_OPENTYPES, DEUR_SOORTEN,
  } from "../../lib/layout.js";

  // Proof-of-concept for the free subdivision model (recursive split tree).
  // Fully local — pure JS/SVG, no backend/wasm.

  const OUTER = { w: 1800, h: 2100 };
  const FRAME = 90;
  const DIVIDER = 60;

  let tree = $state(melkmeisje1());
  let selectedId = $state(null);

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
  function changeVulling(e) {
    if (!selectedLeaf) return;
    const val = e.target.value;
    let v;
    if (val.startsWith("raam:")) v = { type: "raam", openType: val.slice(5) };
    else if (val.startsWith("deur:")) v = { type: "deur", doorKind: val.slice(5) };
    else v = { type: val };
    tree = setVulling(tree, selectedLeaf.id, v);
  }

  // Centre of a rect for label placement
  const cx = (r) => r.x + r.width / 2;
  const cy = (r) => r.y + r.height / 2;
</script>

<div class="view">
  <div class="toolbar">
    <h2>Vrije indeling (proto) — vakvullingen &amp; splitsboom</h2>
    <span class="hint-inline">{vakken} vak(ken)</span>
  </div>

  <div class="body">
    <div class="controls">
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
            </select>
          </label>
        {:else}
          <span class="hint">Klik een vak in de tekening om te splitsen of een vulling te kiezen.</span>
        {/if}
      </div>
    </div>

    <div class="canvas-wrap">
      <svg viewBox="0 0 {OUTER.w} {OUTER.h}" class="frame-svg">
        <!-- outer frame -->
        <rect x="0" y="0" width={OUTER.w} height={OUTER.h} fill="#6b5b3e" />
        <rect x={FRAME} y={FRAME} width={OUTER.w - 2 * FRAME} height={OUTER.h - 2 * FRAME} fill="#1a1a2e" />

        <!-- leaf vakken -->
        {#each geom.leaves as l (l.node.id)}
          <g class="vak" class:sel={l.node.id === selectedId} role="button" tabindex="0"
             onclick={() => (selectedId = l.node.id)} onkeydown={(e) => e.key === "Enter" && (selectedId = l.node.id)}>
            <rect x={l.rect.x} y={l.rect.y} width={l.rect.width} height={l.rect.height}
                  fill={FILL[l.node.vulling.type] || "#888"} stroke="#3a3a52" stroke-width="4" />
            {#if l.node.vulling.type === "raam" || l.node.vulling.type === "deur"}
              <!-- sash border -->
              <rect x={l.rect.x + 30} y={l.rect.y + 30} width={Math.max(0, l.rect.width - 60)} height={Math.max(0, l.rect.height - 60)}
                    fill="none" stroke="#caa06a" stroke-width="22" />
            {/if}
            {#if l.node.id === selectedId}
              <rect x={l.rect.x + 4} y={l.rect.y + 4} width={Math.max(0, l.rect.width - 8)} height={Math.max(0, l.rect.height - 8)}
                    fill="none" stroke="#D97706" stroke-width="14" />
            {/if}
            <text x={cx(l.rect)} y={cy(l.rect)} text-anchor="middle" dominant-baseline="middle"
                  font-size="78" fill="#e8e8f0">{vullingLabel(l.node.vulling)}</text>
          </g>
        {/each}

        <!-- dividers (tussenstijlen / tussendorpels) -->
        {#each geom.dividers as d}
          <rect x={d.rect.x} y={d.rect.y} width={d.rect.width} height={d.rect.height} fill="#6b5b3e" />
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
  .hint { color: var(--text-muted); font-size: 12px; font-style: italic; }
  .canvas-wrap { flex: 1; display: flex; align-items: center; justify-content: center; background: var(--bg-app); border-radius: var(--radius-sm); overflow: hidden; }
  .frame-svg { max-height: 100%; max-width: 100%; height: 100%; }
  .vak { cursor: default; }
  .vak:hover rect:first-child { fill-opacity: 0.5; }
</style>
