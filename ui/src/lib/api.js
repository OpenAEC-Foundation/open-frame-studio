/**
 * Centralized Tauri command wrapper with error handling via toast.
 */
import { invoke } from "./tauri.js";
import { toast } from "../stores/toast.js";

export async function api(cmd, args = {}) {
  try {
    return await invoke(cmd, args);
  } catch (e) {
    toast.error(String(e));
    throw e;
  }
}
