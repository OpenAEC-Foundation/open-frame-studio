<script>
  import { currentKozijn, selectedLayoutLeafId, setKozijnLayout } from "../../stores/kozijn.js";
  import {
    ensureIds, findNode, setVulling, patchVulling,
    RAAM_OPENTYPES, DEUR_SOORTEN,
  } from "../../lib/layout.js";

  // Vulling-editor voor het geselecteerde vak van een vrije-indeling-kozijn.
  // Spiegelt de proto-tab-controls maar muteert de boom op het echte kozijn,
  // zodat vakvullingen in de hoofd-editor bewerkbaar zijn (klacht: na
  // "Toepassen" kon glas niet meer naar raam/deur/paneel worden gezet).

  let tree = $derived(ensureIds($currentKozijn?.layout ?? null));
  let leaf = $derived(tree && $selectedLayoutLeafId ? findNode(tree, $selectedLayoutLeafId) : null);
  let v = $derived(leaf && leaf.kind === "leaf" ? leaf.vulling : null);

  const isGlazed = (t) => t === "glas" || t === "raam";
  const isOperable = (t) => t === "raam" || t === "deur";

  let typeValue = $derived(
    !v ? "" : v.type === "raam" ? `raam:${v.openType || "draai"}`
      : v.type === "deur" ? `deur:${v.doorKind || "enkel"}`
      : v.type
  );

  function changeType(e) {
    if (!leaf || !v) return;
    const val = e.target.value;
    // Compatibele extras (glaslat/scharnier/draairichting/beslag) overleven een
    // type-wissel — anders raakt de gebruiker zijn instellingen kwijt.
    const keep = (t) => ({
      ...(isGlazed(t) && v.glaslat != null ? { glaslat: v.glaslat } : {}),
      ...(isOperable(t) && v.hingeSide != null ? { hingeSide: v.hingeSide } : {}),
      ...(isOperable(t) && v.opensOutward != null ? { opensOutward: v.opensOutward } : {}),
      ...(isOperable(t) && v.hardware != null ? { hardware: v.hardware } : {}),
    });
    let nv;
    if (val.startsWith("raam:")) nv = { ...keep("raam"), type: "raam", openType: val.slice(5) };
    else if (val.startsWith("deur:")) nv = { ...keep("deur"), type: "deur", doorKind: val.slice(5) };
    else nv = { ...keep(val), type: val };
    setKozijnLayout(setVulling(tree, leaf.id, nv));
  }

  function patch(p) {
    if (leaf) setKozijnLayout(patchVulling(tree, leaf.id, p));
  }
</script>

{#if v}
  <div class="section">
    <h3>Vakvulling</h3>
    <div class="field">
      <label for="vv-type">Vulling</label>
      <select id="vv-type" value={typeValue} onchange={changeType}>
        <option value="glas">Vast glas</option>
        <optgroup label="Raam">
          {#each Object.entries(RAAM_OPENTYPES) as [k, lbl]}
            <option value={`raam:${k}`}>{lbl}</option>
          {/each}
        </optgroup>
        <optgroup label="Deur">
          {#each Object.entries(DEUR_SOORTEN) as [k, lbl]}
            <option value={`deur:${k}`}>{lbl}</option>
          {/each}
        </optgroup>
        <option value="paneel">Paneel</option>
        <option value="rooster">Ventilatie</option>
        <option value="buiten">Buiten kozijn (trapt)</option>
      </select>
    </div>
    {#if isGlazed(v.type)}
      <div class="field">
        <label for="vv-glaslat">Glaslat</label>
        <select id="vv-glaslat" value={v.glaslat || "geen"}
                onchange={(e) => patch({ glaslat: e.target.value === "geen" ? null : e.target.value })}>
          <option value="geen">Geen</option>
          <option value="binnen">Binnen</option>
          <option value="buiten">Buiten</option>
        </select>
      </div>
    {/if}
    {#if isOperable(v.type)}
      <div class="field">
        <label for="vv-hinge">Scharnierzijde</label>
        <select id="vv-hinge" value={v.hingeSide || "links"} onchange={(e) => patch({ hingeSide: e.target.value })}>
          <option value="links">Links</option>
          <option value="rechts">Rechts</option>
        </select>
      </div>
      <div class="field">
        <label for="vv-dir">Draairichting</label>
        <select id="vv-dir" value={v.opensOutward ? "buiten" : "binnen"}
                onchange={(e) => patch({ opensOutward: e.target.value === "buiten" })}>
          <option value="binnen">Binnendraaiend</option>
          <option value="buiten">Buitendraaiend</option>
        </select>
      </div>
      <label class="check">
        <input type="checkbox" checked={!!v.hardware} onchange={(e) => patch({ hardware: e.target.checked })} />
        Hang- en sluitwerk
      </label>
    {/if}
  </div>
{/if}

<style>
  .section {
    padding: var(--sp-3) 0;
    border-bottom: 1px solid var(--border-color, #333);
  }
  .section h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: 0 0 var(--sp-2) 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: var(--sp-2);
  }
  .field label {
    font-size: 11px;
    color: var(--text-muted);
  }
  .field select {
    padding: var(--sp-1) var(--sp-2);
    background: var(--bg-surface-alt);
    border: 1px solid var(--border-color, #333);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: 12px;
    color: var(--text-primary);
  }
</style>
