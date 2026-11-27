<script>
  import { _, locale } from "svelte-i18n";
  import BlueprintIcon from "./BlueprintIcon.svelte";
  import { showFeedback } from "../../stores/ui.js";
  import { APP_VERSION } from "../../lib/version.js";
  import { openGithubUrl } from "../../lib/shell-actions.js";
  import {
    FEEDBACK_CATEGORIES, FEEDBACK_MAX, FEEDBACK_MIN, FEEDBACK_REPO, FEEDBACK_SENTIMENTS,
    buildFeedbackPayload, describeFeedbackContext, detectPlatform, sendFeedback, validateFeedback,
  } from "../../lib/feedback.js";

  // Feedback goes to Open Feedback Studio, which opens the GitHub issue itself:
  // the user does not need a GitHub account (see lib/feedback.js).
  const ICONS = { bug: "status.warning", feature: "frame.new", general: "ui.help" };

  let dialog;
  let category = $state("bug");
  let message = $state("");
  let email = $state("");
  let name = $state("");
  let sentiment = $state(null);
  let state = $state("writing"); // writing | sending | sent
  let issueUrl = $state(null);
  let error = $state("");

  let context = $derived({ version: APP_VERSION, platform: detectPlatform(), language: $locale || "?" });
  let problem = $derived(validateFeedback({ category, message, email }));
  let touched = $derived(message.length > 0 || email.length > 0);
  let shownProblem = $derived(touched && problem && !(problem === "tooShort" && message.trim().length === 0) ? problem : null);

  function openModal(node) {
    node.showModal();
    return { destroy() { node.close(); } };
  }

  function close() {
    showFeedback.set(false);
    // Start fresh next time, but keep nothing the user typed after sending.
    if (state === "sent") reset();
  }

  function reset() {
    category = "bug";
    message = "";
    email = "";
    name = "";
    sentiment = null;
    state = "writing";
    issueUrl = null;
    error = "";
  }

  async function submit(event) {
    event.preventDefault();
    if (problem || state === "sending") return;
    state = "sending";
    error = "";
    const result = await sendFeedback(buildFeedbackPayload({ category, message, email, name, sentiment }, context));
    if (result.ok) {
      issueUrl = result.issueUrl;
      state = "sent";
    } else {
      error = $_("feedback.error." + result.reason);
      state = "writing";
    }
  }
</script>

