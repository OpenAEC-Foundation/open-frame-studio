import assert from "node:assert/strict";

const base = new URL(process.argv[2]);
const token = process.env.OFS_API_TOKEN;
assert.ok(token && token.length >= 32, "Set OFS_API_TOKEN for the service under test");
assert.ok(base.protocol === "https:" || (base.protocol === "http:" &&
  ["localhost", "127.0.0.1", "[::1]"].includes(base.hostname)), "Use HTTPS or HTTP on loopback");

function request(path, options = {}) {
  return fetch(new URL(path, base), { redirect: "error", signal: AbortSignal.timeout(5000), ...options });
}

function estimate(body, bearer = token, contentType = "application/json") {
  const headers = { "Content-Type": contentType };
  if (bearer) headers.Authorization = "Bearer " + bearer;
  return request("/api/v1/estimates/fixed-frame", { method: "POST", headers, body });
}

const health = await request("/api/v1/health");
assert.equal(health.status, 200);
const { status, version } = await health.json();
assert.equal(status, "ok");
assert.ok(version);
const standard = JSON.stringify({ widthMm: 1200, heightMm: 1400 });
for (const bearer of ["", "invalid-test-token"]) {
  const response = await estimate(standard, bearer);
  assert.equal(response.status, 401);
  assert.equal(response.headers.get("www-authenticate"), "Bearer");
}
assert.equal((await estimate("{")).status, 400);
assert.equal((await estimate(standard, token, "text/plain")).status, 415);
assert.equal((await estimate(JSON.stringify({ widthMm: 1200, heightMm: 1400, other: 1 }))).status, 422);
assert.equal((await estimate(JSON.stringify({ widthMm: 6001, heightMm: 1400 }))).status, 422);
assert.equal((await estimate(" ".repeat(4096) + standard)).status, 413);
assert.equal((await request("/api/v1/projects")).status, 404);

async function checkedEstimate(body) {
  const response = await estimate(body);
  assert.equal(response.status, 200);
  const result = await response.json();
  const sum = result.lineItems.reduce((total, item) => total + item.total, 0);
  assert.ok(result.materialCost > 0 && result.glassCost > 0 && result.laborHours > 0);
  assert.ok(Math.abs(result.totalCost - sum) < 0.01);
  return result;
}
const small = await checkedEstimate(standard);
const large = await checkedEstimate(JSON.stringify({ widthMm: 2400, heightMm: 1400 }));
assert.ok(large.totalCost > small.totalCost);
console.log(`Open Frame Studio ${version}: health, authentication, validation, body limit and two estimates passed. Totals EUR ${small.totalCost.toFixed(2)} / ${large.totalCost.toFixed(2)}.`);
