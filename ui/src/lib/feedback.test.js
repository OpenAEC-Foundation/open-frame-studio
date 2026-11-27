import test from "node:test";
import assert from "node:assert/strict";
import {
  FEEDBACK_APP, FEEDBACK_URL, buildFeedbackPayload, sendFeedback, validateFeedback,
} from "./feedback.js";

const ok = { category: "bug", message: "De maatlijn verspringt na opslaan." };

test("validation: length, category and optional email", () => {
  assert.equal(validateFeedback(ok), null);
  assert.equal(validateFeedback({ ...ok, message: "kort" }), "tooShort");
  assert.equal(validateFeedback({ ...ok, message: "   " + "x".repeat(5) + "   " }), "tooShort");
  assert.equal(validateFeedback({ ...ok, message: "x".repeat(5001) }), "tooLong");
  assert.equal(validateFeedback({ ...ok, category: "spam" }), "category");
  assert.equal(validateFeedback({ ...ok, email: "" }), null);
  assert.equal(validateFeedback({ ...ok, email: "jan@bedrijf.nl" }), null);
  assert.equal(validateFeedback({ ...ok, email: "geen-adres" }), "badEmail");
  assert.equal(validateFeedback({ ...ok, email: "a@b" }), "badEmail");
});

test("payload carries only what the user wrote plus version/platform, email only when given", () => {
  const ctx = { version: "2026.48.4", platform: "Windows", language: "nl" };
  const p = buildFeedbackPayload({ ...ok, email: "  " }, ctx);
  assert.equal(p.app, FEEDBACK_APP);
  assert.equal(p.category, "bug");
  assert.equal(p.appVersion, "2026.48.4");
  assert.ok(!("email" in p));
  assert.ok(p.message.startsWith("De maatlijn verspringt na opslaan."));
  assert.match(p.message, /Open Frame Studio v2026\.48\.4 · Windows · nl/);
  assert.deepEqual(Object.keys(p).sort(), ["app", "appVersion", "category", "message"]);

  const withMail = buildFeedbackPayload({ ...ok, email: " jan@bedrijf.nl " }, ctx);
  assert.equal(withMail.email, "jan@bedrijf.nl");
  assert.ok(!withMail.message.includes("jan@bedrijf.nl"), "email must not end up in the public message");
});

test("optional name and mood travel along under the service's field names", () => {
  const ctx = { version: "2026.48.5", platform: "Windows", language: "nl" };
  const p = buildFeedbackPayload({ ...ok, name: "  Jan Jansen ", sentiment: "Frustrated" }, ctx);
  assert.equal(p.fullname, "Jan Jansen");
  assert.equal(p.sentiment, "Frustrated");
  const bare = buildFeedbackPayload({ ...ok, name: "   ", sentiment: "Furious" }, ctx);
  assert.ok(!("fullname" in bare), "blank name is not sent");
  assert.ok(!("sentiment" in bare), "unknown mood is dropped, never sent");
});

function fakeFetch(status, body, calls = []) {
  return async (url, init) => {
    calls.push({ url, init });
    return { ok: status >= 200 && status < 300, status, json: async () => { if (body === undefined) throw new Error("no body"); return body; } };
  };
}

test("send: posts JSON to the feedback service and returns the issue link", async () => {
  const calls = [];
  const r = await sendFeedback({ app: FEEDBACK_APP }, fakeFetch(200, { success: true, githubIssueUrl: "https://github.com/OpenAEC-Foundation/open-frame-studio/issues/99" }, calls));
  assert.deepEqual(r, { ok: true, issueUrl: "https://github.com/OpenAEC-Foundation/open-frame-studio/issues/99" });
  assert.equal(calls[0].url, FEEDBACK_URL);
  assert.equal(calls[0].init.method, "POST");
  assert.equal(calls[0].init.headers["Content-Type"], "application/json");
});

test("send: an issue URL outside github.com is never offered as a link", async () => {
  const r = await sendFeedback({}, fakeFetch(200, { success: true, githubIssueUrl: "https://evil.example/x" }));
  assert.deepEqual(r, { ok: true, issueUrl: null });
});

test("send: a stored submission without an issue link is still a success", async () => {
  assert.deepEqual(await sendFeedback({}, fakeFetch(200, { success: true })), { ok: true, issueUrl: null });
});

test("send: failures map to a reason and never throw", async () => {
  assert.deepEqual(await sendFeedback({}, fakeFetch(429, {})), { ok: false, reason: "rate" });
  assert.deepEqual(await sendFeedback({}, fakeFetch(409, {})), { ok: false, reason: "duplicate" });
  assert.deepEqual(await sendFeedback({}, fakeFetch(400, {})), { ok: false, reason: "rejected" });
  assert.deepEqual(await sendFeedback({}, fakeFetch(500, undefined)), { ok: false, reason: "failed" });
  assert.deepEqual(await sendFeedback({}, fakeFetch(200, { success: false })), { ok: false, reason: "failed" });
  assert.deepEqual(await sendFeedback({}, async () => { throw new Error("offline"); }), { ok: false, reason: "failed" });
});
