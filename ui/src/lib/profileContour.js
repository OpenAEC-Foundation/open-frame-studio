/**
 * Parametrische, realistische profieldoorsnedes voor kunststof (PVC),
 * aluminium en hout-aluminium kozijnprofielen.
 *
 * Coördinatenconventie (alle maten in mm):
 *   x: 0 = muurzijde ........ w = vakzijde (dagkant / glasopening)
 *   y: 0 = buitenzijde ...... d = binnenzijde (bouwdiepte)
 *
 * Output per profiel: { crossSection, innerWalls }
 *   - crossSection: buitencontour — impliciet gesloten, enkelvoudige polygoon
 *     (zelfde vorm als altijd: Array<[x, y]>), tekenbaar door ProfileCanvas
 *     en ProfileCrossSection zonder wijziging.
 *   - innerWalls: Array van impliciet gesloten polygonen (Array<Array<[x, y]>>)
 *     met de binnenstructuur: kamer-holtes, staalversterking (U-band),
 *     polyamide isolatorstroken (alu) en de alu-schaal + houtkern (hout-alu).
 *     De tussenruimtes tussen de holtes vormen de zichtbare binnenwanden
 *     (~2,8 mm PVC / ~2,0 mm aluminium).
 *
 * Serde-/compatibiliteitsnotitie:
 *   - `innerWalls` is een NIEUW, puur additief veld in de profiel-JSON's.
 *     De Tauri-kant (src-tauri/src/commands/profiles.rs::load_profile_library)
 *     leest de JSON's als serde_json::Value en geeft ze ongewijzigd door.
 *   - ofs-core::profile::ProfileDefinition heeft GEEN #[serde(deny_unknown_fields)];
 *     standaard serde negeert onbekende velden. Wordt een bibliotheekprofiel als
 *     eigen profiel opgeslagen (add_custom_profile), dan valt innerWalls stil weg —
 *     de buitencontour blijft intact. Geen Rust-wijziging nodig.
 *
 * Deterministisch: geen Date/Math.random; alles afgerond op 0,1 mm.
 */

const TAN15 = Math.tan((15 * Math.PI) / 180);

/** Rond af op 0,1 mm zodat gegenereerde JSON stabiel en leesbaar blijft. */
function r1(v) {
  return Math.round(v * 10) / 10;
}

function clamp(v, lo, hi) {
  return Math.min(Math.max(v, lo), hi);
}

function rect(x0, y0, x1, y1) {
  return [
    [r1(x0), r1(y0)],
    [r1(x1), r1(y0)],
    [r1(x1), r1(y1)],
    [r1(x0), r1(y1)],
  ];
}

function pt(x, y) {
  return [r1(x), r1(y)];
}

/**
 * Buitencontour voor een kader-/vleugelprofiel in PVC of aluminium:
 * rechthoekig lijf met glas-/vleugelsponning op de vak-binnenhoek,
 * buitenaanslag (optioneel met 15°-afschuining), dichtingsgroef op de
 * aanslag en een klikgroef voor de glaslat in de sponningbodem.
 */
function bodyContour(w, d, rH, aY, opts) {
  const bevel = opts.bevel || 0; // afschuiningshoogte (y) op de aanslag, 0 = haaks
  const bevRun = bevel > 0 ? r1(TAN15 * bevel) : 0;
  const gD = 4; // dichtingsgroef diepte
  const gx1 = w - rH + 3; // dichtingsgroef in de aanslag, 3 mm breed
  const gx2 = gx1 + 3;
  const kD = 3; // klikgroef (glaslat) diepte in de sponningbodem
  const kY1 = d - 8;
  const kY2 = d - 3;

  const pts = [pt(0, 0), pt(w, 0)];
  if (bevel > 0) {
    pts.push(pt(w, aY - bevel));
    pts.push(pt(w - bevRun, aY)); // 15° afwateringsafschuining (Gealan/VEKA-silhouet)
  } else {
    pts.push(pt(w, aY));
  }
  // Aanslagbovenvlak met dichtingsgroef
  pts.push(pt(gx2, aY));
  pts.push(pt(gx2, aY - gD));
  pts.push(pt(gx1, aY - gD));
  pts.push(pt(gx1, aY));
  pts.push(pt(w - rH, aY));
  // Sponningbodem met klikgroef voor de glaslat
  pts.push(pt(w - rH, kY1));
  pts.push(pt(w - rH - kD, kY1));
  pts.push(pt(w - rH - kD, kY2));
  pts.push(pt(w - rH, kY2));
  pts.push(pt(w - rH, d));
  pts.push(pt(0, d));
  return pts;
}

