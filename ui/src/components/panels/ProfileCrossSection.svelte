<script>
  import { _ } from "svelte-i18n";
  let { crossSection = [], width = 80, height = 60, sponning = null } = $props();

  let points = $derived(crossSection && crossSection.length > 2 ? crossSection : null);

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
      <path d={pathD} fill="var(--bg-surface-alt)" stroke="var(--amber)" stroke-width="0.8" />
      {#if sponning}
        <text
          x={points[0][0] + 2}
          y={points[0][1] + 6}
          font-size="4"
          fill="var(--text-muted)"
          font-family="var(--font-body)"
        >
          SP {sponning.width}x{sponning.depth}
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
