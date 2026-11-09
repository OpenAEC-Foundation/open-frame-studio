/** Transport-neutral, read-only OFS assistant tools.
 *
 * A browser adapter and a future desktop MCP transport can pass the same
 * project snapshot and selection into runFrameTool. No tool mutates the model.
 */
import toolDefinitions from "./mcp-tools.json" with { type: "json" };

const MAX_OUTPUT_LENGTH = 30_000;
const MAX_LIST_LIMIT = 50;
const MAX_CELL_PREVIEW = 20;

export class FrameToolError extends Error {
  constructor(code, detail = "") {
    super(detail ? `${code}: ${detail}` : code);
    this.name = "FrameToolError";
    this.code = code;
  }
}

export function frameToolNames() {
  return toolDefinitions.map(({ name }) => name);
}

export function frameToolDefinitions() {
  return structuredClone(toolDefinitions);
}

function text(value, max = 120) {
  return String(value ?? "").slice(0, max);
}

function number(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function materialLabel(material) {
  if (typeof material === "string") return text(material, 40);
  if (material && typeof material === "object") {
    const [kind, subtype] = Object.entries(material)[0] || [];
    return kind ? text(subtype ? `${kind}:${subtype}` : kind, 40) : "";
  }
  return "";
}

function requireArgs(args, permitted) {
  if (args === undefined) return {};
  if (!args || typeof args !== "object" || Array.isArray(args)) {
    throw new FrameToolError("ongeldige_argumenten", "verwacht een object");
  }
  for (const key of Object.keys(args)) {
    if (!permitted.includes(key)) throw new FrameToolError("ongeldige_argumenten", `onbekend veld ${key}`);
  }
  return args;
}

function requireProject(context) {
  const project = context?.project;
  if (!project || !Array.isArray(project.kozijnen)) throw new FrameToolError("geen_project");
  return project;
}

function kozijnSummary(kozijn) {
  return {
    id: text(kozijn.id, 80),
    mark: text(kozijn.mark, 60),
    name: text(kozijn.name),
    material: materialLabel(kozijn.frame?.material),
    widthMm: number(kozijn.frame?.outerWidth),
    heightMm: number(kozijn.frame?.outerHeight),
    shape: text(kozijn.frame?.shape?.shapeType || "rectangular", 40),
    cellCount: Array.isArray(kozijn.cells) ? kozijn.cells.length : 0,
    recoveredFromIfc: !!kozijn.recoveredFromIfc,
  };
}

function stand(project, context) {
  const selectedKozijn = project.kozijnen.find((item) => item.id === context.selectedKozijnId);
  const vliesgevels = Array.isArray(project.vliesgevels) ? project.vliesgevels : [];
  const selectedVliesgevel = vliesgevels.find((item) => item.id === context.selectedVliesgevelId);
  const file = context.hostFileRef;
  return {
    project: {
      id: text(project.id, 80),
      name: text(project.projectInfo?.name),
      number: text(project.projectInfo?.number, 80),
      formatVersion: text(project.formatVersion, 30),
    },
    file: file ? {
      id: text(file.id, 80), name: text(file.name),
      version: file.version ?? null, readOnly: !!file.readOnly,
    } : null,
    dirty: !!context.dirty,
    selection: selectedVliesgevel
      ? { type: "vliesgevel", id: text(selectedVliesgevel.id, 80), mark: text(selectedVliesgevel.mark, 60) }
      : selectedKozijn
        ? { type: "kozijn", id: text(selectedKozijn.id, 80), mark: text(selectedKozijn.mark, 60) }
        : null,
    counts: { kozijnen: project.kozijnen.length, vliesgevels: vliesgevels.length },
  };
}

function kozijnen(project, args) {
  if (args.query !== undefined && typeof args.query !== "string") {
    throw new FrameToolError("ongeldige_argumenten", "query moet tekst zijn");
  }
  const offset = args.offset ?? 0;
  const limit = args.limit ?? 20;
  if (!Number.isInteger(offset) || offset < 0 || !Number.isInteger(limit) || limit < 1 || limit > MAX_LIST_LIMIT) {
    throw new FrameToolError("ongeldige_argumenten", "offset >= 0 en limit 1..50 vereist");
  }
  const query = (args.query || "").trim().toLocaleLowerCase("nl");
  const filtered = project.kozijnen.filter((item) =>
    !query || `${item.mark || ""} ${item.name || ""}`.toLocaleLowerCase("nl").includes(query)
  );
  return {
    total: filtered.length,
    offset,
    limit,
    items: filtered.slice(offset, offset + limit).map(kozijnSummary),
  };
}

function kozijnDetail(project, args, context) {
  if (args.id !== undefined && (typeof args.id !== "string" || !args.id.trim())) {
    throw new FrameToolError("ongeldige_argumenten", "id moet een niet-lege tekst zijn");
  }
  const id = args.id || context.selectedKozijnId;
  if (!id) throw new FrameToolError("geen_kozijn_geselecteerd", "geef een id op of selecteer een kozijn");
  const kozijn = project.kozijnen.find((item) => item.id === id);
  if (!kozijn) throw new FrameToolError("kozijn_niet_gevonden", text(id, 80));
  const cells = Array.isArray(kozijn.cells) ? kozijn.cells : [];
  const byPanelType = Object.create(null);
  for (const cell of cells) {
    const type = text(cell.panelType, 40) || "onbekend";
    byPanelType[type] = (byPanelType[type] || 0) + 1;
  }
  const frame = kozijn.frame || {};
  return {
    ...kozijnSummary(kozijn),
    frame: {
      profile: frame.profile ? { id: text(frame.profile.id, 80), name: text(frame.profile.name) } : null,
      sillProfile: frame.sillProfile ? { id: text(frame.sillProfile.id, 80), name: text(frame.sillProfile.name) } : null,
      colorInside: text(frame.colorInside, 50),
      colorOutside: text(frame.colorOutside, 50),
      frameWidthMm: number(frame.frameWidth),
      frameDepthMm: number(frame.frameDepth),
      shape: {
        type: text(frame.shape?.shapeType || "rectangular", 40),
        archRadiusMm: number(frame.shape?.archRadius),
        archHeightMm: number(frame.shape?.archHeight),
        topWidthMm: number(frame.shape?.topWidth),
      },
    },
    grid: {
      columns: Array.isArray(kozijn.grid?.columns) ? kozijn.grid.columns.length : 0,
      rows: Array.isArray(kozijn.grid?.rows) ? kozijn.grid.rows.length : 0,
    },
    hasFreeLayout: !!kozijn.layout,
    placement: kozijn.placement ? {
      xMm: number(kozijn.placement.x), yMm: number(kozijn.placement.y),
      zMm: number(kozijn.placement.z), orientationDegrees: number(kozijn.placement.orientation),
    } : null,
    cells: {
      total: cells.length,
      byPanelType,
      first: cells.slice(0, MAX_CELL_PREVIEW).map((cell, index) => ({
        index,
        panelType: text(cell.panelType, 40),
        openingDirection: cell.openingDirection ? text(cell.openingDirection, 40) : null,
        glassType: text(cell.glazing?.glassType, 60),
        glassThicknessMm: number(cell.glazing?.thicknessMm),
        ugValue: number(cell.glazing?.ugValue),
        isEscape: !!cell.isEscape,
      })),
      omitted: Math.max(0, cells.length - MAX_CELL_PREVIEW),
    },
  };
}

/** Pure dispatcher used by the browser bridge and future desktop MCP transport. */
export function runFrameTool(name, args, context) {
  if (!frameToolNames().includes(name)) throw new FrameToolError("onbekende_tool", text(name, 80));
  const project = requireProject(context);
  let result;
  switch (name) {
    case "frame_stand": result = stand(project, context); requireArgs(args, []); break;
    case "frame_kozijnen": result = kozijnen(project, requireArgs(args, ["query", "offset", "limit"])); break;
    case "frame_kozijn": result = kozijnDetail(project, requireArgs(args, ["id"]), context); break;
  }
  if (JSON.stringify(result).length > MAX_OUTPUT_LENGTH) {
    throw new FrameToolError("antwoord_te_groot", "verklein de selectie of filters");
  }
  return result;
}
