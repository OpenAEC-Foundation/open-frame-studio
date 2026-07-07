/**
 * Tauri invoke wrapper.
 *
 * In Tauri: delegates to the real invoke.
 * In browser: delegates to WASM module (ofs-wasm).
 * Falls back to sensible defaults if WASM isn't loaded yet.
 */
export const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
export const isWeb = !isTauri;

let wasm = null;

/**
 * Initialize WASM module for web mode.
 * Call once at startup. Safe to call multiple times.
 */
export async function initWasm() {
  if (isTauri || wasm) return;
  try {
    // Use fetch + instantiate to avoid Rollup resolving the path
    const jsUrl = new URL("/wasm/ofs_wasm.js", window.location.origin).href;
    const wasmUrl = new URL("/wasm/ofs_wasm_bg.wasm", window.location.origin).href;
    const module = await import(/* @vite-ignore */ jsUrl);
    await module.default(wasmUrl);
    wasm = module;
    console.log("[web] WASM module loaded");
  } catch (e) {
    console.warn("[web] WASM not available, using fallback:", e);
  }
}

export async function invoke(cmd, args) {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke(cmd, args);
  }

  // Web mode: try WASM first, fall back to defaults
  if (wasm) {
    return wasmCommand(cmd, args);
  }
  return browserFallback(cmd, args);
}

// ── Shared demo/fallback shapes ─────────────────────────────

function emptyDiffJson() {
  return JSON.stringify({ added: [], removed: [], modified: [], unchanged: 0 });
}

function emptyCostEstimate() {
  return {
    nlSfb: { code: "31.21", description: "Buitenkozijnen, -ramen, -deuren; hout" },
    materialCost: 0, glassCost: 0, hardwareCost: 0, gasketCost: 0, panelCost: 0,
    surfaceTreatmentCost: 0, laborHours: 0, laborCost: 0, totalCost: 0, lineItems: [],
  };
}

function defaultPricingConfig() {
  return {
    discountPercentage: 0, btwPercentage: 21, btwVerlegd: false,
    transportCost: 0, montageCostPerHour: 45, montageHours: 0,
  };
}

function bcfTopicStub(args, comments = []) {
  const now = new Date().toISOString();
  return {
    guid: args?.guid || String(Date.now()),
    title: args?.title || "Demo topic",
    description: args?.description || "",
    status: args?.status || "Open",
    priority: "Normal",
    creationDate: now,
    modifiedDate: now,
    assignedTo: null,
    relatedKozijnIds: [],
    comments,
  };
}

function quotationStub(args, version = 1) {
  return {
    id: args?.quotationId || `demo-${Date.now()}`,
    version,
    status: args?.status || "draft",
    createdAt: new Date().toISOString(),
    validUntil: "",
    kozijnMarks: [],
    totalInclBtw: args?.newTotal ?? args?.totalInclBtw ?? 0,
    notes: "",
    changeDescription: args?.changeDescription || "",
  };
}

function vgProductionStub() {
  return { mark: "VG01", name: "", mullionList: [], transomList: [], glassList: [], gasketList: [], bom: [] };
}

function circularityStub() {
  return {
    kozijnPassports: [], categories: [],
    totalMassKg: 0, totalRecyclableKg: 0, totalCo2Kg: 0,
    circularityScore: 0, renewableMassKg: 0, renewablePct: 0,
  };
}

function dopStub() {
  return {
    dopNumber: "", productType: "", kozijnMark: "", kozijnName: "",
    intendedUse: "", harmonisedStandard: "", avcpSystem: "", notifiedBody: "",
    manufacturer: "", material: "", widthMm: 0, heightMm: 0, declaredUw: 0,
    characteristics: [], conformityStatement: "",
  };
}

function plausibilityStub() {
  return { windPressurePa: 0, limitRatio: 200, kozijnResults: [], overallPass: true };
}

function energyStub(args) {
  const maxUw = args?.maxUw ?? 1.65;
  return {
    kozijnContributions: [], totalTransmissionLoss: 0, totalSolarGainFactor: 0,
    averageUw: 0, bouwbesluitMaxUw: maxUw, compliant: false,
  };
}

function certStub() {
  const empty = (standard) => ({ standard, checks: [], overallPass: false });
  return {
    ceMarking: empty("CE EN 14351-1"),
    skhKomo: empty("SKH/KOMO Houten Kozijnen"),
    performanceClass: {
      airPermeability: "", waterTightness: "", windResistance: "",
      thermalTransmittance: "", soundInsulation: "", burglarResistance: "",
    },
  };
}

// ── Custom sjablonen (web mode) ─────────────────────────────
// The Tauri backend stores custom sjablonen in the project state; in web
// mode we persist them in localStorage so save/delete actually stick.

