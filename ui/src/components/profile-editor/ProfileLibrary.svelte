<script>
  import { _ } from "svelte-i18n";
  import { profileCategories, profileFilter, filteredProfiles } from "../../stores/profiles.js";
  import { profileEditor } from "../../stores/profileEditor.js";
  import { toast } from "../../stores/toast.js";
  import ProfileCrossSection from "../panels/ProfileCrossSection.svelte";

  let searchQuery = $state("");
  let expandedCategories = $state({});

  function handleSearch(e) {
    searchQuery = e.target.value;
    profileFilter.set(searchQuery);
  }

  function toggleCategory(catId) {
    expandedCategories = {
      ...expandedCategories,
      [catId]: !expandedCategories[catId],
    };
  }

  function loadProfile(profile) {
    profileEditor.loadProfile(profile);
  }

  function copyProfile(profile, e) {
    e.stopPropagation();
    const copy = {
      ...profile,
      id: null,
      name: profile.name + " (kopie)",
    };
    profileEditor.loadProfile(copy);
    toast.info(($_("profileEditor.copied") || "Profiel gekopieerd. Pas aan en sla op."));
  }

  // Group filtered profiles by category
  let grouped = $derived.by(() => {
    if (searchQuery) {
      // Show flat filtered list
      return null;
    }
    return $profileCategories;
  });
</script>

<div class="profile-library">
  <div class="lib-header">
    <span class="lib-title">{$_("profileEditor.library") || "Profielen"}</span>
    <button class="new-btn" onclick={() => profileEditor.newProfile()}
      title={$_("profileEditor.newProfile") || "Nieuw profiel"}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </button>
  </div>

  <div class="search-box">
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <input
      type="text"
      value={searchQuery}
      oninput={handleSearch}
      placeholder={$_("profileEditor.searchProfiles") || "Zoek profiel..."}
    />
  </div>

  <div class="lib-list">
    {#if searchQuery && $filteredProfiles.length > 0}
      <!-- Flat search results -->
      {#each $filteredProfiles as p}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="profile-item" onclick={() => loadProfile(p)}>
          <div class="profile-thumb">
            {#if p.crossSection}
              <ProfileCrossSection crossSection={p.crossSection} />
            {:else}
              <div class="no-thumb">{p.width}×{p.depth}</div>
            {/if}
          </div>
          <div class="profile-info">
            <span class="profile-name">{p.name}</span>
            <span class="profile-meta">{p.width}×{p.depth}mm · {p.manufacturer || p.material}</span>
          </div>
          <button class="copy-btn" onclick={(e) => copyProfile(p, e)} title="Kopieer">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2"/>
              <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
            </svg>
          </button>
        </div>
      {/each}
    {:else if searchQuery}
      <div class="no-results">{$_("profileEditor.noResults") || "Geen profielen gevonden."}</div>
    {:else if grouped}
      <!-- Grouped by category -->
      {#each grouped as cat}
        <div class="category">
          <button class="category-header" onclick={() => toggleCategory(cat.id)}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
              style="transform: rotate({expandedCategories[cat.id] ? '90deg' : '0deg'}); transition: transform 0.15s">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
            <span>{cat.label}</span>
            <span class="count">{cat.profiles.length}</span>
          </button>

          {#if expandedCategories[cat.id]}
            {#each cat.profiles as p}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="profile-item" onclick={() => loadProfile(p)}>
                <div class="profile-thumb">
                  {#if p.crossSection}
                    <ProfileCrossSection crossSection={p.crossSection} />
                  {:else}
                    <div class="no-thumb">{p.width}×{p.depth}</div>
                  {/if}
                </div>
                <div class="profile-info">
                  <span class="profile-name">{p.name}</span>
                  <span class="profile-meta">{p.width}×{p.depth}mm</span>
                </div>
                <button class="copy-btn" onclick={(e) => copyProfile(p, e)} title="Kopieer">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="9" y="9" width="13" height="13" rx="2"/>
                    <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                  </svg>
                </button>
              </div>
            {/each}
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .profile-library {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .lib-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: var(--border-default);
  }

  .lib-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .new-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    transition: all 0.15s;
  }
  .new-btn:hover {
    background: var(--amber);
    color: #fff;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    margin: 6px 8px;
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
  }
  .search-box svg {
    flex-shrink: 0;
    color: var(--text-muted);
  }
  .search-box input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
  }

  .lib-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    text-align: left;
    transition: color 0.15s;
  }
  .category-header:hover {
    color: var(--text-primary);
  }
  .count {
    margin-left: auto;
    font-size: 10px;
    font-weight: 400;
    opacity: 0.6;
  }

  .profile-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 5px 12px 5px 20px;
    text-align: left;
    transition: background 0.1s;
    border-left: 2px solid transparent;
  }
  .profile-item:hover {
    background: var(--bg-surface-alt);
    border-left-color: var(--amber);
  }

  .profile-thumb {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    border-radius: 2px;
    overflow: hidden;
    background: var(--bg-surface-alt);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .no-thumb {
    font-size: 8px;
    color: var(--text-muted);
    font-family: "JetBrains Mono", monospace;
  }

  .profile-info {
    flex: 1;
    min-width: 0;
  }

  .profile-name {
    display: block;
    font-size: 12px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .profile-meta {
    display: block;
    font-size: 10px;
    color: var(--text-muted);
  }

  .copy-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    padding: 4px;
    border-radius: 2px;
    color: var(--text-muted);
    opacity: 0;
    transition: all 0.15s;
  }
  .profile-item:hover .copy-btn {
    opacity: 1;
  }
  .copy-btn:hover {
    color: var(--amber);
    background: var(--bg-surface);
  }

  .no-results {
    padding: 20px;
    text-align: center;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
