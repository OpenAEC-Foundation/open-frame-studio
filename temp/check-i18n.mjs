// Cross-check every literal i18n key used in ui/src against nl/en/de locale files.
import { readFileSync, readdirSync, statSync } from "fs";
import { join } from "path";

const ROOT = "C:/Bronbestanden/Open Frame Studio/ui/src";
const keys = {};
for (const l of ["nl", "en", "de"]) {
  keys[l] = new Set(Object.keys(JSON.parse(readFileSync(`${ROOT}/locales/${l}.json`, "utf8"))));
}

const used = new Map();
function walk(d) {
  for (const n of readdirSync(d)) {
    const p = join(d, n);
    if (statSync(p).isDirectory()) { if (n !== "node_modules") walk(p); continue; }
    if (!/\.(svelte|js)$/.test(n) || p.includes("locales")) continue;
    const src = readFileSync(p, "utf8");
    for (const m of src.matchAll(/\$_\(\s*['"]([\w.-]+)['"]/g)) used.set(m[1], p);
    for (const m of src.matchAll(/get\(_\)\(\s*['"]([\w.-]+)['"]/g)) used.set(m[1], p);
  }
}
walk(ROOT);

let bad = 0;
for (const [k, p] of [...used].sort()) {
  const miss = ["nl", "en", "de"].filter((l) => !keys[l].has(k));
  if (miss.length) { console.log(`${k}  [${miss}]  ${p.replace(ROOT, "ui/src").replace(/\\/g, "/")}`); bad++; }
}
console.log(bad ? `${bad} missing keys` : `All ${used.size} literal i18n keys resolve in nl/en/de`);
