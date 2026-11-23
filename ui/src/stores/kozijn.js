import { writable, derived, get } from "svelte/store";
import { invoke } from "../lib/tauri.js";
import { refreshProject, markDirty } from "./project.js";
import { pushSnapshot, clearHistory } from "./history.js";
import { selectedJoints } from "../lib/dividerJoinery.js";

export const currentKozijn = writable(null);
export const selectedCellIndex = writable(null);
// Selected member: { type: "frame_top"|"frame_bottom"|"frame_left"|"frame_right"|"divider_v"|"divider_h", index: number }
export const selectedMember = writable(null);
// Selected leaf (vak) in the free-subdivision layout tree, by node id. Lives in
// the store so the side panels can edit the vulling of the selected vak.
export const selectedLayoutLeafId = writable(null);

export function selectDividerPiece(piece) {
  selectedLayoutLeafId.set(null);
  selectedCellIndex.set(null);
  selectedMember.set({
    type: piece.vertical ? "divider_v" : "divider_h", index: piece.sourceIndex, pieceIndex: piece.pieceIndex,
  });
}

export const currentGeometry = writable(null);
export const selectedTimberJoints = derived([currentGeometry, selectedMember], ([geometry, selected]) =>
  selectedJoints(geometry, selected));

export async function createKozijn(name, mark, width, height) {
  const k = await invoke("create_kozijn", { name, mark, width, height });
  await refreshProject();
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  await refreshGeometry(k.id);
  markDirty();
  return k;
}

export async function createFromTemplate(template, width, height, sjabloonId) {
  const k = await invoke("create_kozijn_from_template", {
    template,
    width,
    height,
    sjabloonId: sjabloonId || null,
  });
  await refreshProject();
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  await refreshGeometry(k.id);
  markDirty();
  return k;
}

export async function selectKozijn(id) {
  clearHistory();
  const k = await invoke("get_kozijn", { id });
  currentKozijn.set(k);
  selectedCellIndex.set(null);
  selectedLayoutLeafId.set(null);
  await refreshGeometry(id);
}

/**
 * Persist the free-subdivision layout tree onto the current kozijn.
 * Optimistically updates the store so the UI reflects it immediately; the IPC
 * persists it server-side (and into .ofs) once the wasm bundle exposes the cmd.
 */
export async function setKozijnLayout(tree) {
  const k = get(currentKozijn);
  if (!k) return;
  // Store a plain (deep-cloned) tree — callers may pass a Svelte $state proxy,
  // which must not leak into the store (it breaks structuredClone-based history
  // and IPC serialization).
  const plain = JSON.parse(JSON.stringify(tree));
  pushSnapshot();
  // Optimistic: keep OUR tree in the store rather than the backend echo — older
  // backends strip vulling extras (glaslat/opensOutward/hardware) on roundtrip.
  currentKozijn.set({ ...k, layout: plain });
  try {
    await invoke("update_kozijn_layout", { id: k.id, layoutJson: JSON.stringify(plain) });
  } catch (e) {
    console.error("Layout opslaan mislukt:", e);
  }
  // The 2D stepped outline (framePolygons), dimension chains and the 3D viewer
  // all render from the geometry payload — without this refresh they keep
  // showing the pre-layout state until an unrelated mutation happens.
  await refreshProject();
  await refreshGeometry(k.id);
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

export async function updateCellPanelFilling(cellIndex, panelFilling) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  // Optimistically apply so fields like the new inzet/diepte (setbackMm, #6)
  // reflect immediately — even when the backend echoes the kozijn unchanged or
  // an older wasm bundle silently drops a field it doesn't know yet.
  const applyLocal = (kz) => ({
    ...kz,
    cells: (kz.cells || []).map((c, i) => (i === cellIndex ? { ...c, panelFilling } : c)),
  });
  currentKozijn.set(applyLocal(k));
  try {
    const updated = await invoke("update_cell_panel_filling", {
      id: k.id,
      cellIndex,
      panelFillingJson: JSON.stringify(panelFilling),
    });
    if (updated && Array.isArray(updated.cells)) currentKozijn.set(applyLocal(updated));
  } catch (e) {
    console.error("Vakvulling opslaan mislukt:", e);
  }
  await refreshProject();
  await refreshGeometry(k.id);
}

