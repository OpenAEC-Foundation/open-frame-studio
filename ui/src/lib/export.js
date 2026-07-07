/**
 * Export/import service — handles file dialogs and Tauri commands.
 * Keeps business logic out of UI components.
 */
import { get } from "svelte/store";
import { _ } from "svelte-i18n";
import { isTauri, isWeb } from "./tauri.js";
import { api } from "./api.js";
import { toast } from "../stores/toast.js";
import { currentKozijn } from "../stores/kozijn.js";
import { refreshCustomProfiles } from "../stores/profiles.js";

// In web mode file export/import commands resolve to null (no filesystem);
// be honest about it instead of showing a success toast. In Tauri mode
// failures throw, so a null result never means this there.
function exportUnavailable(result) {
  if (!isWeb || result !== null) return false;
  toast.warning(get(_)("alert.exportDesktopOnly"));
  return true;
}

function importUnavailable(result) {
  if (!isWeb || result !== null) return false;
  toast.warning(get(_)("alert.importDesktopOnly"));
  return true;
}

async function getSaveDialog() {
  if (isTauri) return await import("@tauri-apps/plugin-dialog");
  return { save: async (opts) => prompt("Save path:", opts?.defaultPath || "file") };
}

async function getOpenDialog() {
  if (isTauri) return await import("@tauri-apps/plugin-dialog");
  return { open: async () => prompt("File path:") };
}

// ── Export functions ────────────────────────────────────────

export async function exportIfc() {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    defaultPath: `${k.mark}.ifc`,
  });
  if (!path) return;
  const result = await api("export_ifc", { id: k.id, outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "IFC", path } }));
}

export async function exportDxf() {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "DXF", extensions: ["dxf"] }],
    defaultPath: `${k.mark}.dxf`,
  });
  if (!path) return;
  const result = await api("export_dxf", { id: k.id, outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "DXF", path } }));
}

export async function exportKozijnstaat(format) {
  const ext = format === "xlsx" ? "xlsx" : "pdf";
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: format.toUpperCase(), extensions: [ext] }],
    defaultPath: `kozijnstaat.${ext}`,
  });
  if (!path) return;
  const result = await api("export_kozijnstaat", { outputPath: path, format });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: `Kozijnstaat ${format.toUpperCase()}`, path } }));
}

export async function exportWorkshop() {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "PDF", extensions: ["pdf"] }],
    defaultPath: `${k.mark}_werkplaats.pdf`,
  });
  if (!path) return;
  const result = await api("export_workshop_drawing", { id: k.id, outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "PDF", path } }));
}

export async function exportGltf() {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "glTF Binary", extensions: ["glb"] }],
    defaultPath: `${k.mark}.glb`,
  });
  if (!path) return;
  const result = await api("export_gltf", { id: k.id, outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "glTF", path } }));
}

export async function exportProduction(format) {
  const extMap = { pdf: "pdf", xlsx: "xlsx", csv: "csv" };
  const ext = extMap[format] || "pdf";
  const defaultName = format === "csv" ? "productiestaten" : `productiestaten.${ext}`;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: format.toUpperCase(), extensions: [ext] }],
    defaultPath: defaultName,
  });
  if (!path) return;
  const result = await api("export_production_lists", { outputPath: path, format });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: `Production ${format.toUpperCase()}`, path } }));
}

export async function exportQuotationPdf() {
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "PDF", extensions: ["pdf"] }],
    defaultPath: "offerte.pdf",
  });
  if (!path) return;
  const result = await api("export_quotation_pdf", { outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "Offerte PDF", path } }));
}

export async function sendToBlender() {
  const k = get(currentKozijn);
  if (!k) return;
  const result = await api("send_to_blender", { id: k.id });
  toast.success(get(_)("alert.blenderSuccess", { values: { result } }));
}

// ── CNC & Labels ───────────────────────────────────────────

export async function exportCncGcode() {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "G-code", extensions: ["nc", "gcode"] }],
    defaultPath: `${k.mark}_cnc`,
  });
  if (!path) return;
  const result = await api("export_cnc_gcode", { id: k.id, outputDir: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "CNC G-code", path } }));
}

