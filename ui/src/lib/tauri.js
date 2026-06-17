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
      case "update_cell_panel_filling": return J(wasm.update_cell_panel_filling(args?.id, args?.cellIndex, args?.panelFillingJson));
      case "update_cell_glaslat": return J(wasm.update_cell_glaslat(args?.id, args?.cellIndex, args?.glaslatJson));
      case "update_cell_escape": return J(wasm.update_cell_escape(args?.id, args?.cellIndex, args?.isEscape));
      case "add_column": return J(wasm.add_column(args?.id, args?.position));
      case "add_row": return J(wasm.add_row(args?.id, args?.position));

      case "get_kozijn_geometry": return J(wasm.get_kozijn_geometry(args?.id));
      case "get_production_data": return J(wasm.get_production_data(args?.id));
      case "get_production_data_project": return J(wasm.get_production_data_project());
      case "calculate_thermal": return J(wasm.calculate_thermal(args?.id));
      case "get_export_data": return J(wasm.get_export_data(args?.id));
      case "get_project_export_data": return J(wasm.get_project_export_data());

      // Commands that return unchanged kozijn (updates not yet in WASM)
      case "update_grid_sizes":
      case "update_frame_profile":
      case "update_sill_profile":
      case "update_divider_profile":
      case "update_member_profile":
      case "update_frame_shape":
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
      case "get_sjablonen": return [];

      case "get_all_vliesgevels": return [];
      case "get_vliesgevel_production": return vgProductionStub();
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
      case "get_project_energy": return { items: [] };
      case "check_certification": return { ceChecks: [], skhChecks: [] };
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
      case "remove_combination": return null;
      case "generate_purchase_proposals": return [];
      case "get_cnc_parts": return [];
      case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
      case "validate_project_ids": return { checks: [] };
      case "get_glass_library": return [];
      case "export_cnc_gcode": return "ok";
      case "export_labels_pdf": return "ok";
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
    case "get_sjablonen": return [];
    case "get_quotations": return [];
    case "create_quotation": return quotationStub(args);
    case "update_quotation_status": return quotationStub(args);
    case "create_quotation_revision": return quotationStub(args, 2);
    case "get_production_plan": return { jobs: [], totalHours: 0, estimatedDays: 0, deliveryDate: "" };
    case "get_project_energy": return { items: [] };
    case "check_certification": return { ceChecks: [], skhChecks: [] };
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
    case "remove_combination": return null;
    case "generate_purchase_proposals": return [];
    case "get_cnc_parts": return [];
    case "get_pricing_config": return defaultPricingConfig();
    case "update_pricing_config": return null;
    case "get_cost_estimate": return emptyCostEstimate();
    case "get_cost_estimate_project": return [];
    case "get_vliesgevel_production": return vgProductionStub();
    case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
    case "validate_project_ids": return { checks: [] };
    case "get_glass_library": return [];
    case "export_cnc_gcode": return "ok";
    case "export_labels_pdf": return "ok";
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
