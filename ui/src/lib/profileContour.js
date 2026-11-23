/**
 * Parametrische, realistische profieldoorsnedes voor hout, kunststof (PVC),
 * aluminium en hout-aluminium kozijnprofielen.
 *
 * Coördinatenconventie (alle maten in mm, identiek aan profiles/*.json en
 * de extrusie in Viewer3D.svelte):
 *   u (x): 0 = muurzijde ........ w = vakzijde (dagkant / glasopening)
 *   v (y): 0 = buitenzijde ...... d = binnenzijde (bouwdiepte)
 * Een binnensponning heeft zijn inkeping dus op de vakzijde-binnen-hoek
 * (u = w, v = d); een buitensponning op (u = w, v = 0).
 *
 * Veldsemantiek (KVT 12.01: h = sponninghoogte in het vlak, m = sponning-
 * breedte in de bouwdiepterichting):
 *   - sponning.depth  = sponninghoogte h (glasinval in het aanzicht; hout
 *     vast 17 mm per KVT 12.2, PVC = Glaseinstand per systeem, alu per
 *     systeem — 13,5 (SL38) t/m 27 (MasterLine 8)).
 *   - sponning.width  = sponningbreedte m (bouwdiepterichting; hout vast
 *     glas 51 mm per KVT-tekening 14.01/DTS, onderdorpel 53 = bodem 45 +
 *     opstand 8).
 *   - glazingRebate   = zelfde m voor hout; voor PVC de Glasfalzhöhe
 *     (aanzichtrichting! — VEKA 82: Falzhöhe 28 ≠ Glaseinstand 20) en voor
 *     aluminium de sponninghoogte van het systeem.
 *   - thermalBreakWidth = breedte isolatorsteg alu (ML8 40; insulbar-reeks
 *     24/34/42 als subtype-fallback).
 *
 * Output per profiel: { crossSection, innerWalls? }
 *   - crossSection: buitencontour — impliciet gesloten, enkelvoudige polygoon
 *     (Array<[x, y]>), tekenbaar door ProfileCanvas en ProfileCrossSection.
 *   - innerWalls: Array van impliciet gesloten polygonen met binnenstructuur:
 *     kamer-holtes, staalversterking (U-band), isolatorstroken (alu) en de
 *     alu-schaal + spouw (hout-alu). Hout is massief: geen innerWalls.
 *
 * Serde-/compatibiliteitsnotitie:
 *   - `innerWalls` en de extra datavelden zijn puur additief in de
 *     profiel-JSON's. De Tauri-kant (load_profile_library) leest de JSON's
 *     als serde_json::Value en geeft ze ongewijzigd door; ProfileDefinition
 *     negeert onbekende velden (geen deny_unknown_fields).
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

/* ──────────────────────────── HOUT ──────────────────────────────── */

function hasApplication(spec, names) {
  const apps = spec.applicableAs || [];
  return apps.some((a) => names.includes(a));
}

/** Library JSON and saved projects use different field names for the same rebate. */
export function profileRebate(sp) {
  const { type: legacyType, opdek_width, rubber_count, second_width, second_depth, ...fields } = sp || {};
  const type = fields.sponningType ?? legacyType ??
    ({ buiten: "buitensponning", dubbel: "dubbele_sponning" }[fields.position] || "binnensponning");
  return {
    ...fields,
    sponningType: type,
    position: type === "dubbele_sponning" ? "dubbel"
      : fields.position || (type === "buitensponning" ? "buiten" : "binnen"),
    width: sp?.width ?? 0,
    depth: sp?.depth ?? 0,
    opdekWidth: fields.opdekWidth ?? opdek_width,
    rubberCount: fields.rubberCount ?? rubber_count,
    secondWidth: fields.secondWidth ?? second_width,
    secondDepth: fields.secondDepth ?? second_depth,
  };
}

export function woodSectionFits(spec) {
  const { width: w, depth: d } = spec;
  if (!(w > 0 && d > 0)) return false;
  const sp = profileRebate(spec.sponning);
  if (sp.sponningType === "geen" || (!sp.width && !sp.depth)) return true;
  const divider = isDedicatedDivider(spec);
  const face = divider ? w / 2 : w;
  if (!(sp.width > 0 && sp.width < d && sp.depth > 0 && sp.depth < face)) return false;
  if (sp.position === "dubbel") {
    const width2 = sp.secondWidth ?? sp.width;
    const depth2 = sp.secondDepth ?? sp.depth;
    return width2 > 0 && width2 + sp.width < d && depth2 > 0 && depth2 < face;
  }
  if (sp.opdekWidth && sp.sponningType === "draaikiep") return sp.opdekWidth + 4 < w - sp.depth;
  if (hasApplication(spec, ["sill", "sash_sill", "onderdorpel"])) {
    const drop = Math.tan((sp.slopeDegrees ?? 10) * Math.PI / 180) * sp.width;
    return sp.width > 8 && sp.depth + drop < w;
  }
  return true;
}

