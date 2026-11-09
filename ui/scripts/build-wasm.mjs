import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const uiDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const wasmDir = resolve(uiDir, "../ofs-wasm");
const pkgDir = join(wasmDir, "pkg");
const publicDir = join(uiDir, "public", "wasm");
const uiVersion = JSON.parse(readFileSync(join(uiDir, "package.json"), "utf8")).version;

const build = spawnSync("wasm-pack", ["build", "--target", "web", "--out-dir", "pkg"], {
  cwd: wasmDir,
  stdio: "inherit",
});
if (build.error) {
  throw new Error(`WASM-build mislukt: ${build.error.message}. Installeer wasm-pack en de wasm32-unknown-unknown Rust-target.`);
}
if (build.status !== 0) process.exit(build.status || 1);

const wasmVersion = JSON.parse(readFileSync(join(pkgDir, "package.json"), "utf8")).version;
if (wasmVersion !== uiVersion) {
  throw new Error(`Versieverschil: UI ${uiVersion}, WASM ${wasmVersion}. Bouw beide uit dezelfde commit.`);
}

mkdirSync(publicDir, { recursive: true });
for (const name of ["ofs_wasm.js", "ofs_wasm_bg.wasm", "package.json"]) {
  copyFileSync(join(pkgDir, name), join(publicDir, name));
}
console.log(`WASM ${wasmVersion} gekopieerd naar ui/public/wasm`);
