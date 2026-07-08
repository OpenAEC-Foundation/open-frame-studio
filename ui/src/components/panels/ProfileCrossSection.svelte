<script>
  import { _ } from "svelte-i18n";
  let { crossSection = [], innerWalls = [], width = 80, height = 60, sponning = null } = $props();

  let points = $derived(crossSection && crossSection.length > 2 ? crossSection : null);

  // Binnenstructuur (kamers/staal/isolatoren) — impliciet gesloten polygonen
  let wallPaths = $derived(
    (innerWalls || [])
      .filter((poly) => Array.isArray(poly) && poly.length > 2)
      .map((poly) => `M ${poly.map((p) => `${p[0]} ${p[1]}`).join(" L ")} Z`)
  );

  let viewBox = $derived((() => {
    if (!points) return "0 0 80 60";
    const xs = points.map(p => p[0]);
    const ys = points.map(p => p[1]);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const pad = 2;
    return `${minX - pad} ${minY - pad} ${maxX - minX + 2 * pad} ${maxY - minY + 2 * pad}`;
  })());

  let pathD = $derived(points
    ? `M ${points.map(p => `${p[0]} ${p[1]}`).join(" L ")} Z`
    : null
  );
</script>

<div class="cross-section" style="width: {width}px; height: {height}px;">
  {#if pathD}
    <svg {width} {height} viewBox={viewBox} preserveAspectRatio="xMidYMid meet">
      <!-- Profile outline with fill -->
      <path d={pathD} fill="var(--bg-surface-alt)" stroke="var(--amber)" stroke-width="0.8" />
      <!-- Hatch pattern for material indication -->
      <defs>
        <pattern id="wood-hatch" patternUnits="userSpaceOnUse" width="4" height="4" patternTransform="rotate(45)">
          <line x1="0" y1="0" x2="0" y2="4" stroke="var(--amber)" stroke-width="0.3" opacity="0.15" />
        </pattern>
      </defs>
      <path d={pathD} fill="url(#wood-hatch)" stroke="none" />
      {#each wallPaths as wp}
        <path d={wp} fill="var(--bg-surface)" fill-opacity="0.6" stroke="var(--amber)" stroke-width="0.5" opacity="0.9" />
      {/each}
      {#if sponning}
        {@const xs = points.map(p => p[0])}
        {@const ys = points.map(p => p[1])}
        {@const maxX = Math.max(...xs)}
        {@const maxY = Math.max(...ys)}
        <!-- Sponning dimension annotation -->
        <text
          x={maxX - 1}
          y={maxY - sponning.depth / 2}
          font-size="3.5"
          fill="#22c55e"
          font-family="var(--font-body)"
          font-weight="600"
          text-anchor="end"
        >
          ↕{sponning.depth}
        </text>
      {/if}
    </svg>
  {:else}
    <div class="empty">
      <span>{$_('crossSection.none')}</span>
    </div>
  {/if}
</div>

<style>
  .cross-section {
    border: var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  svg {
    display: block;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .empty span {
    font-size: 9px;
    color: var(--scaffold-gray);
    font-style: italic;
  }
</style>
