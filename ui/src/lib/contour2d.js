/** SVG uses the same y-down plane as the core. A complete circle arrives
 * as two arcs since SVG ignores an arc whose endpoints are identical. */
export function segmentPath(edges) {
  if (!edges?.length) return "";
  const commands = [`M${edges[0].from}`];
  for (const edge of edges) {
    commands.push(edge.kind === "arc"
      ? `A${edge.radius} ${edge.radius} 0 ${Math.abs(edge.sweepAngle) > Math.PI ? 1 : 0} ${edge.sweepAngle > 0 ? 1 : 0} ${edge.to}`
      : `L${edge.to}`);
  }
  return commands.join(" ");
}

export function contourPath(contour) {
  const path = segmentPath(contour?.segments);
  return path ? path + " Z" : "";
}

/** Dashed extent of the unmachined blank beyond a qualified straight end. */
export function sawAllowancePath(piece, side) {
  const end = piece?.[side];
  if (!(end?.sawAllowanceMm > 0) || end.segments?.length !== 1 || end.segments[0].kind !== "line") return "";
  const {from, to} = end.segments[0];
  const extension = side === "start" ? -end.sawAllowanceMm : end.sawAllowanceMm;
  const shifted = ([x, y]) => piece.vertical ? [x, y + extension] : [x + extension, y];
  return `M${from} L${shifted(from)} L${shifted(to)} L${to}`;
}

export function openingPath(contour, rect) {
  if (contour) return contourPath(contour);
  return `M${rect.x},${rect.y}h${rect.width}v${rect.height}h${-rect.width}Z`;
}

export function circleDiameter(contour) {
  const edges = contour?.segments;
  if (!edges?.length) return null;
  const first = edges[0];
  if (!edges.every(e => e.kind === "arc" && Math.abs(e.radius - first.radius) < 1e-6
    && Math.hypot(e.center[0] - first.center[0], e.center[1] - first.center[1]) < 1e-6)) return null;
  const sweep = edges.reduce((sum, e) => sum + e.sweepAngle, 0);
  return Math.abs(Math.abs(sweep) - 2 * Math.PI) < 1e-6 ? 2 * first.radius : null;
}
