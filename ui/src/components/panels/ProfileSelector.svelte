<script>
  import { _ } from "svelte-i18n";
  import { allProfiles } from "../../stores/profiles.js";

  let { value = null, filter = "", label = "Profiel", onchange = null } = $props();

  let filteredProfiles = $derived(($allProfiles || []).filter(p => {
    if (!filter) return true;
    return (p.applicableAs || []).includes(filter);
  }));

  // Positive finite number or null — 0 means "not specified" in library data.
  function mm(v) {
    return typeof v === "number" && isFinite(v) && v > 0 ? v : null;
  }

  /**
   * Resolve the drawing-relevant dimensions from the full ProfileDefinition.
   * Field semantics follow ofs-core::profile: SponningInfo.depth is the
   * sponninghoogte (face plane; for PVC the Falzhöhe), glazingRebate the
   * sponningdiepte (bouwdiepte direction), glasinval the glass edge cover
   * where it differs from the sponninghoogte (PVC: VEKA 82 MD Glaseinstand
   * 20 vs Falzhöhe 28), and glaslatHeight the bead height (KVT 12.3.2).
   *
   * Values the profile does not carry stay null: ofs-core resolves them to
   * the norm for the kozijn material (hout 17 / PVC 20 / alu 25 — see
   * profile::norm_glasinval). The former `sponningHoogte ?? glazingRebate`
   * crutch is gone deliberately: glazingRebate is the sponningDIEPTE
   * (bouwdiepte, e.g. 51 mm for wood fixed glazing after the 2026-07 data
   * correction) and must never be drawn as the 17 mm face rebate.
   */
  function resolveSnapshot(p) {
    return {
      sponningDiepte: mm(p.glazingRebate),
      sponningHoogte: mm(p.sponning?.depth),
      glasinval: mm(p.glasinval),
      glaslatBreedte: mm(p.glaslatWidth),
      glaslatHoogte: mm(p.glaslatHeight),
      aanzichtbreedte: mm(p.sightline),
    };
  }

  function handleChange(e) {
    const selectedId = e.target.value;
    const profile = filteredProfiles.find(p => p.id === selectedId);
    if (profile) {
      onchange?.({
        id: profile.id,
        name: profile.name,
        width: profile.width,
        depth: profile.depth,
        // Resolved profile values: consumers that apply the profile to the
        // frame store these as profile_snapshot so the drawing follows the
        // real profile instead of hard-coded defaults.
        snapshot: resolveSnapshot(profile),
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