export async function exportLabels() {
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "PDF", extensions: ["pdf"] }],
    defaultPath: "labels.pdf",
  });
  if (!path) return;
  const result = await api("export_labels_pdf", { outputPath: path });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: "Labels PDF", path } }));
}

export async function exportIfcWithLod(lod) {
  const k = get(currentKozijn);
  if (!k) return;
  const { save } = await getSaveDialog();
  const path = await save({
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    defaultPath: `${k.mark}_lod${lod}.ifc`,
  });
  if (!path) return;
  // Rust expects lod as Option<String> — always pass a string.
  const result = await api("export_ifc", { id: k.id, outputPath: path, lod: String(lod) });
  if (exportUnavailable(result)) return;
  toast.success(get(_)("alert.exportSuccess", { values: { type: `IFC LOD${lod}`, path } }));
}

// ── IFC Import & Compare ───────────────────────────────────

export async function importIfcFile() {
  const { open } = await getOpenDialog();
  const path = await open({
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    multiple: false,
  });
  if (!path) return;
  const result = await api("import_ifc_file", { filePath: path });
  if (importUnavailable(result)) return;
  toast.success(`IFC bestand geimporteerd: ${path}`);
  return result;
}

function toastIfcDiffSummary(result) {
  let diff = null;
  try {
    diff = typeof result === "string" ? JSON.parse(result) : result;
  } catch {
    diff = null;
  }
  if (!diff) {
    toast.warning(get(_)("alert.ifcCompareNoResult"));
    return;
  }
  toast.success(
    get(_)("alert.ifcCompareSummary", {
      values: {
        added: diff.added?.length ?? 0,
        removed: diff.removed?.length ?? 0,
        modified: diff.modified?.length ?? 0,
        unchanged: diff.unchanged ?? 0,
      },
    })
  );
}

export async function compareIfcRoundtrip() {
  const { open } = await getOpenDialog();
  const path = await open({
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    multiple: false,
  });
  if (!path) return;
  const result = await api("compare_ifc_roundtrip", { filePath: path });
  toastIfcDiffSummary(result);
  return result;
}

export async function compareIfcFiles() {
  const { open } = await getOpenDialog();
  const oldPath = await open({
    title: get(_)("dialog.selectOldIfc"),
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    multiple: false,
  });
  if (!oldPath) return;
  const newPath = await open({
    title: get(_)("dialog.selectNewIfc"),
    filters: [{ name: "IFC", extensions: ["ifc"] }],
    multiple: false,
  });
  if (!newPath) return;
  const result = await api("compare_ifc_files", { oldPath, newPath });
  toastIfcDiffSummary(result);
  return result;
}

// ── Import functions ────────────────────────────────────────

export async function importDxfProfile() {
  const { open } = await getOpenDialog();
  const path = await open({
    filters: [{ name: "DXF", extensions: ["dxf"] }],
    multiple: false,
  });
  if (!path) return;
  const result = await api("import_dxf_profile", { filePath: path });
  if (importUnavailable(result)) return;
  const profile = JSON.parse(result);
  await api("add_custom_profile", { profileJson: JSON.stringify(profile) });
  await refreshCustomProfiles();
  toast.success(
    get(_)("alert.profileImported", {
      values: { name: profile.name, width: profile.width, depth: profile.depth },
    })
  );
}

export async function importCatalog() {
  const { open } = await getOpenDialog();
  const path = await open({
    filters: [{ name: "Catalog", extensions: ["json", "xlsx", "xls", "csv"] }],
    multiple: false,
  });
  if (!path) return;
  const result = await api("import_catalog", { filePath: path, supplier: null });
  if (importUnavailable(result)) return;
  const profiles = JSON.parse(result);
  for (const profile of profiles) {
    await api("add_custom_profile", { profileJson: JSON.stringify(profile) });
  }
  await refreshCustomProfiles();
  toast.success(
    get(_)("alert.catalogImported", { values: { count: profiles.length } })
  );
}
