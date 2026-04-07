<script>
  import { _ } from "svelte-i18n";
  import { profileEditor, editorProfile, editorBounds, editorIsDirty } from "../../stores/profileEditor.js";
  import { toast } from "../../stores/toast.js";

  let profile = $derived($editorProfile);
  let bounds = $derived($editorBounds);

  let sponning = $state({ width: 12, depth: 17, position: "buiten" });

  // Sync sponning from store
  $effect(() => {
    let s;
    profileEditor.subscribe(v => s = v)();
    sponning = { ...s.sponning };
  });

  function updateName(e) {
    profileEditor.updateProfile({ name: e.target.value });
  }

  function updateMaterial(e) {
    profileEditor.updateProfile({ material: e.target.value });
  }

  function updateSeries(e) {
    profileEditor.updateProfile({ series: e.target.value });
  }

  function updateUf(e) {
    profileEditor.updateProfile({ ufValue: parseFloat(e.target.value) || 0 });
  }

  function updateWidth(e) {
    const newW = parseFloat(e.target.value);
    if (newW > 0 && newW !== bounds.width) {
      profileEditor.scaleToSize(newW, bounds.height);
    }
  }

  function updateDepth(e) {
    const newH = parseFloat(e.target.value);
    if (newH > 0 && newH !== bounds.height) {
      profileEditor.scaleToSize(bounds.width, newH);
    }
  }

  function updateSponningWidth(e) {
    sponning.width = parseFloat(e.target.value) || 0;
    profileEditor.updateSponning(sponning);
  }

  function updateSponningDepth(e) {
    sponning.depth = parseFloat(e.target.value) || 0;
    profileEditor.updateSponning(sponning);
  }

  function updateSponningPos(e) {
    sponning.position = e.target.value;
    profileEditor.updateSponning(sponning);
  }

  function toggleApplicable(type) {
    const current = profile?.applicableAs || [];
    const next = current.includes(type)
      ? current.filter((t) => t !== type)
      : [...current, type];
    profileEditor.updateProfile({ applicableAs: next });
  }

  async function handleSave() {
    if (!profile?.name?.trim()) {
      toast.warning($_("profileEditor.nameRequired") || "Voer een profielnaam in.");
      return;
    }

    const data = profileEditor.getExportData();
    const id = data.id || `custom-${data.name.toLowerCase().replace(/\s+/g, "-")}-${Date.now()}`;
    const profileJson = JSON.stringify({ ...data, id });

    try {
      const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
      if (isTauri) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("add_custom_profile", { profileJson });
      }
      toast.success($_("profileEditor.saved") || `Profiel "${data.name}" opgeslagen.`);
      profileEditor.update((s) => ({ ...s, isDirty: false, profile: { ...s.profile, id } }));
    } catch (e) {
      toast.error(($_("profileEditor.saveFailed") || "Opslaan mislukt") + ": " + e);
    }
  }

  const applicableTypes = [
    { key: "frame", label: "Kozijn" },
    { key: "sash", label: "Draairaam" },
    { key: "divider", label: "Verdeler" },
    { key: "sill", label: "Dorpel" },
    { key: "kozijn_stijl", label: "Kozijnstijl" },
    { key: "bovendorpel", label: "Bovendorpel" },
    { key: "onderdorpel", label: "Onderdorpel" },
    { key: "tussenstijl", label: "Tussenstijl" },
  ];
</script>

