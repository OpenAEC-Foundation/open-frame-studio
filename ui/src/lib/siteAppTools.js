/** SuperCloud transport for the transport-neutral read-only frame tools. */
import { get } from "svelte/store";
import { invoke, getWebRuntimeMode } from "./tauri.js";
import { hostFileRef, isDirty } from "../stores/project.js";
import { currentKozijn } from "../stores/kozijn.js";
import { currentVliesgevel } from "../stores/vliesgevel.js";
import { frameToolDefinitions, frameToolNames, runFrameTool, FrameToolError } from "./frameTools.js";

export async function tools() {
  return frameToolNames();
}

export async function definitions() {
  return frameToolDefinitions();
}

export async function run(name, args) {
  if (getWebRuntimeMode() !== "wasm") {
    throw new FrameToolError("niet_in_browser", "OFS-rekenkern ontbreekt");
  }
  // Re-read the authoritative core snapshot on every invocation. The Svelte
  // project store can briefly lag behind a model mutation or optimistic edit.
  const project = await invoke("get_project");
  return runFrameTool(name, args, {
    project,
    selectedKozijnId: get(currentKozijn)?.id,
    selectedVliesgevelId: get(currentVliesgevel)?.id,
    hostFileRef: get(hostFileRef),
    dirty: get(isDirty),
  });
}
