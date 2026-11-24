// Run after npm run build:platform. Playwright may be installed separately.
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { extname, join, resolve, relative, isAbsolute } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.OFS_PLAYWRIGHT || "playwright");
const dist = resolve(process.argv[2] || fileURLToPath(new URL("../dist", import.meta.url)));
const wasm = await import(pathToFileURL(join(dist, "wasm/ofs_wasm.js")));
wasm.initSync({ module: await readFile(join(dist, "wasm/ofs_wasm_bg.wasm")) });
wasm.new_project("Docs rookproef", "SMOKE");
wasm.create_kozijn("Voorgevel", "K01", 1200, 1500);
const model = wasm.save_project_json();
const types = { ".html": "text/html", ".js": "application/javascript", ".css": "text/css", ".json": "application/json",
  ".wasm": "application/wasm", ".woff2": "font/woff2", ".svg": "image/svg+xml", ".png": "image/png" };

const server = createServer(async (req, res) => {
  try {
    if (req.url === "/") {
      res.setHeader("Content-Type", "text/html");
      return res.end('<iframe title="Werkplek" sandbox="allow-scripts allow-modals" style="width:100%;height:95vh" src="/werkplek/app/frame/index.html"></iframe>');
    }
    const prefix = "/werkplek/app/frame/";
    const pathname = new URL(req.url, "http://localhost").pathname;
    if (!pathname.startsWith(prefix)) { res.writeHead(404); return res.end(); }
    const path = resolve(dist, decodeURIComponent(pathname.slice(prefix.length)));
    const suffix = relative(dist, path);
    if (isAbsolute(suffix) || suffix.startsWith("..")) { res.writeHead(403); return res.end(); }
    const body = await readFile(path);
    const origin = "http://127.0.0.1:" + server.address().port;
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Content-Security-Policy", "default-src 'none'; script-src " + origin + " 'wasm-unsafe-eval'; connect-src " +
      origin + "; style-src " + origin + " 'unsafe-inline'; img-src " + origin + " data:; font-src " + origin + ";");
    res.setHeader("Content-Type", types[extname(path)] || "application/octet-stream");
    res.end(body);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
let browser;
try {
  browser = await chromium.launch({ headless: true, executablePath: process.env.OFS_BROWSER || undefined });
  const page = await browser.newPage();
  page.setDefaultTimeout(10000);
  const errors = [];
  const external = [];
  const origin = "http://127.0.0.1:" + server.address().port;
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
  await page.route("**/*", (route) => {
    if (new URL(route.request().url()).origin === origin) return route.continue();
    external.push(route.request().url());
    return route.abort();
  });
  await page.addInitScript((model) => {
    if (window === top) return;
    const settings = new Map([["frame.left_panel_open", false]]);
    const reads = [], saves = [], preferences = [];
    window.testDocs = { reads, saves, preferences };
    window.openaecHost = {
      initialFileId: "first", taal: "nl", thema: "dark",
      settings: { get: async k => settings.get(k), set: async (k,v) => { preferences.push([k,v]); settings.set(k,v); } },
      list: async () => [{ id: "first", name: "Project.ofs" }, { id: "old", name: "Historisch.ofs" }],
      open: async id => { reads.push(id); return { id, name: id === "old" ? "Historisch.ofs" : "Project.ofs", version: 1,
        readOnly: id === "old", bytes: new TextEncoder().encode(model) }; },
      save: async (bytes, ref) => {
        saves.push({ ref, project: JSON.parse(new TextDecoder().decode(bytes)).projectInfo.name });
        return ref.id ? { id: "conflict", name: "Project (conflict).ofs", version: 2 } : { id: "copy", name: ref.name, version: 1 };
      },
    };
  }, model);
  await page.goto(origin);
  const frame = page.frames().find(f => f.url().includes("/werkplek/app/frame/"));
  assert.ok(frame);
  await frame.waitForFunction(async () => {
    try { return (await window.__OPDS_APP__?.run("frame_stand")).file?.id === "first"; } catch { return false; }
  }, undefined, { timeout: 20000 }).catch((error) => { throw new Error(error.message + "\n" + errors.join("\n")); });
  assert.equal(await frame.evaluate(() => window.origin), "null");
  assert.equal(await frame.evaluate(() => document.documentElement.dataset.oaTheme), "dark");
  const leftPanel = frame.locator("aside.side-panel:not(.right) button.panel-toggle");
  assert.equal(await leftPanel.getAttribute("aria-expanded"), "false");
  await leftPanel.click();
  await frame.waitForFunction(() => window.testDocs.preferences.some(([key, value]) => key === "frame.left_panel_open" && value === true));
  const tools = await frame.evaluate(() => window.__OPDS_APP__.definitions());
  assert.deepEqual(tools, JSON.parse(await readFile(join(dist, "mcp-tools.json"), "utf8")));
  assert.equal(await frame.evaluate(async () => (await window.__OPDS_APP__.run("frame_oppervlakte")).areaM2), 1.8);
  const menu = frame.locator("button.file-tab");
  await menu.click();
  await frame.getByRole("button", { name: /Openen.*Ctrl\+O/ }).first().click();
  const dialog = frame.getByRole("dialog", { name: /Docs/ });
  await dialog.getByRole("listbox").selectOption("old");
  await dialog.getByRole("button", { name: "Openen" }).click();
  await frame.waitForFunction(async () => (await window.__OPDS_APP__.run("frame_stand")).file?.readOnly);
  await menu.press("Control+Shift+s");
  await dialog.getByRole("textbox").fill("Eigen kopie");
  await dialog.getByRole("button", { name: "Opslaan" }).click();
  await frame.waitForFunction(async () => (await window.__OPDS_APP__.run("frame_stand")).file?.id === "copy");
  await menu.press("Control+s");
  await frame.waitForFunction(async () => (await window.__OPDS_APP__.run("frame_stand")).file?.id === "conflict");
  const actions = await frame.evaluate(() => window.testDocs);
  assert.deepEqual(actions.reads, ["first", "old"]);
  assert.deepEqual(actions.saves.map(s => s.ref), [{ name: "Eigen kopie.ofs" }, { id: "copy", expectedVersion: 1 }]);
  assert.ok(actions.saves.every(s => s.project === "Docs rookproef"));
  await frame.evaluate(() => {
    Object.assign(window.openaecHost, { taal: "en", thema: "light" });
    window.dispatchEvent(new Event("openaec-host-preferences"));
  });
  await frame.getByRole("button", { name: "File", exact: true }).first().waitFor();
  assert.equal(await frame.evaluate(() => document.documentElement.dataset.oaTheme), "light");
  assert.deepEqual(external, []);
  assert.deepEqual(errors, []);
  console.log("Opaque iframe: Docs open, read-only, Save As, conflict, preferences and all tool definitions passed.");
} finally {
  await browser?.close();
  server.close();
}
