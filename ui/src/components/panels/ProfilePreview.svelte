<script>
  import { woodCrossSection } from "../../lib/profileContour.js";
  let { width = 67, depth = 114, sponningWidth = 51, sponningDepth = 17, sponningPos = "binnen", svgWidth = 200, svgHeight = 280 } = $props();

  // Generate cross-section polygon from parameters
  let points = $derived(woodCrossSection({ width, depth, sponning: { width: sponningWidth, depth: sponningDepth, position: sponningPos } }));
  let pathD = $derived(points.length > 2
    ? `M ${points.map(p => `${p[0]} ${p[1]}`).join(" L ")} Z`
    : ""
  );

  // ViewBox with padding for dimension labels
  let pad = $derived(20);
  let viewBox = $derived(`${-pad} ${-pad} ${width + 2 * pad + 40} ${depth + 2 * pad + 20}`);

</script>

<svg width={svgWidth} height={svgHeight} viewBox={viewBox} preserveAspectRatio="xMidYMid meet">
  <!-- Profile shape -->
  <path d={pathD} fill="var(--bg-surface-alt, #E7E5E4)" stroke="var(--amber, #D97706)" stroke-width="1.2" />

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

  <text x={width / 2} y={depth + 12} text-anchor="middle" font-size="6"
    fill="var(--text-muted)" font-family="var(--font-body)">
    ↔ {sponningDepth} · ↕ {sponningWidth} mm
  </text>
</svg>