/** Aanslagkamer: holte in de buitenaanslag, vrij van dichtingsgroef en afschuining. */
function aanslagCavity(w, rH, aY, t, bevRun) {
  const x0 = w - rH + t;
  const x1 = w - bevRun - t;
  const y1 = aY - 4 - 2; // 2 mm wand onder de dichtingsgroef (diepte 4)
  if (x1 - x0 < 3 || y1 - t < 3) return null;
  return rect(x0, t, x1, y1);
}

function parseChambers(spec, fallback) {
  if (typeof spec.chambers === "number" && spec.chambers >= 2) return Math.round(spec.chambers);
  const m = /^(\d+)_kamer/.exec(spec.materialSubtype || "");
  if (m) return parseInt(m[1], 10);
  return fallback;
}

/**
 * PVC-profiel (meerkamer, bv. VEKA Softline / Gealan S-serie).
 * Kamers gestapeld in de dieptrichting met een grotere staalkamer in het
 * midden; wanddikte ~2,8 mm; 15°-aanslagafschuining; klik- en dichtingsgroef.
 */
export function pvcProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const t = 2.8; // wanddikte PVC
  const rH = clamp(spec.glazingRebate || 25, 15, w * 0.45); // sponninghoogte (x)
  const aY = clamp(25, 12, d * 0.4); // sponningdiepte / aanslag (y), ~25 mm
  // 15°-afwateringsafschuining op de aanslag (Gealan/VEKA-silhouet);
  // vlakke designs (bv. Gealan-KUBUS, flushDesign) blijven haaks.
  const bevel = spec.flushDesign ? 0 : Math.min(10, aY - 8);
  const bevRun = r1(TAN15 * bevel);

  const crossSection = bodyContour(w, d, rH, aY, { bevel });
  const innerWalls = [];

  // Hoofdkamerrij in de dieptrichting (y): nOut kamers buiten, staalkamer, nIn binnen
  let n = clamp(parseChambers(spec, 5), 3, 8);
  let free, hOther, hSteel;
  for (;;) {
    free = d - 2 * t - (n - 1) * t;
    hSteel = free * (n >= 5 ? 0.4 : 0.5);
    hOther = (free - hSteel) / (n - 1);
    if (hOther >= 3.5 || n <= 3) break;
    n -= 1;
  }
  const nOut = Math.floor((n - 1) / 2);
  const xL = t;
  const xR = w - rH - t;
  const xRLast = w - rH - 3 - t; // binnenste kamer wijkt voor de klikgroef

  let y = t;
  for (let i = 0; i < n; i++) {
    const isSteel = i === nOut;
    const h = isSteel ? hSteel : hOther;
    const isLast = i === n - 1;
    innerWalls.push(rect(xL, y, isLast ? xRLast : xR, y + h));
    if (isSteel && spec.steelReinforced !== false) {
      // Staalversterking: U-profiel als gesloten dunne band, opening naar binnen
      const sx0 = xL + 2;
      const sx1 = xR - 2;
      const sy0 = y + 2;
      const sy1 = y + h - 2;
      const bt = 1.5;
      if (sx1 - sx0 > 10 && sy1 - sy0 > 8) {
        innerWalls.push([
          pt(sx0, sy1), pt(sx0, sy0), pt(sx1, sy0), pt(sx1, sy1),
          pt(sx1 - bt, sy1), pt(sx1 - bt, sy0 + bt), pt(sx0 + bt, sy0 + bt), pt(sx0 + bt, sy1),
        ]);
      }
    }
    y += h + t;
  }

  const aanslag = aanslagCavity(w, rH, aY, t, bevRun);
  if (aanslag) innerWalls.push(aanslag);

  return { crossSection, innerWalls };
}

