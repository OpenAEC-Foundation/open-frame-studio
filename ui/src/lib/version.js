// Single source of truth for the app version shown in the UI.
//
// Reads straight from ui/package.json, so bumping the version there (or via
// scripts/bump-version.mjs) propagates automatically to the title bar, the
// backstage app menu and the settings dialog — no more hand-editing "v0.4.0"
// in three components and forgetting one.
import pkg from "../../package.json";

export const APP_VERSION = pkg.version;