function isDedicatedDivider(spec) {
  return hasApplication(spec, ["divider", "divider_horizontal", "tussenstijl", "tussendorpel"]) &&
    !hasApplication(spec, ["frame", "sash"]);
}

/**
 * Massieve houtdoorsnede volgens de KVT-maatketen.
 *
 * Vormen:
 *   - binnensponning (standaard): inkeping h×m op de vakzijde-binnen-hoek;
 *   - buitensponning: inkeping op de vakzijde-buiten-hoek;
 *   - tussenstijl/tussendorpel (divider): inkeping aan beide vakzijden;
 *   - onderdorpel (sill): sponningbodem afwaterend ≥9° (praktijk 10°,
 *     KVT 14.2) met opstand 8 breed × 15 hoog (KVT-tekening 14.01);
 *   - draaikiep-raamhout: sponning + opdeklip 13 en rubbergroeven;
 *   - dubbele sponning: sponning aan binnen- én buitenzijde met kernhout;
 *   - geen sponning: rechthoek (glaslat, spouwlat, stomp kozijn).
 *
 * h = spec.sponning.depth (sponninghoogte, KVT-min 17), m = spec.sponning
 * .width of glazingRebate (sponningbreedte).
 */
export function woodCrossSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const sp = profileRebate(spec.sponning);
  const h = r1(sp.depth);
  const m = r1(spec.sponning?.width ?? spec.glazingRebate ?? 0);

  // Geen sponning: rechthoek.
  if (h <= 0 || m <= 0 || sp.sponningType === "geen") {
    return [pt(0, 0), pt(w, 0), pt(w, d), pt(0, d)];
  }

  const position = sp.position;
  const isSill = hasApplication(spec, ["sill", "sash_sill", "onderdorpel"]);
  // Alleen zuivere tussenstijlen/-dorpels dubbelzijdig tekenen; een profiel
  // dat óók stijl/raamhout kan zijn houdt de enkelzijdige standaardvorm.
  const isDivider = isDedicatedDivider(spec);
  const isDraaikiep =
    sp.opdekWidth > 0 && (sp.sponningType === "draaikiep" || hasApplication(spec, ["sash", "raam_stijl", "raam_dorpel"]));

  // Onderdorpel: afwaterende sponningbodem + opstand (KVT 14.2 / 14.01).
  if (isSill && (position === "binnen" || position === "buiten")) {
    const slope = (sp.slopeDegrees ?? 10) * (Math.PI / 180);
    if (position === "buiten") {
      // The open outer end drains away from the inner stop.
      return [
        pt(0, 0), pt(w - h - Math.tan(slope) * m, 0),
        pt(w - h, m), pt(w, m), pt(w, d), pt(0, d),
      ];
    }
    const upW = 8; // opstand breedte (KVT 14.01: >= 8)
    const upH = 15; // opstand hoogte (KVT 14.01: >= 15 bij draaiende delen)
    const bodem = Math.max(m - upW, 10); // afwaterende bodembreedte (~45 bij m=53)
    const drop = r1(Math.tan(slope) * bodem);
    const xTop = w - h; // bodem aan binnenzijde (= dagmaat - sponninghoogte)
    const xLow = xTop - drop; // bodem aan buitenzijde (afschot omlaag)
    const xUp = Math.min(xTop + upH, w - 2); // bovenkant opstand
    const y0 = d - m; // sponningrand (buitenzijde van de sponning)
    const yUp = d - upW; // voorkant opstand
    return [
      pt(0, 0), pt(w, 0), pt(w, y0), pt(xLow, y0),
      pt(xTop, yUp), pt(xUp, yUp), pt(xUp, d), pt(0, d),
    ];
  }

  // Tussenstijl / tussendorpel: sponning aan beide vakzijden (binnenzijde).
  let glassSide;
  if (position === "dubbel") {
    const h2 = sp.secondDepth ?? h;
    const m2 = sp.secondWidth ?? m;
    glassSide = [pt(w - h2, 0), pt(w - h2, m2), pt(w, m2),
      pt(w, d - m), pt(w - h, d - m), pt(w - h, d)];
  } else if (position === "buiten") {
    glassSide = [pt(w - h, 0), pt(w - h, m), pt(w, m), pt(w, d)];
  } else if (position === "midden") {
    const start = (d - m) / 2;
    glassSide = [pt(w, 0), pt(w, start), pt(w - h, start),
      pt(w - h, start + m), pt(w, start + m), pt(w, d)];
  } else {
    glassSide = [pt(w, 0), pt(w, d - m), pt(w - h, d - m), pt(w - h, d)];
  }

  if (isDivider) {
    const opposite = glassSide.map(([u, v]) => pt(w - u, v)).reverse();
    return [opposite.pop(), ...glassSide, ...opposite];
  }

  // The sash overlap is on the frame side. Keep the glass pocket open on u=w.
  if (isDraaikiep) {
    const lap = sp.opdekWidth;
    const lipDepth = Math.min(13, d / 3);
    const grooveWidth = Math.min(spec.kaderdichtingGroef?.width ?? 3, lap - 2);
    const grooveDepth = Math.min(spec.kaderdichtingGroef?.depth ?? 4, lipDepth - 2);
    const left = (lap - grooveWidth) / 2;
    const shoulder = d - lipDepth;
    const frameSide = [pt(0, d), pt(0, shoulder)];
    if (grooveWidth > 0 && grooveDepth > 0) frameSide.push(
      pt(left, shoulder), pt(left, shoulder + grooveDepth),
      pt(left + grooveWidth, shoulder + grooveDepth), pt(left + grooveWidth, shoulder)
    );
    frameSide.push(pt(lap, shoulder));
    const grooveCount = clamp(sp.rubberCount ?? 2, 1, 3);
    for (let i = 1; i < grooveCount; i++) {
      const center = shoulder * (1 - i / grooveCount);
      frameSide.push(pt(lap, center + 1.5), pt(lap + 4, center + 1.5),
        pt(lap + 4, center - 1.5), pt(lap, center - 1.5));
    }
    return [pt(lap, 0), ...glassSide, ...frameSide];
  }
  return [pt(0, 0), ...glassSide, pt(0, d)];
}

