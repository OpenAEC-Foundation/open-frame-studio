function mm(value) {
  return Number.isFinite(value) && value > 0 ? value : null;
}

// Face-plane dimensions and build depths are independent. Keep this conversion
// in step with ofs-core/build.rs for catalog profiles opened by ID.
export function resolveProfileSnapshot(profile) {
  const system = profile.material === "pvc" || profile.material === "aluminum";
  const rebate = profile.sponning;
  const side = rebate?.position || ({ binnensponning: "binnen", buitensponning: "buiten" })[rebate?.type ?? rebate?.sponningType];
  const frame = mm(profile.sightline);
  const vent = mm(profile.sightlineVent);
  const combined = mm(profile.sightlineCombination);
  const overlap = mm(rebate?.opdekWidth) ?? mm(rebate?.opdek_width) ?? mm(profile.overslagHoogte) ?? mm(profile.ventOverlap);
  const levels = profile.sealingLevels;
  return {
    sponningDiepte: system ? null : mm(rebate?.width) ?? mm(profile.glazingRebate),
    sponningHoogte: mm(rebate?.depth),
    beadPosition: ["binnen", "buiten"].includes(side) ? side : null,
    sashOverlap: overlap ?? (system && frame && vent && combined ? mm(frame + vent - combined) : null),
    sashFaceWidth: system && frame && combined && overlap ? mm(combined - frame + overlap) ?? vent : vent,
    sashBuildDepth: mm(profile.ventDepth),
    sashClearance: mm(profile.aanslagSponning?.clearance) ?? mm(profile.falzluft),
    sashSeatDepth: mm(profile.aanslagSponning?.depth),
    meetingGap: mm(profile.stulpNaad),
    sealingLevels: Number.isInteger(levels) && levels >= 1 && levels <= 3 ? levels : null,
    glasinval: mm(profile.glasinval),
    glaslatBreedte: mm(profile.glaslatWidth),
    beadWidthBase: mm(profile.glazingBead?.widthBase),
    glaslatHoogte: mm(profile.glaslatHeight) ?? mm(profile.glazingBeadHeight) ?? mm(profile.glazingBead?.clipHeight),
    aanzichtbreedte: frame,
  };
}
