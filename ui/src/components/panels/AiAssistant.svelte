<script>
  import { _ } from "svelte-i18n";
  import { onMount, tick } from "svelte";
  import {
    aiMessages, aiLoading, aiConfigured,
    aiEndpoint, aiModel, aiKey,
    sendMessage, configureAi, clearChat,
  } from "../../stores/aiAssistant.js";

  let inputText = $state("");
  let showConfig = $state(false);
  let messagesEl;

  // Config form state
  let cfgEndpoint = $state($aiEndpoint || "http://localhost:11434/v1");
  let cfgModel = $state($aiModel || "gpt-4o-mini");
  let cfgKey = $state($aiKey || "");

  // Auto-scroll on new messages
  $effect(() => {
    void $aiMessages;
    tick().then(() => {
      if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
    });
  });

  // Show config on first use
  onMount(() => {
    if (!$aiConfigured) showConfig = true;
  });

  function handleSend() {
    if (!inputText.trim() || $aiLoading) return;
    const text = inputText;
    inputText = "";
    sendMessage(text);
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  function saveConfig() {
    configureAi(cfgEndpoint, cfgKey, cfgModel);
    showConfig = false;
  }
</script>

<div class="ai-assistant">
  <!-- Header -->
  <div class="ai-header">
    <div class="ai-title">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2a5 5 0 015 5v1h1a3 3 0 013 3v1a3 3 0 01-3 3h-1v4a3 3 0 01-3 3H10a3 3 0 01-3-3v-4H6a3 3 0 01-3-3v-1a3 3 0 013-3h1V7a5 5 0 015-5z"/>
        <circle cx="9" cy="10" r="1" fill="currentColor"/>
        <circle cx="15" cy="10" r="1" fill="currentColor"/>
      </svg>
      <span>{$_("ai.title") || "AI Assistent"}</span>
    </div>
    <div class="ai-actions">
      <button class="icon-btn" onclick={() => clearChat()} title={$_("ai.clear") || "Chat wissen"}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
        </svg>
      </button>
      <button class="icon-btn" onclick={() => (showConfig = !showConfig)} title={$_("ai.settings") || "Instellingen"}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Config panel -->
  {#if showConfig}
    <div class="config-panel">
      <label class="config-field">
        <span>API Endpoint</span>
        <input type="text" bind:value={cfgEndpoint}
          placeholder="http://localhost:11434/v1" />
      </label>
      <label class="config-field">
        <span>Model</span>
        <input type="text" bind:value={cfgModel}
          placeholder="gpt-4o-mini, llama3, qwen2.5..." />
      </label>
      <label class="config-field">
        <span>API Key {$_("ai.optional") || "(optioneel)"}</span>
        <input type="password" bind:value={cfgKey}
          placeholder="sk-..." />
      </label>
      <div class="config-presets">
        <button class="preset-btn" onclick={() => { cfgEndpoint = "http://localhost:11434/v1"; cfgModel = "qwen2.5"; cfgKey = ""; }}>
          Ollama
        </button>
        <button class="preset-btn" onclick={() => { cfgEndpoint = "https://api.openai.com/v1"; cfgModel = "gpt-4o-mini"; }}>
          OpenAI
        </button>
        <button class="preset-btn" onclick={() => { cfgEndpoint = "https://api.anthropic.com/v1"; cfgModel = "claude-sonnet-4-5-20250514"; }}>
          Claude
        </button>
      </div>
      <button class="save-btn" onclick={saveConfig}>
        {$_("ai.saveConfig") || "Opslaan"}
      </button>
    </div>
  {/if}

  <!-- Messages -->
  <div class="messages" bind:this={messagesEl}>
    {#if $aiMessages.length === 0}
      <div class="empty-state">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
          <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"/>
        </svg>
        <p>{$_("ai.emptyHint") || "Stel een vraag of geef een opdracht."}</p>
        <div class="suggestions">
          <button class="suggestion" onclick={() => { inputText = "Maak een draaikiep kozijn 900x1400"; handleSend(); }}>
            Maak een draaikiep 900×1400
          </button>
          <button class="suggestion" onclick={() => { inputText = "Maak een schuifpui 3000x2400"; handleSend(); }}>
            Maak een schuifpui 3000×2400
          </button>
          <button class="suggestion" onclick={() => { inputText = "Maak een voordeur 1000x2400 RAL 7016"; handleSend(); }}>
            Voordeur RAL 7016
          </button>
        </div>
      </div>
    {:else}
      {#each $aiMessages as msg}
        <div class="message" class:user={msg.role === "user"} class:error={msg.isError}>
          <div class="msg-content">{msg.content || ""}</div>
          {#if msg.toolCalls}
            <div class="tool-calls">
              {#each msg.toolCalls as tc}
                <div class="tool-badge" class:success={tc.success} class:failed={!tc.success}>
                  <span class="tool-icon">{tc.success ? "✓" : "✗"}</span>
                  <span class="tool-name">{tc.name}</span>
                  <span class="tool-result">{tc.result}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
      {#if $aiLoading}
        <div class="message loading">
          <div class="typing-indicator">
            <span></span><span></span><span></span>
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Input -->
  <div class="input-area">
    <textarea
      bind:value={inputText}
      onkeydown={handleKeyDown}
      placeholder={$_("ai.placeholder") || "Typ een opdracht..."}
      rows="1"
      disabled={$aiLoading || !$aiConfigured}
    ></textarea>
    <button class="send-btn" onclick={handleSend}
      disabled={$aiLoading || !inputText.trim() || !$aiConfigured}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="22" y1="2" x2="11" y2="13"/>
        <polygon points="22 2 15 22 11 13 2 9 22 2"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .ai-assistant {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
  }

  .ai-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: var(--border-default);
  }

  .ai-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--amber);
  }

  .ai-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    padding: 4px;
    border-radius: 3px;
    color: var(--text-muted);
    transition: all 0.15s;
  }
  .icon-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-alt);
  }

  /* Config panel */
  .config-panel {
    padding: 10px 12px;
    border-bottom: var(--border-default);
    background: var(--bg-surface-alt);
  }

  .config-field {
    display: block;
    margin-bottom: 6px;
  }
  .config-field span {
    display: block;
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    margin-bottom: 2px;
  }
  .config-field input {
    width: 100%;
    height: 26px;
    padding: 4px 6px;
    font-size: 11px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }
  .config-field input:focus {
    border-color: var(--amber);
  }

  .config-presets {
    display: flex;
    gap: 4px;
    margin: 6px 0;
  }
  .preset-btn {
    padding: 3px 8px;
    font-size: 10px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.15s;
  }
  .preset-btn:hover {
    border-color: var(--amber);
    color: var(--amber);
  }

  .save-btn {
    width: 100%;
    padding: 6px;
    font-size: 11px;
    font-weight: 600;
    background: var(--amber);
    color: #fff;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .save-btn:hover { background: #c2790a; }

  /* Messages */
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--text-muted);
  }
  .empty-state p {
    font-size: 12px;
  }

  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
    width: 100%;
  }
  .suggestion {
    padding: 6px 10px;
    font-size: 11px;
    text-align: left;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.15s;
  }
  .suggestion:hover {
    border-color: var(--amber);
    color: var(--amber);
    background: var(--bg-surface-alt);
  }

  .message {
    padding: 6px 10px;
    border-radius: var(--radius-md);
    font-size: 12px;
    line-height: 1.4;
    max-width: 95%;
    word-wrap: break-word;
  }

  .message.user {
    align-self: flex-end;
    background: var(--amber);
    color: #fff;
    border-bottom-right-radius: 2px;
  }

  .message:not(.user) {
    align-self: flex-start;
    background: var(--bg-surface-alt);
    color: var(--text-primary);
    border-bottom-left-radius: 2px;
  }

  .message.error {
    background: rgba(220, 38, 38, 0.15);
    color: var(--error, #dc2626);
  }

  .msg-content {
    white-space: pre-wrap;
  }

  /* Tool call badges */
  .tool-calls {
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tool-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-family: "JetBrains Mono", monospace;
  }

  .tool-badge.success {
    background: rgba(22, 163, 74, 0.15);
    color: var(--success, #16a34a);
  }
  .tool-badge.failed {
    background: rgba(220, 38, 38, 0.15);
    color: var(--error, #dc2626);
  }

  .tool-icon { font-size: 11px; }
  .tool-name { font-weight: 600; }
  .tool-result { opacity: 0.8; }

  /* Loading animation */
  .typing-indicator {
    display: flex;
    gap: 4px;
    padding: 4px 0;
  }
  .typing-indicator span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-muted);
    animation: typing 1.2s infinite;
  }
  .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
  .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

  @keyframes typing {
    0%, 60%, 100% { opacity: 0.3; transform: translateY(0); }
    30% { opacity: 1; transform: translateY(-3px); }
  }

  /* Input area */
  .input-area {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-top: var(--border-default);
  }

  .input-area textarea {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    font-family: Inter, sans-serif;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--bg-surface-alt);
    color: var(--text-primary);
    resize: none;
    outline: none;
    min-height: 32px;
    max-height: 80px;
  }
  .input-area textarea:focus {
    border-color: var(--amber);
  }

  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--amber);
    color: #fff;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .send-btn:hover:not(:disabled) { background: #c2790a; }
  .send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