const WEB_SJABLONEN_KEY = "ofs-custom-sjablonen";

function loadWebSjablonen() {
  try {
    const list = JSON.parse(localStorage.getItem(WEB_SJABLONEN_KEY) || "[]");
    return Array.isArray(list) ? list : [];
  } catch {
    return [];
  }
}

function saveWebSjabloon(sjabloonJson) {
  const sjabloon = JSON.parse(sjabloonJson || "{}");
  const list = loadWebSjablonen();
  const idx = list.findIndex((s) => s.id === sjabloon.id);
  if (idx >= 0) list[idx] = sjabloon;
  else list.push(sjabloon);
  localStorage.setItem(WEB_SJABLONEN_KEY, JSON.stringify(list));
  return null;
}

function deleteWebSjabloon(sjabloonId) {
  const list = loadWebSjablonen().filter((s) => s.id !== sjabloonId);
  localStorage.setItem(WEB_SJABLONEN_KEY, JSON.stringify(list));
  return null;
}

// ── WASM command dispatch ───────────────────────────────────

// ofs-wasm functions return JSON strings; the Tauri commands return parsed
// values. Normalize entity-returning commands so both modes look identical.
const J = (v) => (typeof v === "string" && (v.startsWith("{") || v.startsWith("[")) ? JSON.parse(v) : v);