{#if $showFeedback}
  <dialog bind:this={dialog} use:openModal aria-labelledby="feedback-heading" oncancel={close} onkeydown={(event) => event.stopPropagation()}>
    <header>
      <h1 id="feedback-heading">{$_('feedback.title')}</h1>
      <button class="close" type="button" title={$_('settings.close')} aria-label={$_('settings.close')} onclick={close}>
        <BlueprintIcon name="ui.close" />
      </button>
    </header>

    {#if state === "sent"}
      <div class="done" role="status">
        <h2>{$_('feedback.thanks')}</h2>
        <p>{issueUrl ? $_('feedback.madeIssue', { values: { repo: FEEDBACK_REPO } }) : $_('feedback.stored')}</p>
        {#if issueUrl}
          <button type="button" class="link" onclick={() => openGithubUrl(issueUrl)}>
            <BlueprintIcon name="link.open-external" />{$_('feedback.viewOnGithub')}
          </button>
        {/if}
      </div>
      <footer>
        <button type="button" class="secondary" onclick={reset}>{$_('feedback.another')}</button>
        <button type="button" class="primary" onclick={close}>{$_('settings.close')}</button>
      </footer>
    {:else}
      <form onsubmit={submit}>
        <div class="body">
          <fieldset class="types">
            <legend>{$_('feedback.kind')}</legend>
            {#each FEEDBACK_CATEGORIES as kind}
              <button type="button" class="type" aria-pressed={category === kind} onclick={() => category = kind}>
                <BlueprintIcon name={ICONS[kind]} />{$_('feedback.kind.' + kind)}
              </button>
            {/each}
          </fieldset>

          <label>
            <span>{$_('feedback.message')}</span>
            <textarea rows="6" maxlength={FEEDBACK_MAX} bind:value={message} placeholder={$_('feedback.messagePlaceholder')} required></textarea>
            <small class="count">{message.trim().length}/{FEEDBACK_MAX}</small>
          </label>

          <fieldset class="moods">
            <legend>{$_('feedback.mood')}</legend>
            {#each FEEDBACK_SENTIMENTS as mood}
              <button type="button" class="mood" aria-pressed={sentiment === mood} onclick={() => sentiment = sentiment === mood ? null : mood}>
                {$_('feedback.mood.' + mood)}
              </button>
            {/each}
          </fieldset>

          <div class="who">
            <label>
              <span>{$_('feedback.email')} <em>{$_('feedback.emailOptional')}</em></span>
              <input type="email" bind:value={email} autocomplete="email" placeholder="naam@bedrijf.nl" />
            </label>
            <label>
              <span>{$_('feedback.name')} <em>{$_('feedback.nameOptional')}</em></span>
              <input type="text" bind:value={name} autocomplete="name" maxlength="100" />
            </label>
          </div>

          <p class="note">
            {$_('feedback.publicNote', { values: { repo: FEEDBACK_REPO } })}
            <strong>{$_('feedback.noAccount')}</strong>
          </p>
          <p class="meta">{$_('feedback.noModel')}</p>
          <p class="meta">{$_('feedback.included')}: {describeFeedbackContext(context)}</p>
          <p class="hint" aria-live="polite">{shownProblem ? $_('feedback.problem.' + shownProblem, { values: { min: FEEDBACK_MIN } }) : error}</p>
        </div>
        <footer>
          <button type="button" class="secondary" onclick={close}>{$_('feedback.cancel')}</button>
          <button type="submit" class="primary" disabled={!!problem || state === "sending"}>
            {state === "sending" ? $_('feedback.sending') : $_('feedback.send')}
          </button>
        </footer>
      </form>
    {/if}
  </dialog>
{/if}

<style>
  dialog {
    position: fixed;
    inset: 0;
    width: min(520px, calc(100% - 24px));
    max-height: calc(100% - 24px);
    margin: auto;
    padding: 0;
    border: 1px solid var(--border-color);
    border-radius: 10px;
    overflow: auto;
    background: var(--oa-raised);
    color: var(--text-primary);
    box-shadow: var(--oa-shadow-dialog);
  }
  dialog::backdrop { background: var(--oa-overlay); }
  header { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px 8px 16px; border-bottom: 1px solid var(--border-color); }
  h1, h2 { font: 600 16px var(--font-heading); }
  .close { display: grid; place-items: center; height: 28px; width: 28px; border-radius: 4px; }
  .close:hover { background: var(--oa-hover); }
  .body { display: grid; gap: 14px; padding: 16px 20px; }
  fieldset { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; border: 0; padding: 0; margin: 0; min-width: 0; }
  legend { grid-column: 1 / -1; padding: 0; margin-bottom: 6px; color: var(--text-secondary); font-size: 12px; }
  .type { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 10px 6px; border: 1px solid var(--border-color); border-radius: 6px; background: var(--bg-app); font-size: 12px; font-weight: 600; }
  .type:hover { background: var(--oa-hover); }
  .type[aria-pressed="true"] { border-color: var(--text-accent); background: var(--oa-selected); color: var(--text-accent); }
  .moods { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .mood { padding: 6px 4px; border: 1px solid var(--border-color); border-radius: 6px; background: var(--bg-app); font-size: 12px; }
  .mood:hover { background: var(--oa-hover); }
  .mood[aria-pressed="true"] { border-color: var(--text-accent); background: var(--oa-selected); color: var(--text-accent); font-weight: 600; }
  .who { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
  @media (max-width: 460px) { .who { grid-template-columns: minmax(0, 1fr); } }
  label { display: grid; gap: 6px; color: var(--text-secondary); font-size: 12px; }
  label em { font-style: normal; opacity: .8; }
  textarea, input { width: 100%; padding: 8px 10px; border: 1px solid var(--border-color); border-radius: 6px; color: var(--text-primary); background: var(--bg-app); font: inherit; font-size: 13px; }
  textarea { min-height: 110px; resize: vertical; }
  textarea:focus, input:focus { outline: 2px solid var(--text-accent); outline-offset: -1px; }
  .count { justify-self: end; font-size: 11px; opacity: .75; }
  .note { margin: 0; padding: 8px 10px; border-inline-start: 3px solid var(--text-accent); background: var(--bg-surface-alt); font-size: 12px; line-height: 1.5; color: var(--text-secondary); }
  .meta { margin: 0; font: 11px var(--font-mono, monospace); color: var(--text-secondary); opacity: .85; }
  .hint { margin: 0; min-height: 1.2em; font-size: 12px; color: var(--error); }
  .done { padding: 24px 20px 8px; text-align: center; }
  .done p { margin: 8px 0 12px; font-size: 13px; color: var(--text-secondary); }
  .link { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-accent); text-decoration: underline; }
  footer { display: flex; justify-content: end; gap: 8px; padding: 10px 14px; border-top: 1px solid var(--border-color); }
  .primary { padding: 6px 14px; font-size: 12px; font-weight: 600; background: var(--oa-accent); color: var(--oa-text-on-accent); border-radius: 6px; }
  .primary:disabled { opacity: .45; cursor: not-allowed; }
  .secondary { padding: 6px 14px; font-size: 12px; border: 1px solid var(--border-color); border-radius: 6px; }
  .secondary:hover { background: var(--oa-hover); }
  :global([data-oa-theme^="high-contrast"]) .type[aria-pressed="true"] { outline: 2px solid var(--text-accent); outline-offset: -2px; }
</style>