export async function updateCellGlaslat(cellIndex, glaslat) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_cell_glaslat", {
    id: k.id,
    cellIndex,
    glaslatJson: JSON.stringify(glaslat),
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateCellGlazing(cellIndex, glazing) {
  const k = get(currentKozijn);
  if (!k || cellIndex == null) return;
  pushSnapshot();
  const updated = await invoke("update_cell_glazing", {
    id: k.id, cellIndex, glazingJson: JSON.stringify(glazing),
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(k.id);
}

export async function updateCellEscape(cellIndex, isEscape) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_cell_escape", {
    id: k.id,
    cellIndex,
    isEscape,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addColumn(position, elevation = false) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("add_column", { id: k.id, position, elevation });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addRow(position, elevation = false) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("add_row", { id: k.id, position, elevation });
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
  await refreshGeometry(updated.id);
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
  await refreshGeometry(updated.id);
}

export async function updateFrameProfile(profileId, profileName, profileWidth, profileDepth, profileSnapshot, material, subtype) {
  const k = get(currentKozijn);
  if (!k) return;
  let frameMaterial = null;
  if (material === "wood") {
    const species = ["meranti", "accoya", "vuren", "eiken"].includes(subtype) ? subtype : k.frame.material?.wood || "meranti";
    frameMaterial = { wood: species };
  } else if (["pvc", "aluminum", "wood_aluminum"].includes(material)) {
    frameMaterial = material;
  }
  pushSnapshot();
  const updated = await invoke("update_frame_profile", {
    id: k.id, profileId, profileName,
    profileWidth: profileWidth || null,
    profileDepth: profileDepth || null,
    // Resolved sponning/glaslat/aanzicht values from the ProfileDefinition;
    // null leaves the stored snapshot untouched (old backends ignore the arg).
    profileSnapshotJson: profileSnapshot ? JSON.stringify(profileSnapshot) : null,
    profileMaterialJson: frameMaterial ? JSON.stringify(frameMaterial) : null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updatePerimeterStop(profile) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_perimeter_stop", { id: k.id, profileJson: JSON.stringify(profile) });
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
  await refreshGeometry(updated.id);
}

export async function updateMemberProfile(memberType, memberIndex, profileId, profileName, profileWidth, profileDepth) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_member_profile", {
    id: k.id,
    memberType,
    memberIndex: memberIndex ?? null,
    profileId,
    profileName,
    profileWidth: profileWidth || null,
    profileDepth: profileDepth || null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function addFrameExtension(extension) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("add_frame_extension", {
    id: k.id,
    extensionJson: JSON.stringify(extension),
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function removeFrameExtension(extensionIndex) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("remove_frame_extension", {
    id: k.id,
    extensionIndex,
  });
  currentKozijn.set(updated);
  await refreshProject();
}

export async function updateFrameShape(shapeType, archHeight, topWidth, leftAngle, rightAngle) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_frame_shape", {
    id: k.id, shapeType,
    archHeight: archHeight || null,
    topWidth: topWidth || null,
    leftAngle: leftAngle || null,
    rightAngle: rightAngle || null,
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateCornerJoints(joints) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_corner_joints", {
    id: k.id, jointsJson: JSON.stringify(joints),
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateGridSizes(columnSizes, rowSizes, rowsFromDay = false) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_grid_sizes", {
    id: k.id,
    columnSizes,
    rowSizes,
    rowsFromDay,
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
  await refreshGeometry(updated.id);
}

export async function updateCellSashProfile(cellIndex, profileId, profileName, sashWidth, sashDepth, snapshot) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  const updated = await invoke("update_cell_sash_profile", {
    id: k.id, cellIndex, profileId, profileName, sashWidth, sashDepth,
    profileSnapshotJson: JSON.stringify(snapshot ?? null),
  });
  currentKozijn.set(updated);
  await refreshProject();
  await refreshGeometry(updated.id);
}

export async function updateFrameColors(colorInside, colorOutside) {
  const k = get(currentKozijn);
  if (!k) return;
  pushSnapshot();
  // Keep both selectors in step when the user changes the two faces in succession.
  currentKozijn.set({ ...k, frame: { ...k.frame, colorInside, colorOutside } });
  try {
    await invoke("update_frame_colors", {
      id: k.id,
      colorInside,
      colorOutside,
    });
  } catch (e) {
    console.error("Kleur opslaan mislukt:", e);
    const current = get(currentKozijn);
    if (current?.id === k.id && current.frame.colorInside === colorInside && current.frame.colorOutside === colorOutside) {
      const project = await refreshProject();
      const saved = project?.kozijnen?.find(item => item.id === k.id);
      if (get(currentKozijn) === current) {
        const previous = saved?.frame ?? k.frame;
        currentKozijn.set({ ...current, frame: { ...current.frame, colorInside: previous.colorInside, colorOutside: previous.colorOutside } });
      }
    }
    return;
  }
  await refreshProject();
}

export async function duplicateKozijn(newMark) {
  const k = get(currentKozijn);
  if (!k) return;
  const dup = await invoke("duplicate_kozijn", { id: k.id, newMark });
  await refreshProject();
  currentKozijn.set(dup);
  selectedCellIndex.set(null);
  await refreshGeometry(dup.id);
  markDirty();
  return dup;
}

export async function calculateThermal() {
  const k = get(currentKozijn);
  if (!k) return null;
  try {
    return await invoke("calculate_thermal", { id: k.id });
  } catch (e) {
    console.error("Thermische berekening mislukt:", e);
    return null;
  }
}

export async function removeKozijn(id) {
  await invoke("remove_kozijn", { id });
  await refreshProject();
  const k = get(currentKozijn);
  if (k && k.id === id) {
    currentKozijn.set(null);
    currentGeometry.set(null);
    selectedCellIndex.set(null);
    selectedLayoutLeafId.set(null);
  }
  markDirty();
}

export async function refreshGeometry(id = get(currentKozijn)?.id) {
  if (!id) return;
  try {
    const geom = await invoke("get_kozijn_geometry", { id });
    if (get(currentKozijn)?.id === id) currentGeometry.set(geom);
  } catch (e) {
    console.error("Geometrie laden mislukt:", e);
  }
}
