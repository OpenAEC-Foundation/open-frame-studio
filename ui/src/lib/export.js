/**
 * Export/import service — handles file dialogs and Tauri commands.
 * Keeps business logic out of UI components.
 */
import { get } from "svelte/store";
import { _ } from "svelte-i18n";
import { isTauri } from "./tauri.js";
import { api } from "./api.js";
import { toast } from "../stores/toast.js";
import { currentKozijn } from "../stores/kozijn.js";

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
  await api("export_ifc", { id: k.id, outputPath: path });
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
  await api("export_dxf", { id: k.id, outputPath: path });
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
  await api("export_kozijnstaat", { outputPath: path, format });
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
  await api("export_workshop_drawing", { id: k.id, outputPath: path });
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
  await api("export_gltf", { id: k.id, outputPath: path });
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
  await api("export_production_lists", { outputPath: path, format });
  toast.success(get(_)("alert.exportSuccess", { values: { type: `Production ${format.toUpperCase()}`, path } }));
}

export async function sendToBlender() {
  const k = get(currentKozijn);
  if (!k) return;
  const result = await api("send_to_blender", { id: k.id });
  toast.success(get(_)("alert.blenderSuccess", { values: { result } }));
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
  const profile = JSON.parse(result);
  await api("add_custom_profile", { profileJson: JSON.stringify(profile) });
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
  const profiles = JSON.parse(result);
  for (const profile of profiles) {
    await api("add_custom_profile", { profileJson: JSON.stringify(profile) });
  }
  toast.success(
    get(_)("alert.catalogImported", { values: { count: profiles.length } })
  );
}
