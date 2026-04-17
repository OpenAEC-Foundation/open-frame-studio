<script>
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { toast } from "../../stores/toast.js";

  let topics = [];
  let loading = false;
  let showCreate = false;
  let newTitle = "";
  let newDescription = "";

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
      toast.success("BCF topic aangemaakt");
    } catch (e) {
      toast.error("Fout bij aanmaken topic: " + e);
    }
  }

  function statusColor(status) {
    if (!status) return "neutral";
    const s = status.toLowerCase();
    if (s === "open") return "open";
    if (s === "closed" || s === "resolved") return "closed";
    if (s === "inprogress" || s === "in_progress") return "progress";
    return "neutral";
  }
</script>

<div class="view">
  <div class="toolbar">
    <h2>BCF Issues</h2>
    <div class="toolbar-actions">
      <button class="action-btn" onclick={loadTopics}>Vernieuwen</button>
      <button class="action-btn primary" onclick={() => showCreate = !showCreate}>+ Nieuw topic</button>
    </div>
  </div>

  {#if showCreate}
    <div class="create-form">
      <div class="field">
        <label>Titel</label>
        <input type="text" bind:value={newTitle} placeholder="Beschrijf het issue..." />
      </div>
      <div class="field">
        <label>Beschrijving</label>
        <textarea bind:value={newDescription} rows="3" placeholder="Aanvullende details..."></textarea>
      </div>
      <div class="form-actions">
        <button class="action-btn" onclick={() => showCreate = false}>Annuleren</button>
        <button class="action-btn primary" onclick={createTopic} disabled={!newTitle.trim()}>Aanmaken</button>
      </div>
    </div>
  {/if}

  {#if loading}
    <p class="hint">Laden...</p>
  {:else if topics.length === 0}
    <p class="hint">Geen BCF topics gevonden. Maak een nieuw topic aan om issues bij te houden.</p>
  {:else}
    <div class="topics-list">
      {#each topics as topic}
        <div class="topic-card">
          <div class="topic-status">
            <span class="status-dot {statusColor(topic.status)}"></span>
          </div>
          <div class="topic-content">
            <span class="topic-title">{topic.title || "\u2014"}</span>
            <span class="topic-meta">
              {topic.status || "Open"}
              {#if topic.assignedTo} &middot; {topic.assignedTo}{/if}
              {#if topic.createdAt} &middot; {topic.createdAt.slice(0, 10)}{/if}
            </span>
            {#if topic.description}
              <p class="topic-desc">{topic.description}</p>
            {/if}
          </div>
          <div class="topic-type">{topic.topicType || "Issue"}</div>
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
  .topic-card { display: flex; align-items: flex-start; gap: var(--sp-3); padding: var(--sp-3); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: 1px solid var(--border-color, #333); }
  .topic-card:hover { border-color: var(--amber); }
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
  .topic-type { font-size: 10px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); background: rgba(255,255,255,0.05); padding: 2px 6px; border-radius: 4px; white-space: nowrap; }
</style>
