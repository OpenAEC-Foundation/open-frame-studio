<script>
  let { width = 67, depth = 114, sponningWidth = 12, sponningDepth = 17, sponningPos = "buiten", noseDepth = 5, svgWidth = 200, svgHeight = 280 } = $props();

  // Generate cross-section polygon from parameters
  let points = $derived(generateCrossSection(width, depth, sponningWidth, sponningDepth, sponningPos, noseDepth));
  let pathD = $derived(points.length > 2
    ? `M ${points.map(p => `${p[0]} ${p[1]}`).join(" L ")} Z`
    : ""
  );

  // ViewBox with padding for dimension labels
  let pad = $derived(20);
  let viewBox = $derived(`${-pad} ${-pad} ${width + 2 * pad + 40} ${depth + 2 * pad + 20}`);

  function generateCrossSection(w, d, sw, sd, pos, nd) {
    if (pos === "buiten") {
      // Sponning at outer (bottom) edges — L-shaped notch on both sides
      return [
        [0, 0],
        [w, 0],
        [w, d - sd],
        [w - sw, d - sd],
        [w - sw, d - nd],
        [w - sw, d],
        [sw, d],
        [sw, d - nd],
        [sw, d - sd],
        [0, d - sd],
      ];
    } else if (pos === "binnen") {
      // Sponning at inner (top) edges
      return [
        [sw, 0],
        [w - sw, 0],
        [w - sw, sd],
        [w, sd],
        [w, d],
        [0, d],
        [0, sd],
        [sw, sd],
      ];
    } else {
      // Midden — sponning centered in depth
      const mid = (d - sd) / 2;
      return [
        [0, 0],
        [w, 0],
        [w, mid],
        [w - sw, mid],
        [w - sw, mid + sd],
        [w, mid + sd],
        [w, d],
        [0, d],
        [0, mid + sd],
        [sw, mid + sd],
        [sw, mid],
        [0, mid],
      ];
    }
  }
</script>

<svg width={svgWidth} height={svgHeight} viewBox={viewBox} preserveAspectRatio="xMidYMid meet">
  <!-- Profile shape -->
  <path d={pathD} fill="var(--bg-surface-alt, #E7E5E4)" stroke="var(--amber, #D97706)" stroke-width="1.2" />

  <!-- Sponning zone highlight -->
  {#if sponningPos === "buiten"}
    <rect
      x={sponningWidth} y={depth - sponningDepth}
      width={width - 2 * sponningWidth} height={sponningDepth}
      fill="rgba(59, 130, 246, 0.08)"
      stroke="rgba(59, 130, 246, 0.3)"
      stroke-width="0.5"
      stroke-dasharray="2 2"
    />
  {/if}

  <!-- Dimension: overall width (top) -->
  <line x1={0} y1={-8} x2={width} y2={-8} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <line x1={0} y1={-12} x2={0} y2={-4} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <line x1={width} y1={-12} x2={width} y2={-4} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <text x={width / 2} y={-12} text-anchor="middle" font-size="7" fill="var(--text-muted, #888)" font-family="var(--font-body, sans-serif)">{width}</text>

  <!-- Dimension: overall depth (right) -->
  <line x1={width + 8} y1={0} x2={width + 8} y2={depth} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <line x1={width + 4} y1={0} x2={width + 12} y2={0} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <line x1={width + 4} y1={depth} x2={width + 12} y2={depth} stroke="var(--text-muted, #888)" stroke-width="0.5" />
  <text x={width + 16} y={depth / 2} text-anchor="start" dominant-baseline="central" font-size="7" fill="var(--text-muted, #888)" font-family="var(--font-body, sans-serif)">{depth}</text>

  <!-- Dimension: sponning width (bottom) -->
  {#if sponningPos === "buiten"}
    <line x1={sponningWidth} y1={depth + 6} x2={width - sponningWidth} y2={depth + 6} stroke="rgba(59, 130, 246, 0.6)" stroke-width="0.4" />
    <text x={width / 2} y={depth + 12} text-anchor="middle" font-size="6" fill="rgba(59, 130, 246, 0.8)" font-family="var(--font-body, sans-serif)">SP {sponningWidth}x{sponningDepth}</text>
  {/if}

  <!-- Dimension: sponning depth (right inner) -->
  {#if sponningPos === "buiten"}
    <line x1={width + 22} y1={depth - sponningDepth} x2={width + 22} y2={depth} stroke="rgba(59, 130, 246, 0.6)" stroke-width="0.4" />
    <line x1={width + 18} y1={depth - sponningDepth} x2={width + 26} y2={depth - sponningDepth} stroke="rgba(59, 130, 246, 0.6)" stroke-width="0.4" />
    <line x1={width + 18} y1={depth} x2={width + 26} y2={depth} stroke="rgba(59, 130, 246, 0.6)" stroke-width="0.4" />
    <text x={width + 30} y={depth - sponningDepth / 2} text-anchor="start" dominant-baseline="central" font-size="6" fill="rgba(59, 130, 246, 0.8)" font-family="var(--font-body, sans-serif)">{sponningDepth}</text>
  {/if}
</svg>
