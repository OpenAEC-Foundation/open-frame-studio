import { writable, get } from "svelte/store";
import { invoke } from "../lib/tauri.js";
import { refreshProject, project } from "./project.js";

export const currentVliesgevel = writable(null);
export const currentVgGeometry = writable(null);
export const selectedPanelIndex = writable(null);

/** All vliesgevels in the current project. */
export const vliesgevels = writable([]);

// Keep the list in sync with the project store. Every mutation (create from
// template, add/remove mullion, remove, ...) calls refreshProject(), and
// open/new project replace the project store — so newly created vliesgevels
// show up here automatically.
project.subscribe(($project) => {
  vliesgevels.set($project?.vliesgevels || []);
});

/** Load all vliesgevels from the backend into the list store. */
export async function loadVliesgevels() {
  try {
    const list = await invoke("get_all_vliesgevels");
    vliesgevels.set(Array.isArray(list) ? list : []);
    return list;
  } catch (e) {
    console.error("Vliesgevels laden mislukt:", e);
    return [];
  }
}

/** Clear the vliesgevel selection so the kozijn editor becomes active again. */
export function clearVliesgevelSelection() {
  currentVliesgevel.set(null);
  currentVgGeometry.set(null);
  selectedPanelIndex.set(null);
}

export async function createVliesgevel(name, mark, width, height, mullionSpacing, transomSpacing) {
  const vg = await invoke("create_vliesgevel", {
    name, mark, width, height, mullionSpacing, transomSpacing,
  });
  await refreshProject();
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(vg.id);
  return vg;
}

export async function createVgFromTemplate(template, width, height) {
  const vg = await invoke("create_vliesgevel_from_template", { template, width, height });
  await refreshProject();
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(vg.id);
  return vg;
}

export async function selectVliesgevel(id) {
  const vg = await invoke("get_vliesgevel", { id });
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(id);
}

export async function addMullion(xPosition) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_add_mullion", { id: vg.id, xPosition });
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function addTransom(yPosition) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_add_transom", { id: vg.id, yPosition });
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function removeMullion(mullionIndex) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_remove_mullion", { id: vg.id, mullionIndex });
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function removeTransom(transomIndex) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_remove_transom", { id: vg.id, transomIndex });
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function updatePanel(col, row, panelType) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_update_panel", { id: vg.id, col, row, panelType });
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function removeVliesgevel(id) {
  await invoke("remove_vliesgevel", { id });
  await refreshProject();
  const vg = get(currentVliesgevel);
  if (vg && vg.id === id) {
    currentVliesgevel.set(null);
    currentVgGeometry.set(null);
    selectedPanelIndex.set(null);
  }
}

async function refreshVgGeometry(id) {
  try {
    const geom = await invoke("get_vliesgevel_geometry", { id });
    currentVgGeometry.set(geom);
  } catch (e) {
    console.error("Vliesgevel geometrie laden mislukt:", e);
  }
}
