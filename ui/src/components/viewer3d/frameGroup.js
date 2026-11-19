import { ralToHex } from "../../lib/ral-colors.js";
import {
  ringBBox, miterEndsFor, layoutHasBuiten, collectBuitenRects,
  expandBuitenRect, decomposeSteppedFrame, rectAdjacency,
  sectionFromPoints, preferredChannel, openingHalf, memberRuns, machineRebate, sashSides, closingStock, sashReturn,
  stoppedSection, machiningSpans, stockStrip,
} from "../../lib/frame3d.js";
import { cellFits } from "../../lib/glazingFit.js";
import { contourPoints, contourSide, contourSlab, clipByLine, clipToRect, insetPolygon, sweepRing, timberPlan, trimAtMasonry, splitTimberWeb } from "./curvedMesh.js";
import { dividerFootprint, frameBandFootprint, masonrySpans, touchesPocket, meshFaces, clipStock, stockMesh, dividerMesh } from "./dividerMesh.js";
import { machineTimberJoints } from "./tenonMesh.js";
import { systemClosure } from "./systemClosure.js";
import { addHardware } from "./hardwareMesh.js";
// Material colors
const MATERIAL_COLORS = {
  "wood(meranti)": 0xC4956A,
  "wood(accoya)": 0xD4C5A9,
  "wood(vuren)": 0xE8D5A3,
  "wood(eiken)": 0xA07040,
  aluminum: 0xB0B0B0,
  pvc: 0xF0F0F0,
  "wood_aluminum": 0xC4956A,
};

const GLASS_COLOR = 0x88CCDD;
const PANEL_COLOR = 0xD0C8B8;
const DOOR_COLOR = 0xA08060;
const MASONRY_COLOR = 0x9C876D; // matte baksteen (borstwering onder 'buiten'-vak)
const SASH_PROUD_MM = 8; // vleugel-binnenvlak steekt iets voor het kozijn (opdek)
const SEAL_MM = 2; // modelled compressed glazing gasket at each glass face

// Operable sash types (mirror PanelType::is_operable in ofs-core).
const OPERABLE_TYPES = new Set([
  "turn_tilt", "turn", "tilt", "sliding", "door",
  "top_hung", "bottom_hung", "lift_slide", "pivot",
]);

// Panel-filling tint per FillingType (serde snake_case).
const FILLING_COLORS = {
  sandwich: 0xD0C8B8,
  solid: 0xB8A888,
  door_panel: 0xA08060,
  ventilation: 0x9AA0A8,
  blind: 0xC0C0C0,
};

function getMaterialKey(material) {
  if (!material) return "wood(meranti)";
  if (typeof material === "string") return material.toLowerCase();
  // Rust enum serialized as { "wood": "meranti" } or "aluminum"
  if (typeof material === "object") {
    const key = Object.keys(material)[0];
    const val = material[key];
    return `${key}(${val})`.toLowerCase();
  }
  return "wood(meranti)";
}

function getFrameColor(kozijn) {
  // #8: prefer the chosen RAL colour so the picked colour shows in 3D too;
  // fall back to the material's natural tint when no RAL is set.
  const ral = kozijn?.frame?.colorInside;
  if (ral) {
    const hex = ralToHex(ral);
    if (typeof hex === "string" && hex[0] === "#") return parseInt(hex.slice(1), 16);
  }
  const key = getMaterialKey(kozijn?.frame?.material);
  return MATERIAL_COLORS[key] || MATERIAL_COLORS["wood(meranti)"];
}

// --- Profiel-doorsnedes (echte kozijnhout-/kamerprofiel-secties) ---
//
// crossSection-conventie van de profielbibliotheek (profiles/*.json en
// ui/src/lib/profileContour.js): [u, v]-punten in mm.
//   u: 0 = muurzijde ..... w = vakzijde (dagkant, waar het glas/vak zit)
//   v: 0 = buitenzijde ... d = binnenzijde (bouwdiepte)
// Preserve imported solid contours too: a four-point rectangle has no rebate.
function resolveProfileSection(ref, profiles) {
  if (!ref?.id || !Array.isArray(profiles)) return null;
  const def = profiles.find((p) => p.id === ref.id);
  const cs = def?.crossSection;
  if (!Array.isArray(cs) || cs.length < 3) return null;
  const pts = [];
  for (const p of cs) {
    const u = Number(p?.[0]);
    const v = Number(p?.[1]);
    if (!Number.isFinite(u) || !Number.isFinite(v)) return null;
    // Opeenvolgende dubbele punten overslaan (DXF-import herhaalt punten).
    const prev = pts[pts.length - 1];
    if (prev && Math.abs(prev[0] - u) < 1e-6 && Math.abs(prev[1] - v) < 1e-6) continue;
    pts.push([u, v]);
  }
  // Expliciet sluitpunt (gelijk aan startpunt) weglaten.
  if (pts.length > 2) {
    const [fu, fv] = pts[0];
    const [lu, lv] = pts[pts.length - 1];
    if (Math.abs(fu - lu) < 1e-6 && Math.abs(fv - lv) < 1e-6) pts.pop();
  }
  if (pts.length < 3) return null;
  let minU = Infinity, maxU = -Infinity, minV = Infinity, maxV = -Infinity;
  for (const [u, v] of pts) {
    if (u < minU) minU = u;
    if (u > maxU) maxU = u;
    if (v < minV) minV = v;
    if (v > maxV) maxV = v;
  }
  const w = maxU - minU;
  const d = maxV - minV;
  if (!(w > 0.5) || !(d > 0.5)) return null;
  // `def` rijdt mee voor sponningmetadata (glas-z in de sponning).
  const innerWalls = (Array.isArray(def.innerWalls) ? def.innerWalls : []).filter(r => Array.isArray(r) && r.length >= 3 &&
    r.every(p => Array.isArray(p) && p.length === 2 && p.every(Number.isFinite)));
  return { id: def.id, def, pts, minU, minV, w, d, innerWalls };
}

