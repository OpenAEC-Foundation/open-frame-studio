import { frameToolDefinitions, frameToolNames, runFrameTool } from "../../ui/src/lib/frameTools.js";
import ui from "../../ui/package.json" with { type: "json" };

export const protocolVersions = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

export function rpcError(id, code, message) {
  return { jsonrpc: "2.0", id, error: { code, message } };
}

const object = (value) => value !== null && typeof value === "object" && !Array.isArray(value);

export async function handleMcpRequest(request, loadContext) {
  if (!object(request) || request.jsonrpc !== "2.0" || typeof request.method !== "string") {
    return rpcError(null, -32600, "Invalid request");
  }
  const { id, method, params } = request;
  if (id === undefined) return undefined;
  if (!(typeof id === "string" || Number.isSafeInteger(id))) return rpcError(null, -32600, "Invalid request id");
  if (params !== undefined && !object(params)) return rpcError(id, -32602, "Invalid parameters");
  let result;
  switch (method) {
    case "initialize": {
      if (typeof params?.protocolVersion !== "string" || !object(params.capabilities) || !object(params.clientInfo)) {
        return rpcError(id, -32602, "Invalid initialization parameters");
      }
      const protocolVersion = protocolVersions.includes(params.protocolVersion) ? params.protocolVersion : protocolVersions[0];
      result = { protocolVersion, capabilities: { tools: {} }, serverInfo: { name: "open-frame-studio", version: ui.version } };
      break;
    }
    case "ping": result = {}; break;
    case "tools/list": result = { tools: frameToolDefinitions() }; break;
    case "tools/call": {
      if (!frameToolNames().includes(params?.name)) return rpcError(id, -32602, "Unknown tool");
      if (params.arguments !== undefined && !object(params.arguments)) return rpcError(id, -32602, "Invalid tool arguments");
      try {
        const data = runFrameTool(params.name, params.arguments, await loadContext());
        result = { content: [{ type: "text", text: JSON.stringify(data) }] };
      } catch (error) {
        result = { isError: true, content: [{ type: "text", text: error?.name === "FrameToolError" ? error.message : "project_onleesbaar" }] };
      }
      break;
    }
    default: return rpcError(id, -32601, "Method not found");
  }
  return { jsonrpc: "2.0", id, result };
}
