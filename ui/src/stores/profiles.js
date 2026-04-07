/**
 * Profielbibliotheek store.
 *
 * Laadt alle profieldefinities uit de /profiles directory en
 * biedt ze aan als een doorzoekbare lijst.
 */
import { writable, derived } from "svelte/store";

// Alle profielen, gegroepeerd per categorie
export const profileCategories = writable([]);

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
    profileCategories.set(categories);
  } catch (e) {
    console.error("Profielen laden mislukt:", e);
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
 * Genereer crossSection polygon op basis van profiel parameters.
 * Buitensponning: sponning aan buitenkant (standaard houten kozijn)
 * Binnensponning: sponning aan binnenkant (raamhout/deurhout)
 * Midden: sponning in het midden (aluminium/PVC)
 * Draaikiep: binnensponning + opdek overlap + rubber groeven
 */
function generateCrossSection(w, d, sp, type) {
  const sw = sp?.width || 0;
  const sd = sp?.depth || 0;
  const pos = sp?.position || "buiten";

  // Eenvoudige rechthoek zonder sponning (glaslat, spouwlat)
  if (!sw || !sd) return [[0,0],[w,0],[w,d],[0,d]];

  if (pos === "buiten") {
    // Buitensponning: inkepingen aan onderkant links+rechts
    return [[0,0],[w,0],[w,d-sd],[w-sw,d-sd],[w-sw,d],[sw,d],[sw,d-sd],[0,d-sd]];
  }
  if (pos === "binnen") {
    // Binnensponning: inkepingen aan bovenkant links+rechts
    return [[sw,0],[w-sw,0],[w-sw,sd],[w,sd],[w,d],[0,d],[0,sd],[sw,sd]];
  }
  if (pos === "midden") {
    // Midden sponning: groef in het midden van de diepte
    const mid = (d - sd) / 2;
    return [[sw,0],[w-sw,0],[w-sw,mid],[w,mid],[w,mid+sd],[w-sw,mid+sd],[w-sw,d],[sw,d],[sw,mid+sd],[0,mid+sd],[0,mid],[sw,mid]];
  }
  if (type === "draaikiep") {
    // Draaikiep: binnensponning + opdek (13mm overlap) + 2 rubber groeven
    const opdek = sp?.opdek_width || 13;
    const rubberW = 3;
    const rubberD = 4;
    // Binnenzijde (boven): sponning met opdek lip
    // Buitenzijde (onder): glad
    return [
      [0, 0],                           // linksonder buiten
      [w, 0],                           // rechtsonder buiten
      [w, d],                           // rechtsboven buiten
      [w - sw, d],                      // begin sponning rechts boven
      [w - sw, d - opdek],              // opdek lip rechts
      [w - sw + rubberW, d - opdek],    // rubber groef 1 rechts
      [w - sw + rubberW, d - opdek - rubberD],
      [w - sw, d - opdek - rubberD],    // einde rubber 1
      [w - sw, d - sd],                 // sponning bodem rechts
      [sw, d - sd],                     // sponning bodem links
      [sw, d - opdek - rubberD],        // begin rubber 1 links
      [sw - rubberW, d - opdek - rubberD],
      [sw - rubberW, d - opdek],        // rubber groef 1 links
      [sw, d - opdek],                  // opdek lip links
      [sw, d],                          // sponning bovenkant links
      [0, d],                           // linksboven buiten
    ];
  }
  // Dubbele sponning
  if (pos === "dubbel") {
    const sw2 = sp?.second_width || sw;
    const sd2 = sp?.second_depth || sd;
    const kern = sp?.kernhout || 20;
    return [
      [0, 0], [w, 0],
      [w, sd], [w - sw, sd],
      [w - sw, sd + kern], [w - sw2, sd + kern],
      [w - sw2, d], [sw2, d],
      [sw2, sd + kern], [sw, sd + kern],
      [sw, sd], [0, sd],
    ];
  }
  // Fallback
  return [[0,0],[w,0],[w,d-sd],[w-sw,d-sd],[w-sw,d],[sw,d],[sw,d-sd],[0,d-sd]];
}

function getEmbeddedProfiles() {
  return [
    {
      id: "wood",
      label: "Hout",
      profiles: [
        { id: "wood-meranti-67x114", name: "Meranti 67x114mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame", "sash", "divider"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: [[0,0],[67,0],[67,97],[55,97],[55,114],[12,114],[12,97],[0,97]] },
        { id: "wood-meranti-67x150", name: "Meranti 67x150mm (dorpel)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 150, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["sill"], sponning: { width: 12, depth: 20, position: "buiten" }, crossSection: [[0,0],[67,0],[67,130],[55,130],[55,150],[12,150],[12,130],[0,130]] },
        { id: "wood-accoya-67x114", name: "Accoya 67x114mm", manufacturer: "Generiek", series: "Accoya", material: "wood", materialSubtype: "accoya", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.5, applicableAs: ["frame", "sash", "divider"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "hebo-67x114", name: "Hebo Kozijn 67x114mm", manufacturer: "Hebo Kozijnen", series: "Classic", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame", "sash"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "hebo-67x130", name: "Hebo Kozijn 67x130mm (zwaar)", manufacturer: "Hebo Kozijnen", series: "Classic", material: "wood", materialSubtype: "meranti", width: 67, depth: 130, sightline: 54, glazingRebate: 30, ufValue: 1.7, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 20, position: "buiten" }, crossSection: generateCrossSection(67, 130, { width: 15, depth: 20, position: "buiten" }) },
        { id: "goemaat-67x114", name: "Goemaat Standaard 67x114mm", manufacturer: "Goemaat Kozijnen", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame", "sash"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "goemaat-67x130", name: "Goemaat Zwaar 67x130mm", manufacturer: "Goemaat Kozijnen", series: "Zwaar", material: "wood", materialSubtype: "meranti", width: 67, depth: 130, sightline: 54, glazingRebate: 30, ufValue: 1.7, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 20, position: "buiten" }, crossSection: generateCrossSection(67, 130, { width: 15, depth: 20, position: "buiten" }) },
        { id: "weekamp-67x114", name: "WeekampGroep Standaard 67x114mm", manufacturer: "WeekampGroep", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame", "door_frame"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "raamwerk-67x114", name: "Raamwerk Standaard 67x114mm", manufacturer: "Het Raamwerk", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame", "sash"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
      ],
    },
    {
      id: "aluminum",
      label: "Aluminium",
      profiles: [
        { id: "reynaers-cs77-frame", name: "Reynaers CS 77", manufacturer: "Reynaers Aluminium", series: "CS 77", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 77, depth: 77, sightline: 54, glazingRebate: 28, ufValue: 1.6, applicableAs: ["frame", "sash"], sponning: { width: 18, depth: 21, position: "midden" }, crossSection: generateCrossSection(77, 77, { width: 18, depth: 21, position: "midden" }) },
        { id: "reynaers-cs86-frame", name: "Reynaers CS 86-HI", manufacturer: "Reynaers Aluminium", series: "CS 86-HI", material: "aluminum", materialSubtype: "high_insulation", width: 86, depth: 86, sightline: 60, glazingRebate: 34, ufValue: 1.1, applicableAs: ["frame", "sash"], sponning: { width: 22, depth: 24, position: "midden" }, crossSection: generateCrossSection(86, 86, { width: 22, depth: 24, position: "midden" }) },
        { id: "schuco-aws70-frame", name: "Schuco AWS 70.HI", manufacturer: "Schuco", series: "AWS 70.HI", material: "aluminum", materialSubtype: "thermisch_onderbroken", width: 70, depth: 70, sightline: 50, glazingRebate: 26, ufValue: 1.6, applicableAs: ["frame", "sash"], sponning: { width: 16, depth: 20, position: "midden" }, crossSection: generateCrossSection(70, 70, { width: 16, depth: 20, position: "midden" }) },
        { id: "schuco-aws75-frame", name: "Schuco AWS 75.SI+", manufacturer: "Schuco", series: "AWS 75.SI+", material: "aluminum", materialSubtype: "super_insulated", width: 75, depth: 75, sightline: 53, glazingRebate: 32, ufValue: 1.0, applicableAs: ["frame", "sash"], sponning: { width: 20, depth: 22, position: "midden" }, crossSection: generateCrossSection(75, 75, { width: 20, depth: 22, position: "midden" }) },
      ],
    },
    {
      id: "pvc",
      label: "Kunststof (PVC)",
      profiles: [
        { id: "gealan-s9000-frame", name: "Gealan S 9000", manufacturer: "Gealan", series: "S 9000", material: "pvc", materialSubtype: "6_kamer", width: 83, depth: 83, sightline: 62, glazingRebate: 28, ufValue: 1.0, applicableAs: ["frame", "sash"], sponning: { width: 18, depth: 20, position: "midden" }, crossSection: generateCrossSection(83, 83, { width: 18, depth: 20, position: "midden" }) },
        { id: "gealan-kubus-frame", name: "Gealan-KUBUS", manufacturer: "Gealan", series: "KUBUS", material: "pvc", materialSubtype: "7_kamer", width: 85, depth: 85, sightline: 0, glazingRebate: 30, ufValue: 0.92, applicableAs: ["frame", "sash"], sponning: { width: 20, depth: 22, position: "midden" }, crossSection: generateCrossSection(85, 85, { width: 20, depth: 22, position: "midden" }) },
        { id: "veka-softline82-frame", name: "VEKA Softline 82", manufacturer: "VEKA", series: "Softline 82", material: "pvc", materialSubtype: "7_kamer", width: 82, depth: 82, sightline: 61, glazingRebate: 28, ufValue: 0.95, applicableAs: ["frame", "sash"], sponning: { width: 18, depth: 20, position: "midden" }, crossSection: generateCrossSection(82, 82, { width: 18, depth: 20, position: "midden" }) },
        { id: "veka-softline70-frame", name: "VEKA Softline 70", manufacturer: "VEKA", series: "Softline 70", material: "pvc", materialSubtype: "5_kamer", width: 70, depth: 70, sightline: 53, glazingRebate: 24, ufValue: 1.3, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 18, position: "midden" }, crossSection: generateCrossSection(70, 70, { width: 15, depth: 18, position: "midden" }) },
      ],
    },
    {
      id: "wood-aluminum",
      label: "Hout-Aluminium",
      profiles: [
        { id: "hout-alu-67x114", name: "Hout-Aluminium 67x114mm", manufacturer: "Generiek", series: "Combi", material: "wood_aluminum", materialSubtype: "meranti_alu", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.5, applicableAs: ["frame", "sash"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "hout-alu-67x130", name: "Hout-Aluminium 67x130mm (zwaar)", manufacturer: "Generiek", series: "Combi", material: "wood_aluminum", materialSubtype: "meranti_alu", width: 67, depth: 130, sightline: 54, glazingRebate: 30, ufValue: 1.4, applicableAs: ["frame", "sash"], sponning: { width: 15, depth: 20, position: "buiten" }, crossSection: generateCrossSection(67, 130, { width: 15, depth: 20, position: "buiten" }) },
      ],
    },
    {
      id: "raamhout",
      label: "Raamhout (draaidelen)",
      profiles: [
        { id: "raam-meranti-54x67", name: "Raamhout Meranti 54x67mm", manufacturer: "Generiek", series: "54mm", material: "wood", materialSubtype: "meranti", width: 54, depth: 67, sightline: 42, glazingRebate: 15, ufValue: 2.2, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 12, depth: 15, position: "buiten" }, crossSection: generateCrossSection(54, 67, { width: 12, depth: 15, position: "buiten" }) },
        { id: "raam-meranti-54x78", name: "Raamhout Meranti 54x78mm (HR++)", manufacturer: "Generiek", series: "54mm", material: "wood", materialSubtype: "meranti", width: 54, depth: 78, sightline: 42, glazingRebate: 20, ufValue: 2.0, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 12, depth: 20, position: "buiten" }, crossSection: generateCrossSection(54, 78, { width: 12, depth: 20, position: "buiten" }) },
        { id: "raam-meranti-54x90", name: "Raamhout Meranti 54x90mm (Triple)", manufacturer: "Generiek", series: "54mm", material: "wood", materialSubtype: "meranti", width: 54, depth: 90, sightline: 42, glazingRebate: 28, ufValue: 1.8, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 12, depth: 28, position: "buiten" }, crossSection: generateCrossSection(54, 90, { width: 12, depth: 28, position: "buiten" }) },
        { id: "raam-accoya-54x78", name: "Raamhout Accoya 54x78mm", manufacturer: "Generiek", series: "54mm Accoya", material: "wood", materialSubtype: "accoya", width: 54, depth: 78, sightline: 42, glazingRebate: 20, ufValue: 1.8, applicableAs: ["raam_stijl", "raam_dorpel", "sash"], sponning: { width: 12, depth: 20, position: "buiten" }, crossSection: generateCrossSection(54, 78, { width: 12, depth: 20, position: "buiten" }) },
      ],
    },
    {
      id: "deurhout",
      label: "Deurhout",
      profiles: [
        { id: "deur-meranti-67x114", name: "Deurhout Meranti 67x114mm", manufacturer: "Generiek", series: "67mm", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["deur_stijl", "deur_dorpel", "sash"], sponning: { width: 12, depth: 17, position: "buiten" }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "buiten" }) },
        { id: "deur-meranti-67x130", name: "Deurhout Meranti 67x130mm (zwaar)", manufacturer: "Generiek", series: "67mm", material: "wood", materialSubtype: "meranti", width: 67, depth: 130, sightline: 54, glazingRebate: 30, ufValue: 1.7, applicableAs: ["deur_stijl", "deur_dorpel", "sash"], sponning: { width: 15, depth: 20, position: "buiten" }, crossSection: generateCrossSection(67, 130, { width: 15, depth: 20, position: "buiten" }) },
      ],
    },
    {
      id: "glaslat-spouwlat",
      label: "Glaslatten & Spouwlatten",
      profiles: [
        { id: "glaslat-17x17", name: "Glaslat 17x17mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 17, depth: 17, sightline: 17, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"], crossSection: [[0,0],[17,0],[17,17],[0,17]] },
        { id: "glaslat-17x22", name: "Glaslat 17x22mm (breed)", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "meranti", width: 17, depth: 22, sightline: 17, glazingRebate: 0, ufValue: 0, applicableAs: ["glaslat"], crossSection: [[0,0],[17,0],[17,22],[0,22]] },
        { id: "spouwlat-22x100", name: "Spouwlat 22x100mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 100, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"], crossSection: [[0,0],[22,0],[22,100],[0,100]] },
        { id: "spouwlat-22x120", name: "Spouwlat 22x120mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 120, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"], crossSection: [[0,0],[22,0],[22,120],[0,120]] },
        { id: "spouwlat-22x140", name: "Spouwlat 22x140mm", manufacturer: "Generiek", series: "Standaard", material: "wood", materialSubtype: "vuren", width: 22, depth: 140, sightline: 0, glazingRebate: 0, ufValue: 0, applicableAs: ["spouwlat"], crossSection: [[0,0],[22,0],[22,140],[0,140]] },
      ],
    },
    {
      id: "draaikiep",
      label: "Draaikiep (kozijn + raam)",
      profiles: [
        { id: "dk-kozijn-67x114", name: "DK Kozijn 67x114mm", manufacturer: "Generiek", series: "67mm DK", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 54, glazingRebate: 24, ufValue: 1.8, applicableAs: ["frame"], sponning: { width: 15, depth: 20, position: "buiten", opdek_width: 13, rubber_count: 2 }, crossSection: generateCrossSection(67, 114, { width: 15, depth: 20, position: "buiten" }) },
        { id: "dk-raam-54x78", name: "DK Raamhout 54x78mm (opdek)", manufacturer: "Generiek", series: "54mm DK", material: "wood", materialSubtype: "meranti", width: 54, depth: 78, sightline: 39, glazingRebate: 20, ufValue: 2.0, applicableAs: ["sash"], sponning: { width: 15, depth: 20, position: "buiten", opdek_width: 13, rubber_count: 2 }, crossSection: generateCrossSection(54, 78, { width: 15, depth: 20, position: "buiten", opdek_width: 13 }, "draaikiep") },
        { id: "dk-raam-54x90", name: "DK Raamhout 54x90mm Triple (opdek)", manufacturer: "Generiek", series: "54mm DK", material: "wood", materialSubtype: "meranti", width: 54, depth: 90, sightline: 39, glazingRebate: 28, ufValue: 1.8, applicableAs: ["sash"], sponning: { width: 15, depth: 28, position: "buiten", opdek_width: 13, rubber_count: 2 }, crossSection: generateCrossSection(54, 90, { width: 15, depth: 28, position: "buiten", opdek_width: 13 }, "draaikiep") },
        { id: "dk-kozijn-78x114", name: "DK Kozijn 78x114mm (zwaar)", manufacturer: "Generiek", series: "78mm DK", material: "wood", materialSubtype: "meranti", width: 78, depth: 114, sightline: 63, glazingRebate: 24, ufValue: 1.6, applicableAs: ["frame"], sponning: { width: 15, depth: 20, position: "buiten", opdek_width: 13, rubber_count: 2 }, crossSection: generateCrossSection(78, 114, { width: 15, depth: 20, position: "buiten" }) },
        { id: "dk-dubbel-67x114", name: "DK Dubbele sponning 67x114mm", manufacturer: "Generiek", series: "67mm Dubbel", material: "wood", materialSubtype: "meranti", width: 67, depth: 114, sightline: 43, glazingRebate: 20, ufValue: 1.7, applicableAs: ["frame"], sponning: { width: 12, depth: 17, position: "dubbel", second_width: 12, second_depth: 17, kernhout: 22 }, crossSection: generateCrossSection(67, 114, { width: 12, depth: 17, position: "dubbel", second_width: 12, second_depth: 17, kernhout: 22 }) },
      ],
    },
  ];
}
