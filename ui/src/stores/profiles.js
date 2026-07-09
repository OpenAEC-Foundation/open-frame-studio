/**
 * Profielbibliotheek store.
 *
 * Laadt alle profieldefinities uit de /profiles directory en
 * biedt ze aan als een doorzoekbare lijst.
 */
import { writable, derived } from "svelte/store";
import { invoke } from "../lib/tauri.js";
import { generateProfileGeometry } from "../lib/profileContour.js";

// Statische bibliotheek uit de profiles/ directory (of embedded fallback)
const libraryCategories = writable([]);

// Eigen/geïmporteerde profielen uit project.custom_profiles
const customProfiles = writable([]);

// Alle profielen, gegroepeerd per categorie — bibliotheek + eigen profielen
export const profileCategories = derived(
  [libraryCategories, customProfiles],
  ([$library, $custom]) =>
    $custom.length
      ? [...$library, { id: "custom", label: "Eigen / geïmporteerd", profiles: $custom }]
      : $library
);

// Platte lijst van alle profielen
export const allProfiles = derived(profileCategories, ($cats) =>
  $cats.flatMap((cat) =>
    cat.profiles.map((p) => ({ ...p, category: cat.id, categoryLabel: cat.label }))
  )
);

// Zoekfilter
export const profileFilter = writable("");

// Gefilterde profielen
export const filteredProfiles = derived(
  [allProfiles, profileFilter],
  ([$all, $filter]) => {
    if (!$filter) return $all;
    const q = $filter.toLowerCase();
    return $all.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        (p.manufacturer || "").toLowerCase().includes(q) ||
        (p.series || "").toLowerCase().includes(q) ||
        p.material.toLowerCase().includes(q)
    );
  }
);

/**
 * Laad de profielbibliotheek.
 * In de browser mock laden we inline data; in Tauri lezen we JSON bestanden.
 */
export async function loadProfiles() {
  try {
    // Embedded profile data for browser preview
    const categories = await fetchProfileData();
    libraryCategories.set(categories);
  } catch (e) {
    console.error("Profielen laden mislukt:", e);
  }
  await refreshCustomProfiles();
}

/**
 * Ververs de eigen/geïmporteerde profielen uit project.custom_profiles.
 * Aanroepen na add_custom_profile-flows en bij project openen/nieuw.
 */
export async function refreshCustomProfiles() {
  try {
    const result = await invoke("get_custom_profiles");
    const list = typeof result === "string" ? JSON.parse(result) : result;
    // Laatste definitie per id wint — add_custom_profile dedupliceert niet.
    const byId = new Map((Array.isArray(list) ? list : []).map((p) => [p.id, p]));
    customProfiles.set([...byId.values()]);
  } catch (e) {
    console.error("Eigen profielen laden mislukt:", e);
  }
}

async function fetchProfileData() {
  const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

  if (isTauri) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const json = await invoke("load_profile_library");
      return JSON.parse(json);
    } catch (e) {
      console.warn("Profielen laden via command mislukt, gebruik fallback:", e);
    }
  }

  return getEmbeddedProfiles();
}

/**
 * Verrijk een profielspec met een doorsnede uit de parametrische generator
 * (ui/src/lib/profileContour.js): buitencontour + innerWalls (kamers, staal,
 * isolatoren, alu-schaal). Hout krijgt de massieve KVT-contour.
 *
 * Conventie (identiek aan profiles/*.json en Viewer3D): u 0=muurzijde ..
 * w=vakzijde, v 0=buitenzijde .. d=binnenzijde. Sponningmaatketen hout per
 * KVT: sponninghoogte (sponning.depth) 17 mm vast, sponningbreedte
 * (sponning.width/glazingRebate) 51 mm bij vast glas, aanslagsponning 29 mm
 * met 6 mm lucht bij draaiende delen (bron: KVT 12.2/14.01, DTS,
 * TO-binnendetaillering 2024 — zie docs/profielmaten-onderzoek.md).
 */
function withRealSection(p) {
  return { ...p, ...(generateProfileGeometry(p) || {}) };
}

