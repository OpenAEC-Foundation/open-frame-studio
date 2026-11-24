# Open Frame Studio in Claude Code

The local stdio server reads the saved `.ofs` file supplied on its command line. Each call reads the current file contents. The four tools show project status, list frames, read one frame and calculate its area. Tool arguments cannot select a different file or change a project.

Use Node.js 20.18.3 or a later LTS with JSON import attributes. Keep the Open Frame Studio checkout available. In the Claude Code project, create `.mcp.json` with absolute paths to Node, the server and the project file. This Windows example also works with forward slashes. Adapt the paths for your machine.

```json
{
  "mcpServers": {
    "open-frame": {
      "command": "C:/Program Files/nodejs/node.exe",
      "args": [
        "C:/src/open-frame-studio/packaging/mcp/stdio.mjs",
        "C:/projects/example.ofs"
      ]
    }
  }
}
```

Start Claude Code in that project and approve the `open-frame` server when prompted. Run `claude mcp get open-frame` to check the connection. In the session, ask for the available frames and the area of one of them. [Claude Code project MCP configuration](https://code.claude.com/docs/en/mcp#project-scope)

## Verified clients

On 24 November 2026, Claude Code 2.1.331 on Windows x64 reported `Connected` for both `claude mcp get open-frame` and `claude mcp list`. The test used a temporary project-scoped configuration with a synthetic frame. The official Claude Agent SDK 0.3.331 queried that Claude Code process and reported a connected project server with `frame_stand`, `frame_kozijnen`, `frame_kozijn` and `frame_oppervlakte`. The official MCP JavaScript SDK also called `frame_oppervlakte` and obtained 1.8 m² for a 1200 × 1500 mm frame. The connection and tool-discovery tests required no model request. [Claude Agent SDK server status](https://code.claude.com/docs/en/agent-sdk/typescript#mcpserverstatus)

For Claude Desktop, add the same `open-frame` entry under `mcpServers` in **Settings → Developer → Edit Config**, fully restart the app and check its list of connectors. Desktop GUI acceptance and an installable desktop extension are still open. [Claude Desktop local server configuration](https://modelcontextprotocol.io/docs/develop/connect-local-servers)
