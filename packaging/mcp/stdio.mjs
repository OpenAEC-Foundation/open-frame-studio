import { once } from "node:events";
import { handleMcpRequest, rpcError } from "./protocol.mjs";
import { createFrameFileContext } from "./project-file.mjs";

if (process.argv.length > 3) {
  console.error("Usage: node packaging/mcp/stdio.mjs [project.ofs]");
  process.exit(2);
}
const loadContext = createFrameFileContext(process.argv[2]);
const maxLine = 64 * 1024;
let buffer = Buffer.alloc(0);
let discarding = false;

async function send(message) {
  if (message && !process.stdout.write(JSON.stringify(message) + "\n")) await once(process.stdout, "drain");
}

// A stdio MCP message occupies one UTF-8 line. Ignore the rest of an oversized line.
for await (const chunk of process.stdin) {
  let start = 0;
  while (start < chunk.length) {
    const end = chunk.indexOf(10, start);
    const part = chunk.subarray(start, end === -1 ? chunk.length : end);
    if (!discarding && buffer.length + part.length > maxLine) {
      discarding = true;
      buffer = Buffer.alloc(0);
      await send(rpcError(null, -32700, "Message too large"));
    }
    if (!discarding) buffer = Buffer.concat([buffer, part]);
    if (end === -1) break;
    if (!discarding && buffer.length) {
      try {
        await send(await handleMcpRequest(JSON.parse(buffer.toString("utf8")), loadContext));
      } catch {
        await send(rpcError(null, -32700, "Invalid JSON"));
      }
    }
    buffer = Buffer.alloc(0);
    discarding = false;
    start = end + 1;
  }
}