/* ─────────────────────── PVC / ALUMINIUM ────────────────────────── */

/**
 * Buitencontour voor een kader-/vleugelprofiel in PVC of aluminium:
 * rechthoekig lijf met glas-/vleugelsponning op de vak-binnen-hoek
 * (glaslat binnen), buitenaanslag (optioneel met 15°-afschuining),
 * dichtingsgroef op de aanslag en een klikgroef voor de glaslat in de
 * sponningbodem.
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
 * midden; wanddikte 2,8 mm (Klasse A zichtvlak, DIN EN 12608);
 * 15°-aanslagafschuining; klik- en dichtingsgroef. glazingRebate wordt hier
 * als Glasfalzhöhe (aanzichtrichting) gebruikt.
 */
export function pvcProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const t = 2.8; // wanddikte PVC — Klasse A zichtvlak >= 2,8 (DIN EN 12608)
  const rH = clamp(spec.glazingRebate || 25, 15, Math.max(15, w - 14)); // Falzhöhe (x)
  const aY = clamp(25, 12, d * 0.4); // aanslagzone (y), ~25 mm
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

  return { crossSection, innerWalls, innerWallTypes: innerWalls.map(r => r.length === 8 ? "reinforcement" : "chamber") };
}

/**
 * Aluminium profiel met thermische onderbreking: buiten- en binnenschaal
 * (wanddikte nominaal 2 mm, band 1,6-2,5 per ML8-bestek) verbonden door twee
 * polyamide isolatorstroken. Stegbreedte uit spec.thermalBreakWidth
 * (MasterLine 8: 40 mm) of de insulbar-referentiereeks per subtype
 * (24 / 34 / 42 mm). glazingRebate = sponninghoogte van het systeem
 * (SL38 13,5 / ML8 27 / CS77 Functional 25 mm).
 */
