/**
 * In-app feedback via Open Feedback Studio — the same service the other OpenAEC
 * apps use, so a user needs NO GitHub account: the service opens the GitHub
 * issue itself (repo mapped to the app id in its `app_repos` table) and returns
 * the issue URL. A reply address is optional and is kept out of the public issue.
 *
 * Contract (see the OpenAEC website `shared/feedback.js` / `docs/feedback.md`):
 *   POST /api/feedback  { app, category, message, appVersion, email? }
 *   200 { success: true, githubIssueUrl? }; 400 rejected, 409 duplicate, 429 rate-limited.
 */

export const FEEDBACK_URL = "https://open-feedback-studio.pages.dev/api/feedback";
/** App id as registered in the service (public repo OpenAEC-Foundation/open-frame-studio). */
export const FEEDBACK_APP = "open-frame-studio";
export const FEEDBACK_REPO = "OpenAEC-Foundation/open-frame-studio";
export const FEEDBACK_MIN = 10;
export const FEEDBACK_MAX = 5000;
export const FEEDBACK_CATEGORIES = ["bug", "feature", "general"];
/** How the writer feels, as the feedback service names it (optional). */
export const FEEDBACK_SENTIMENTS = ["Frustrated", "Neutral", "Happy"];

const EMAIL = /^[^\s@]+@[^\s@.][^\s@]*\.[^\s@.]+$/;

/** @returns {null | "category" | "tooShort" | "tooLong" | "badEmail"} first problem, or null */
export function validateFeedback({ category, message, email }) {
  if (!FEEDBACK_CATEGORIES.includes(category)) return "category";
  const n = (message || "").trim().length;
  if (n < FEEDBACK_MIN) return "tooShort";
  if (n > FEEDBACK_MAX) return "tooLong";
  const mail = (email || "").trim();
  if (mail && !EMAIL.test(mail)) return "badEmail";
  return null;
}

/**
 * What is sent: only what the user wrote, the app version and a one-line
 * environment note (shown to the user in the dialog before sending).
 */
export function buildFeedbackPayload({ category, message, email, name, sentiment }, { version, platform, language }) {
  const note = `_Via Open Frame Studio v${version} · ${platform || "onbekend platform"} · ${language || "?"}_`;
  const body = (message || "").trim();
  const payload = {
    app: FEEDBACK_APP,
    category,
    message: `${body}\n\n${note}`.slice(0, FEEDBACK_MAX + note.length + 2),
    appVersion: String(version),
  };
  const mail = (email || "").trim();
  if (mail) payload.email = mail;
  const fullname = (name || "").trim();
  if (fullname) payload.fullname = fullname.slice(0, 100);
  // Same vocabulary as Open BIM Suite and the service: Frustrated | Neutral | Happy.
  if (FEEDBACK_SENTIMENTS.includes(sentiment)) payload.sentiment = sentiment;
  return payload;
}

/** The context line the dialog shows under "Meegestuurd". */
export function describeFeedbackContext({ version, platform, language }) {
  return `v${version} · ${platform || "onbekend platform"} · ${language || "?"}`;
}

/**
 * Send the payload. Never throws: returns `{ ok: true, issueUrl }` or
 * `{ ok: false, reason }` with reason one of rate | duplicate | rejected | failed.
 */
export async function sendFeedback(payload, fetchImpl = globalThis.fetch) {
  let response;
  try {
    response = await fetchImpl(FEEDBACK_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
  } catch {
    return { ok: false, reason: "failed" };
  }
  let json = {};
  try {
    json = await response.json();
  } catch {
    // Body is optional; the status decides.
  }
  if (response.ok && json?.success) {
    const url = typeof json.githubIssueUrl === "string" && json.githubIssueUrl.startsWith("https://github.com/")
      ? json.githubIssueUrl
      : null;
    return { ok: true, issueUrl: url };
  }
  const reason = response.status === 429 ? "rate"
    : response.status === 409 ? "duplicate"
    : response.status === 400 ? "rejected"
    : "failed";
  return { ok: false, reason };
}

/** Short platform name for the context line (no identifiers beyond the OS family). */
export function detectPlatform() {
  const uaData = typeof navigator !== "undefined" ? navigator.userAgentData?.platform : "";
  const raw = (uaData || (typeof navigator !== "undefined" ? navigator.platform : "") || "").toLowerCase();
  if (raw.includes("win")) return "Windows";
  if (raw.includes("mac")) return "macOS";
  if (raw.includes("linux") || raw.includes("x11")) return "Linux";
  return "onbekend platform";
}
