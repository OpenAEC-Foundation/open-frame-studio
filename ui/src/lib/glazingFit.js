// The elevation and 3D viewer use the millimetre rectangles from ofs-core.
export function insetOpening(rect, amount) {
  const dx = Math.min(rect.width / 2, amount);
  const dy = Math.min(rect.height / 2, amount);
  return { x: rect.x + dx, y: rect.y + dy, width: rect.width - 2 * dx, height: rect.height - 2 * dy };
}

export function cellFits(cellRect, cell, frame, geometry) {
  if (cellRect.fit) return cellRect.secondFit ? [cellRect.fit, cellRect.secondFit] : [cellRect.fit];
  if (cellRect.openingContour) return [];

  // Older geometry payloads only carry the clear opening. Keep those projects
  // drawable until the matching core has supplied the assembled sections.
  const { rect, vulling } = cellRect;
  const type = vulling?.type ?? cell?.panelType ?? "fixed_glass";
  if (["panel", "paneel", "ventilation", "rooster", "buiten"].includes(type)) return [];
  const fixed = type === "fixed_glass" || type === "glas";
  const isDoor = type === "door" || type === "deur";
  const sashWidth = cell?.sashWidth || (isDoor ? frame.frameWidth : 69);
  const snap = (!fixed && cell?.sashProfileSnapshot) || frame.profileSnapshot;
  const rebateHeight = snap?.sponningHoogte ?? geometry?.sponningHoogte ?? 17;
  const cover = Math.min(rebateHeight, snap?.glasinval ?? Math.max(0, rebateHeight - 5));
  const leaves = [rect];
  if (vulling?.doorKind === "dubbel") {
    const width = Math.max(0, (rect.width - 6) / 2);
    leaves.splice(0, 1, { ...rect, width }, { ...rect, x: rect.x + rect.width - width, width });
  }
  return leaves.map((leaf) => {
    const day = fixed ? leaf : insetOpening(leaf, sashWidth);
    return {
      sashOuter: fixed ? null : leaf, day,
      rebate: insetOpening(day, -rebateHeight), glass: insetOpening(day, -cover),
      beadHeight: cell?.glaslat?.heightMm ?? snap?.glaslatHoogte ?? geometry?.glaslatHoogte ?? rebateHeight,
      beadWidth: cell?.glaslat?.widthMm ?? snap?.glaslatBreedte ?? geometry?.glaslatBreedte ?? 15,
      beadPosition: vulling?.glaslat ?? cell?.glaslat?.position ?? snap?.beadPosition ?? "binnen",
      beadMitered: cell?.glaslat?.mitered ?? true,
    };
  });
}

// A split handle only commits to the core on mouse-up. Keep the fitted
// offsets from each edge constant while the two adjacent openings resize.
export function moveFit(fit, before, after) {
  const move = (r) => {
    if (!r) return r;
    const dx = r.x - before.x, dy = r.y - before.y;
    const width = Math.max(0, after.width + r.width - before.width);
    const height = Math.max(0, after.height + r.height - before.height);
    return { x: after.x + dx, y: after.y + dy, width, height };
  };
  return { ...fit, day: move(fit.day), glass: move(fit.glass), rebate: move(fit.rebate), sashOuter: move(fit.sashOuter) };
}
