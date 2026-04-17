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

// ── WASM command dispatch ───────────────────────────────────

function wasmCommand(cmd, args) {
  try {
    switch (cmd) {
      case "get_project": return wasm.get_project();
      case "new_project": return wasm.new_project(args?.name || "New", args?.number || "");
      case "open_project": return wasm.open_project_json(args?.json || "{}");
      case "save_project": return wasm.save_project_json();

      case "create_kozijn": return wasm.create_kozijn(args?.name, args?.mark, args?.width, args?.height);
      case "create_kozijn_from_template": return wasm.create_kozijn(args?.name || "Kozijn", args?.mark || "K01", args?.width || 1200, args?.height || 1500);
      case "get_kozijn": return wasm.get_kozijn(args?.id);
      case "get_all_kozijnen": return wasm.get_all_kozijnen();
      case "remove_kozijn": return wasm.remove_kozijn(args?.id);
      case "duplicate_kozijn": return wasm.duplicate_kozijn(args?.id, args?.newMark || "K01");

      case "update_kozijn_dimensions": return wasm.update_kozijn_dimensions(args?.id, args?.width, args?.height);
      case "update_cell_type": return wasm.update_cell_type(args?.id, args?.cellIndex, args?.panelType, args?.openingDirection);
      case "add_column": return wasm.add_column(args?.id, args?.position);
      case "add_row": return wasm.add_row(args?.id, args?.position);

      case "get_kozijn_geometry": return wasm.get_kozijn_geometry(args?.id);
      case "get_production_data": return wasm.get_production_data(args?.id);
      case "get_production_data_project": return wasm.get_production_data_project();
      case "calculate_thermal": return wasm.calculate_thermal(args?.id);

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
        return args?.id ? wasm.get_kozijn(args.id) : null;

      case "load_profile_library":
      case "add_custom_profile":
      case "get_sjablonen":
        return "[]";

      case "get_all_vliesgevels": return [];
      case "get_cost_estimate": return { items: [], total: 0 };
      case "get_cost_estimate_project": return [];

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
      case "create_quotation": return { version: 1, status: "draft" };
      case "get_production_plan": return { jobs: [], totalHours: 0, estimatedDays: 0, deliveryDate: "" };
      case "get_project_energy": return { items: [] };
      case "check_certification": return { ceChecks: [], skhChecks: [] };
      case "get_bcf_topics": return [];
      case "create_bcf_topic": return { id: "new", title: args?.title };
      case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
      case "validate_project_ids": return { checks: [] };
      case "get_glass_library": return [];
      case "export_cnc_gcode": return "ok";
      case "export_labels_pdf": return "ok";
      case "import_ifc_file": return null;
      case "compare_ifc_roundtrip": return null;

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
    case "create_quotation": return { version: 1, status: "draft" };
    case "get_production_plan": return { jobs: [], totalHours: 0, estimatedDays: 0, deliveryDate: "" };
    case "get_project_energy": return { items: [] };
    case "check_certification": return { ceChecks: [], skhChecks: [] };
    case "get_bcf_topics": return [];
    case "create_bcf_topic": return { id: "new" };
    case "optimize_project_cut_list": return { bars: [], totalBars: 0, wastePercent: 0, totalWasteMm: 0 };
    case "validate_project_ids": return { checks: [] };
    case "get_glass_library": return [];
    case "export_cnc_gcode": return "ok";
    case "export_labels_pdf": return "ok";
    case "import_ifc_file": return null;
    case "compare_ifc_roundtrip": return null;
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
