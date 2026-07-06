import { writable, get } from "svelte/store";
import { _ } from "svelte-i18n";
import { invoke, isWeb } from "../lib/tauri.js";
import { refreshProject, project } from "./project.js";
import { toast } from "./toast.js";

// Vliesgevel commands are not implemented in the web (wasm) build yet;
// invoke() resolves to null there. Tell the user instead of crashing on
// the null result. In Tauri mode failures throw, so null never means this.
function unavailableInWeb(result) {
  if (!isWeb || (result !== null && result !== undefined)) return false;
  toast.warning(get(_)("alert.vliesgevelWebUnavailable"));
  return true;
}

export const currentVliesgevel = writable(null);
export const currentVgGeometry = writable(null);
export const selectedPanelIndex = writable(null);

/** All vliesgevels in the current project. */
export const vliesgevels = writable([]);

// Keep the list in sync with the project store. Every mutation (create from
// template, panel update, remove, ...) calls refreshProject(), and
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

export async function createVgFromTemplate(template, width, height) {
  const vg = await invoke("create_vliesgevel_from_template", { template, width, height });
  if (unavailableInWeb(vg)) return null;
  await refreshProject();
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(vg.id);
  return vg;
}

/** Next free VG-mark (VG01, VG02, ...) based on the project's vliesgevels. */
function nextVgMark() {
  const used = get(vliesgevels)
    .map((v) => /^VG(\d+)$/.exec(v.mark || ""))
    .filter(Boolean)
    .map((m) => parseInt(m[1], 10));
  const n = (used.length ? Math.max(...used) : 0) + 1;
  return `VG${String(n).padStart(2, "0")}`;
}

/** Create a blank vliesgevel (one open bay; add stijlen/regels in the editor). */
export async function createVliesgevel(width = 6000, height = 3600) {
  const vg = await invoke("create_vliesgevel", {
    name: "Vliesgevel",
    mark: nextVgMark(),
    width,
    height,
    // spacing >= size → no interior members: a single empty bay
    mullionSpacing: width,
    transomSpacing: height,
  });
  if (unavailableInWeb(vg)) return null;
  await refreshProject();
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(vg.id);
  return vg;
}

/** Midpoint of the largest gap between existing positions within [0, total]. */
function largestGapMidpoint(positions, total) {
  const sorted = [0, ...positions.slice().sort((a, b) => a - b), total];
  let bestMid = total / 2;
  let bestGap = -1;
  for (let i = 0; i + 1 < sorted.length; i++) {
    const gap = sorted[i + 1] - sorted[i];
    if (gap > bestGap) {
      bestGap = gap;
      bestMid = sorted[i] + gap / 2;
    }
  }
  return Math.round(bestMid);
}

/** Add a mullion (tussenstijl) in the middle of the widest bay. */
export async function addMullion() {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const x = largestGapMidpoint(vg.mullions.map((m) => m.xPosition), vg.overallWidth);
  const updated = await invoke("vliesgevel_add_mullion", { id: vg.id, xPosition: x });
  if (unavailableInWeb(updated)) return;
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

/** Add a transom (regel) in the middle of the tallest bay. */
export async function addTransom() {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const y = largestGapMidpoint(vg.transoms.map((t) => t.yPosition), vg.overallHeight);
  const updated = await invoke("vliesgevel_add_transom", { id: vg.id, yPosition: y });
  if (unavailableInWeb(updated)) return;
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

/** Remove the last (rightmost) mullion. */
export async function removeMullion() {
  const vg = get(currentVliesgevel);
  if (!vg || !vg.mullions.length) return;
  const updated = await invoke("vliesgevel_remove_mullion", { id: vg.id, mullionIndex: vg.mullions.length - 1 });
  if (unavailableInWeb(updated)) return;
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

/** Remove the last (topmost) transom. */
export async function removeTransom() {
  const vg = get(currentVliesgevel);
  if (!vg || !vg.transoms.length) return;
  const updated = await invoke("vliesgevel_remove_transom", { id: vg.id, transomIndex: vg.transoms.length - 1 });
  if (unavailableInWeb(updated)) return;
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function selectVliesgevel(id) {
  const vg = await invoke("get_vliesgevel", { id });
  if (unavailableInWeb(vg)) return;
  currentVliesgevel.set(vg);
  selectedPanelIndex.set(null);
  await refreshVgGeometry(id);
}

export async function updatePanel(col, row, panelType) {
  const vg = get(currentVliesgevel);
  if (!vg) return;
  const updated = await invoke("vliesgevel_update_panel", { id: vg.id, col, row, panelType });
  if (unavailableInWeb(updated)) return;
  currentVliesgevel.set(updated);
  await refreshProject();
  await refreshVgGeometry(updated.id);
}

export async function removeVliesgevel(id) {
  const result = await invoke("remove_vliesgevel", { id });
  // In Tauri this command returns () → null, so only web-mode null is a gap.
  if (unavailableInWeb(result)) return;
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
