import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { createFrameFileContext } from "./project-file.mjs";
import { frameToolNames } from "../../ui/src/lib/frameTools.js";

test("stdio negotiates MCP and calls the browser's area tool on a saved OFS file", async (t) => {
  const dir = await mkdtemp(join(tmpdir(), "ofs-mcp-"));
  t.after(() => rm(dir, { recursive: true, force: true }));
  const file = join(dir, "voorbeeld.ofs");
  await writeFile(file, JSON.stringify({ kozijnen: [{ id: "one", frame: { outerWidth: 1000, outerHeight: 2100 } }] }));
  const requests = [
    { id: 1, method: "initialize", params: { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "test", version: "1" } } },
    { method: "notifications/initialized" },
    { id: 2, method: "tools/list" },
    { id: 3, method: "tools/call", params: { name: "frame_oppervlakte", arguments: { id: "one" } } },
    { id: 4, method: "tools/call", params: { name: "frame_kozijn", arguments: { path: "other.ofs" } } },
  ].map(request => JSON.stringify({ jsonrpc: "2.0", ...request })).join("\n") + "\n";
  const child = spawnSync(process.execPath, [fileURLToPath(new URL("./stdio.mjs", import.meta.url)), file], {
    input: requests, encoding: "utf8", timeout: 10000,
  });
  assert.equal(child.status, 0, child.stderr);
  const results = child.stdout.trim().split("\n").map((line) => JSON.parse(line));
  assert.deepEqual(results.map(r => r.id), [1, 2, 3, 4]);
  assert.equal(results[0].result.protocolVersion, "2025-06-18");
  assert.deepEqual(results[1].result.tools.map(t => t.name), frameToolNames());
  assert.equal(JSON.parse(results[2].result.content[0].text).areaM2, 2.1);
  assert.ok(results[3].result.isError);
  assert.match(results[3].result.content[0].text, /ongeldige_argumenten/);
  const load = createFrameFileContext(file, { root: dir });
  assert.equal((await load()).project.kozijnen.length, 1);
  await writeFile(file, '{"kozijnen":[]}');
  assert.equal((await load()).project.kozijnen.length, 0);
});
