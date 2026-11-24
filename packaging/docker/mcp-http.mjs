import { createHash, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { pathToFileURL } from "node:url";
import { createFrameFileContext } from "../mcp/project-file.mjs";
import { handleMcpRequest, protocolVersions, rpcError } from "../mcp/protocol.mjs";

const maxBody = 64 * 1024;
const digest = (text) => createHash("sha256").update(text).digest();

export function createMcpServer(token, loadContext) {
  if (!/^[A-Za-z0-9._~+\/-]{32,512}=*$/.test(token)) throw new Error("A Bearer token of at least 32 characters is required");
  const expected = digest("Bearer " + token);
  let active = 0;
  return createServer({ requestTimeout: 15000, headersTimeout: 10000, maxHeaderSize: 8192 }, async (req, res) => {
    function reply(status, json, headers = {}) {
      res.writeHead(status, { "Cache-Control": "no-store", ...(json ? { "Content-Type": "application/json" } : {}), ...headers });
      res.end(json ? JSON.stringify(json) : undefined);
    }
    if (req.url !== "/mcp") return reply(404);
    // This internal endpoint is called server-to-server by the SuperCloud.
    if (req.headers.origin !== undefined) return reply(403);
    if (!timingSafeEqual(expected, digest(req.headers.authorization || ""))) {
      return reply(401, undefined, { "WWW-Authenticate": 'Bearer realm="Open Frame Studio"' });
    }
    if (req.method !== "POST") return reply(405, undefined, { Allow: "POST" });
    const version = req.headers["mcp-protocol-version"];
    if (version && !protocolVersions.includes(version)) return reply(400, rpcError(null, -32600, "Unsupported protocol version"));
    if (req.headers["content-type"]?.split(";")[0].trim().toLowerCase() !== "application/json") return reply(415);
    const accepts = (req.headers.accept || "").toLowerCase().split(",").map((item) => item.split(";")[0].trim());
    if (!accepts.includes("application/json") || !accepts.includes("text/event-stream")) return reply(406);
    if (Number(req.headers["content-length"]) > maxBody) return reply(413);
    if (active >= 4) return reply(503, undefined, { "Retry-After": "1" });

    active++;
    try {
      let length = 0;
      const chunks = [];
      for await (const chunk of req) {
        length += chunk.length;
        if (length > maxBody) return reply(413);
        chunks.push(chunk);
      }
      let request;
      try {
        request = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks)));
      } catch {
        return reply(400, rpcError(null, -32700, "Parse error"));
      }
      const response = await handleMcpRequest(request, loadContext);
      if (response === undefined) return reply(202);
      return reply(response.error?.code === -32600 ? 400 : 200, response);
    } catch {
      if (!res.destroyed) reply(500, rpcError(null, -32603, "Internal error"));
    } finally {
      active--;
    }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const token = (await readFile(process.env.OPENAEC_MCP_TOKEN_FILE, "utf8")).trim();
  const loadContext = createFrameFileContext(process.env.OPENAEC_OPEN, { root: "/openaec" });
  createMcpServer(token, loadContext).listen(3001, "0.0.0.0");
}
