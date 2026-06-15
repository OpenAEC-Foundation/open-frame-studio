<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  const STATUSES = ["Open", "InProgress", "Closed"];

  let topics = [];
  let loading = false;
  let showCreate = false;
  let newTitle = "";
  let newDescription = "";
  let expanded = {};
  let commentDrafts = {};
  let commentAuthor = "";

  onMount(loadTopics);

  async function loadTopics() {
    loading = true;
    try {
      topics = await invoke("get_bcf_topics", {}) || [];
    } catch (e) {
      console.error("BCF topics laden mislukt:", e);
    }
    loading = false;
  }

  async function createTopic() {
    if (!newTitle.trim()) return;
    try {
      await invoke("create_bcf_topic", { title: newTitle.trim(), description: newDescription.trim() });
      newTitle = "";
      newDescription = "";
      showCreate = false;
      await loadTopics();
      toast.success($_("bcf.topicCreated"));
    } catch (e) {
      toast.error($_("bcf.createError") + ": " + e);
    }
  }

  async function updateStatus(topic, status) {
    if (!status || status === topic.status) return;
    try {
      await invoke("update_bcf_topic_status", { guid: topic.guid, status });
      await loadTopics();
      toast.success($_("bcf.statusUpdated"));
    } catch (e) {
      toast.error($_("bcf.statusUpdateError") + ": " + e);
      await loadTopics();
    }
  }

  async function addComment(topic) {
    const text = (commentDrafts[topic.guid] || "").trim();
    if (!text) return;
    const author = commentAuthor.trim() || $_("bcf.anonymous");
    try {
      await invoke("add_bcf_comment", { guid: topic.guid, author, comment: text });
      commentDrafts = { ...commentDrafts, [topic.guid]: "" };
      await loadTopics();
      toast.success($_("bcf.commentAdded"));
    } catch (e) {
      toast.error($_("bcf.commentError") + ": " + e);
    }
  }

  function toggleComments(guid) {
    expanded = { ...expanded, [guid]: !expanded[guid] };
  }

  function statusLabelKey(status) {
    if (status === "InProgress") return "bcf.statusInProgress";
    if (status === "Closed") return "bcf.statusClosed";
    return "bcf.statusOpen";
  }

  function statusColor(status) {
    if (!status) return "neutral";
    const s = status.toLowerCase();
    if (s === "open") return "open";
    if (s === "closed" || s === "resolved") return "closed";
    if (s === "inprogress" || s === "in_progress") return "progress";
    return "neutral";
  }

  function fmtDateTime(iso) {
    if (!iso) return "—";
    return iso.slice(0, 16).replace("T", " ");
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>{$_("bcf.title")}</h2>
    <div class="toolbar-actions">
      <button class="action-btn" onclick={loadTopics}>{$_("bcf.refresh")}</button>
      <button class="action-btn primary" onclick={() => showCreate = !showCreate}>{$_("bcf.newTopic")}</button>
    </div>
  </div>

  {#if showCreate}
    <div class="create-form">
      <div class="field">
        <label for="bcf-new-title">{$_("bcf.fieldTitle")}</label>
        <input id="bcf-new-title" type="text" bind:value={newTitle} placeholder={$_("bcf.titlePlaceholder")} />
      </div>
      <div class="field">
        <label for="bcf-new-desc">{$_("bcf.fieldDescription")}</label>
        <textarea id="bcf-new-desc" bind:value={newDescription} rows="3" placeholder={$_("bcf.descriptionPlaceholder")}></textarea>
      </div>
      <div class="form-actions">
        <button class="action-btn" onclick={() => showCreate = false}>{$_("bcf.cancel")}</button>
        <button class="action-btn primary" onclick={createTopic} disabled={!newTitle.trim()}>{$_("bcf.create")}</button>
      </div>
    </div>
  {/if}

  {#if loading}
    <p class="hint">{$_("bcf.loading")}</p>
  {:else if topics.length === 0}
    <p class="hint">{$_("bcf.empty")}</p>
  {:else}
    <div class="topics-list">
      {#each topics as topic (topic.guid)}
        <div class="topic-card">
          <div class="topic-row">
            <div class="topic-status">
              <span class="status-dot {statusColor(topic.status)}"></span>
            </div>
            <div class="topic-content">
              <span class="topic-title">{topic.title || "—"}</span>
              <span class="topic-meta">
                {#if topic.assignedTo}{topic.assignedTo} &middot; {/if}
                {#if topic.creationDate}{topic.creationDate.slice(0, 10)}{/if}
              </span>
              {#if topic.description}
                <p class="topic-desc">{topic.description}</p>
              {/if}
            </div>
            <div class="topic-side">
              <select
                class="status-select"
                value={topic.status || "Open"}
                aria-label={$_("bcf.status")}
                onchange={(e) => updateStatus(topic, e.target.value)}
              >
                {#if topic.status && !STATUSES.includes(topic.status)}
                  <option value={topic.status}>{topic.status}</option>
                {/if}
                {#each STATUSES as s}
                  <option value={s}>{$_(statusLabelKey(s))}</option>
                {/each}
              </select>
              <span class="topic-priority">{topic.priority || "—"}</span>
            </div>
          </div>

          <button class="comments-toggle" onclick={() => toggleComments(topic.guid)}>
            <span class="chevron">{expanded[topic.guid] ? "▾" : "▸"}</span>
            {$_("bcf.comments")} ({topic.comments?.length || 0})
          </button>

          {#if expanded[topic.guid]}
            <div class="comments-section">
              {#if topic.comments?.length}
                <ul class="comments-list">
                  {#each topic.comments as c (c.guid)}
                    <li class="comment">
                      <div class="comment-head">
                        <span class="comment-author">{c.author || "—"}</span>
                        <span class="comment-date">{fmtDateTime(c.date)}</span>
                      </div>
                      <p class="comment-body">{c.comment}</p>
                    </li>
                  {/each}
                </ul>
              {:else}
                <p class="hint">{$_("bcf.noComments")}</p>
              {/if}
              <div class="comment-form">
                <input
                  class="author-input"
                  type="text"
                  bind:value={commentAuthor}
                  placeholder={$_("bcf.authorPlaceholder")}
                />
                <input
                  class="comment-input"
                  type="text"
                  bind:value={commentDrafts[topic.guid]}
                  placeholder={$_("bcf.commentPlaceholder")}
                  onkeydown={(e) => { if (e.key === "Enter") addComment(topic); }}
                />
                <button
                  class="action-btn primary"
                  onclick={() => addComment(topic)}
                  disabled={!(commentDrafts[topic.guid] || "").trim()}
                >{$_("bcf.addComment")}</button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: var(--sp-4); overflow: hidden; }
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--sp-3); }
  .toolbar h2 { font-size: 16px; color: var(--text-primary); margin: 0; }
  .toolbar-actions { display: flex; gap: var(--sp-2); }
  .action-btn { padding: var(--sp-2) var(--sp-4); background: var(--bg-surface-alt); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 12px; font-weight: 600; cursor: default; }
  .action-btn.primary { background: var(--amber); color: var(--bg-surface); border-color: transparent; }
  .action-btn:disabled { opacity: 0.4; }
  .hint { color: var(--text-muted); font-size: 13px; font-style: italic; }

  .create-form { padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); margin-bottom: var(--sp-3); }
  .field { margin-bottom: var(--sp-2); }
  .field label { display: block; font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: var(--sp-1); }
  .field input, .field textarea { width: 100%; padding: var(--sp-2) var(--sp-3); background: var(--bg-surface); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; font-family: inherit; resize: vertical; }
  .field input:focus, .field textarea:focus { outline: none; border-color: var(--amber); }
  .form-actions { display: flex; gap: var(--sp-2); justify-content: flex-end; }

  .topics-list { flex: 1; overflow: auto; display: flex; flex-direction: column; gap: var(--sp-2); }
  .topic-card { display: flex; flex-direction: column; padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .topic-card:hover { border-color: var(--amber); }
  .topic-row { display: flex; align-items: flex-start; gap: var(--sp-3); }
  .topic-status { padding-top: 2px; }
  .status-dot { display: block; width: 10px; height: 10px; border-radius: 50%; }
  .status-dot.open { background: #f59e0b; }
  .status-dot.closed { background: #22c55e; }
  .status-dot.progress { background: #3b82f6; }
  .status-dot.neutral { background: #6b7280; }
  .topic-content { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .topic-title { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .topic-meta { font-size: 11px; color: var(--text-muted); }
  .topic-desc { font-size: 12px; color: var(--text-secondary); margin: var(--sp-1) 0 0; }
  .topic-side { display: flex; flex-direction: column; align-items: flex-end; gap: var(--sp-1); }
  .topic-priority { font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); background: rgba(255,255,255,0.05); padding: 2px 6px; border-radius: 4px; white-space: nowrap; }

  .status-select { padding: var(--sp-1) var(--sp-2); background: var(--bg-surface); color: var(--text-primary); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); font-size: 11px; font-weight: 600; font-family: inherit; }
  .status-select:focus { outline: none; border-color: var(--amber); }

  .comments-toggle { align-self: flex-start; margin-top: var(--sp-2); padding: 0; background: none; border: none; color: var(--text-muted); font-size: 11px; font-weight: 600; cursor: default; display: flex; align-items: center; gap: var(--sp-1); }
  .comments-toggle:hover { color: var(--amber); }
  .chevron { font-size: 10px; }

  .comments-section { margin-top: var(--sp-2); padding-top: var(--sp-2); border-top: 1px solid var(--border-color, #333); display: flex; flex-direction: column; gap: var(--sp-2); }
  .comments-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: var(--sp-2); }
  .comment { padding: var(--sp-2) var(--sp-3); background: var(--bg-surface); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .comment-head { display: flex; justify-content: space-between; gap: var(--sp-2); margin-bottom: 2px; }
  .comment-author { font-size: 11px; font-weight: 700; color: var(--text-primary); }
  .comment-date { font-size: 10px; color: var(--text-muted); font-variant-numeric: tabular-nums; }
  .comment-body { font-size: 12px; color: var(--text-secondary); margin: 0; white-space: pre-wrap; }

  .comment-form { display: flex; gap: var(--sp-2); align-items: center; }
  .author-input { width: 140px; flex: none; }
  .comment-input { flex: 1; }
  .comment-form input { padding: var(--sp-2) var(--sp-3); background: var(--bg-surface); border: 1px solid var(--border-color, #333); border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; font-family: inherit; }
  .comment-form input:focus { outline: none; border-color: var(--amber); }
</style>
