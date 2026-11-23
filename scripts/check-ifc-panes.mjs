// Generate the same exports users receive, then validate their IFC records with Open BIM Suite.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const base = resolve(process.argv[2] ?? join(root, "target", "ifc-ruitcontrole"));
mkdirSync(base, { recursive: true });
const run = mkdtempSync(join(base, "run-"));
const ids = join(root, "ofs-core", "examples", "ifc", "ruit-onderdelen.ids");
const tool = join(root, "tools", "ifc-ids-check", "Cargo.toml");
const read = path => readFileSync(path, "utf8");
const json = path => JSON.parse(read(path));

function cargo(args, expectedExit = 0) {
  const result = spawnSync("cargo", args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  assert.equal(result.status, expectedExit, `cargo ${args[0]}: exit ${result.status ?? result.signal}, verwacht ${expectedExit}`);
}

function validate(name, file, expected, shouldPass) {
  const output = join(run, name + ".json");
  console.log(`\n${name}: ${basename(file)}, ${expected.length} verwachte ruiten`);
  cargo(["run", "--locked", "--quiet", "--manifest-path", tool, "--", ids, file,
    JSON.stringify(expected), output], shouldPass ? 0 : 1);
  const result = json(output);
  assert.equal(result.status, shouldPass ? "passed" : "failed", name);
  return result;
}

function failingIds(result, facet) {
  return result.report.specifications.flatMap(spec => spec.findings)
    .filter(finding => finding.status === "failed" && finding.facet === facet)
    .map(finding => finding.global_id);
}

function changedFile(name, original, changed) {
  assert.notEqual(changed, original, `De proef "${name}" wijzigde geen IFC-record`);
  const file = join(run, name + ".ifc");
  writeFileSync(file, changed);
  return file;
}

console.log("IFC-proefbestanden en rapporten:", run);
cargo(["run", "--locked", "--quiet", "-p", "ofs-core", "--example", "ifc_acceptance_fixture", "--", run]);
assert.ok(statSync(join(run, "01-los-kozijn.ifc")).size > 0, "Lege export van het losse kozijn");
const beforeFile = join(run, "03-onderdelen.ifc");
const afterFile = join(run, "05-onderdelen-revisie.ifc");
const manifest = json(join(run, "ruit-verwachting.json"));
const before = manifest[basename(beforeFile)];
const after = manifest[basename(afterFile)];
assert.equal(before.length, 3, "De bronproef bevat drie ruiten");
assert.equal(after.length, 4, "De revisie voegt één ruit toe");
assert.ok(before.every(id => after.includes(id)), "De bronrevisie moet de bestaande ruit-ID's behouden");
validate("voor-revisie", beforeFile, before, true);
validate("na-revisie", afterFile, after, true);

// Restore the former IfcPlate representation of one pane, keeping its GlobalId,
// relationships and shape. IfcPlate shares the first eight attributes of IfcWindow.
const original = read(beforeFile);
const windowRow = /\bIFCWINDOW\(('([0-3][0-9A-Za-z_$]{21})',[^;\r\n]*),[^,]+,[^,]+,\.WINDOW\.,\$,\$\);/;
const match = original.match(windowRow);
assert.ok(match && before.includes(match[2]), "Geen verwachte IfcWindow gevonden voor de klasseproef");
const wrongClass = changedFile("fout-klasse", original,
  original.replace(windowRow, (_, common) => `IFCPLATE(${common},.SHEET.);`));
const classResult = validate("controle-klasse", wrongClass, before, false);
assert.deepEqual(failingIds(classResult, "entity"), [match[2]], "Verwacht een klassefout op de gewijzigde ruit");

// Change just one Onderdeel value. The other panes must still pass the IDS.
const wrongLabel = changedFile("fout-onderdeel", original, original.replace(
  /(IFCPROPERTYSINGLEVALUE\('Onderdeel',(?:''|\$),IFCLABEL\()'Ruit'/, "$1'Paneel'"));
const labelResult = validate("controle-onderdeel", wrongLabel, before, false);
const badLabels = failingIds(labelResult, "property");
assert.ok(badLabels.length === 1 && before.includes(badLabels[0]), "Verwacht één eigenschapsfout met ruit-ID");

// A valid earlier export is missing exactly the pane added by the revision.
const missing = validate("controle-ontbrekende-ruit", beforeFile, after, false);
assert.deepEqual(missing.missingGlobalIds, after.filter(id => !before.includes(id)));

// Equal counts and a passing IDS must still catch a replaced GlobalId.
const replacement = "0000000000000000000001";
assert.ok(!original.includes(replacement), "De vervangende proef-ID moet ongebruikt zijn");
const changedId = changedFile("fout-globalid", original,
  original.replace(windowRow, row => row.replace(match[2], replacement)));
const identity = validate("controle-globalid", changedId, before, false);
assert.equal(identity.report.status, "passed", "De klasse en eigenschappen van de ruit zijn behouden");
assert.deepEqual(identity.missingGlobalIds, [match[2]]);
assert.deepEqual(identity.unexpectedGlobalIds, [replacement]);

console.log("\nIFC-ruitcontrole geslaagd. De revisie behoudt drie ruit-ID's en voegt één ruit toe.");