/**
 * Aluminium profiel met thermische onderbreking: buiten- en binnenschaal
 * (wanddikte ~2 mm) verbonden door twee polyamide isolatorstroken van
 * 24–34 mm (zichtbare isolatorzone als middelste kamer).
 */
export function aluProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const t = 2.0; // wanddikte aluminium
  const rH = clamp(spec.glazingRebate || 25, 15, w * 0.45);
  const aY = clamp(25, 12, d * 0.4);

  const sub = spec.materialSubtype || "";
  let bZ = 24; // polyamide-strookbreedte (dieptrichting)
  if (sub === "high_insulation") bZ = 30;
  if (sub === "super_insulated" || sub === "passivhaus") bZ = 34;
  bZ = Math.min(bZ, d - aY - 12); // binnenschaal minimaal ~12 mm
  const y1 = r1(Math.max(aY, (d - bZ) * 0.45)); // onderbrekingszone direct achter de aanslag
  const y2 = r1(Math.min(y1 + bZ, d - 10));

  const crossSection = bodyContour(w, d, rH, aY, { bevel: 0 });
  const innerWalls = [];

  // Buitenschaalkamer
  if (y1 - 2 * t > 4) innerWalls.push(rect(t, t, w - rH - t, y1 - t));
  // Aanslagkamer in de buitenschaal
  const aanslag = aanslagCavity(w, rH, aY, t, 0);
  if (aanslag) innerWalls.push(aanslag);
  // Twee polyamide isolatorstroken (elk ~2 mm dik) overbruggen de schalen
  innerWalls.push(rect(1.2, y1, 3.2, y2));
  innerWalls.push(rect(w - rH - 3.2, y1, w - rH - 1.2, y2));
  // Isolatorzone als aparte (lucht)kamer tussen de stroken
  if (w - rH - 10.4 > 4 && y2 - y1 > 4) {
    innerWalls.push(rect(5.2, y1 + 1, w - rH - 5.2, y2 - 1));
  }
  // Binnenschaalkamer (wijkt voor de klikgroef in de sponningbodem)
  if (d - y2 - 2 * t > 4) innerWalls.push(rect(t, y2 + t, w - rH - 3 - t, d - t));

  return { crossSection, innerWalls };
}

/**
 * Schuifprofiel (PVC of aluminium): kamvormig kader met twee sponningkanalen
 * voor de schuivende vleugels, kamers in het gesloten lijf.
 */
export function slidingSection(spec, material) {
  const w = spec.width;
  const d = spec.depth;
  const t = material === "pvc" ? 2.8 : 2.0;
  const chD = r1(clamp(w - (spec.sightline || w * 0.55), w * 0.3, w * 0.55)); // kanaaldiepte (x)
  const fw = 5; // randvingers (y)
  const mw = 6; // middenvinger tussen de kanalen
  const chW = r1((d - 2 * fw - mw) / 2); // kanaalbreedte (y)

  const crossSection = [
    pt(0, 0), pt(w, 0),
    pt(w, fw), pt(w - chD, fw), pt(w - chD, fw + chW), pt(w, fw + chW),
    pt(w, fw + chW + mw), pt(w - chD, fw + chW + mw), pt(w - chD, d - fw), pt(w, d - fw),
    pt(w, d), pt(0, d),
  ];

  // Drie kamers in het gesloten lijf (x ∈ [0, w - chD]), gestapeld in y
  const innerWalls = [];
  const xR = w - chD - t;
  const h = (d - 4 * t) / 3;
  let y = t;
  for (let i = 0; i < 3; i++) {
    innerWalls.push(rect(t, y, xR, y + h));
    if (i === 1 && material === "pvc" && spec.steelReinforced !== false) {
      const bt = 1.5;
      const sx0 = t + 2, sx1 = xR - 2, sy0 = y + 2, sy1 = y + h - 2;
      if (sx1 - sx0 > 10 && sy1 - sy0 > 8) {
        innerWalls.push([
          pt(sx0, sy1), pt(sx0, sy0), pt(sx1, sy0), pt(sx1, sy1),
          pt(sx1 - bt, sy1), pt(sx1 - bt, sy0 + bt), pt(sx0 + bt, sy0 + bt), pt(sx0 + bt, sy1),
        ]);
      }
    }
    y += h + t;
  }

  return { crossSection, innerWalls };
}