export function buildFrameGroup(THREE, kozijn, geometry, profiles) {
  const kozijnGroup = new THREE.Group();
  const frameColor = getFrameColor(kozijn);
  const frameDepth = kozijn.frame?.frameDepth || 114;

  // Frame material — PBR properties depend on material type
  const matKey = getMaterialKey(kozijn?.frame?.material);
  const isAluminum = matKey === "aluminum";
  const frameMat = new THREE.MeshStandardMaterial({
    color: frameColor,
    roughness: isAluminum ? 0.3 : 0.7,
    metalness: isAluminum ? 0.8 : 0.0,
  });

  // Glass material — realistic PBR glass
  const glassMat = new THREE.MeshPhysicalMaterial({
    color: GLASS_COLOR,
    transparent: true,
    opacity: 0.3,
    roughness: 0.05,
    metalness: 0.0,
    transmission: 0.9,
    ior: 1.5,
    thickness: 0.5,
    side: THREE.DoubleSide,
  });

  // Door material
  const doorMat = new THREE.MeshStandardMaterial({
    color: DOOR_COLOR,
    roughness: 0.6,
    metalness: 0.0,
  });

  const sealMat = new THREE.MeshStandardMaterial({ color: 0x343536, roughness: 0.88 });
  const sectionLineMat = new THREE.LineBasicMaterial({ color: 0x4c5054 });

  // Divider material (same as frame)
  const dividerMat = frameMat.clone();

  // Sash frame material (matches the frame; slightly different sheen)
  const sashMat = new THREE.MeshStandardMaterial({
    color: frameColor,
    roughness: isAluminum ? 0.3 : 0.65,
    metalness: isAluminum ? 0.8 : 0.0,
  });

  // Glazing-bead material (glaslat) — matches the frame colour
  const beadMat = new THREE.MeshStandardMaterial({
    color: frameColor,
    roughness: isAluminum ? 0.35 : 0.6,
    metalness: isAluminum ? 0.7 : 0.0,
  });

  // Helper: role-tag voor de debug-samenvatting (__OFS_3D_SUMMARY__)
  function tag(mesh, role) {
    if (mesh) mesh.userData.role = role;
    return mesh;
  }

  // Helper: create a box mesh from a 2D rect, extruded into Z
  function makeBox(rect, depth, material, zOffset = 0) {
    const geo = new THREE.BoxGeometry(rect.width, rect.height, depth);
    const mesh = new THREE.Mesh(geo, material);
    // Position: rect x,y are top-left in 2D, convert to center for Three.js
    mesh.position.set(
      rect.x + rect.width / 2,
      -(rect.y + rect.height / 2), // flip Y (2D y-down to 3D y-up)
      zOffset
    );
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    return mesh;
  }

  // Helper: shrink a rect inwards by `by` mm on every side
  function insetRectBy(rect, by) {
    return {
      x: rect.x + by,
      y: rect.y + by,
      width: Math.max(1, rect.width - 2 * by),
      height: Math.max(1, rect.height - 2 * by),
    };
  }

  // --- Echte profiel-extrusies voor kozijnleden ---
  // Hergebruik binnen deze build: één THREE.Shape per profiel en één
  // ExtrudeGeometry per uniek (profiel, lengte)-paar — vakken met gelijke
  // maten delen zo hun geometrie. Meshes mogen geometrie delen: de teardown
  // in build3DKozijn/onDestroy mag dispose() dan meermaals aanroepen
  // (BufferGeometry.dispose is idempotent).
  const sectionCache = new Map(); // profileRef.id -> section | null
  const shapeCache = new Map(); // section.id -> THREE.Shape
  const extrudeCache = new Map(); // `${section.id}|${lengte}` -> ExtrudeGeometry
  const profileList = Array.isArray(profiles) ? profiles : [];

  function getSection(ref) {
    const key = ref?.id;
    if (!key) return null;
    if (sectionCache.has(key)) return sectionCache.get(key);
    const selected = resolveProfileSection(ref, profileList);
    const section = miterCorners && selected?.def?.material !== baseMat ? null : selected;
    sectionCache.set(key, section);
    return section;
  }

  function getExtrudedGeometry(section, length, mirrored) {
    const shapeKey = mirrored ? `${section.id}|m` : section.id;
    const key = `${shapeKey}|${length}`;
    let geo = extrudeCache.get(key);
    if (!geo) {
      let shape = shapeCache.get(shapeKey);
      if (!shape) {
        // Gespiegeld: u' = w - u (vakzijde wisselt van kant); ExtrudeGeometry
        // normaliseert de omgekeerde winding zelf.
        const su = (u) => (mirrored ? section.w - (u - section.minU) : u - section.minU);
        shape = new THREE.Shape();
        shape.moveTo(su(section.pts[0][0]), section.pts[0][1] - section.minV);
        for (let i = 1; i < section.pts.length; i++) {
          shape.lineTo(su(section.pts[i][0]), section.pts[i][1] - section.minV);
        }
        shape.closePath();
        shapeCache.set(shapeKey, shape);
      }
      geo = new THREE.ExtrudeGeometry(shape, { depth: length, bevelEnabled: false });
      extrudeCache.set(key, geo);
    }
    return geo;
  }

  // Eén kozijnlid als profiel-extrusie.
  //
  // Viewer-assen: X = breedte (2D x), Y = hoogte (2D y omgeklapt), Z = diepte
  // met -Z = buiten en +Z = binnen (glas en muur zitten richting -Z). De
  // shape leeft lokaal in XY (x = u = aanzichtbreedte, y = v = bouwdiepte,
  // v=0 = buiten) en wordt langs lokaal +Z over de lidlengte geëxtrudeerd.
  // De rotaties (Euler-order ZYX, numeriek geverifieerd tegen three r183):
  //   stijl  ("v"): (u, v, t) -> wereld (u, -t, v) — doorsnede in het
  //                 XZ-vlak, extrusie langs Y; u loopt naar +X
  //   dorpel ("h"): (u, v, t) -> wereld (t, u, v)  — doorsnede in het
  //                 YZ-vlak, extrusie langs X; u loopt naar +Y
  // `mirrored` klapt de doorsnede in de aanzichtrichting zodat de vakzijde
  // (u=w) naar het vak wijst: linkerstijl/onderdorpel normaal,
  // rechterstijl/bovendorpel gespiegeld; tussenleden (glas aan twee kanten)
  // blijven ongespiegeld/gecentreerd.
  // Binnenvlakken liggen vlak in één front: v=d (binnen) op Z=+frameDepth/2,
  // zodat een dieper profiel (bv. onderdorpel met waterhol) aan de
  // buitenzijde uitsteekt — zoals in het echt. De doorsnede wordt in de
  // aanzichtrichting gecentreerd op het lid als profiel- en lidbreedte
  // verschillen.
  // opts: { zInner, side, miterStart, miterEnd }
  //   zInner     — wereld-z van het binnenvlak (v=d); default frameDepth/2.
  //   side       — 'top'|'bottom'|'left'|'right': welke buitenrand het lid
  //                heeft (nodig om de verstekrichting te bepalen).
  //   miterStart — 45°-verstek op het startvlak van de extrusie (links voor
  //                "h", boven voor "v"); miterEnd idem op het eindvlak.
  // Het verstek wordt geschoren op een kloon van de extrusie: eindkap-
  // vertices schuiven langs de lid-as evenredig met hun afstand tot de
  // buitenrand, zodat twee versteklede elkaar exact op de 45°-diagonaal
  // ontmoeten (zoals de framePolygons uit de payload).
  function makeProfileMember(rect, section, material, orientation, mirrored = false, opts = {}) {
    const length = orientation === "v" ? rect.height : rect.width;
    if (!(length > 0) || !(rect.width > 0) || !(rect.height > 0)) return null;
    const zInner = typeof opts.zInner === "number" ? opts.zInner : frameDepth / 2;
    let geo = getExtrudedGeometry(section, length, mirrored);
    const crossW = orientation === "v" ? rect.width : rect.height;
    const off = (crossW - section.w) / 2;
    const shiftAt = (u) => {
      const fromEdge = opts.side === "top" || opts.side === "right" ? crossW - off - u : off + u;
      return Math.min(crossW, length / 2, Math.max(0, fromEdge));
    };
    if (opts.miterStart || opts.miterEnd) {
      geo = geo.clone();
      const pos = geo.getAttribute("position");
      for (let i = 0; i < pos.count; i++) {
        const shift = shiftAt(pos.getX(i));
        const z = pos.getZ(i);
        if (opts.miterStart && Math.abs(z) < 1e-3) pos.setZ(i, shift);
        else if (opts.miterEnd && Math.abs(z - length) < 1e-3) pos.setZ(i, length - shift);
      }
      pos.needsUpdate = true;
      geo.computeVertexNormals();
    }
    const mesh = new THREE.Mesh(geo, material);
    // The catalogue's interior paths include both air chambers and solids
    // (steel, isolators, cover caps). Show their boundaries on the cut faces.
    if (section.innerWalls?.length) {
      const lines = [];
      for (const ring of section.innerWalls) {
        for (let i = 0; i < ring.length; i++) {
          for (const end of [false, true]) {
            for (const [x, y] of [ring[i], ring[(i+1) % ring.length]]) {
              const u = mirrored ? section.w - (x-section.minU) : x-section.minU;
              const t = end ? length - (opts.miterEnd ? shiftAt(u) : 0) : opts.miterStart ? shiftAt(u) : 0;
              lines.push(u, y - section.minV, t + (end ? 0.01 : -0.01));
            }
          }
        }
      }
      const detail = new THREE.BufferGeometry();
      detail.setAttribute("position", new THREE.Float32BufferAttribute(lines, 3));
      mesh.add(new THREE.LineSegments(detail, sectionLineMat));
    }
    mesh.rotation.order = "ZYX"; // eerst de X-, dan de Z-rotatie op de vector
    const zBinnen = zInner - section.d; // v=0 hier → v=d op zInner
    if (orientation === "v") {
      mesh.rotation.set(Math.PI / 2, 0, 0);
      mesh.position.set(
        rect.x + (rect.width - section.w) / 2, // aanzicht gecentreerd op het lid
        -rect.y, // extrusie start bovenaan en loopt omlaag
        zBinnen
      );
    } else {
      mesh.rotation.set(Math.PI / 2, 0, Math.PI / 2);
      mesh.position.set(
        rect.x, // extrusie start links
        -(rect.y + rect.height) + (rect.height - section.w) / 2,
        zBinnen
      );
    }
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    return mesh;
  }

  const f = kozijn.frame || {};
  const fw = f.frameWidth || 67;
  const baseMat = matKey.startsWith("wood(") ? "wood" : matKey; // wood|pvc|aluminum|wood_aluminum
  const miterCorners = baseMat === "aluminum" || baseMat === "pvc";
  const layoutTree = kozijn.layout || null;
  const memberRefs = {
    top: f.topProfile || f.profile,
    bottom: f.bottomProfile || f.sillProfile || f.profile,
    left: f.leftProfile || f.profile,
    right: f.rightProfile || f.profile,
  };
  const memberDefaults = {
    top: { orientation: "h", mirrored: true },
    bottom: { orientation: "h", mirrored: false },
    left: { orientation: "v", mirrored: false },
    right: { orientation: "v", mirrored: true },
  };

  const defaultSection = getSection(f.profile);
  const frameInner = frameDepth / 2;
  const timber = baseMat === "wood" || baseMat === "wood_aluminum";
  const rebateH = f.profileSnapshot?.sponningHoogte || defaultSection?.def?.sponning?.depth ||
    geometry.sponningHoogte || (timber ? 17 : baseMat === "pvc" ? 20 : 25);

  function simpleRebate(width, depth, height, ledge, outside) {
    const h = Math.min(height, Math.max(0, width - 1));
    const v = Math.max(1, Math.min(depth - 1, ledge));
    const pts = [[0, 0], [width, 0], [width, v], [width-h, v], [width-h, depth], [0, depth]];
    if (outside) for (const p of pts) p[1] = depth - p[1];
    return sectionFromPoints(["rebate", width, depth, h, v, outside].join("|"), pts);
  }

  function fitSection(section, width, depth = section?.d || frameDepth, fit) {
    if (!section) {
      const fallbackLedge = timber ? depth - (f.profileSnapshot?.sponningDiepte || 51) : Math.min(25, depth * 0.4);
      const ledge = fit?.rebateDepth > 0 ? depth - fit.rebateDepth : fallbackLedge;
      const h = fit?.contours?.rebateHeight ?? (fit ? (fit.rebate.width - fit.day.width) / 2 : rebateH);
      return simpleRebate(width, depth, h, ledge, false);
    }
    if (section.w === width && section.d === depth) return section;
    // Keep the rebate at catalogue size when fitting a wider/narrower stock.
    const h = Math.min(section.def?.sponning?.depth || rebateH, width / 2, section.w / 2);
    const body = section.w - h;
    const resize = ([x, y]) => {
      const u = x - section.minU;
      return [u >= body ? width - (section.w - u) : u * (width-h) / body,
        (y-section.minV) * depth / section.d];
    };
    return { ...sectionFromPoints([section.id, width, depth].join("|"), section.pts.map(resize)), def: section.def,
      innerWalls: section.innerWalls?.map(r => r.map(resize)) };
  }

  function flipDepth(section) {
    const flip = ([u, v]) => [u, 2 * section.minV + section.d - v];
    return { ...section, id: section.id + "|flipped", pts: section.pts.map(flip),
      innerWalls: section.innerWalls?.map(r => r.map(flip)) };
  }

  function orientPocket(section, outside) {
    if (outside == null) return section;
    const channel = preferredChannel(section, Math.min(2, section.w / 4), outside);
    if (!channel) return section;
    const opensOutside = channel[0] < 0.01, opensInside = channel[1] > section.d - 0.01;
    if (outside ? !opensInside || opensOutside : !opensOutside || opensInside) return section;
    return flipDepth(section);
  }

  function findSashSection(cell, isDoor) {
    const selected = getSection(cell?.sashProfile);
    if (selected) return selected;
    const preferred = isDoor ? ["deur-meranti-67x114", "deurhout-54x118"]
      : ["raam-meranti-69x90", "raamhout-69x90", "raamhout-54x78"];
    for (const id of timber ? preferred : []) {
      const section = getSection({ id });
      if (section) return section;
    }
    if (defaultSection?.def?.applicableAs?.includes("sash")) return defaultSection;
    const candidates = profileList.filter(p => p.material === baseMat && p.applicableAs?.includes("sash"));
    const candidate = candidates.find(p => p.manufacturer === defaultSection?.def?.manufacturer) || candidates[0];
    return getSection(candidate) || defaultSection;
  }

  const leafCount = node => node.kind === "leaf" ? 1 : (node.children || []).reduce((n, c) => n + leafCount(c.node), 0);
  const aligned = !layoutTree || leafCount(layoutTree) === kozijn.cells?.length;
  const cells = (geometry.cellRects || []).map(cr => {
    const v = cr.vulling;
    const cell = aligned ? kozijn.cells?.[cr.cellIndex] : null;
    const type = v?.type || cell?.panelType || "fixed_glass";
    const door = type === "deur" || type === "door";
    const operable = door || type === "raam" || OPERABLE_TYPES.has(type);
    const fits = cellFits(cr, cell, f, geometry);
    const fit = fits[0];
    const source = operable ? findSashSection(cell, door) : defaultSection;
    const snap = operable ? cell?.sashProfileSnapshot || f.profileSnapshot : f.profileSnapshot;
    const side = v?.glaslat || cell?.glaslat?.position || snap?.beadPosition;
    const sourceChannel = source && preferredChannel(source, 4, false);
    const outside = side ? side === "buiten" : sourceChannel
      ? sourceChannel[0] < 0.01 && sourceChannel[1] < source.d - 0.01
      : fit?.beadPosition === "buiten";
    const opensOut = fit?.contours?.sashOpensOutward ?? (v ? v.opensOutward ?? (v.openType === "uitzet") :
      cell?.openingDirection === "outward" || (type === "top_hung" && cell?.openingDirection !== "inward"));
    const w = fit?.contours?.sashWidth ?? (fit?.sashOuter ? (fit.sashOuter.width - fit.day.width) / 2 : 0);
    const system = fit?.systemSeals;
    const depth = cell?.sashDepth || (system && (snap?.sashBuildDepth || source?.def?.ventDepth));
    const blank = operable && w > 0 ? fitSection(source, w, depth || undefined, fit) : null;
    const sash = blank && orientPocket(blank, outside);
    // The opening direction controls the frame stop, independently of the
    // side from which glass is installed in the movable sash.
    let sashInner = opensOut && sash ? -frameInner - SASH_PROUD_MM + sash.d : frameInner + SASH_PROUD_MM;
    // A projecting lip closes on a second, shallower shoulder. Measure its
    // underside on the actual frame-facing contour (u = 0 in sash coordinates).
    const lip = sash?.def?.sponning?.opdekWidth ?? sash?.def?.sponning?.opdek_width;
    let lipContact = null, sashParts = sash ? [sash] : [];
    if (lip > 0 && sash.def.material === "wood") {
      const frameChannel = s => preferredChannel({ ...s,
        pts: s.pts.map(([u,v]) => [2*s.minU + s.w-u,v]) }, Math.min(2, lip/2), opensOut);
      let frameSide = blank, channel = frameChannel(blank);
      if (channel && (opensOut ? channel[0] === 0 : channel[1] === blank.d)) {
        frameSide = flipDepth(blank);
        channel = frameChannel(frameSide);
      }
      if (channel && (opensOut ? channel[1] === sash.d : channel[0] === 0)) {
        lipContact = sashInner - sash.d + channel[opensOut ? 0 : 1];
        sashParts = sashSides(sash, frameSide);
      }
    }
    const closure = fit?.contours;
    const clearance = closure?.sashClearance;
    const seatDepth = closure?.sashSeatDepth ?? defaultSection?.def?.aanslagSponning?.depth;
    const frameMaterial = defaultSection?.def?.material || baseMat;
    let seatStop = null;
    if (sash && closure?.sashSeat && closure.sashOverlap > 0 && seatDepth > 0 && frameMaterial === "wood") {
      const sign = opensOut ? -1 : 1;
      seatStop = sign * (frameInner - Math.min(frameDepth - 1, seatDepth));
      const face = seatStop + sign * SEAL_MM;
      const back = opensOut ? sashInner : sashInner - sash.d;
      const returnDepth = sign * (face - back);
      const returnWidth = closure.sashOverlap + (clearance ?? defaultSection?.def?.aanslagSponning?.clearance ?? 4);
      if (lipContact != null) {
        // Keep the factory lip and seal groove intact. Its underside seats
        // against the shallow frame stop; its body enters the second pocket.
        sashInner += face - lipContact;
        lipContact = face;
      } else if ((sash.def?.material || baseMat) === "wood" && returnDepth > 0 && returnDepth < sash.d &&
                 returnWidth < sash.w - closure.rebateHeight) {
        sashParts = sashReturn(sash, returnWidth, face - (sashInner - sash.d), opensOut);
      } else {
        sashInner += face - back;
      }
    }
    let systemFit = null;
    if (system?.hinged && sash && miterCorners) {
      const section = orientPocket(fitSection(defaultSection,fw),opensOut);
      const overlap = closure?.sashOverlap ?? system.sashOverlapMm ?? Math.max(0,cr.rect.x-fit.sashOuter.x);
      systemFit = systemClosure(section,sash,frameInner,sashInner,overlap,system);
      if (systemFit) {
        sashParts = systemFit.parts;
        systemFit.byStock = new Map([[section.id,systemFit]]);
        systemFit.bySide = new Map();
      }
    }
    return { rect: cr.rect, opening: cr.openingContour, fits, cellIndex: cr.cellIndex, cell, v, type, door, sash, outside, opensOut,
      sashInner, sashParts, lip, lipContact, seatStop, clearance, seats: [], systemFit, hardwareLeaves:cr.hardwareLeaves };
  });

  function rebateAtMember(section, job, side) {
    if (!job || !job.fits.length) return [section];
    const fit = job.fits[0];
    if (!job.sash) {
      const stock = orientPocket(section, job.outside);
      job.seats.push({ section: stock, side });
      return [stock];
    }
    const last = job.fits.at(-1);
    const outer = fit.sashOuter;
    const overlap = fit.contours?.sashOverlap ?? {
      left: job.rect.x - outer.x,
      right: last.sashOuter.x + last.sashOuter.width - job.rect.x - job.rect.width,
      top: job.rect.y - outer.y,
      bottom: outer.y + outer.height - job.rect.y - job.rect.height,
    }[side];
    const stock = orientPocket(section, job.opensOut);
    const material = stock.def?.material || baseMat;
    if (material !== "wood") {
      if (job.systemFit) {
        const {byStock,bySide} = job.systemFit;
        if (!byStock.has(stock.id)) {
          byStock.set(stock.id,systemClosure(stock,job.sash,frameInner,job.sashInner,overlap,fit.systemSeals));
        }
        bySide.set(side,byStock.get(stock.id));
        return [stock];
      }
      // PVC, aluminium and clad sections have manufactured channels. Set
      // the leaf against the existing stop without machining a new slot.
      if (overlap > 0) {
        const channel = preferredChannel(stock, overlap, job.opensOut);
        if (channel) {
          const stop = frameInner - stock.d + channel[job.opensOut ? 1 : 0];
          job.sashInner = job.opensOut ? Math.min(job.sashInner, stop - SEAL_MM)
            : Math.max(job.sashInner, stop + SEAL_MM + job.sash.d);
        }
      }
      return [stock];
    }
    const clearance = job.clearance ?? section.def?.aanslagSponning?.clearance ?? 4;
    const stop = job.opensOut ? job.sashInner + SEAL_MM : job.sashInner - job.sash.d - SEAL_MM;
    const v = stop - (frameInner - section.d);
    const cutHeight = Math.min(stock.w-1, Math.max(0, overlap) + clearance);
    if (job.seatStop != null) {
      const seat = job.seatStop - (frameInner - section.d);
      const stockWidth = Math.min(stock.w, Math.max(cutHeight, stock.def?.sponning?.depth || rebateH));
      const shallow = closingStock(stock, stockWidth).flatMap(s => machineRebate(s, cutHeight, seat, job.opensOut));
      if (job.lipContact == null) return shallow;
      const bodyHeight = Math.max(0, overlap - job.lip) + clearance;
      return shallow.flatMap(s => machineRebate(s, bodyHeight, v, job.opensOut));
    }
    if (job.lipContact != null && overlap > clearance) {
      const underLip = job.lipContact + (job.opensOut ? SEAL_MM : -SEAL_MM) - (frameInner-stock.d);
      const bodyHeight = Math.max(0, overlap - job.lip) + clearance;
      const parts = machineRebate(stock, cutHeight, underLip, job.opensOut)
        .flatMap(part => machineRebate(part, bodyHeight, v, job.opensOut));
      // A glazing section is already hollow at the front of the shallow lip.
      // Complete its sash stop only within that empty channel.
      const channel = preferredChannel(stock, overlap, job.opensOut);
      if (channel && overlap > bodyHeight) {
        const lo = job.opensOut ? Math.max(underLip, channel[0]) : channel[0];
        const hi = job.opensOut ? channel[1] : Math.min(underLip, channel[1]);
        if (hi > lo) {
          const x0 = stock.minU + stock.w - overlap, x1 = stock.minU + stock.w - bodyHeight;
          const y0 = stock.minV + lo, y1 = stock.minV + hi;
          parts.push({ ...stock, id: [stock.id, "lip", x0,x1,y0,y1].join("|"), innerWalls: [],
            pts: [[x0,y0], [x1,y0], [x1,y1], [x0,y1]] });
        }
      }
      return parts;
    }
    return machineRebate(stock, cutHeight, v, job.opensOut);
  }

  function addMember(rect, side, base, opts = {}, half = false) {
    const { orientation, mirrored } = memberDefaults[side];
    const vertical = orientation === "v";
    const width = vertical ? rect.width : rect.height;
    if (!(width > 0) || !(vertical ? rect.height : rect.width)) return;
    const runs = memberRuns(rect, side, cells);
    for (const [i, run] of runs.entries()) {
      const stock = fitSection(base, width, undefined, !run.cell?.sash && run.cell?.fits[0]);
      const r = { ...run.rect };
      if (half) {
        if (vertical) { r.width /= 2; if (side === "left") r.x += r.width; }
        else { r.height /= 2; if (side === "top") r.y += r.height; }
      }
      const job = run.cell;
      const cutParts = rebateAtMember(stock, job, side);
      const miterStart = i === 0 && opts.start, miterEnd = i === runs.length - 1 && opts.end;
      let pocket;
      if ((stock.def?.material || baseMat) === "wood" && job?.fits.length) {
        const first = job.fits[0], last = job.fits.at(-1);
        if (first.sashOuter) {
          const pair = { ...first.sashOuter, width: last.sashOuter.x + last.sashOuter.width - first.sashOuter.x };
          pocket = insetRectBy(pair, -(stock.def?.aanslagSponning?.clearance || 4));
        } else pocket = first.rebate;
      }
      for (const part of machiningSpans(r, vertical, pocket, miterStart, miterEnd)) {
        const pieces = part.cut ? cutParts : stoppedSection(stock);
        for (const piece of pieces) {
          const section = half ? openingHalf(piece) : piece;
          if (section.pts.length < 3) continue;
          const mesh = makeProfileMember(part.rect, section, half ? dividerMat : frameMat, orientation, mirrored,
            { side, miterStart: part.cut && miterStart, miterEnd: part.cut && miterEnd });
          if (mesh) kozijnGroup.add(tag(mesh, half ? "divider" : "frame-member"));
        }
      }
    }
  }

  // Frame members — de payload is leidend (leden mogen elkaar niet
  // doorsnijden of gaten laten):
  //  1. Stepped (melkmeisje, layout met 'buiten'-vak): framePolygons zijn
  //     ringen per vak; decomponeer hun unie (min vakken/dividers) tot
  //     disjuncte leden. Zo bestaat de dorpel onder het zijlicht, loopt er
  //     geen lid door de borstweringzone en ontstaan geen dubbele
  //     coplanaire boxen (z-fighting).
  //  2. Per-lid polygonen (verstek / gemengde hoeken): volgorde [top,
  //     bottom, left, right]; lidlengte = bbox van de polygon, diagonale
  //     uiteinden worden als 45°-verstek geschoren.
  //  3. Anders: de klassieke frameRects (byte-identiek grid-pad).
  const framePolys = Array.isArray(geometry.framePolygons) ? geometry.framePolygons : [];
  const steppedFrame = framePolys.length > 0 && layoutTree && layoutHasBuiten(layoutTree);
  const curve = geometry.curvedFrame;
  const timberCuts = geometry.curvedTimber?.length ? timberPlan(geometry.curvedTimber) : null;
  let masonryGaps=[];
  function addSweep(outer, inner, sections, z, mat, role, pathAtDepth, plan) {
    const at = typeof sections === "function" ? sections : () => sections;
    for (const mesh of sweepRing(THREE, outer, inner, at, z, mat, role, pathAtDepth, plan?.cuts, sectionLineMat)) kozijnGroup.add(mesh);
  }
  function addSlab(outer, holes, z, thickness, mat, role) {
    const slab = contourSlab(THREE, outer, holes, z, thickness, mat);
    if (slab) kozijnGroup.add(tag(slab, role));
  }
  function addCurvedFrame() {
    const outer = contourPoints(curve.outer);
    if (outer.length < 3) return;
    const openings = cells.filter(j => j.opening?.segments.length);
    const split = geometry.vDividers?.length || geometry.hDividers?.length || layoutHasBuiten(layoutTree);
    // Between adjacent pockets leave a continuous web, also where a mullion
    // terminates obliquely against the arch. Bound the cuts by actual stock.
    const widths = [fw, ...(geometry.vDividers || []).map(r => r.width), ...(geometry.hDividers || []).map(r => r.height)];
    const band = Math.min(...widths) * 0.48;
    const outsideBand = fw * 0.46;
    const web = split ? insetPolygon(outer, outsideBand) : outer;
    const day = contourPoints(curve.day);
    const dividerParts=geometry.curvedDividerTimber ??
      (layoutTree && !geometry.cellRects?.length || geometry.curvedDividers?.every(c=>!c.segments.length) ? [] : null);
    const dividers = split && dividerParts?.map(dividerFootprint);
    const frameParts = timberCuts && geometry.curvedTimber.map(frameBandFootprint);
    if (dividers && layoutHasBuiten(layoutTree)) {
      masonryGaps=masonrySpans(day,dividers,geometry.vDividers,geometry.hDividers);
    }
    const dividerFaces = new Map();
    function dividerStock(part, job) {
      const {vertical,sourceIndex}=part.member;
      const source=(vertical ? geometry.vDividers : geometry.hDividers)[sourceIndex];
      const ref=(vertical ? kozijn.grid?.columns : kozijn.grid?.rows)?.[sourceIndex+1]?.dividerProfile;
      return fitSection(getSection(!layoutTree && ref) || defaultSection,
        vertical ? source.width : source.height, undefined, !job?.sash && job?.fits[0]);
    }
    function cutFrame(mesh, parts, owner) {
      let faces=meshFaces(mesh);
      for (const part of parts) faces=clipStock(faces,part,false);
      if (owner) faces=clipStock(faces,owner,true);
      const cut=stockMesh(THREE,faces,mesh.material,"frame-member");
      mesh.geometry.dispose();
      if (cut) {
        Object.assign(cut.userData,mesh.userData);
        for (const line of [...mesh.children]) cut.add(line);
      }
      return cut;
    }
    function pieceSection(piece, side) {
      if (piece?.role === "arc") {
        return getSection(piece.profileSide === "bottom" ? f.bottomProfile || f.profile : memberRefs.top);
      }
      return getSection(memberRefs[piece?.role || side]);
    }
    function withPiece(section, piece) {
      return piece ? { ...section, id: section.id + "|" + piece.role + piece.index, timber: piece } : section;
    }
    const frameOffsets = new Map();
    const pockets = [];
    for (const job of openings) {
      const opening = contourPoints(job.opening);
      const paths = job.fits[0]?.contours;
      const planes = job.opening.segments.filter(s => s.kind === "line").map(s => {
        const dx = s.to[0]-s.from[0], dy = s.to[1]-s.from[1], length = Math.hypot(dx,dy);
        const nx = dy/length, ny = -dx/length;
        return { nx, ny, limit: nx*s.from[0] + ny*s.from[1] };
      });
      const pocketAt = depth => {
        if (!job.sash && paths && Math.abs(depth - paths.rebateHeight) < 1e-6) return contourPoints(paths.rebate);
        if (job.sash && paths?.sashSeat && Math.abs(depth - paths.sashOverlap - paths.sashClearance) < 1e-6) {
          return contourPoints(paths.sashSeat);
        }
        if (!frameOffsets.has(depth)) frameOffsets.set(depth, insetPolygon(day, -depth));
        return planes.reduce((polygon, p) => clipByLine(polygon, p.nx,p.ny,p.limit+depth), frameOffsets.get(depth));
      };
      const pocket = split ? pocketAt(band) : outer;
      pockets.push(pocket);
      const pocketBox=ringBBox(pocket);
      const neighbours=dividers && dividers.filter(part=>touchesPocket(part,pocket,pocketBox));
      const wallGaps=masonryGaps.filter(part=>touchesPocket(part,pocket,pocketBox));
      const stocks = new Map();
      const sectionsAt = (x,y,midX=x,midY=y) => {
        const { side } = contourSide(job.opening, x, y);
        const vertical = side === "left" || side === "right";
        const dividers = (vertical ? geometry.vDividers : geometry.hDividers) || [];
        const offset = vertical ? 0 : geometry.vDividers?.length || 0;
        const dividerIndex = !neighbours && split && contourSide(curve.day, x, y).distance > 0.2
          ? dividers.findIndex((r,i) => {
            const actual = geometry.curvedDividers?.[offset+i];
            if (actual) return contourSide(actual,x,y).distance < 0.2;
            return x >= r.x-0.2 && x <= r.x+r.width+0.2 && y >= r.y-0.2 && y <= r.y+r.height+0.2;
          }) : -1;
        const piece = dividerIndex < 0 ? timberCuts?.at(midX,midY) || (neighbours && timberCuts?.nearest(midX,midY)) : null;
        const key = side + "|" + dividerIndex + "|" + (piece ? piece.role + piece.index : "");
        if (stocks.has(key)) return stocks.get(key);
        let base = pieceSection(piece,side) || defaultSection, width = fw;
        if (dividerIndex >= 0) {
          const ref = (vertical ? kozijn.grid?.columns : kozijn.grid?.rows)?.[dividerIndex+1]?.dividerProfile;
          base = getSection(!layoutTree && ref) || defaultSection;
          width = vertical ? dividers[dividerIndex].width : dividers[dividerIndex].height;
        }
        const stock = fitSection(base, width, undefined, !job.sash && job.fits[0]);
        const parts = rebateAtMember(stock, job, side).map(s => {
          const strip = split ? stockStrip(s, width-band, band) : s;
          return dividerIndex >= 0 ? { ...strip, id: strip.id + "|divider", meshRole: "divider" } : withPiece(strip,piece);
        });
        stocks.set(key, parts);
        return parts;
      };
      if (!neighbours) {
        addSweep(pocket, opening, sectionsAt, frameInner, frameMat, "frame-member", pocketAt, timberCuts);
        continue;
      }
      // A rebated band crosses the butt joint into the adjoining stock. Fit
      // both profiles right up to the same nominal day and cap their ends.
      if (!timberCuts || pocket.some(([x,y])=>timberCuts.at(x,y))) {
        for (const owner of frameParts?.filter(o=>touchesPocket(o,pocket,pocketBox)) ?? [null]) {
          const piece=owner?.member;
          const stock=piece && fitSection(pieceSection(piece) || defaultSection,fw,undefined,!job.sash && job.fits[0]);
          const strips=piece && rebateAtMember(stock,job,piece.profileSide).map(s=>withPiece(stockStrip(s,fw-band,band),piece));
          for (const mesh of sweepRing(THREE,pocket,opening,strips ? ()=>strips : sectionsAt,frameInner,frameMat,"frame-member",
            pocketAt,timberCuts?.cuts,sectionLineMat)) {
            const cut=cutFrame(mesh,neighbours.concat(wallGaps),owner);
            if (cut) kozijnGroup.add(cut);
          }
        }
      }
      for (const part of neighbours) {
        const stock=dividerStock(part,job);
        const [x0,y0]=part.box;
        const side = part.member.vertical ? (job.rect.x<x0 ? "right" : "left") : (job.rect.y<y0 ? "bottom" : "top");
        const sections=rebateAtMember(stock,job,side).map(s=>stockStrip(s,stock.w-band,band));
        const collector=dividerFaces.get(part) || [];
        for (const mesh of sweepRing(THREE,pocket,opening,()=>sections,frameInner,dividerMat,"divider",pocketAt)) {
          collector.push(...clipStock(meshFaces(mesh),part,true));
          mesh.geometry.dispose();
        }
        dividerFaces.set(part,collector);
      }
    }
    if (split) {
      let frameOnlySlab;
      if (dividers) {
        const webSlab=contourSlab(THREE,web,pockets,frameInner-frameDepth,frameDepth,frameMat);
        if (webSlab) {
          const faces=meshFaces(webSlab);
          let frameFaces=faces;
          for (const part of dividers) {
            const core=clipStock(faces,part,true);
            const depth=dividerStock(part).d;
            const scaled=core.map(face=>face.map(([x,y,z])=>[x,y,frameInner+(z-frameInner)*depth/frameDepth]));
            const mesh=dividerMesh(THREE,scaled.concat(dividerFaces.get(part) || []),part,dividerMat,sectionLineMat);
            if (mesh) kozijnGroup.add(mesh);
            frameFaces=clipStock(frameFaces,part,false);
          }
          for (const wall of masonryGaps) frameFaces=clipStock(frameFaces,wall,false);
          webSlab.geometry.dispose();
          frameOnlySlab=stockMesh(THREE,frameFaces,frameMat,"frame-member");
        }
      }
      if (timberCuts) {
        const depthOf = piece => (pieceSection(piece) || defaultSection)?.d || frameDepth;
        if (frameOnlySlab || !dividers) {
          for (const mesh of splitTimberWeb(THREE,web,pockets,day,frameInner-frameDepth,frameDepth,frameMat,sectionLineMat,timberCuts,depthOf,frameOnlySlab)) kozijnGroup.add(mesh);
        }
      } else if (frameOnlySlab) kozijnGroup.add(frameOnlySlab);
      else if (!dividers) addSlab(web, pockets, frameInner-frameDepth, frameDepth, frameMat, "frame-member");
      const edges = new Map();
      addSweep(outer, web, (x,y,midX=x,midY=y) => {
        const piece = timberCuts?.at(midX,midY);
        const side = contourSide(curve.outer,x,y).side, key = piece || side;
        if (!edges.has(key)) edges.set(key, [withPiece(stockStrip(fitSection(pieceSection(piece,side) || defaultSection, fw), 0, outsideBand),piece)]);
        return edges.get(key);
      }, frameInner, frameMat, "frame-member", undefined, timberCuts);
    } else if (!openings.length) {
      addSlab(outer, [], frameInner-frameDepth, frameDepth, frameMat, "frame-member");
    }
  }
  if (curve) {
    addCurvedFrame();
    machineTimberJoints(THREE, kozijnGroup, geometry.curvedDividerTimber, frameInner, geometry.archJoints);
  } else if (steppedFrame) {
    const ow = f.outerWidth || geometry.outerRect?.width || 0;
    const oh = f.outerHeight || geometry.outerRect?.height || 0;
    const vakRects = (geometry.cellRects || []).map((c) => c.rect);
    const holeRects = [
      ...vakRects,
      ...(geometry.vDividers || []),
      ...(geometry.hDividers || []),
    ];
    const rings = framePolys.map(ringBBox);
    for (const m of decomposeSteppedFrame(rings, holeRects)) {
      const orientation = m.width >= m.height ? "h" : "v";
      const adj = rectAdjacency(m, vakRects);
      let side;
      if (orientation === "h") {
        // Vak boven het lid (2D) → het gedraagt zich als onderdorpel;
        // vak eronder → bovendorpel. Sponning wijst zo naar het vak.
        if (adj.top && !adj.bottom) side = "bottom";
        else if (adj.bottom && !adj.top) side = "top";
        else side = m.y < oh / 2 ? "top" : "bottom";
      } else {
        if (adj.right && !adj.left) side = "left";
        else if (adj.left && !adj.right) side = "right";
        else side = m.x < ow / 2 ? "left" : "right";
      }
      addMember(m, side, getSection(memberRefs[side]));
    }
  } else if (framePolys.length === 4) {
    ["top", "bottom", "left", "right"].forEach((side, i) => {
      const poly = framePolys[i];
      addMember(ringBBox(poly), side, getSection(memberRefs[side]),
        miterEndsFor(poly, memberDefaults[side].orientation));
    });
  } else {
    (geometry.frameRects || []).forEach((rect, i) => {
      const side = ["top", "bottom", "left", "right"][i];
      if (side) addMember(rect, side, getSection(memberRefs[side]));
    });
  }

  // 'Buiten'-vakken (melkmeisje-borstwering): metselwerkvolume in plaats van
  // een zwart gat, doorgetrokken tot de buitenrand aan zijden zonder kozijn.
  if (layoutTree && geometry.innerRect && layoutHasBuiten(layoutTree)) {
    const masonryMat = new THREE.MeshStandardMaterial({
      color: MASONRY_COLOR, roughness: 0.95, metalness: 0.0,
    });
    for (const gap of masonryGaps) addSlab(gap.loop,[],-frameInner,frameDepth,masonryMat,"masonry");
    for (const b of collectBuitenRects(layoutTree, geometry.innerRect, fw)) {
      const r = expandBuitenRect(b, geometry.innerRect, fw);
      if (curve) {
        const boundary = clipToRect(contourPoints(curve.outer), r);
        if (boundary.length >= 3) {
          for (const mesh of [...kozijnGroup.children]) {
            if (mesh.userData.role !== "masonry") trimAtMasonry(THREE, mesh, r);
          }
          addSlab(boundary, [], -frameInner, frameDepth, masonryMat, "masonry");
        }
        continue;
      }
      // 1 mm terugliggend aan beide dieptezijden: geen coplanaire vlakken
      // met muur (achter) of kozijnbinnenvlak (voor).
      kozijnGroup.add(tag(makeBox(r, frameDepth - 2, masonryMat, 0), "masonry"));
    }
  }

  // In system grids the transoms butt against continuous mullions. A free
  // subdivision has its own split order and already has disjoint rectangles.
  function dividerSegments(rect, vertical, others) {
    const [along,length,across,width] = vertical ? ["y","height","x","width"] : ["x","width","y","height"];
    const end = rect[along]+rect[length];
    const stops = others.filter(r => r[across] <= rect[across] && r[across]+r[width] >= rect[across]+rect[width] &&
      r[along] < end && r[along]+r[length] > rect[along]).sort((a,b)=>a[along]-b[along]);
    const segments = [];
    let start = rect[along];
    for (const stop of stops) {
      if (stop[along] > start) segments.push({...rect,[along]:start,[length]:stop[along]-start});
      start = Math.max(start,stop[along]+stop[length]);
    }
    if (end > start) segments.push({...rect,[along]:start,[length]:end-start});
    return segments;
  }
  const continuousMullions = miterCorners && !layoutTree;
  if (!curve) for (const vertical of [true,false]) {
    const members = (vertical ? geometry.vDividers : geometry.hDividers) || [];
    const cross = (vertical ? geometry.hDividers : geometry.vDividers) || [];
    const grid = vertical ? kozijn.grid?.columns : kozijn.grid?.rows;
    for (const [i,rect] of members.entries()) {
      const profile = getSection(!layoutTree && grid?.[i+1]?.dividerProfile) || defaultSection;
      const segments = vertical === continuousMullions ? [rect] : dividerSegments(rect,vertical,cross);
      for (const segment of segments) for (const side of vertical ? ["left","right"] : ["top","bottom"]) {
        addMember(segment,side,profile,{},true);
      }
    }
  }

  function addRing(rect, section, zInner, mat, role, miter = false, stilesThrough = false, sash = null) {
    const w = section.w;
    const cutV = miter || stilesThrough ? 0 : w;
    const cutH = !miter && stilesThrough ? w : 0;
    const members = {
      top: { x: rect.x + cutH, y: rect.y, width: rect.width - 2*cutH, height: w },
      bottom: { x: rect.x + cutH, y: rect.y + rect.height-w, width: rect.width - 2*cutH, height: w },
      left: { x: rect.x, y: rect.y + cutV, width: w, height: rect.height - 2*cutV },
      right: { x: rect.x + rect.width-w, y: rect.y + cutV, width: w, height: rect.height - 2*cutV },
    };
    for (const [side, r] of Object.entries(members)) {
      const { orientation, mirrored } = memberDefaults[side];
      for (const part of machiningSpans(r, orientation === "v", sash?.pocket, miter, miter)) {
        const stocks = sash?.bySide?.get(side)?.parts ?? sash?.profiles;
        const pieces = !sash || sash.meetingSide===side ? [section] : part.cut ? stocks : stoppedSection(stocks.at(-1));
        for (const piece of pieces) {
          const mesh = makeProfileMember(part.rect, piece, mat, orientation, mirrored,
            { side, zInner, miterStart: miter, miterEnd: miter });
          if (mesh) kozijnGroup.add(tag(mesh, role));
        }
      }
    }
  }

  function ringSection(w, d, bevel = 0, outside = false) {
    const b = Math.min(bevel, w / 2, d / 2);
    const pts = b > 0 ? [[0,0], [w,0], [w,d-b], [w-b,d], [0,d]] : [[0,0], [w,0], [w,d], [0,d]];
    if (outside) for (const p of pts) p[1] = d - p[1];
    return sectionFromPoints(["ring", w, d, b, outside].join("|"), pts);
  }

  function addGlazing(job, fit) {
    const { day, glass, rebate } = fit;
    if (!(glass?.width > 0 && glass.height > 0 && day?.width > 0 && day.height > 0)) return;
    const hostInner = job.sash ? job.sashInner : frameInner;
    const seats = job.sash ? [{ section: job.sash }] : job.seats;
    if (!seats.length) seats.push({ section: fitSection(defaultSection, fw) });
    const paths = fit.contours;
    const paneCover = paths ? paths.rebateHeight - paths.edgeClearance : (glass.width - day.width) / 2;
    // A sill upstand or a deeper head can limit the common glazing channel.
    const channels = seats.map(({ section }) => {
      const channel = preferredChannel(section, paneCover, job.outside);
      if (!channel) return null;
      const origin = hostInner - section.d;
      return { lo: origin + channel[0], hi: origin + channel[1],
        outside: channel[0] < 0.01 && channel[1] < section.d - 0.01 };
    }).filter(Boolean);
    const outside = job.outside ?? (channels.length ? channels.every(c => c.outside) : fit.beadPosition === "buiten");
    const lo = channels.length ? Math.max(...channels.map(c => c.lo)) : hostInner;
    const hi = channels.length ? Math.min(...channels.map(c => c.hi)) : hostInner;
    const filling = job.door && job.cell?.panelFilling;
    const thickness = filling?.thicknessMm || fit.glassThickness || job.cell?.glazing?.thicknessMm || 24;
    const sealDepth = fit.sealThickness ?? SEAL_MM;
    const face = outside ? hi - sealDepth : lo + sealDepth;
    const dir = outside ? -1 : 1;
    const beadFace = face + dir * thickness;
    if (paths) {
      addSlab(contourPoints(paths.glass), [], Math.min(face,beadFace), thickness, filling ? doorMat : glassMat, filling ? "door" : "glass");
    } else {
      kozijnGroup.add(tag(makeBox(glass, thickness, filling ? doorMat : glassMat,
        face + dir * thickness / 2), filling ? "door" : "glass"));
    }

    const beadHeight = fit.beadHeight || job.cell?.glaslat?.heightMm || geometry.glaslatHoogte || 17;
    // Use the same stock dimensions as the drawing and cut list. A selected
    // bead can stand proud of the frame when the glazing package is thick.
    const foot = fit.beadWidth || seats[0].section.def?.glaslatWidth || geometry.glaslatBreedte || 15;
    if (foot > 0.1) {
      const z = beadFace + dir * sealDepth + (outside ? 0 : foot);
      const bead = ringSection(beadHeight, foot, 2, outside);
      if (paths) addSweep(contourPoints(paths.rebate), contourPoints(paths.beadInner), [bead], z, beadMat, "bead");
      else addRing(rebate, bead, z, beadMat, "bead", fit.beadMitered ?? true);
    }
    const sealHeight = Math.min(5, Math.max(1, paneCover));
    if (!(sealDepth > 0)) return;
    if (paths) {
      const gasket = ringSection(sealHeight, sealDepth);
      function gasketAt(contour, z) {
        const edge = contourPoints(contour);
        addSweep(insetPolygon(edge, 1-sealHeight), insetPolygon(edge, 1), [gasket], z, sealMat, "seal");
      }
      gasketAt(paths.day, face + (outside ? sealDepth : 0));
      if (foot > 0.1) gasketAt(paths.beadInner, beadFace + (outside ? 0 : sealDepth));
      return;
    }
    const sealOuter = insetRectBy(day, -sealHeight + 1);
    addRing(sealOuter, ringSection(sealHeight, sealDepth), face + (outside ? sealDepth : 0), sealMat, "seal");
    if (foot > 0.1) {
      const beadSealRect = insetRectBy(rebate, beadHeight-sealHeight+1);
      addRing(beadSealRect, ringSection(sealHeight, sealDepth), beadFace + (outside ? 0 : sealDepth), sealMat, "seal");
    }
  }

  function addCurvedSash(job, fit) {
    const paths = fit.contours;
    const outer = contourPoints(paths.sashOuter), day = contourPoints(paths.day);
    const plan = paths.sashTimber?.length ? timberPlan(paths.sashTimber) : null;
    const stocks = new Map();
    function sectionsAt(x, y, midX = x, midY = y) {
      if (job.systemFit && job.fits.length===2) {
        const left=fit===job.fits[0], meetingX=left ? fit.day.x+fit.day.width : fit.day.x;
        if (Math.abs(x-meetingX)<0.15) return [job.sash];
      }
      if (job.systemFit) return job.systemFit.bySide.get(contourSide(paths.day,x,y).side)?.parts || job.sashParts;
      const member = plan?.at(midX, midY);
      if (!member) return job.sashParts;
      if (!stocks.has(member)) {
        stocks.set(member, job.sashParts.map(s => ({ ...s,
          id: s.id + "|" + member.role + member.index, timber: member,
          leaf: { cellIndex: job.cellIndex, side: paths.sashLeaf || null } })));
      }
      return stocks.get(member);
    }
    // Erosion from the outside also preserves a short edge which disappears
    // at the narrower day contour of an off-centre curved leaf.
    const profileAt = depth => Math.abs(depth - paths.rebateHeight) < 1e-6
      ? contourPoints(paths.rebate) : insetPolygon(outer, job.sash.w - depth);
    addSweep(outer, day, sectionsAt, job.sashInner, sashMat, "sash", profileAt, plan);
  }

  function addSystemSeals(job) {
    const {rect} = job;
    const corners = [[rect.x,rect.y],[rect.x+rect.width,rect.y],[rect.x+rect.width,rect.y+rect.height],[rect.x,rect.y+rect.height]];
    const opening = job.opening || {segments:corners.map((from,i)=>({kind:"line",from,to:corners[(i+1)%4]}))};
    const day = contourPoints(opening);
    const systems = new Set([job.systemFit,...job.systemFit.bySide.values()]);
    for (const level of ["inner","middle","outer"]) {
      for (const [field,material,role] of [["stop",frameMat,"system-stop"],["rubber",sealMat,"sash-seal"]]) {
        const shapes = new Map();
        for (const system of systems) {
          const points = system?.seals.find(seal=>seal.level===level)?.[field];
          if (points) shapes.set(system,points);
        }
        if (!shapes.size) continue;
        const common = sectionFromPoints(role,[...shapes.values()].flat());
        const sections = new Map([...shapes].map(([system,pts],i)=>[system,[{...common,pts,id:role+i}]]));
        const at = (x,y) => sections.get(job.systemFit.bySide.get(contourSide(opening,x,y).side) || job.systemFit) || [];
        const path = depth => insetPolygon(day,common.minU+common.w-depth);
        for (const mesh of sweepRing(THREE,path(common.w),path(0),at,common.minV+common.d,material,role,path)) {
          mesh.userData.systemSeal = {cellIndex:job.cellIndex,level};
          kozijnGroup.add(mesh);
        }
      }
    }
  }

  for (const job of cells) {
    if (job.opening && !job.opening.segments.length) continue;
    if (job.fits.length) {
      const outer = job.fits[0].sashOuter;
      const overlap = job.fits[0].contours?.sashOverlap ?? (outer && job.rect.x - outer.x);
      if (job.systemFit) addSystemSeals(job);
      else if (job.sash && overlap > 1) {
        const contact = job.seatStop != null ? job.seatStop + (job.opensOut ? -SEAL_MM : SEAL_MM) :
          job.lipContact ?? (job.opensOut ? job.sashInner : job.sashInner - job.sash.d);
        const compressionFace = contact + (job.opensOut ? SEAL_MM : 0);
        const sealWidth = Math.min(overlap, 5, job.lipContact == null ? overlap : job.lip / 2);
        const reach = job.lipContact == null ? sealWidth : overlap;
        const seal = ringSection(sealWidth, SEAL_MM);
        if (job.opening) {
          const opening = contourPoints(job.opening);
          addSweep(insetPolygon(opening, -reach), insetPolygon(opening, sealWidth-reach), [seal], compressionFace, sealMat, "sash-seal");
        } else addRing(insetRectBy(job.rect, -reach), seal, compressionFace, sealMat, "sash-seal");
      }
      for (const fit of job.fits) {
        if (job.sash && fit.sashOuter) {
          if (fit.contours) addCurvedSash(job, fit);
          else addRing(fit.sashOuter, job.sash, job.sashInner, sashMat, "sash", miterCorners, true,
            { pocket: fit.rebate, profiles: job.sashParts, bySide:job.systemFit?.bySide,
              meetingSide:job.systemFit && job.fits.length===2 ? fit===job.fits[0] ? "right" : "left" : null });
        }
        addGlazing(job, fit);
      }
      if (job.fits.length === 2 && job.sash) {
        const a = job.fits[0].sashOuter, b = job.fits[1].sashOuter;
        const seam = (a.x + a.width + b.x) / 2;
        const z = job.opensOut ? job.sashInner - job.sash.d - 7.5 : job.sashInner + 7.5;
        if (job.opening) {
          const closure = insetPolygon(contourPoints(job.opening), -overlap);
          addSlab(clipByLine(clipByLine(closure, -1, 0, 20-seam), 1, 0, seam+20), [], z-7.5, 15, sashMat, "sash");
        } else kozijnGroup.add(tag(makeBox({ x: seam-20, y: a.y, width: 40, height: a.height }, 15, sashMat, z), "sash"));
      }
      continue;
    }
    if (!["panel", "paneel", "ventilation", "rooster"].includes(job.type)) continue;
    const ventilation = job.type === "rooster" || job.type === "ventilation";
    const filling = job.v?.filling || job.cell?.panelFilling;
    const thickness = filling?.thicknessMm || frameDepth * 0.6;
    const z = filling?.setbackMm || 0;
    const r = insetRectBy(job.rect, 4);
    const color = FILLING_COLORS[filling?.fillingType] || (ventilation ? FILLING_COLORS.ventilation : PANEL_COLOR);
    const mat = new THREE.MeshStandardMaterial({color, roughness: 0.8});
    const panel = job.opening && insetPolygon(contourPoints(job.opening), 4);
    if (panel) addSlab(panel, [], z-thickness/2, thickness, mat, "panel");
    else kozijnGroup.add(tag(makeBox(r, thickness, mat, z), "panel"));
    if (ventilation) {
      const slats = Math.max(2, Math.min(8, Math.floor(r.height / 90)));
      const slatMat = new THREE.MeshStandardMaterial({ color: 0x6B7178, roughness: 0.5, metalness: 0.3 });
      for (let i = 1; i <= slats; i++) {
        const top = r.y + i*r.height/(slats+1)-4;
        if (panel) {
          const strip = clipByLine(clipByLine(insetPolygon(panel, 6), 0,-1,-top),0,1,top+8);
          addSlab(strip, [], z+thickness/2, 6, slatMat, "panel");
        } else kozijnGroup.add(tag(makeBox({ x: r.x+6, y: top, width: Math.max(1,r.width-12), height: 8 },
          6, slatMat, z + thickness/2 + 3), "panel"));
      }
    }
  }

  addHardware(THREE,kozijnGroup,cells,frameInner,frameDepth);
  const stop = geometry.perimeterStop;
  if (stop?.thicknessMm > 0) {
    addSlab(contourPoints(stop.outer), [contourPoints(stop.inner)],
      -frameInner, stop.thicknessMm, frameMat, "perimeter-stop");
  }

  // Four wall returns leave a real opening, also when orbiting to the exterior.
  const wallMat = new THREE.MeshStandardMaterial({ color: 0xd0c8b8, roughness: 0.9 });
  const outer = geometry.outerRect || { x: 0, y: 0, width: f.outerWidth, height: f.outerHeight };
  const wallDepth = 300;
  const strips = [
    { x: outer.x-300, y: outer.y-200, width: outer.width+600, height: 200 },
    { x: outer.x-300, y: outer.y+outer.height, width: outer.width+600, height: 200 },
    { x: outer.x-300, y: outer.y, width: 300, height: outer.height },
    { x: outer.x+outer.width, y: outer.y, width: 300, height: outer.height },
  ];
  if (curve) {
    const x0 = outer.x-300, x1 = outer.x+outer.width+300, y0 = outer.y-200, y1 = outer.y+outer.height+200;
    addSlab([[x0,y0],[x1,y0],[x1,y1],[x0,y1]], [contourPoints(curve.outer)], -wallDepth-frameInner, wallDepth, wallMat, "wall");
  } else for (const strip of strips) kozijnGroup.add(tag(makeBox(strip, wallDepth, wallMat, -wallDepth/2-frameInner), "wall"));

  return kozijnGroup;
}
