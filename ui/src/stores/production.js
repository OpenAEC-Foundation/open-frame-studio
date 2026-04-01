import { writable, get } from "svelte/store";
import { invoke } from "../lib/tauri.js";
import { currentKozijn } from "./kozijn.js";

export const productionData = writable(null);
export const productionDataProject = writable([]);

export async function loadProductionData() {
  const k = get(currentKozijn);
  if (!k) {
    productionData.set(null);
    return;
  }
  try {
    const data = await invoke("get_production_data", { id: k.id });
    productionData.set(data);
  } catch (e) {
    console.error("Productiedata laden mislukt:", e);
  }
}

export async function loadProductionDataProject() {
  try {
    const data = await invoke("get_production_data_project", {});
    productionDataProject.set(data);
  } catch (e) {
    console.error("Projectproductiedata laden mislukt:", e);
  }
}

export async function exportProductionLists(outputPath, format) {
  return await invoke("export_production_lists", {
    outputPath,
    format,
  });
}
