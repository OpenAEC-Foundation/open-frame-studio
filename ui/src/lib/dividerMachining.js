// VEKA SOFTLINE 82 MD, Technische Information 06/2012, sheet 5.12.
// x is depth from the exterior. Positive axial setbacks are drawn upwards
// from the unmachined end at the bead groove (GLM). The cubic reproduces
// the R4 transition at detail X from the supplier's vector drawing.
const vekaMdToolProfile = [
  "M 0 -20.839 L 0.716 -20.123",
  "C 1.279 -20.932 2.012 -21.447 2.965 -21.702",
  "L 19 -26 V -15 H 43.5 V -0.9 H 61.5 V -4.1 H 77.5 V 0 H 81.25 L 82 -0.75",
].join(" ");

export function dividerToolReference(end) {
  if (end?.machiningStatus !== "veka_md_glm" || end.sawAllowanceMm !== 25 ||
      end.radiusMm != null || !Number.isFinite(end.angleDeg) || Math.abs(end.angleDeg - 90) > 1e-6 ||
      !end.segments?.length || !end.segments.every(s => s.kind === "line")) return null;
  return vekaMdToolProfile;
}
