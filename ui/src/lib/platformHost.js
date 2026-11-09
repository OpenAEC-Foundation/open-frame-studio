/** OpenAEC SuperCloud Docs bridge for the browser build.
 *
 * The host owns permissions, version conflicts, and storage. OFS only handles
 * its own UTF-8 .ofs project JSON. This module has no browser storage fallback.
 */

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });

export function getOpenAecHost(target = globalThis.window) {
  const host = target?.openaecHost;
  return host && typeof host.open === "function" && typeof host.save === "function"
    ? host
    : null;
}

export function ensureOfsName(name) {
  const trimmed = String(name || "").trim();
  if (!trimmed) throw new Error("Geef een naam voor het .ofs-project op");
  return /\.ofs$/i.test(trimmed) ? trimmed : `${trimmed}.ofs`;
}

export function decodeOfsBytes(bytes) {
  if (bytes instanceof ArrayBuffer) return decoder.decode(new Uint8Array(bytes));
  if (ArrayBuffer.isView(bytes)) {
    return decoder.decode(new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength));
  }
  throw new Error("Docs leverde geen leesbare .ofs-bestandsinhoud");
}

function fileRef(value, readOnly = false) {
  if (!value || typeof value.id !== "string" || !value.id || typeof value.name !== "string" || !value.name) {
    throw new Error("Docs leverde geen geldig bestandskenmerk");
  }
  return { id: value.id, name: value.name, version: value.version, readOnly: !!readOnly };
}

export async function readHostProject(host, id, version) {
  const file = await host.open(id, version);
  const ref = fileRef(file, file?.readOnly);
  if (!/\.ofs$/i.test(ref.name)) throw new Error("Alleen .ofs-projecten kunnen worden geopend");
  return { json: decodeOfsBytes(file.bytes), ref };
}

/** Ignore a late Docs response if the local project changed while it loaded. */
export async function openHostProjectIfCurrent(host, id, version, { captureState, isStateCurrent, apply }) {
  const state = await captureState();
  const file = await readHostProject(host, id, version);
  if (!await isStateCurrent(state)) return false;
  await apply(file);
  return true;
}

export async function listHostProjects(host) {
  if (typeof host.list !== "function") throw new Error("Docs ondersteunt de bestandslijst niet");
  const files = await host.list();
  if (!Array.isArray(files)) throw new Error("Docs leverde geen geldige bestandslijst");
  return files.filter((file) => typeof file?.id === "string" && /\.ofs$/i.test(file?.name || ""));
}

export async function writeHostProject(host, json, { current = null, name = null } = {}) {
  if (typeof json !== "string") throw new Error("Projectinhoud ontbreekt");
  if (current?.readOnly && !name) throw new Error("Deze projectversie is alleen-lezen. Gebruik Opslaan als.");
  if (current && !name && (current.version === undefined || current.version === null)) {
    throw new Error("Docs leverde geen versie voor veilig opslaan; gebruik Opslaan als");
  }
  const options = name
    ? { name: ensureOfsName(name) }
    : current
      ? { id: current.id, expectedVersion: current.version }
      : { name: "project.ofs" };
  const saved = await host.save(encoder.encode(json), options);
  return fileRef(saved, false);
}
