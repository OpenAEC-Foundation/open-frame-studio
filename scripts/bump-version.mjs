#!/usr/bin/env node
// Bump the Open Frame Studio version in EVERY place at once.
//
//   node scripts/bump-version.mjs --next
//   node scripts/bump-version.mjs 2026.46.0
//
// Most of the app already derives its version from a single source:
//   • the Svelte UI reads ui/package.json      (via ui/src/lib/version.js)
//   • every Rust crate inherits [workspace.package].version  (Cargo.toml)
//   • ofs-cloud's /health reads env!("CARGO_PKG_VERSION")    (Cargo.toml)
// So the truly authoritative files are Cargo.toml + ui/package.json +
// src-tauri/tauri.conf.json. Lockfiles are updated with these source versions.
// The remaining spots (the standalone ofs-web
// pages and the README title) are static, so this script rewrites them too —
// that way "bump the version" is one command and nothing is forgotten.
//
// It does NOT touch historical changelog headings (e.g. "## Changelog v0.4.0").

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// The current version is the single source we key off (ui/package.json).
const pkgPath = join(root, "ui/package.json");
const prev = JSON.parse(readFileSync(pkgPath, "utf8")).version;
const yearWeekVersion = /^(\d{4})\.([1-9]|[1-4]\d|5[0-3])\.(0|[1-9]\d*)$/;

function currentIsoYearWeek() {
  const fields = Object.fromEntries(
    new Intl.DateTimeFormat("en-GB", {
      timeZone: "Europe/Amsterdam", year: "numeric", month: "2-digit", day: "2-digit",
    }).formatToParts(new Date()).filter((part) => part.type !== "literal")
      .map((part) => [part.type, Number(part.value)]),
  );
  const thursday = new Date(Date.UTC(fields.year, fields.month - 1, fields.day));
  thursday.setUTCDate(thursday.getUTCDate() + 4 - (thursday.getUTCDay() || 7));
  const year = thursday.getUTCFullYear();
  const firstDay = new Date(Date.UTC(year, 0, 1));
  const week = Math.ceil((((thursday - firstDay) / 86400000) + 1) / 7);
  return { year, week };
}

let next = process.argv[2];
const showNext = next === "--show-next";
if (next === "--next" || showNext) {
  const { year, week } = currentIsoYearWeek();
  const previous = yearWeekVersion.exec(prev);
  const sequence = previous && Number(previous[1]) === year && Number(previous[2]) === week
    ? Number(previous[3]) + 1 : 0;
  next = `${year}.${week}.${sequence}`;
}
if (!next || !yearWeekVersion.test(next)) {
  console.error("Usage: node scripts/bump-version.mjs --next | --show-next | <JJJJ.W.N>   (e.g. 2026.46.0)");
  process.exit(1);
}
if (showNext) {
  console.log(next);
  process.exit(0);
}
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
// 4. Keep the workspace package entries and npm root package in sync.
for (const name of ["ofs-core", "ofs-wasm", "ofs-cloud", "open-frame-studio"]) {
  edit("Cargo.lock", new RegExp(`(\\[\\[package\\]\\]\\s+name = "${name}"\\s+version = ")[^"]+(")`), `$1${next}$2`);
}
const packageLockPath = join(root, "ui/package-lock.json");
const packageLock = JSON.parse(readFileSync(packageLockPath, "utf8"));
packageLock.version = next;
if (!packageLock.packages?.[""]) throw new Error("ui/package-lock.json mist het rootpakket");
packageLock.packages[""].version = next;
writeFileSync(packageLockPath, JSON.stringify(packageLock, null, 2) + "\n");
console.log("ok    ui/package-lock.json");
// 5. Standalone ofs-web pages + README title (static — replace the old vX.Y.Z).
edit("ofs-web/index.html", new RegExp(`v${esc(prev)}\\b`), `v${next}`, { optional: true });
edit("ofs-web/workshop.html", new RegExp(`v${esc(prev)}\\b`), `v${next}`, { optional: true });
edit("README.md", new RegExp(`^(# Open Frame Studio )v${esc(prev)}\\b`, "m"), `$1v${next}`, { optional: true });

console.log(`\nBumped ${prev} -> ${next}.`);
console.log("Reminder: werk release-notities bij en draai `node scripts/check-version.mjs`.");
