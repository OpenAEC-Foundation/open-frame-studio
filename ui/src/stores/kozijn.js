import { writable, derived, get } from "svelte/store";
import { invoke } from "../lib/tauri.js";
import { refreshProject } from "./project.js";
import { pushSnapshot, clearHistory } from "./history.js";

export const currentKozijn = writable(null);
export const selectedCellIndex = writable(null);
// Selected member: { type: "frame_top"|"frame_bottom"|"frame_left"|"frame_right"|"divider_v"|"divider_h", index: number }
export const selectedMember = writable(null);

export const currentGeometry = writable(null);

export async function createKozijn(name, mark, width, height) {
  const k = await invoke("create_kozijn", { name, mark, width, height });
  await refreshProject();
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  await refreshGeometry(k.id);
  return k;
}

export async function createFromTemplate(template, width, height) {
  const k = await invoke("create_kozijn_from_template", {
    template,
    width,
    height,
  });
  await refreshProject();
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  await refreshGeometry(k.id);
  return k;
}

export async function selectKozijn(id) {
  clearHistory();
  const k = await invoke("get_kozijn", { id });
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  await refreshGeometry(id);
}

export async function updateDimensions(width, height) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_kozijn_dimensions", {
    id: k.id,
    width,
    height,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateCellType(cellIndex, panelType, openingDirection) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_cell_type", {
    id: k.id,
    cellIndex,
    panelType,
    openingDirection: openingDirection || null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addColumn(position) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("add_column", { id: k.id, position });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addRow(position) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("add_row", { id: k.id, position });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateCellHardware(cellIndex, hardwareSet) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_cell_hardware", {
    id: k.id,
    cellIndex,
    hardwareSetJson: JSON.stringify(hardwareSet),
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function autoSelectHardware(cellIndex) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("auto_select_hardware", {
    id: k.id,
    cellIndex,
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function updateFrameProfile(profileId, profileName, profileWidth, profileDepth) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_frame_profile", {
    id: k.id, profileId, profileName,
    profileWidth: profileWidth || null,
    profileDepth: profileDepth || null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateSillProfile(profileId, profileName) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_sill_profile", { id: k.id, profileId, profileName });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function updateDividerProfile(dividerIndex, isColumn, profileId, profileName) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_divider_profile", {
    id: k.id, dividerIndex, isColumn, profileId, profileName,
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function updateFrameShape(shapeType, archHeight) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_frame_shape", {
    id: k.id, shapeType, archHeight: archHeight || null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addCustomProfile(profileJson) {
  await invoke("add_custom_profile", { profileJson: JSON.stringify(profileJson) });
  await refreshProject();
}

export async function updateGridSizes(columnSizes, rowSizes) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_grid_sizes", {
    id: k.id,
    columnSizes,
    rowSizes,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateSecurityClass(cellIndex, securityClass) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_security_class", {
    id: k.id,
    cellIndex,
    securityClass,
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function removeKozijn(id) {
  await invoke("remove_kozijn", { id });
  await refreshProject();
  const k = get(currentKozijn);
  if (k && k.id === id) {
    currentKozijn.set(null);
    currentGeometry.set(null);
    selectedCellIndex.set(null);
  }
}

async function refreshGeometry(id) {
  try {
    const geom = await invoke("get_kozijn_geometry", { id });
    currentGeometry.set(geom);
  } catch (e) {
    console.error("Geometrie laden mislukt:", e);
  }
}
