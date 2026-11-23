/**
 * Shared label lookups for panel types, member types, gasket types.
 * Uses i18n keys — call with the `$_` function from svelte-i18n.
 */

export const PANEL_TYPE_KEYS = {
  fixed_glass: "panel.fixedGlass",
  turn_tilt: "panel.turnTilt",
  turn: "panel.turn",
  tilt: "panel.tilt",
  top_hung: "panel.topHung",
  bottom_hung: "panel.bottomHung",
  sliding: "panel.sliding",
  lift_slide: "panel.liftSlide",
  pivot: "panel.pivot",
  door: "panel.door",
  panel: "panel.panel",
  ventilation: "panel.ventilation",
};

const MEMBER_TYPE_KEYS = {
  perimeter_stop: "system.strip",
  frame_arc: "member.arc",
  sash_arc: "member.sashArc",
  frame_top: "member.topRail",
  frame_bottom: "member.bottomRail",
  frame_left: "member.leftStile",
  frame_right: "member.rightStile",
  divider_v: "member.dividerV",
  divider_h: "member.dividerH",
  sash_top: "member.sashTop",
  sash_bottom: "member.sashBottom",
  sash_left: "member.sashLeft",
  sash_right: "member.sashRight",
};

/** A circle uses the top and bottom profile slots for its two half-rings. */
export function timberMemberType(piece) {
  if (piece.profileSide) return `frame_${piece.profileSide}`;
  if (piece.role === "arc") return piece.index === 0 ? "frame_top" : "frame_bottom";
  return `frame_${piece.role}`;
}

const GASKET_TYPE_KEYS = {
  glazing_inner: "production.gasketInner",
  glazing_outer: "production.gasketOuter",
  sash_seal: "production.gasketSash",
  frame_seal: "production.gasketFrame",
  closing_inner: "production.closingInner",
  closing_centre: "production.closingCentre",
  closing_outer: "production.closingOuter",
  closing_single: "production.closingSingle",
  closing_unspecified: "production.closingUnspecified",
};

/**
 * Get translated label for a panel type.
 * @param {Function} t - the `$_` translation function
 * @param {string} type - panel type key (e.g. "fixed_glass")
 */
export function panelLabel(t, type) {
  return t(PANEL_TYPE_KEYS[type] || type);
}

/**
 * Get translated label for a member type.
 * @param {Function} t - the `$_` translation function
 * @param {string} type - member type key (e.g. "frame_top")
 */
export function dividerPieceLabel(t, piece, members) {
  const name = memberLabel(t, piece.vertical ? "divider_v" : "divider_h") + " " + (piece.sourceIndex + 1);
  const split = members.some(p => p.vertical === piece.vertical && p.sourceIndex === piece.sourceIndex && p.pieceIndex > 0);
  return split ? name + " · " + t("divider.partNumber", { values: { n: piece.pieceIndex + 1 } }) : name;
}

export function memberLabel(t, type, side) {
  if (type === "frame_arc" && side) return t(side === "bottom" ? "member.lowerArc" : "member.upperArc");
  if (type === "sash_arc" && ["top", "bottom", "left", "right"].includes(side)) return t(`member.sashArc.${side}`);
  return t(MEMBER_TYPE_KEYS[type] || type);
}

const PROFILE_SASH_KEYS = {
  sash_top: "member.profileSashTop",
  sash_bottom: "member.profileSashBottom",
  sash_left: "member.profileSashLeft",
  sash_right: "member.profileSashRight",
};

export function cutMemberLabel(t, item) {
  // CutListItem.material comes from Material::label_nl in all UI languages.
  if (item.material === "Kunststof" || item.material === "Aluminium") {
    const key = PROFILE_SASH_KEYS[item.memberType];
    if (key) return t(key);
  }
  return memberLabel(t, item.memberType, item.profileSide);
}

/**
 * Get translated label for a gasket type.
 * @param {Function} t - the `$_` translation function
 * @param {string} type - gasket type key (e.g. "glazing_inner")
 */
export function gasketLabel(t, type) {
  return t(GASKET_TYPE_KEYS[type] || type);
}

/**
 * Get panel type summary string (e.g. "2x Fixed glass, 1x Door")
 * @param {Function} t - the `$_` translation function
 * @param {Array} cells - array of cell objects with panelType
 */
export function panelTypeSummary(t, cells) {
  const counts = {};
  for (const cell of cells) {
    const label = panelLabel(t, cell.panelType);
    counts[label] = (counts[label] || 0) + 1;
  }
  return Object.entries(counts)
    .map(([label, count]) => `${count}x ${label}`)
    .join(", ");
}
