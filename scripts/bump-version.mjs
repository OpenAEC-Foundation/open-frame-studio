#!/usr/bin/env node
// Bump the Open Frame Studio version in EVERY place at once.
//
//   node scripts/bump-version.mjs 0.5.2
//
// Most of the app already derives its version from a single source:
//   • the Svelte UI reads ui/package.json      (via ui/src/lib/version.js)
//   • every Rust crate inherits [workspace.package].version  (Cargo.toml)
//   • ofs-cloud's /health reads env!("CARGO_PKG_VERSION")    (Cargo.toml)
// So the truly authoritative files are Cargo.toml + ui/package.json +
// src-tauri/tauri.conf.json. The remaining spots (the standalone ofs-web
// pages and the README title) are static, so this script rewrites them too —
// that way "bump the version" is one command and nothing is forgotten.
//
// It does NOT touch historical changelog headings (e.g. "## Changelog v0.4.0").

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const next = process.argv[2];
if (!next || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(next)) {
  console.error("Usage: node scripts/bump-version.mjs <semver>   (e.g. 0.5.2)");
  process.exit(1);
}

// The current version is the single source we key off (ui/package.json).
const pkgPath = join(root, "ui/package.json");
const prev = JSON.parse(readFileSync(pkgPath, "utf8")).version;
if (prev === next) {
  console.log(`Version is already ${next} — nothing to do.`);
  process.exit(0);
}

const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Replace exactly one occurrence matched by `re`; throw if not found. */
function edit(relPath, re, replacement, { optional = false } = {}) {
  const abs = join(root, relPath);
  let text;
  try {
    text = readFileSync(abs, "utf8");
  } catch (e) {
    if (optional) return console.warn(`skip  ${relPath} (not found)`);
    throw e;
  }
  if (!re.test(text)) {
    if (optional) return console.warn(`skip  ${relPath} (pattern not found)`);
    throw new Error(`Pattern not found in ${relPath}: ${re}`);
  }
  writeFileSync(abs, text.replace(re, replacement));
  console.log(`ok    ${relPath}`);
}

// 1. Rust workspace — every crate inherits this; ofs-cloud /health reads it.
edit("Cargo.toml", /(\[workspace\.package\][\s\S]*?version\s*=\s*")[^"]+(")/, `$1${next}$2`);
// 2. UI — the Svelte shell reads this via ui/src/lib/version.js.
edit("ui/package.json", /("version":\s*")[^"]+(")/, `$1${next}$2`);
// 3. Tauri bundle metadata.
edit("src-tauri/tauri.conf.json", /("version":\s*")[^"]+(")/, `$1${next}$2`);
// 4. Standalone ofs-web pages + README title (static — replace the old vX.Y.Z).
edit("ofs-web/index.html", new RegExp(`v${esc(prev)}\\b`), `v${next}`, { optional: true });
edit("ofs-web/workshop.html", new RegExp(`v${esc(prev)}\\b`), `v${next}`, { optional: true });
edit("README.md", new RegExp(`^(# Open Frame Studio )v${esc(prev)}\\b`, "m"), `$1v${next}`, { optional: true });

console.log(`\nBumped ${prev} -> ${next}.`);
console.log("Reminder: add a changelog entry to README.md and run `npm install` in ui/ to refresh the lockfile.");
