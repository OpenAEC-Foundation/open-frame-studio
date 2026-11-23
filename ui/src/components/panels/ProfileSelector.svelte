<script>
  import { _ } from "svelte-i18n";
  import { allProfiles } from "../../stores/profiles.js";
  import { resolveProfileSnapshot } from "../../lib/profileSnapshot.js";

  let { value = null, filter = "", material = null, systemProfile = null, inwardWindow = false, label = "Profiel", onchange = null } = $props();

  let filteredProfiles = $derived(($allProfiles || []).filter(p => {
    if (material && p.material !== material) return false;
    if (!filter) return true;
    if (p.inwardWindowFrame && (!inwardWindow || p.inwardWindowFrame !== systemProfile?.id)) return false;
    if (p.dividerFrame && p.dividerFrame !== systemProfile?.id) return false;
    if (!(p.applicableAs || []).includes(filter) && p.id !== systemProfile?.id) return false;
    if (["sash", "sliding_sash", "divider"].includes(filter) && systemProfile && ["aluminum", "pvc"].includes(p.material)) {
      const active = ($allProfiles || []).find(candidate => candidate.id === systemProfile.id);
      const activeSystem = active && resolveProfileSnapshot(active).systemId;
      const profileSystem = resolveProfileSnapshot(p).systemId;
      if (!activeSystem || profileSystem !== activeSystem) return p.id === systemProfile.id;
    }
    return true;
  }).map(p => {
    const snapshot = resolveProfileSnapshot(p);
    const systemSash = ["sash", "sliding_sash"].includes(filter) && ["aluminum", "pvc"].includes(p.material);
    const systemDivider = filter === "divider" && ["aluminum", "pvc"].includes(p.material);
    return {
      id: p.id, name: p.name, material: p.material, materialSubtype: p.materialSubtype, snapshot,
      width: (systemSash && snapshot.sashFaceWidth) || (systemDivider && snapshot.dividerFaceWidth) || p.width,
      depth: (systemSash && snapshot.sashBuildDepth) || p.depth,
    };
  }));

  function handleChange(e) {
    const selectedId = e.target.value;
    const profile = filteredProfiles.find(p => p.id === selectedId);
    if (profile) onchange?.(profile);
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
    <span class="current">{filteredProfiles.find(p => p.id === value.id)?.name ?? value.name}</span>
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
