import test from "node:test";
import assert from "node:assert/strict";
import { canStoreWorkspaceSetting, hostPreferences, readWorkspaceSettings } from "./hostWorkspaceSettings.js";

test("restores only UI preferences with values supported by the workspace", async () => {
  const fields = { "frame.locale": "tr", "frame.right_panel_open": false, "frame.left_panel_width": "900px",
    "frame.ribbon_layout": { minimized: true, hiddenGroups: "x", hiddenTabs: [] } };
  const keys = [];
  const settings = { get: async (key) => { keys.push(key); return fields[key]; } };
  assert.deepEqual(await readWorkspaceSettings(settings), { locale: "tr", right_panel_open: false });
  assert.ok(keys.every((key) => key.startsWith("frame.")));
  assert.equal(canStoreWorkspaceSetting("ai_key", "private"), false);
});

test("host preferences accept supported regional locales and light or dark", () => {
  assert.deepEqual(hostPreferences({ taal: "TR-tr", thema: "dark" }, ["nl", "tr"]), { locale: "tr", theme: "dark" });
  assert.deepEqual(hostPreferences({ taal: "xx", thema: "invalid" }, ["nl"]), { locale: null, theme: null });
});
