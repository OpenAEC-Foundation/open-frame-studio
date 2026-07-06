import { writable } from "svelte/store";
import { invoke } from "../lib/tauri.js";

export const productionDataProject = writable([]);

export async function loadProductionDataProject() {
  try {
    const data = await invoke("get_production_data_project", {});
    productionDataProject.set(data);
  } catch (e) {
    console.error("Projectproductiedata laden mislukt:", e);
  }
}
