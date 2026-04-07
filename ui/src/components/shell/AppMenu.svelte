<script>
  import { showAppMenu } from "../../stores/ui.js";
  import { isTauri } from "../../lib/tauri.js";
  import { fileNew, fileOpen, fileSave, fileSaveAs } from "../../lib/project-actions.js";
  import { _ } from "svelte-i18n";

  async function handleNew() {
    await fileNew();
    showAppMenu.set(false);
  }

  async function handleOpen() {
    const opened = await fileOpen();
    if (opened) showAppMenu.set(false);
  }

  async function handleSave() {
    const saved = await fileSave();
    if (saved) showAppMenu.set(false);
  }

  async function handleSaveAs() {
    const saved = await fileSaveAs();
    if (saved) showAppMenu.set(false);
  }

  async function openExternal(url) {
    if (isTauri) {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open(url);
      } catch {
        window.open(url, "_blank");
      }
    } else {
      window.open(url, "_blank");
    }
  }

  function handleKeydown(e) {
    if (e.key === "Escape") showAppMenu.set(false);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $showAppMenu}
  <div class="overlay" onclick={() => showAppMenu.set(false)}>
    <div class="backstage" onclick={(e) => { e.stopPropagation(); }}>

      <div class="sidebar">
        <button class="nav-btn back-btn" onclick={() => showAppMenu.set(false)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5m0 0l7 7m-7-7l7-7"/>
          </svg>
          {$_('backstage.back')}
        </button>

        <nav>
          <span class="nav-group-label">{$_('backstage.project')}</span>
          <button class="nav-btn" onclick={handleNew}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="12" y1="11" x2="12" y2="17"/>
              <line x1="9" y1="14" x2="15" y2="14"/>
            </svg>
            <span class="nav-label">{$_('backstage.newProject')}</span>
            <span class="nav-shortcut">Ctrl+N</span>
          </button>
          <button class="nav-btn" onclick={handleOpen}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
            </svg>
            <span class="nav-label">{$_('backstage.open')}</span>
            <span class="nav-shortcut">Ctrl+O</span>
          </button>
          <button class="nav-btn" onclick={handleSave}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            <span class="nav-label">{$_('backstage.save')}</span>
            <span class="nav-shortcut">Ctrl+S</span>
          </button>
          <button class="nav-btn" onclick={handleSaveAs}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            <span class="nav-label">{$_('backstage.saveAs')}</span>
            <span class="nav-shortcut">Ctrl+Shift+S</span>
          </button>

          <div class="nav-divider"></div>

          <span class="nav-group-label">{$_('backstage.help')}</span>
          <button class="nav-btn" onclick={() => openExternal('https://github.com/OpenAEC-Foundation/Open-Frame-Studio')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 00-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0020 4.77 5.07 5.07 0 0019.91 1S18.73.65 16 2.48a13.38 13.38 0 00-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 005 4.77a5.44 5.44 0 00-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 009 18.13V22"/>
            </svg>
            <span class="nav-label">GitHub</span>
          </button>
          <button class="nav-btn" onclick={() => openExternal('https://github.com/OpenAEC-Foundation/Open-Frame-Studio/issues')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4m0-4h.01"/>
            </svg>
            <span class="nav-label">{$_('backstage.reportIssue')}</span>
          </button>
          <button class="nav-btn" onclick={() => openExternal('https://github.com/OpenAEC-Foundation/Open-Frame-Studio/wiki')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4 19.5A2.5 2.5 0 016.5 17H20"/>
              <path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/>
            </svg>
            <span class="nav-label">{$_('backstage.documentation')}</span>
          </button>
        </nav>

        <div class="sidebar-footer">
          <span class="footer-text">v0.1.1</span>
        </div>
      </div>

      <div class="content">
        <div class="content-inner">
          <div class="hero">
            <img class="hero-icon" src="/icon.png" alt="" />
            <div class="hero-text">
              <h2>{$_('app.name')}</h2>
              <p class="hero-sub">{$_('backstage.description')}</p>
            </div>
          </div>

          <div class="section">
            <h3>{$_('backstage.quickstart')}</h3>
            <div class="cards">
              <button class="card" onclick={handleNew}>
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--amber)" stroke-width="1.5">
                  <rect x="3" y="3" width="18" height="18" rx="1"/>
                  <line x1="12" y1="8" x2="12" y2="16"/>
                  <line x1="8" y1="12" x2="16" y2="12"/>
                </svg>
                <span class="card-title">{$_('backstage.newProject')}</span>
                <span class="card-shortcut">Ctrl+N</span>
              </button>
              <button class="card" onclick={handleOpen}>
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--amber)" stroke-width="1.5">
                  <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
                </svg>
                <span class="card-title">{$_('backstage.open')}</span>
                <span class="card-shortcut">Ctrl+O</span>
              </button>
            </div>
          </div>

          <div class="footer-brand">
            <span>{$_('app.org')}</span>
            <span class="dot"></span>
            <span>AGPL-3.0</span>
          </div>
        </div>
      </div>

    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    top: 36px;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 50;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
  }

  .backstage {
    display: flex;
    width: 100%;
    height: 100%;
  }

  .sidebar {
    width: 240px;
    background: var(--bg-titlebar);
    color: var(--text-on-dark);
    display: flex;
    flex-direction: column;
    padding: var(--sp-3) 0;
  }

  nav {
    display: flex;
    flex-direction: column;
    flex: 1;
  }

  .nav-group-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgba(255, 255, 255, 0.3);
    padding: 8px 20px 4px;
  }

  .nav-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 8px 16px;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: 9px 20px;
    text-align: left;
    font-size: 13px;
    color: var(--text-on-dark);
    transition: background 0.12s;
  }

  .nav-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .nav-label {
    flex: 1;
  }

  .nav-shortcut {
    font-size: 10px;
    color: rgba(255, 255, 255, 0.25);
    font-family: var(--font-code, monospace);
  }

  .back-btn {
    color: var(--scaffold-gray);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    margin-bottom: var(--sp-2);
    padding: 8px 20px;
    font-size: 12px;
  }

  .sidebar-footer {
    padding: 8px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .footer-text {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.25);
  }

  .content {
    flex: 1;
    background: var(--bg-surface);
    overflow-y: auto;
  }

  .content-inner {
    max-width: 560px;
    margin: 0 auto;
    padding: 48px 40px;
  }

  .hero {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 40px;
  }

  .hero-icon {
    width: 56px;
    height: 56px;
  }

  .hero-text h2 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 4px 0;
  }

  .hero-sub {
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.5;
    margin: 0;
  }

  .section {
    margin-bottom: 32px;
  }

  .section h3 {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    margin: 0 0 12px 0;
  }

  .cards {
    display: flex;
    gap: 12px;
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 20px 24px;
    background: var(--bg-surface-alt, #f5f5f5);
    border: 1px solid var(--border-color, rgba(0,0,0,0.08));
    border-radius: 6px;
    min-width: 130px;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .card:hover {
    border-color: var(--amber);
    box-shadow: 0 2px 8px rgba(217, 119, 6, 0.1);
  }

  .card-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .card-shortcut {
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-surface, #eee);
    padding: 1px 6px;
    border-radius: 3px;
    border: 1px solid var(--border-color, rgba(0,0,0,0.1));
  }

  .footer-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.5;
    margin-top: 40px;
  }

  .dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: currentColor;
  }
</style>
