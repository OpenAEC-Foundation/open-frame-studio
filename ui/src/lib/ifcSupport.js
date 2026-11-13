/**
 * IFC export preflight — tells the user BEFORE the save dialog why a kozijn
 * cannot be exported, instead of a red toast after choosing a file name.
 *
 * Mirrors the structural rules of `validate_ifc_export`
 * (ofs-core/src/export/ifc.rs): the Rust side stays the authority (fail-closed,
 * no file is written on a refusal); this only front-runs the rules a user can
 * actually run into so the app can explain them or pick a workable LOD.
 */

/** Structural reason a kozijn can never be exported to IFC (any LOD), or null. */
export function ifcStructuralBlocker(k) {
  if (!k) return null;
  const shape = k.frame?.shape?.shapeType;
  if (shape && shape !== "rectangular") {
    return "alleen rechthoekige kozijnen worden naar IFC geëxporteerd (dit kozijn heeft een andere vorm)";
  }
  if (k.layout) {
    return "vrije indeling wordt nog niet naar IFC geëxporteerd — gebruik 'Terug naar raster' in de eigenschappen om dit kozijn te exporteren";
  }
  if (Array.isArray(k.extensions) && k.extensions.length) {
    return "kozijnaanbouw (uitbreidingen) wordt nog niet naar IFC geëxporteerd";
  }
  return null;
}

/** True when LOD 300 cannot describe this kozijn (anything but fixed glass). */
export function needsLod200(k) {
  return (k?.cells || []).some((c) => (c.panelType || "fixed_glass") !== "fixed_glass");
}

/**
 * Preflight for one or more kozijnen.
 * @param {object[]} kozijnen
 * @param {string|number} lod requested LOD ("200" | "300")
 * @param {{ allowLodFallback?: boolean }} [opts] fall back 300 → 200 (with notice)
 * @returns {{ ok: boolean, lod: string, downgraded: boolean, message: string|null }}
 */
export function ifcPreflight(kozijnen, lod, opts = {}) {
  const list = (kozijnen || []).filter(Boolean);
  const blocked = list
    .map((k) => ({ mark: k.mark || k.name || "?", reason: ifcStructuralBlocker(k) }))
    .filter((b) => b.reason);
  if (blocked.length) {
    const lines = blocked.map((b) => `${b.mark}: ${b.reason}`);
    return {
      ok: false,
      lod: String(lod),
      downgraded: false,
      message: `IFC-export niet mogelijk — ${lines.join("; ")}.`,
    };
  }
  let effective = String(lod);
  let downgraded = false;
  if (effective === "300") {
    const marks = list.filter(needsLod200).map((k) => k.mark || k.name || "?");
    if (marks.length) {
      if (opts.allowLodFallback) {
        effective = "200";
        downgraded = true;
      } else {
        return {
          ok: false,
          lod: effective,
          downgraded: false,
          message: `LOD 300 ondersteunt alleen vaste beglazing (${marks.join(", ")} heeft draaiende delen of deuren) — kies LOD 200.`,
        };
      }
    }
  }
  return {
    ok: true,
    lod: effective,
    downgraded,
    message: downgraded
      ? "LOD 300 ondersteunt alleen vaste beglazing — dit project is op LOD 200 geëxporteerd."
      : null,
  };
}