function wasmCommand(cmd, args) {
  try {
    switch (cmd) {
      case "get_project": return J(wasm.get_project());
      case "new_project": return J(wasm.new_project(args?.name || "New", args?.number || ""));
      case "open_project": return J(wasm.open_project_json(args?.json || "{}"));
      case "save_project": return wasm.save_project_json();

      case "create_kozijn": return J(wasm.create_kozijn(args?.name, args?.mark, args?.width, args?.height));
      case "create_kozijn_from_template":
        // Use the template-aware builder when the (rebuilt) wasm exposes it;
        // gracefully fall back to a plain kozijn on older wasm bundles.
        return typeof wasm.create_kozijn_from_template === "function"
          ? J(wasm.create_kozijn_from_template(args?.template || "single_turn_tilt", args?.width || 1200, args?.height || 1500, args?.sjabloonId || null))
          : J(wasm.create_kozijn(args?.name || "Kozijn", args?.mark || "K01", args?.width || 1200, args?.height || 1500));
      case "get_kozijn": return J(wasm.get_kozijn(args?.id));
      case "get_all_kozijnen": return J(wasm.get_all_kozijnen());
      case "remove_kozijn": return wasm.remove_kozijn(args?.id);
      case "duplicate_kozijn": return J(wasm.duplicate_kozijn(args?.id, args?.newMark || "K01"));

      case "update_kozijn_dimensions": return J(wasm.update_kozijn_dimensions(args?.id, args?.width, args?.height));
      case "update_cell_type": return J(wasm.update_cell_type(args?.id, args?.cellIndex, args?.panelType, args?.openingDirection));
      case "update_cell_panel_filling":
        return typeof wasm.update_cell_panel_filling === "function"
          ? J(wasm.update_cell_panel_filling(args?.id, args?.cellIndex, args?.panelFillingJson))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_cell_glaslat":
        return typeof wasm.update_cell_glaslat === "function"
          ? J(wasm.update_cell_glaslat(args?.id, args?.cellIndex, args?.glaslatJson))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_cell_escape":
        return typeof wasm.update_cell_escape === "function"
          ? J(wasm.update_cell_escape(args?.id, args?.cellIndex, args?.isEscape))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_cell_sash_profile":
        return typeof wasm.update_cell_sash_profile === "function"
          ? J(wasm.update_cell_sash_profile(args?.id, args?.cellIndex, args?.profileId, args?.profileName, args?.sashWidth, args?.sashDepth))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_edge_config":
        return typeof wasm.update_edge_config === "function"
          ? J(wasm.update_edge_config(args?.id, args?.edgeIndex, args?.edgeJson))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_corner_joints":
        return typeof wasm.update_corner_joints === "function"
          ? J(wasm.update_corner_joints(args?.id, args?.jointsJson))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_frame_shape":
        return typeof wasm.update_frame_shape === "function"
          ? J(wasm.update_frame_shape(args?.id, args?.shapeType, args?.archHeight, args?.topWidth, args?.leftAngle, args?.rightAngle))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "update_kozijn_layout":
        return typeof wasm.update_kozijn_layout === "function"
          ? J(wasm.update_kozijn_layout(args?.id, args?.layoutJson))
          : (args?.id ? J(wasm.get_kozijn(args.id)) : null);
      case "add_column": return J(wasm.add_column(args?.id, args?.position));
      case "add_row": return J(wasm.add_row(args?.id, args?.position));

      case "get_kozijn_geometry": return J(wasm.get_kozijn_geometry(args?.id));
      case "get_production_data_project": return J(wasm.get_production_data_project());
      case "calculate_thermal": return J(wasm.calculate_thermal(args?.id));
      case "get_export_data": return J(wasm.get_export_data(args?.id));
      case "get_project_export_data": return J(wasm.get_project_export_data());

      // Commands that return unchanged kozijn (updates not yet in WASM)
      case "update_grid_sizes":
      case "update_frame_profile":
      case "update_sill_profile":
      case "update_member_profile":
      case "update_cell_hardware":
      case "auto_select_hardware":
      case "update_security_class":
      case "update_cell_glazing":
      case "update_frame_colors":
      case "add_frame_extension":
      case "remove_frame_extension":
        return args?.id ? J(wasm.get_kozijn(args.id)) : null;

      case "load_profile_library":
      case "add_custom_profile":
        return "[]";
      case "get_sjablonen": return loadWebSjablonen();
      case "save_custom_sjabloon": return saveWebSjabloon(args?.sjabloonJson);
      case "delete_custom_sjabloon": return deleteWebSjabloon(args?.sjabloonId);

      case "get_all_vliesgevels": return [];
      case "get_vliesgevel_production": return vgProductionStub();

      // Vliesgevel state does not exist in the wasm build yet — return null
      // so callers can tell the user honestly instead of pretending.
      case "create_vliesgevel":
      case "create_vliesgevel_from_template":
      case "get_vliesgevel":
      case "get_vliesgevel_geometry":
      case "remove_vliesgevel":
      case "vliesgevel_add_mullion":
      case "vliesgevel_add_transom":
      case "vliesgevel_remove_mullion":
      case "vliesgevel_remove_transom":
      case "vliesgevel_update_panel":
        console.warn(`[web] vliesgevel command not available in web mode: ${cmd}`);
        return null;
      case "get_cost_estimate": return emptyCostEstimate();
      case "get_cost_estimate_project": return [];
      case "get_pricing_config": return defaultPricingConfig();
      case "update_pricing_config": return null;

      case "get_platform": return "web";
      case "load_settings":
        return localStorage.getItem("ofs-settings") || JSON.stringify({
          theme: "default", locale: "nl",
          left_panel_width: 220, right_panel_width: 290,
          left_panel_open: true, right_panel_open: true,
        });
      case "save_settings":
        localStorage.setItem("ofs-settings", args?.settingsJson);
        return "ok";

      case "send_to_blender":
      case "check_blender_connection":
        return "not_available";

      // New feature commands — return empty defaults in WASM mode
      case "get_quotations": return [];
      case "create_quotation": return quotationStub(args);
      case "update_quotation_status": return quotationStub(args);
      case "create_quotation_revision": return quotationStub(args, 2);
      case "get_production_plan": return { jobs: [], totalHours: 0, estimatedDays: 0, deliveryDate: "" };
      case "get_project_energy":
        return typeof wasm.get_project_energy === "function"
          ? J(wasm.get_project_energy(args?.maxUw ?? 1.65))
          : energyStub(args);
      case "get_project_circularity":
        // Real computation once the (rebuilt) wasm exposes it; empty shape otherwise.
        return typeof wasm.get_project_circularity === "function"
          ? J(wasm.get_project_circularity())
          : circularityStub();
      case "get_project_plausibility":
        return typeof wasm.get_project_plausibility === "function"
          ? J(wasm.get_project_plausibility(args?.windPressurePa ?? 1000))
          : plausibilityStub();
      case "check_certification":
        return typeof wasm.check_certification === "function"
          ? J(wasm.check_certification(args?.id))
          : certStub();
      case "generate_dop_for_kozijn":
        return typeof wasm.generate_dop_for_kozijn === "function"
          ? J(wasm.generate_dop_for_kozijn(args?.id))
          : dopStub();
      case "get_bcf_topics": return [];
      case "create_bcf_topic": return bcfTopicStub(args);
      case "update_bcf_topic_status": return bcfTopicStub(args);
      case "add_bcf_comment":
        return bcfTopicStub(args, [{ guid: String(Date.now()), author: args?.author || "", date: new Date().toISOString(), comment: args?.comment || "" }]);
      case "get_combinations": return [];
      case "create_combination":
        return { id: `demo-${Date.now()}`, name: args?.name || "", mark: args?.mark || "", members: [], couplings: [] };
      case "add_to_combination":
        return { id: args?.combinationId, name: "", mark: "", members: [{ kozijnId: args?.kozijnId, offsetX: args?.offsetX || 0, offsetY: args?.offsetY || 0 }], couplings: [] };
      case "add_coupling":
        return { id: args?.combinationId, name: "", mark: "", members: [], couplings: [{ memberAId: args?.memberAId, memberBId: args?.memberBId, couplingType: args?.couplingType, couplingWidth: args?.couplingWidth || 0 }] };
      case "remove_combination": return null;
      case "generate_purchase_proposals": return [];
      case "generate_purchase_orders":
        return typeof wasm.generate_purchase_orders === "function"
          ? J(wasm.generate_purchase_orders())
          : [];
      case "get_cnc_parts": return [];
      case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
      case "validate_project_ids": return [];
      case "get_glass_library": return [];
      // File exports/imports need the filesystem — desktop only.
      case "export_cnc_gcode": return null;
      case "export_labels_pdf": return null;
      case "export_quotation_pdf": return null;
      case "import_ifc_file": return null;
      case "compare_ifc_roundtrip":
      case "compare_ifc_files":
        return emptyDiffJson();

      default:
        console.warn(`[web] unhandled WASM command: ${cmd}`, args);
        return null;
    }
  } catch (e) {
    console.error(`[web] WASM command ${cmd} failed:`, e);
    throw e;
  }
}

