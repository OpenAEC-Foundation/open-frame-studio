import test from "node:test";
import assert from "node:assert/strict";
import { workspaceDialogOptions } from "./desktopDialogs.js";

test("native dialogs keep desktop defaults and start in the container mount when managed", () => {
  const options = { defaultPath: "K01_werkplaats.pdf", filters: [] };
  assert.equal(workspaceDialogOptions(options, null), options);
  assert.deepEqual(workspaceDialogOptions(options, "/openaec"), { ...options, defaultPath: "/openaec/K01_werkplaats.pdf" });
  assert.deepEqual(workspaceDialogOptions({ directory: true }, "/openaec"), { directory: true, defaultPath: "/openaec" });
  assert.equal(workspaceDialogOptions({ defaultPath: "../K01.ifc" }, "/openaec").defaultPath, "/openaec/K01.ifc");
});