export function aluProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const t = 2.0; // wanddikte aluminium (nominaal; 1,6-2,5 per EN 12020-2/ML8)
  const rH = clamp(spec.glazingRebate || 25, 10, Math.max(10, w - 14));
  const aY = clamp(25, 12, d * 0.4);

  const sub = spec.materialSubtype || "";
  // Isolatorsteg: systeemwaarde of insulbar-reeks 24/34/42 (Uf-trap 2,6/1,9/1,5)
  let bZ = 24;
  if (sub === "high_insulation") bZ = 34;
  if (sub === "super_insulated" || sub === "passivhaus") bZ = 42;
  if (typeof spec.thermalBreakWidth === "number" && spec.thermalBreakWidth > 0) {
    bZ = Math.min(spec.thermalBreakWidth, d - aY - 10);
  } else {
    bZ = Math.min(bZ, d - aY - 12);
  }
  const y1 = r1(Math.max(aY, (d - bZ) * 0.45)); // onderbrekingszone direct achter de aanslag
  const y2 = r1(Math.min(y1 + bZ, d - 10));

  const crossSection = bodyContour(w, d, rH, aY, { bevel: 0 });
  const innerWalls = [];

  if (spec.thermallyBroken === false) {
    innerWalls.push(rect(t, t, w - rH - 3 - t, d - t));
    const nose = aanslagCavity(w, rH, aY, t, 0);
    if (nose) innerWalls.push(nose);
    return { crossSection, innerWalls, innerWallTypes: innerWalls.map(() => "chamber") };
  }

  // Buitenschaalkamer
  if (y1 - 2 * t > 4) innerWalls.push(rect(t, t, w - rH - t, y1 - t));
  // Aanslagkamer in de buitenschaal
  const aanslag = aanslagCavity(w, rH, aY, t, 0);
  if (aanslag) innerWalls.push(aanslag);
  // Twee polyamide isolatorstroken (elk ~2 mm dik) overbruggen de schalen
  const bridges = [rect(1.2, y1, 3.2, y2), rect(w - rH - 3.2, y1, w - rH - 1.2, y2)];
  innerWalls.push(...bridges);
  // Isolatorzone als aparte (lucht)kamer tussen de stroken
  if (w - rH - 10.4 > 4 && y2 - y1 > 4) {
    innerWalls.push(rect(5.2, y1 + 1, w - rH - 5.2, y2 - 1));
  }
  // Binnenschaalkamer (wijkt voor de klikgroef in de sponningbodem)
  if (d - y2 - 2 * t > 4) innerWalls.push(rect(t, y2 + t, w - rH - 3 - t, d - t));

  return { crossSection, innerWalls, innerWallTypes: innerWalls.map(r => bridges.includes(r) ? "insulator" : "chamber") };
}

/**
 * Schuifprofiel (PVC of aluminium): kamvormig kader met twee sponningkanalen
 * voor de schuivende vleugels (kanalen gestapeld over de bouwdiepte),
 * kamers in het gesloten lijf.
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

  return { crossSection, innerWalls, innerWallTypes: innerWalls.map(r => r.length === 8 ? "reinforcement" : "chamber") };
}

/**
 * Hout-aluminium: massieve houten kern met glassponning (h×m per KVT, zoals
 * hout) aan de binnenzijde en een aluminium schaal (2 mm) met geventileerde
 * spouw (4 mm) op de buitenzijde. Alleen schaal en spouw staan in innerWalls;
 * de houten kern blijft massief. Tussenstijlen (divider) krijgen sponning en
 * schaalomslag aan beide zijden.
 */
export function woodAluProfileSection(spec) {
  const w = spec.width;
  const d = spec.depth;
  const sp = spec.sponning || null;
  const rW = clamp((sp && sp.depth) || 17, 8, w * 0.35); // sponninghoogte (x)
  const rD = clamp(
    (sp && sp.width) || spec.glazingRebate || 51,
    12,
    Math.max(12, d - 20)
  ); // sponningbreedte (y)
  const capT = 2; // schaaldikte aluminium
  const gap = 4; // geventileerde spouw hout/alu
  const capD = clamp(spec.aluCapDepth || 25, 10, d * 0.4); // omslagdiepte van de schaal
  const isDivider =
    hasApplication(spec, ["divider", "divider_horizontal", "tussenstijl", "tussendorpel"]) &&
    !hasApplication(spec, ["frame", "sash"]);

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
 * Retourneert { crossSection, innerWalls? } of null (onbekende materialen
 * behouden hun bestaande contour). Hout krijgt een massieve contour zonder
 * innerWalls.
 */
export function generateProfileGeometry(spec) {
  const sub = spec.materialSubtype || "";
  if (spec.material === "wood") {
    return { crossSection: woodCrossSection(spec) };
  }
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