// ── Browser fallback (no WASM) ──────────────────────────────

function browserFallback(cmd, args) {
  switch (cmd) {
    case "get_project":
      return {
        formatVersion: "1.3",
        projectInfo: { name: "Demo", number: "", client: "", address: "" },
        kozijnen: [], vliesgevels: [], customProfiles: [],
      };
    case "new_project": return browserFallback("get_project");
    case "get_all_kozijnen": return [];
    case "get_all_vliesgevels": return [];
    case "load_profile_library": return [];
    case "get_sjablonen": return loadWebSjablonen();
    case "save_custom_sjabloon": return saveWebSjabloon(args?.sjabloonJson);
    case "delete_custom_sjabloon": return deleteWebSjabloon(args?.sjabloonId);
    case "get_quotations": return [];
    case "create_quotation": return quotationStub(args);
    case "update_quotation_status": return quotationStub(args);
    case "create_quotation_revision": return quotationStub(args, 2);
    case "get_production_plan": return { jobs: [], totalHours: 0, estimatedDays: 0, deliveryDate: "" };
    case "get_project_energy": return energyStub(args);
    case "get_project_circularity": return circularityStub();
    case "get_project_plausibility": return plausibilityStub();
    case "update_kozijn_layout": return null;
    case "check_certification": return certStub();
    case "generate_dop_for_kozijn": return dopStub();
    case "get_bcf_topics": return [];
    case "create_bcf_topic": return bcfTopicStub(args);
    case "update_bcf_topic_status": return bcfTopicStub(args);
    case "add_bcf_comment":
      return bcfTopicStub(args, [{ guid: String(Date.now()), author: args?.author || "", date: new Date().toISOString(), comment: args?.comment || "" }]);
    case "get_combinations": return [];
    case "create_combination":
      return { id: `demo-${Date.now()}`, name: args?.name || "", mark: args?.mark || "", members: [], couplings: [] };
    case "add_to_combination":
      return { id: args?.combinationId, name: "", mark: "", members: [{ kozijnId: args?.kozijnId, offsetX: args?.offsetX || 0, offsetY: args?.offsetY || 0 }], couplings: [] };
    case "add_coupling":
      return { id: args?.combinationId, name: "", mark: "", members: [], couplings: [{ memberAId: args?.memberAId, memberBId: args?.memberBId, couplingType: args?.couplingType, couplingWidth: args?.couplingWidth || 0 }] };
    case "remove_combination": return null;
    case "generate_purchase_proposals": return [];
    case "generate_purchase_orders": return [];
    case "get_cnc_parts": return [];
    case "get_pricing_config": return defaultPricingConfig();
    case "update_pricing_config": return null;
    case "get_cost_estimate": return emptyCostEstimate();
    case "get_cost_estimate_project": return [];
    case "get_vliesgevel_production": return vgProductionStub();
    case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
    case "validate_project_ids": return [];
    case "get_glass_library": return [];
    // File exports/imports need the filesystem — desktop only.
    case "export_cnc_gcode": return null;
    case "export_labels_pdf": return null;
    case "export_quotation_pdf": return null;
    case "import_ifc_file": return null;
    case "compare_ifc_roundtrip":
    case "compare_ifc_files":
      return emptyDiffJson();
    case "get_platform": return "web";
    case "load_settings":
      return localStorage.getItem("ofs-settings") || JSON.stringify({
        theme: "default", locale: "nl",
        left_panel_width: 220, right_panel_width: 290,
        left_panel_open: true, right_panel_open: true,
      });
    case "save_settings":
      localStorage.setItem("ofs-settings", args?.settingsJson);
      return "ok";
    default:
      console.warn(`[web] unhandled command (no WASM): ${cmd}`);
      return null;
  }
}
