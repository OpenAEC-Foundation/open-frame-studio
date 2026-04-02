<script>
  import { _ } from "svelte-i18n";
  import { allProfiles } from "../../stores/profiles.js";

  let { value = null, filter = "", label = "Profiel", onchange = null } = $props();

  let filteredProfiles = $derived(($allProfiles || []).filter(p => {
    if (!filter) return true;
    return (p.applicableAs || []).includes(filter);
  }));

  function handleChange(e) {
    const selectedId = e.target.value;
    const profile = filteredProfiles.find(p => p.id === selectedId);
    if (profile) {
      onchange?.({
        id: profile.id,
        name: profile.name,
        width: profile.width,
        depth: profile.depth,
      });
    }
  }
</script>

<div class="profile-selector">
  <label>{label}</label>
  <select value={value?.id || ""} onchange={handleChange}>
    <option value="" disabled>{$_('profileSelector.choose')}</option>
    {#each filteredProfiles as profile}
      <option value={profile.id}>
        {profile.name} ({profile.width}x{profile.depth}mm)
      </option>
    {/each}
  </select>
  {#if value}
    <span class="current">{value.name}</span>
  {/if}
</div>

<style>
  .profile-selector {
    margin-bottom: var(--sp-3);
  }

  label {
    display: block;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--sp-1);
  }

  select {
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
  }

  select:focus {
    outline: none;
    border-color: var(--amber);
    box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.2);
  }

  .current {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    margin-top: var(--sp-1);
  }
</style>
