// Backend <-> frontend consistency check.
// Extracts #[tauri::command] fns + generate_handler! registrations from src-tauri,
// and invoke()/api() calls from ui/src, then diffs names and parameter keys.
import { readFileSync, readdirSync, statSync } from "fs";
import { join } from "path";

const ROOT = "C:/Bronbestanden/Open Frame Studio";

function walk(dir, exts, skip = ["node_modules", "target", "dist"]) {
  const out = [];
  for (const name of readdirSync(dir)) {
    if (skip.includes(name)) continue;
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) out.push(...walk(p, exts, skip));
    else if (exts.some((e) => name.endsWith(e))) out.push(p);
  }
  return out;
}

// Tauri 2 uses heck's lower-camel-case: leading underscores are dropped
const toCamel = (s) => s.replace(/^_+/, "").replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase());

// ---------- Rust side ----------
const rustFiles = walk(join(ROOT, "src-tauri/src"), [".rs"]);
const commands = new Map(); // name -> { file, params: [rust names], jsKeys: [camelCase] }

for (const f of rustFiles) {
  const src = readFileSync(f, "utf8");
  const re = /#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(([\s\S]*?)\)\s*->/g;
  let m;
  while ((m = re.exec(src))) {
    const [, name, rawParams] = m;
    const params = [];
    for (const seg of rawParams.split(",")) {
      const pm = seg.match(/^\s*(?:mut\s+)?(\w+)\s*:\s*(.+)/s);
      if (!pm) continue;
      const [, pname, ptype] = pm;
      if (/State|AppHandle|Window|Webview/.test(ptype)) continue;
      params.push(pname);
    }
    commands.set(name, { file: f.replace(ROOT + "\\", "").replace(/\\/g, "/"), params, jsKeys: params.map(toCamel) });
  }
}

// Registered in generate_handler!
const mainSrc = readFileSync(join(ROOT, "src-tauri/src/main.rs"), "utf8");
const handlerBlock = mainSrc.match(/generate_handler!\[([\s\S]*?)\]/);
const registered = new Set();
if (handlerBlock) {
  for (const m of handlerBlock[1].matchAll(/(\w+)\s*(?:,|$)/gm)) {
    // entries look like commands::project::new_project — last path segment is the name
  }
  for (const line of handlerBlock[1].split(/[,\n]/)) {
    const t = line.trim().replace(/\/\/.*$/, "").trim();
    if (!t) continue;
    const seg = t.split("::").pop().trim();
    if (/^\w+$/.test(seg)) registered.add(seg);
  }
}

// ---------- JS side ----------
const uiFiles = walk(join(ROOT, "ui/src"), [".js", ".svelte"]);
const calls = []; // { cmd, keys, file, line, dynamic }
for (const f of uiFiles) {
  const rel = f.replace(ROOT + "\\", "").replace(/\\/g, "/");
  if (rel.endsWith("lib/tauri.js")) continue; // defines the layer itself
  const src = readFileSync(f, "utf8");
  const re = /\b(?:invoke|api)\(\s*(['"`])([\w-]+)\1\s*(?:,\s*(\{[\s\S]*?\}))?\s*\)/g;
  let m;
  while ((m = re.exec(src))) {
    const line = src.slice(0, m.index).split("\n").length;
    const keys = [];
    if (m[3]) {
      // first-level keys: strip nested braces crudely
      let depth = 0, buf = "";
      const inner = m[3].slice(1, -1);
      const parts = [];
      for (const ch of inner) {
        if (ch === "{" || ch === "[" || ch === "(") depth++;
        if (ch === "}" || ch === "]" || ch === ")") depth--;
        if (ch === "," && depth === 0) { parts.push(buf); buf = ""; } else buf += ch;
      }
      if (buf.trim()) parts.push(buf);
      for (const p of parts) {
        const km = p.match(/^\s*(?:\.\.\.)?(\w+)\s*(?::|$|\s*$)/);
        if (km) keys.push(km[1]);
      }
    }
    calls.push({ cmd: m[2], keys, file: rel, line });
  }
  // dynamic command names (variable instead of literal)
  const dyn = /\b(?:invoke|api)\(\s*(?!['"`])([\w.]+)\s*[,)]/g;
  while ((m = dyn.exec(src))) {
    if (["cmd", "command"].includes(m[1]) && rel.endsWith("lib/api.js")) continue;
    calls.push({ cmd: `<dynamic:${m[1]}>`, keys: [], file: rel, line: src.slice(0, m.index).split("\n").length, dynamic: true });
  }
}

// ---------- Diff ----------
const calledNames = new Set(calls.filter((c) => !c.dynamic).map((c) => c.cmd));

const definedNotRegistered = [...commands.keys()].filter((n) => !registered.has(n));
const registeredNotDefined = [...registered].filter((n) => !commands.has(n));
const calledNotRegistered = [...calledNames].filter((n) => !registered.has(n));
const registeredNeverCalled = [...registered].filter((n) => !calledNames.has(n));

// Param mismatches: JS keys must match camelCase(rust param) set
const paramIssues = [];
for (const c of calls) {
  if (c.dynamic || !commands.has(c.cmd)) continue;
  const def = commands.get(c.cmd);
  const expected = new Set(def.jsKeys);
  const missing = def.jsKeys.filter((k) => !c.keys.includes(k));
  const extra = c.keys.filter((k) => !expected.has(k));
  if (extra.length || missing.length)
    paramIssues.push({ cmd: c.cmd, at: `${c.file}:${c.line}`, sent: c.keys, expectedJsKeys: def.jsKeys, missing, extra });
}

console.log(JSON.stringify({
  counts: {
    commandFns: commands.size,
    registered: registered.size,
    uiCallSites: calls.filter(c => !c.dynamic).length,
    uniqueCalledCommands: calledNames.size,
    dynamicCalls: calls.filter(c => c.dynamic).length,
  },
  definedNotRegistered,
  registeredNotDefined,
  calledNotRegistered,
  registeredNeverCalled,
  paramIssues,
  dynamicCallSites: calls.filter(c => c.dynamic).map(c => `${c.file}:${c.line} (${c.cmd})`),
}, null, 2));
