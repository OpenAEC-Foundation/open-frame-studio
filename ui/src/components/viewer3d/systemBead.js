import { rebateChannels, sectionFromPoints } from "../../lib/frame3d.js";

// An intermediate sealing land can project into the glass pocket of an MD
// frame. Use the first glazing channel that accepts the entire glass edge,
// and seat the exterior rubber on the actual ledge at that depth.
export function fixedGlazingSeat(section, cover, thickness, sealDepth) {
  if (!["pvc", "aluminum"].includes(section.def?.material)) return null;
  const glassEdge = section.w - cover;
  for (const [lo, hi] of rebateChannels(section, cover)) {
    if (lo <= 0 || (hi < section.d && hi - lo < thickness + 2 * sealDepth)) continue;
    let tip = null;
    for (let i = 0; i < section.pts.length; i++) {
      const a = section.pts[i], b = section.pts[(i + 1) % section.pts.length];
      if (Math.abs(a[1] - section.minV - lo) > 1e-5 || Math.abs(a[1] - b[1]) > 1e-5) continue;
      const end = Math.max(a[0], b[0]) - section.minU;
      if (end > glassEdge) tip = Math.max(tip ?? end, end);
    }
    if (tip !== null) return { channel: [lo, hi], sealSetback: section.w - tip };
  }
  return null;
}

// Glass-side boundary of a catalogue section, measured at a depth between
// two vertex levels. Only a notch accessible from the glazing pocket can
// receive a foot; an internal air chamber is not a clip groove.
function pocketBands(section) {
  const points = section.pts.map(([u, v]) => [u - section.minU, v - section.minV]);
  const levels = [...new Set(points.map(([, v]) => v))].sort((a, b) => a - b);
  const bands = [];
  for (let i = 1; i < levels.length; i++) {
    const lo = levels[i - 1], hi = levels[i], mid = (lo + hi) / 2;
    let edge = null;
    for (let j = 0; j < points.length; j++) {
      const [u, v] = points[j], [x, y] = points[(j + 1) % points.length];
      if ((v < mid) === (y < mid)) continue;
      const at = z => u + (x - u) * (z - v) / (y - v);
      if (!edge || at(mid) > edge.mid) edge = { mid: at(mid), start: at(lo), end: at(hi) };
    }
    if (!edge) return [];
    const previous = bands.at(-1);
    if (previous && previous.end === edge.start && previous.start === edge.end && edge.start === edge.end) {
      previous.hi = hi;
    } else {
      bands.push({ lo, hi, start: edge.start, end: edge.end });
    }
  }
  return bands;
}

// Schematic clip and hollow cover. The outside envelope at the glass still
// uses the fit's bead dimensions. Wall proportions represent a hollow
// extrusion for inspection; they are not dimensions of a supplier article.
export function systemBeadSection(seats, hostInner, glassSide, rebateHeight, height, width, sealHeight, edgeClearance) {
  const stock = seats[0]?.section;
  if (!["pvc", "aluminum"].includes(stock?.def?.material) ||
      seats.some(s => s.section.id !== stock.id) || !(width > 1 && height > 2)) return null;
  const bands = pocketBands(stock);
  const datum = stock.w - rebateHeight, plane = hostInner - stock.d;
  const near = glassSide - plane, far = near + width;
  // Look for the square-backed pocket on the interior half near the rebate
  // floor. The two neighbouring lips bound the actually accessible opening.
  const slot = bands.find((b, i) =>
    b.lo >= stock.d / 2 && b.start === b.end &&
    Math.abs(b.start - datum) < height / 2 &&
    bands[i - 1] && Math.max(bands[i - 1].start, bands[i - 1].end) > b.start &&
    bands[i + 1] && Math.max(bands[i + 1].start, bands[i + 1].end) > b.end);
  if (!slot) return null;
  const clipZ = (slot.lo + slot.hi) / 2;
  const crossed = bands.filter(b => b.hi > Math.min(near, slot.lo) && b.lo < Math.max(far, slot.hi));
  const wall = Math.max(...crossed.map(b => Math.max(b.start, b.end))) - datum;
  const t = Math.min(width / 8, height / 10, (slot.hi - slot.lo) / 4,
    clipZ < near ? (edgeClearance - Math.max(wall, 0)) / 3 : Infinity);
  const root = Math.max(wall, 0) + t;
  const nose = height - Math.min(sealHeight, height / 3);
  if (!(nose > root + t && t > 0)) return null;
  // Open space between the cover and stock isolates it from the outer shell.
  // Its only solid contact with the receiver is inside this interior pocket.
  const backStart = near + 0.25 * width;
  const outline = [[nose, near], [height, near], [height, near + t],
    [root + (height - root) * 0.65, far], [root, far], [root, backStart]];
  const centre = outline.reduce(([x, z], [u, v]) => [x + u / outline.length, z + v / outline.length], [0, 0]);
  const cavity = outline.map(([x, z]) => [
    centre[0] + 0.65 * (x - centre[0]), centre[1] + 0.65 * (z - centre[1]),
  ]);
  const outlines = outline.map((p, i) => {
    const j = (i + 1) % outline.length;
    return [p, outline[j], cavity[j], cavity[i]];
  });
  const floor = slot.start - datum, half = t / 2;
  // The return can point in either depth direction. A return underneath
  // thick glass must stay within the actual edge clearance.
  const legLo = Math.min(clipZ - half, far), legHi = Math.max(clipZ + half, backStart);
  if (legLo < backStart) outlines.push([[root,legLo],[root+t,legLo],[root+t,backStart],[root,backStart]]);
  if (legHi > far) outlines.push([[root,far],[root+t,far],[root+t,legHi],[root,legHi]]);
  outlines.push([[root,clipZ-half],[root,clipZ+half],[floor+t,clipZ+half],
    [floor+t,slot.hi],[floor,slot.hi],[floor,clipZ-half]]);
  const section = sectionFromPoints(["clip-bead",stock.id,rebateHeight,height,width,near,sealHeight].join("|"), outlines.flat());
  section.outlines = outlines;
  return { section, zInner: plane + section.minV + section.d };
}