{#if profile}
<div class="params-panel">
  <div class="section">
    <div class="section-header">{$_("profileEditor.identity") || "Identiteit"}</div>
    <label class="field">
      <span class="label">{$_("profileEditor.name") || "Naam"}</span>
      <input type="text" value={profile.name} oninput={updateName}
        placeholder={$_("profileEditor.namePlaceholder") || "Profielnaam..."} />
    </label>
    <div class="field-row">
      <label class="field">
        <span class="label">{$_("profileEditor.material") || "Materiaal"}</span>
        <select value={profile.material} onchange={updateMaterial}>
          <option value="wood">Hout</option>
          <option value="aluminum">Aluminium</option>
          <option value="pvc">Kunststof</option>
          <option value="wood_aluminum">Hout-Alu</option>
        </select>
      </label>
      <label class="field">
        <span class="label">{$_("profileEditor.series") || "Serie"}</span>
        <input type="text" value={profile.series || ""} oninput={updateSeries} placeholder="bijv. 67mm" />
      </label>
    </div>
  </div>

  <div class="section">
    <div class="section-header">{$_("profileEditor.dimensions") || "Afmetingen"}</div>
    <div class="field-row">
      <label class="field">
        <span class="label">{$_("profileEditor.width") || "Breedte (mm)"}</span>
        <input type="number" value={Math.round(bounds.width * 10) / 10}
          onchange={updateWidth} min="10" step="1" />
      </label>
      <label class="field">
        <span class="label">{$_("profileEditor.depth") || "Diepte (mm)"}</span>
        <input type="number" value={Math.round(bounds.height * 10) / 10}
          onchange={updateDepth} min="10" step="1" />
      </label>
    </div>
    <div class="computed-row">
      <span>{$_("profileEditor.sightline") || "Zichtlijn"}: <strong>{Math.round((bounds.width - sponning.width) * 10) / 10}mm</strong></span>
      <span>{$_("profileEditor.glazingRebate") || "Glasfalz"}: <strong>{sponning.depth}mm</strong></span>
    </div>
  </div>

  <div class="section">
    <div class="section-header">{$_("profileEditor.sponning") || "Sponning"}</div>
    <label class="field">
      <span class="label">{$_("profileEditor.position") || "Positie"}</span>
      <select value={sponning.position} onchange={updateSponningPos}>
        <option value="buiten">{$_("profileEditor.outside") || "Buiten"}</option>
        <option value="binnen">{$_("profileEditor.inside") || "Binnen"}</option>
        <option value="midden">{$_("profileEditor.center") || "Midden"}</option>
        <option value="dubbel">{$_("profileEditor.double") || "Dubbel (voor+achter)"}</option>
      </select>
    </label>
    <div class="field-row">
      <label class="field">
        <span class="label">{$_("profileEditor.sponningWidth") || "Breedte (mm)"}</span>
        <input type="number" value={sponning.width} onchange={updateSponningWidth}
          min="5" max="40" step="1" />
      </label>
      <label class="field">
        <span class="label">{$_("profileEditor.sponningDepth") || "Diepte (mm)"}</span>
        <input type="number" value={sponning.depth} onchange={updateSponningDepth}
          min="10" max="50" step="1" />
      </label>
    </div>
  </div>

  <div class="section">
    <div class="section-header">{$_("profileEditor.thermal") || "Thermisch"}</div>
    <label class="field">
      <span class="label">Uf ({$_("profileEditor.ufUnit") || "W/m²K"})</span>
      <input type="number" value={profile.ufValue || 0} onchange={updateUf}
        min="0.5" max="5.0" step="0.1" />
    </label>
  </div>

  <div class="section">
    <div class="section-header">{$_("profileEditor.application") || "Toepassing"}</div>
    <div class="checkbox-grid">
      {#each applicableTypes as t}
        <label class="checkbox-label">
          <input type="checkbox"
            checked={(profile.applicableAs || []).includes(t.key)}
            onchange={() => toggleApplicable(t.key)} />
          <span>{t.label}</span>
        </label>
      {/each}
    </div>
  </div>

  <div class="section actions">
    <button class="btn-primary" onclick={handleSave} disabled={!$editorIsDirty && !!profile.id}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M19 21H5a2 2 0 01-2-2V5a2 2 0 012-2h11l5 5v11a2 2 0 01-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/>
        <polyline points="7 3 7 8 15 8"/>
      </svg>
      {$_("profileEditor.save") || "Opslaan"}
    </button>
    <button class="btn-secondary" onclick={() => profileEditor.newProfile()}>
      {$_("profileEditor.newProfile") || "Nieuw profiel"}
    </button>
  </div>

  <div class="section vertices-info">
    <div class="section-header">Geometrie</div>
    <span class="meta">{$editorBounds.width ? `${Math.round(bounds.width)}×${Math.round(bounds.height)}mm` : "—"}</span>
    <span class="meta">{$editorProfile ? `${($editorProfile.crossSection || []).length || bounds.width ? "✓" : "—"} vertices` : "—"}</span>
  </div>
</div>
{/if}

<style>
  .params-panel {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .section {
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
  }
  .section:last-child {
    border-bottom: none;
  }

  .section-header {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .field {
    display: block;
    margin-bottom: 4px;
  }

  .label {
    display: block;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary, var(--text-muted));
    margin-bottom: 2px;
  }

  input, select {
    width: 100%;
    height: 26px;
    padding: 4px 6px;
    font-size: 12px;
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
  }

  input:focus, select:focus {
    border-color: var(--amber);
    box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.15);
  }

  .field-row {
    display: flex;
    gap: 6px;
  }
  .field-row .field {
    flex: 1;
  }

  .computed-row {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  .computed-row strong {
    color: var(--text-primary);
  }

  .checkbox-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .checkbox-label input {
    width: auto;
    height: auto;
    accent-color: var(--amber);
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-top: 10px;
  }

  .btn-primary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 16px;
    background: var(--amber);
    color: #fff;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-sm);
    transition: background 0.15s;
  }
  .btn-primary:hover {
    background: #c2790a;
  }
  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text-muted);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    transition: all 0.15s;
  }
  .btn-secondary:hover {
    background: var(--bg-surface-alt);
    color: var(--text-primary);
  }

  .vertices-info {
    font-size: 11px;
  }
  .meta {
    display: block;
    color: var(--text-muted);
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
  }
</style>
