import assert from "node:assert/strict";
import { once } from "node:events";
import test from "node:test";
import { createMcpServer } from "./mcp-http.mjs";

test("internal MCP accepts authenticated tool calls and rejects browser origins", async () => {
  const token = "x".repeat(32);
  const project = { projectInfo: { name: "Demo" }, kozijnen: [] };
  const server = createMcpServer(token, async () => ({ project }));
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    const endpoint = "http://127.0.0.1:" + server.address().port + "/mcp";
    const headers = { authorization: "Bearer " + token, accept: "application/json, text/event-stream", "content-type": "application/json" };
    const body = JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: "frame_stand", arguments: {} } });
    const post = (extra = {}, data = body) => fetch(endpoint, { method: "POST", headers: { ...headers, ...extra }, body: data });
    assert.equal((await post({ authorization: "" })).status, 401);
    assert.equal((await post({ origin: "https://example.org" })).status, 403);
    assert.equal((await fetch(endpoint, { headers })).status, 405);
    assert.equal((await post({ "mcp-protocol-version": "unknown" })).status, 400);
    assert.equal((await post({ "content-type": "text/plain" })).status, 415);
    assert.equal((await post({}, " ".repeat(65537))).status, 413);
    assert.equal((await post({}, "{")).status, 400);
    assert.equal((await post({}, '{"jsonrpc":"2.0","method":"notifications/initialized"}')).status, 202);
    const response = await post();
    assert.equal(response.status, 200);
    const { result } = await response.json();
    assert.match(result.content[0].text, /Demo/);
  } finally {
    server.close();
    server.closeAllConnections();
  }
});
