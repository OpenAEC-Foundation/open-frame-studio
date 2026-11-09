#!/usr/bin/env node
// Fail CI when a candidate build displays or packages a different version.
import { readFileSync } from "node:fs";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const cargo = read("Cargo.toml");
const version = cargo.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/)?.[1];
if (!version) throw new Error("Workspace version ontbreekt in Cargo.toml");
if (!/^\d{4}\.([1-9]|[1-4]\d|5[0-3])\.(0|[1-9]\d*)$/.test(version)) {
  throw new Error(`Versie ${version} volgt niet JJJJ.W.N (ISO-week zonder voorloopnul)`);
}
const requestedTag = process.argv[2];
if (requestedTag && requestedTag !== `v${version}`) {
  throw new Error(`Release-tag ${requestedTag} komt niet overeen met bronversie v${version}`);
}

const actual = {
  "ui/package.json": JSON.parse(read("ui/package.json")).version,
  "ui/package-lock.json": JSON.parse(read("ui/package-lock.json")).packages?.[""]?.version,
  "src-tauri/tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
  "README.md": read("README.md").match(/^# Open Frame Studio v([^\r\n]+)/m)?.[1],
  "ofs-web/index.html": read("ofs-web/index.html").match(/Kozijn Configurator v([^<\s]+)/)?.[1],
  "ofs-web/workshop.html": read("ofs-web/workshop.html").match(/>v([^<\s]+)<\/span>/)?.[1],
};

const lock = read("Cargo.lock");
for (const name of ["ofs-core", "ofs-wasm", "ofs-cloud", "open-frame-studio"]) {
  actual[`Cargo.lock:${name}`] = lock.match(new RegExp(`\\[\\[package\\]\\]\\s+name = "${name}"\\s+version = "([^"]+)"`))?.[1];
}

const mismatches = Object.entries(actual).filter(([, value]) => value !== version);
if (mismatches.length) {
  for (const [name, value] of mismatches) {
    console.error(`${name}: ${value ?? "ontbreekt"} (verwacht ${version})`);
  }
  process.exit(1);
}
console.log(`Alle bron- en pakketversies zijn ${version}`);