function getEmbeddedProfiles() {
  return [
    {
      id: "wood",
      label: "Hout",
      profiles: [
        { id: "wood-meranti-67x114", name: "Meranti 67x114mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["frame", "sash", "divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "wood-meranti-67x150", name: "Meranti 67x114mm (onderdorpel)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 53, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, waterhol: true, ufValue: 1.8, applicableAs: ["sill"], sponning: { width: 53, depth: 17, position: "binnen", type: "binnensponning", slopeDegrees: 10 } },
        { id: "wood-meranti-67x139-dorpel", name: "Meranti 67x139mm (onderdorpel zwaar)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 139, sightline: 50, glazingRebate: 53, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, waterhol: true, ufValue: 1.7, applicableAs: ["sill"], sponning: { width: 53, depth: 17, position: "binnen", type: "binnensponning", slopeDegrees: 10 } },
        { id: "wood-meranti-90x114-tussenstijl", name: "Meranti 90x114mm (tussenstijl)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 90, depth: 114, sightline: 73, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "wood-accoya-67x114", name: "Accoya 67x114mm", manufacturer: "Generiek", series: "Accoya", material: "wood", materialSubtype: "accoya", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.5, applicableAs: ["frame", "sash", "divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "naaldhout-67x114", name: "Naaldhout (vuren/grenen) 67x114mm", manufacturer: "Generiek", series: "Naaldhout", material: "wood", materialSubtype: "naaldhout", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["frame", "sash", "divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "fsc-hardhout-67x114", name: "FSC Hardhout 67x114mm", manufacturer: "Generiek", series: "FSC Duurzaam", material: "wood", materialSubtype: "fsc_hardhout", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.6, applicableAs: ["frame", "sash", "divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "wood-meranti-67x114-bovendorpel", name: "Meranti 67x114mm (bovendorpel, waterhol)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, waterhol: true, ufValue: 1.8, applicableAs: ["frame"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "wood-meranti-67x114-tussendorpel", name: "Meranti 67x114mm (tussendorpel/kalf)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, waterhol: true, ufValue: 1.8, applicableAs: ["divider"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "wood-meranti-glasroede-32x12", name: "Meranti glasroede 32x12mm (opgelegd)", manufacturer: "Generiek", series: "Roede", material: "wood", materialSubtype: "meranti", width: 32, depth: 12, sightline: 32, glazingRebate: 0, ufValue: 0, applicableAs: ["divider"] },
      ].map(withRealSection),
    },
    {
      id: "aluminum",
      label: "Aluminium",
      profiles: [
        { id: "reynaers-cs77-standard", name: "Reynaers CS 77", manufacturer: "Reynaers Aluminium", series: "ConceptSystem 77", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 51, depth: 77, ventDepth: 87, sightline: 51, glazingRebate: 25, ufValue: 2.2, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 25, position: "midden" } },
        { id: "reynaers-masterline8", name: "Reynaers MasterLine 8", manufacturer: "Reynaers Aluminium", series: "MasterLine 8", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 53, depth: 77, ventDepth: 87, sightline: 53, sightlineVent: 37, sightlineCombination: 97, glazingRebate: 27, thermalBreakWidth: 40, glazingBeadHeight: 25, glaslatHeight: 25, minGlassThickness: 24, maxGlassThickness: 72, ufValue: 1.9, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 27, position: "midden" } },
        { id: "reynaers-slimline38-classic", name: "Reynaers SlimLine 38 Classic", manufacturer: "Reynaers Aluminium", series: "SlimLine 38", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 33.5, depth: 99, ventDepth: 86, sightline: 33.5, sightlineVent: 23, sightlineCombination: 67, glazingRebate: 13.5, minGlassThickness: 16, maxGlassThickness: 55, ufValue: 1.9, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 13.5, position: "midden" } },
        { id: "schuco-aws70hi", name: "Schuco AWS 70.HI", manufacturer: "Schuco", series: "AWS 70.HI", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 51, depth: 70, ventDepth: 80, sightline: 51, glazingRebate: 25, ufValue: 1.5, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 25, position: "midden" } },
        { id: "schuco-aws75si-plus", name: "Schuco AWS 75.SI+", manufacturer: "Schuco", series: "AWS 75.SI+", material: "aluminum", materialSubtype: "super_insulated", width: 41, depth: 75, ventDepth: 85, sightline: 41, sightlineVent: 59, sightlineCombination: 107, glazingRebate: 25, ufValue: 1.2, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 25, position: "midden" } },
      ].map(withRealSection),
    },
    {
      id: "pvc",
      label: "Kunststof (PVC)",
      profiles: [
        { id: "gealan-s9000", name: "Gealan S 9000", manufacturer: "Gealan", series: "S 9000", material: "pvc", materialSubtype: "6_kamer", chambers: 6, sealingLevels: 3, steelReinforced: true, width: 70, depth: 82.5, sightline: 70, sightlineCombination: 110, sightlineMullion: 92, glazingRebate: 28, glasinval: 18, overslagHoogte: 26, falzluft: 12, stulpNaad: 6, maxGlassThickness: 54, ufValue: 0.89, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 28, position: "midden" } },
        { id: "gealan-kubus", name: "Gealan-KUBUS", manufacturer: "Gealan", series: "KUBUS", material: "pvc", materialSubtype: "7_kamer", chambers: 7, sealingLevels: 3, steelReinforced: true, flushDesign: true, width: 85, depth: 85, sightline: 0, glazingRebate: 30, maxGlassThickness: 54, ufValue: 0.92, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 30, position: "midden" } },
        { id: "veka-softline82", name: "VEKA Softline 82 MD", manufacturer: "VEKA", series: "Softline 82", variant: "MD", material: "pvc", materialSubtype: "7_kamer", chambers: 7, chambersSash: 6, sealingLevels: 3, steelReinforced: true, width: 73, depth: 82, sightline: 73, sightlineVent: 84, sightlineCombination: 124, sightlineMullion: 94, glazingRebate: 28, glasinval: 20, falzluft: 12, stulpNaad: 8, minGlassThickness: 24, maxGlassThickness: 52, glazingBead: { clipHeight: 25, widthBase: 59.5 }, glaslatHeight: 25, wallThicknessVisible: 2.8, wallThicknessOther: 2.5, ufValue: 1.1, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 28, position: "midden" } },
        { id: "veka-softline70", name: "VEKA Softline 70", manufacturer: "VEKA", series: "Softline 70", material: "pvc", materialSubtype: "5_kamer", chambers: 5, sealingLevels: 2, steelReinforced: true, width: 70, depth: 70, sightline: 53, glazingRebate: 24, maxGlassThickness: 40, ufValue: 1.3, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 24, position: "midden" } },
        { id: "kommerling-88md", name: "Kommerling 88 MD", manufacturer: "Kommerling (profine)", series: "88 MD", variant: "MD", material: "pvc", materialSubtype: "7_kamer", chambers: 7, sealingLevels: 3, steelReinforced: true, width: 74, depth: 88, sightline: 74, glazingRebate: 30, maxGlassThickness: 58, ufValue: 0.95, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 30, position: "midden" } },
        { id: "schuco-living82-md", name: "Schüco LivIng 82 MD", manufacturer: "Schüco", series: "LivIng 82", variant: "MD", material: "pvc", materialSubtype: "7_kamer", chambers: 7, sealingLevels: 3, steelReinforced: true, width: 70, depth: 82, sightline: 70, sightlineCombination: 120, minGlassThickness: 16, maxGlassThickness: 54, ufValue: 0.96, applicableAs: ["frame", "sash"] },
      ].map(withRealSection),
    },
    {
      id: "wood-aluminum",
      label: "Hout-Aluminium",
      profiles: [
        { id: "hout-alu-67x114", name: "Hout-Aluminium 67x114mm", manufacturer: "Generiek", series: "Combi", material: "wood_aluminum", materialSubtype: "meranti_alu", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, aluCapDepth: 25, ufValue: 1.5, applicableAs: ["frame", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "hout-alu-67x130", name: "Hout-Aluminium 67x130mm (zwaar)", manufacturer: "Generiek", series: "Combi", material: "wood_aluminum", materialSubtype: "meranti_alu", width: 67, depth: 130, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, aluCapDepth: 25, ufValue: 1.4, applicableAs: ["frame", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" }, aanslagSponning: { depth: 29, clearance: 6 } },
      ].map(withRealSection),
    },
    {
      id: "raamhout",
      label: "Raamhout (draaidelen)",
      profiles: [
        { id: "raam-meranti-54x78", name: "Raamhout Meranti 54x78mm", manufacturer: "Generiek", series: "KVT 54mm", material: "wood", materialSubtype: "meranti", width: 54, depth: 78, sightline: 37, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 2.0, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "raam-meranti-54x90", name: "Raamhout Meranti 54x90mm (triple)", manufacturer: "Generiek", series: "KVT 54mm", material: "wood", materialSubtype: "meranti", width: 54, depth: 90, sightline: 37, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "raam-meranti-69x90", name: "Raamhout Meranti 69x90mm (draaikiep standaard)", manufacturer: "Generiek", series: "Draaikiep", material: "wood", materialSubtype: "meranti", width: 69, depth: 90, sightline: 52, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "raam-accoya-54x78", name: "Raamhout Accoya 54x78mm", manufacturer: "Generiek", series: "KVT 54mm Accoya", material: "wood", materialSubtype: "accoya", width: 54, depth: 78, sightline: 37, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
      ].map(withRealSection),
    },
    {
      id: "deurhout",
      label: "Deurhout",
      profiles: [
        { id: "deur-meranti-67x114", name: "Deurhout Meranti 67x114mm", manufacturer: "Generiek", series: "67mm", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["deur_stijl", "deur_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
        { id: "deur-meranti-67x130", name: "Deurhout Meranti 67x130mm (zwaar)", manufacturer: "Generiek", series: "67mm", material: "wood", materialSubtype: "meranti", width: 67, depth: 130, sightline: 50, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.7, applicableAs: ["deur_stijl", "deur_dorpel", "sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "binnensponning" } },
      ].map(withRealSection),
    },
    {
      id: "glaslat-spouwlat",
      label: "Glaslatten & Spouwlatten",
      profiles: [
        // Glaslatten: notatie voet×hoogte (KVT 12.3.2: hoogte >= 17 en >=
        // sponninghoogte; voet >= 13 binnen / 15 buiten). width = hoogte
        // (aanzicht), depth = voetbreedte (bouwdiepte).
        { id: "glaslat-15x17", name: "Glaslat 15x17mm (norm-minimum)", manufacturer: "Generiek", series: "KVT", material: "wood", materialSubtype: "meranti", width: 17, depth: 15, sightline: 17, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"] },
        { id: "glaslat-17x17", name: "Glaslat 17x17mm (fabrieksstandaard)", manufacturer: "Generiek", series: "KVT", material: "wood", materialSubtype: "meranti", width: 17, depth: 17, sightline: 17, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"] },
        { id: "glaslat-15x28", name: "Glaslat 15x28mm (handelsmaat)", manufacturer: "Generiek", series: "Handel", material: "wood", materialSubtype: "hardhout", width: 28, depth: 15, sightline: 28, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"] },
        { id: "glaslat-17x28", name: "Glaslat 17x28mm (handelsmaat)", manufacturer: "Generiek", series: "Handel", material: "wood", materialSubtype: "hardhout", width: 28, depth: 17, sightline: 28, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"] },
        { id: "glaslat-20x34-5", name: "Glaslat 20x34,5mm (isolatie/triple)", manufacturer: "Generiek", series: "Handel", material: "wood", materialSubtype: "hardhout", width: 34.5, depth: 20, sightline: 34.5, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"] },
        { id: "spouwlat-22x100", name: "Spouwlat 22x100mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 100, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"] },
        { id: "spouwlat-22x120", name: "Spouwlat 22x120mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 120, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"] },
        { id: "spouwlat-22x140", name: "Spouwlat 22x140mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 140, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"] },
      ].map(withRealSection),
    },
    {
      id: "draaikiep",
      label: "Draaikiep (kozijn + raam)",
      profiles: [
        { id: "dk-kozijn-67x114", name: "DK Kozijn 67x114mm", manufacturer: "Generiek", series: "67mm DK", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 29, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["frame"], sponning: { width: 29, depth: 17, position: "binnen", type: "binnensponning", opdek_width: 13, rubber_count: 2 }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "dk-raam-54x78", name: "DK Raamhout 54x78mm (opdek)", manufacturer: "Generiek", series: "54mm DK", material: "wood", materialSubtype: "meranti", width: 54, depth: 78, sightline: 37, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 2.0, applicableAs: ["sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "draaikiep", opdek_width: 13, rubber_count: 2 } },
        { id: "dk-raam-54x90", name: "DK Raamhout 54x90mm Triple (opdek)", manufacturer: "Generiek", series: "54mm DK", material: "wood", materialSubtype: "meranti", width: 54, depth: 90, sightline: 37, glazingRebate: 51, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.8, applicableAs: ["sash"], sponning: { width: 51, depth: 17, position: "binnen", type: "draaikiep", opdek_width: 13, rubber_count: 2 } },
        { id: "dk-kozijn-78x114", name: "DK Kozijn 78x114mm (zwaar)", manufacturer: "Generiek", series: "78mm DK", material: "wood", materialSubtype: "meranti", width: 78, depth: 114, sightline: 61, glazingRebate: 29, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.6, applicableAs: ["frame"], sponning: { width: 29, depth: 17, position: "binnen", type: "binnensponning", opdek_width: 13, rubber_count: 2 }, aanslagSponning: { depth: 29, clearance: 6 } },
        { id: "dk-dubbel-67x114", name: "DK Dubbele sponning 67x114mm", manufacturer: "Generiek", series: "67mm Dubbel", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 50, glazingRebate: 29, glaslatWidth: 17, glaslatHeight: 17, achterhout: 13, ufValue: 1.7, applicableAs: ["frame"], sponning: { width: 29, depth: 17, position: "dubbel", type: "dubbele_sponning", second_width: 29, second_depth: 17, kernhout: 56 } },
      ].map(withRealSection),
    },
  ];
}