/**
 * Hout-aluminium: massieve houten kern met glassponning aan de binnenzijde
 * en een aluminium schaal (2 mm) met geventileerde spouw (4 mm) op de
 * buitenzijde. Alleen schaal en spouw staan in innerWalls; de houten kern
 * blijft massief. Tussenstijlen (divider) krijgen sponning en schaalomslag
 * aan beide zijden.
 */
export function woodAluProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const rW = clamp((spec.sponning && spec.sponning.width) || 12, 8, w * 0.3); // sponninghoogte (x)
  const rD = clamp(spec.glazingRebate || 24, 12, d * 0.35); // sponningdiepte (y)
  const capT = 2; // schaaldikte aluminium
  const gap = 4; // geventileerde spouw hout/alu
  const capD = clamp(spec.aluCapDepth || 25, 10, d * 0.4); // omslagdiepte van de schaal
  const isDivider = (spec.applicableAs || []).includes("divider");

  const crossSection = isDivider
    ? [
        pt(0, 0), pt(w, 0), pt(w, d - rD), pt(w - rW, d - rD), pt(w - rW, d),
        pt(rW, d), pt(rW, d - rD), pt(0, d - rD),
      ]
    : [pt(0, 0), pt(w, 0), pt(w, d - rD), pt(w - rW, d - rD), pt(w - rW, d), pt(0, d)];

  const innerWalls = [];
  const g = capT + gap;
  if (isDivider) {
    // Alu-schaal: band over de buitenzijde, omgeslagen langs beide vakzijden
    innerWalls.push([
      pt(0, 0), pt(w, 0), pt(w, capD), pt(w - capT, capD),
      pt(w - capT, capT), pt(capT, capT), pt(capT, capD), pt(0, capD),
    ]);
    // Ventilatiespouw tussen schaal en houten kern (U-band)
    innerWalls.push([
      pt(capT, capT), pt(w - capT, capT), pt(w - capT, capD), pt(w - g, capD),
      pt(w - g, g), pt(g, g), pt(g, capD), pt(capT, capD),
    ]);
  } else {
    innerWalls.push([
      pt(0, 0), pt(w, 0), pt(w, capD), pt(w - capT, capD), pt(w - capT, capT), pt(0, capT),
    ]);
    // Ventilatiespouw tussen schaal en houten kern (L-band)
    innerWalls.push([
      pt(0, capT), pt(w - capT, capT), pt(w - capT, capD), pt(w - g, capD),
      pt(w - g, g), pt(0, g),
    ]);
  }

  return { crossSection, innerWalls };
}

/**
 * Kies per materiaal de juiste generator.
 * Retourneert { crossSection, innerWalls } of null (hout en onbekende
 * materialen behouden hun bestaande, massieve contour).
 */
export function generateProfileGeometry(spec) {
  const sub = spec.materialSubtype || "";
  if (spec.material === "pvc") {
    return sub === "sliding" ? slidingSection(spec, "pvc") : pvcProfileSection(spec);
  }
  if (spec.material === "aluminum") {
    return sub === "sliding" ? slidingSection(spec, "aluminum") : aluProfileSection(spec);
  }
  if (spec.material === "wood_aluminum") {
    return woodAluProfileSection(spec);
  }
  return null;
}
