<script>
  import { _ } from "svelte-i18n";
  import { invoke } from "../../lib/tauri.js";
  import { refreshProject } from "../../stores/project.js";
  import { loadProfiles } from "../../stores/profiles.js";

  let { visible = $bindable(false), editProfile = null, onsaved = null, onclose = null } = $props();

  // === Profile Parameters ===
  let name = $_('shapeManager.defaultName');
  let material = "wood";
  let materialSubtype = "meranti";
  let kvtType = "A";
  let profileSeries = "67";
  let width = 67;
  let depth = 114;

  // Sponning dagzijde (glass side)
  let sponningType = "binnensponning";
  let sponningWidth = 12;
  let sponningDepth = 17;
  let opdekWidth = 13;
  let rubberCount = 2;

  // Dubbele sponning (2nd)
  let secondWidth = 12;
  let secondDepth = 17;
  let kernhout = 20;

  // Randsponning (wall side)
  let hasRandsponning = true;
  let randWidth = 10;
  let randDepth = 12;

  // Glaslat
  let glaslatWidth = 15;
  let glaslatHeight = 17;

  // Thermal
  let ufValue = 1.8;

  // Applicability
  let applicableAs = { frame: true, sash: true, divider: true, sill: false };

  // Slope (onderdorpel)
  let slopeDegrees = 9;

  // Selected point for editing
  let selectedPointIdx = -1;

  // === KVT Defaults per Type ===
  let KVT_DEFAULTS = $derived({
    A:  { width: 67, depth: 114, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.stijl') },
    A1: { width: 67, depth: 114, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.stijlWaterhol') },
    B:  { width: 90, depth: 114, sponningType: "dubbele_sponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.tussenstijl') },
    B1: { width: 90, depth: 114, sponningType: "dubbele_sponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.tussenstijlWaterhol') },
    C:  { width: 90, depth: 150, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.onderdorpel'), slope: 9 },
    D:  { width: 67, depth: 114, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.tussendorpel') },
    R:  { width: 54, depth: 67, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 15, name: $_('shapeManager.kvt.raamhout') },
    Custom: { width: 67, depth: 114, sponningType: "binnensponning", sponningWidth: 12, sponningDepth: 17, name: $_('shapeManager.kvt.vrijProfiel') },
  });

  const SERIES_DEFAULTS = {
    "54": { width: 54, uf: 2.2 },
    "67": { width: 67, uf: 1.8 },
    "78": { width: 78, uf: 1.5 },
    "90": { width: 90, uf: 1.1 },
  };

  // Initialize from editProfile
  $effect(() => {
    if (editProfile && visible) {
      name = editProfile.name || $_('profileEditor.editProfile');
      material = editProfile.material || "wood";
      width = editProfile.width || 67;
      depth = editProfile.depth || 114;
      ufValue = editProfile.ufValue || 1.8;
      if (editProfile.sponning) {
        sponningWidth = editProfile.sponning.width || 12;
        sponningDepth = editProfile.sponning.depth || 17;
        sponningType = editProfile.sponning.sponningType || "binnensponning";
      }
    }
  });

  // Apply KVT defaults when type changes
  function applyKvtDefaults() {
    const d = KVT_DEFAULTS[kvtType];
    if (!d) return;
    width = SERIES_DEFAULTS[profileSeries]?.width || d.width;
    depth = d.depth;
    sponningType = d.sponningType;
    sponningWidth = d.sponningWidth;
    sponningDepth = d.sponningDepth;
    name = `${d.name} ${width}x${depth}`;
    if (d.slope) slopeDegrees = d.slope;
    ufValue = SERIES_DEFAULTS[profileSeries]?.uf || 1.8;
  }

  function applySeriesDefaults() {
    const s = SERIES_DEFAULTS[profileSeries];
    if (s) {
      width = s.width;
      ufValue = s.uf;
      // Force min 90mm for dubbele sponning
      if (sponningType === "dubbele_sponning" && width < 90) width = 90;
      name = `${KVT_DEFAULTS[kvtType]?.name || $_('shapeManager.profile')} ${width}x${depth}`;
    }
  }

  // === Cross-section Generation ===
  let crossSection = $derived(generateCrossSection());
  let pathD = $derived(crossSection.length > 2
    ? `M ${crossSection.map(p => `${p[0]} ${p[1]}`).join(" L ")} Z`
    : ""
  );
  let achterhoutCalc = $derived(width - sponningWidth - (hasRandsponning ? randWidth : 0));
  let sightline = $derived(width - sponningWidth);

  function generateCrossSection() {
    const w = width, d = depth, sw = sponningWidth, sd = sponningDepth;

    switch (sponningType) {
      case "binnensponning":
        return [
          [0, 0], [w, 0],
          [w, d - sd],
          [w - sw, d - sd],
          [w - sw, d],
          [sw, d],
          [sw, d - sd],
          [0, d - sd],
        ];

      case "buitensponning":
        return [
          [sw, 0],
          [w - sw, 0],
          [w - sw, sd],
          [w, sd],
          [w, d],
          [0, d],
          [0, sd],
          [sw, sd],
        ];

      case "dubbele_sponning": {
        const sw2 = secondWidth, sd2 = secondDepth;
        const cw = Math.max(10, w - sw - sw2 - (hasRandsponning ? randWidth * 2 : 0));
        return [
          [sw, 0],
          [w - sw2, 0],
          [w - sw2, sd2],
          [w, sd2],
          [w, d - sd],
          [w - sw2, d - sd],
          [w - sw2, d],
          [sw, d],
          [sw, d - sd],
          [0, d - sd],
          [0, sd],
          [sw, sd],
        ];
      }

      case "draaikiep": {
        const opdek = opdekWidth;
        const rubberOffset = 3;
        return [
          [0, 0], [w, 0],
          [w, d - sd - opdek],       // start opdek aanslag
          [w - sw + opdek, d - sd - opdek],
          [w - sw + opdek, d - sd],  // sponningbodem buiten
          [w - sw, d - sd],
          [w - sw, d],               // sponningbodem binnen
          [sw, d],
          [sw, d - sd],
          [0, d - sd],
        ];
      }

      default:
        return [[0, 0], [w, 0], [w, d], [0, d]];
    }
  }

  // === KVT Validation ===
  let validations = $derived(computeValidations());

  function computeValidations() {
    const results = [];
    // Sponninghoogte
    const minSH = kvtType === "D" ? 14 : 17;
    results.push({
      label: `${$_('shapeManager.validation.sponningHeight')} >= ${minSH}mm`,
      ok: sponningDepth >= minSH,
      value: sponningDepth,
    });
    // Achterhout
    const minAH = sponningDepth >= 20 ? 15 : 13;
    results.push({
      label: `${$_('shapeManager.validation.achterhout')} >= ${minAH}mm`,
      ok: achterhoutCalc >= minAH,
      value: Math.round(achterhoutCalc),
    });
    // Glaslat
    results.push({
      label: `${$_('shapeManager.validation.glazingBead')} >= 13mm`,
      ok: glaslatWidth >= 13,
      value: glaslatWidth,
    });
    // Dubbele sponning breedte
    if (sponningType === "dubbele_sponning") {
      results.push({
        label: `${$_('shapeManager.validation.widthDouble')} >= 90mm`,
        ok: width >= 90,
        value: width,
      });
      const kern = width - sponningWidth - secondWidth;
      results.push({
        label: `${$_('shapeManager.validation.kernhout')} >= 20mm`,
        ok: kern >= 20,
        value: Math.round(kern),
      });
    }
    // Onderdorpel helling
    if (kvtType === "C") {
      results.push({
        label: `${$_('shapeManager.validation.slope')} >= 9°`,
        ok: slopeDegrees >= 9,
        value: slopeDegrees,
      });
    }
    return results;
  }

  let allValid = $derived(validations.every(v => v.ok));

  // === Save ===
  async function handleSave() {
    const aa = [];
    if (applicableAs.frame) aa.push("frame");
    if (applicableAs.sash) aa.push("sash");
    if (applicableAs.divider) aa.push("divider");
    if (applicableAs.sill) aa.push("sill");

    const profileId = editProfile?.id || `custom-${kvtType.toLowerCase()}-${width}x${depth}-${Date.now()}`;

    const profile = {
      id: profileId,
      name,
      material,
      materialSubtype,
      width,
      depth,
      sightline,
      glazingRebate: sponningDepth,
      crossSection: crossSection,
      ufValue,
      applicableAs: aa,
      kvtType,
      profileSeries,
      isParametric: true,
      sponning: {
        sponningType,
        width: sponningWidth,
        depth: sponningDepth,
        position: sponningType === "buitensponning" ? "buiten" : sponningType === "dubbele_sponning" ? "midden" : "binnen",
        opdekWidth: sponningType === "draaikiep" ? opdekWidth : null,
        rubberCount: sponningType === "draaikiep" ? rubberCount : null,
        secondWidth: sponningType === "dubbele_sponning" ? secondWidth : null,
        secondDepth: sponningType === "dubbele_sponning" ? secondDepth : null,
        kernhout: sponningType === "dubbele_sponning" ? (width - sponningWidth - secondWidth) : null,
        slopeDegrees: kvtType === "C" ? slopeDegrees : null,
      },
      grooveFigures: [],
      glaslatWidth,
      glaslatHeight,
      achterhout: achterhoutCalc,
    };

    try {
      await invoke("add_custom_profile", { profileJson: JSON.stringify(profile) });
      await refreshProject();
      await loadProfiles();
      onsaved?.(profile);
      visible = false;
    } catch (e) {
      alert($_('profileEditor.saveFailed') + ": " + e);
    }
  }

  function handleClose() {
    visible = false;
    onclose?.();
  }

  // SVG preview settings
  const PAD = 25;
  let viewBox = $derived(`${-PAD} ${-PAD} ${width + 2 * PAD + 50} ${depth + 2 * PAD + 20}`);

  // Point interaction
  function handlePointClick(idx) {
    selectedPointIdx = idx === selectedPointIdx ? -1 : idx;
  }
</script>

{#if visible}
  <div class="overlay" onclick={handleClose}>
    <div class="modal" onclick={(e) => { e.stopPropagation(); }}>
      <div class="header">
        <h2>{$_('shapeManager.title')}</h2>
        <button class="close-btn" onclick={handleClose}>
          <svg width="16" height="16" viewBox="0 0 16 16"><path d="M4 4L12 12M12 4L4 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        </button>
      </div>

      <div class="body">
        <!-- LEFT: Interactive SVG Preview -->
        <div class="preview-col">
          <div class="preview-label">{$_('shapeManager.crossSectionLabel')}</div>
          <svg width="300" height="380" viewBox={viewBox} preserveAspectRatio="xMidYMid meet" class="profile-svg">
            <!-- Profile shape -->
            <path d={pathD} fill="var(--bg-surface-alt, #E7E5E4)" stroke="var(--amber, #D97706)" stroke-width="1.5" />

            <!-- Sponning zones -->
            {#if sponningType === "binnensponning" || sponningType === "draaikiep"}
              <rect x={sponningWidth} y={depth - sponningDepth} width={width - 2 * sponningWidth} height={sponningDepth}
                fill="rgba(59, 130, 246, 0.08)" stroke="rgba(59, 130, 246, 0.3)" stroke-width="0.4" stroke-dasharray="2 2" />
            {/if}

            <!-- Rubber groeven (draaikiep) -->
            {#if sponningType === "draaikiep"}
              <line x1={width - sponningWidth + 2} y1={depth - sponningDepth + 3} x2={width - sponningWidth + opdekWidth - 2} y2={depth - sponningDepth + 3}
                stroke="#DC2626" stroke-width="1.5" stroke-dasharray="1 1" />
              <line x1={width - sponningWidth + 2} y1={depth - 3} x2={width - sponningWidth + opdekWidth - 2} y2={depth - 3}
                stroke="#DC2626" stroke-width="1.5" stroke-dasharray="1 1" />
            {/if}

            <!-- Clickable contour points -->
            {#each crossSection as pt, i}
              <circle cx={pt[0]} cy={pt[1]} r={selectedPointIdx === i ? 3 : 2}
                fill={selectedPointIdx === i ? "var(--amber)" : "#fff"}
                stroke={selectedPointIdx === i ? "var(--amber)" : "#888"}
                stroke-width="0.8"
                class="point"
                onclick={() => handlePointClick(i)}
              />
            {/each}

            <!-- Dimension: width (top) -->
            <line x1={0} y1={-8} x2={width} y2={-8} stroke="#888" stroke-width="0.4" />
            <line x1={0} y1={-12} x2={0} y2={-4} stroke="#888" stroke-width="0.4" />
            <line x1={width} y1={-12} x2={width} y2={-4} stroke="#888" stroke-width="0.4" />
            <text x={width / 2} y={-14} text-anchor="middle" font-size="7" fill="#888">{width}</text>

            <!-- Dimension: depth (right) -->
            <line x1={width + 8} y1={0} x2={width + 8} y2={depth} stroke="#888" stroke-width="0.4" />
            <line x1={width + 4} y1={0} x2={width + 12} y2={0} stroke="#888" stroke-width="0.4" />
            <line x1={width + 4} y1={depth} x2={width + 12} y2={depth} stroke="#888" stroke-width="0.4" />
            <text x={width + 16} y={depth / 2} text-anchor="start" dominant-baseline="central" font-size="7" fill="#888">{depth}</text>

            <!-- Dimension: sponning -->
            {#if sponningType !== "geen"}
              <text x={width / 2} y={depth + 12} text-anchor="middle" font-size="6" fill="rgba(59,130,246,0.8)">
                {$_('shapeManager.spAbbrev')} {sponningWidth}x{sponningDepth}
              </text>
            {/if}

            <!-- Dimension: sponning depth (far right) -->
            <line x1={width + 22} y1={depth - sponningDepth} x2={width + 22} y2={depth} stroke="rgba(59,130,246,0.5)" stroke-width="0.3" />
            <text x={width + 28} y={depth - sponningDepth / 2} text-anchor="start" dominant-baseline="central" font-size="5" fill="rgba(59,130,246,0.7)">{sponningDepth}</text>
          </svg>

          <!-- Point coordinates -->
          {#if selectedPointIdx >= 0 && selectedPointIdx < crossSection.length}
            <div class="point-info">
              {$_('shapeManager.point')} {selectedPointIdx + 1}: X={crossSection[selectedPointIdx][0]}  Y={crossSection[selectedPointIdx][1]}
            </div>
          {/if}

          <div class="computed-row">
            <span>{$_('profileEditor.sightline')}: {sightline}mm</span>
            <span>{$_('shapeManager.achterhout')}: {Math.round(achterhoutCalc)}mm</span>
          </div>
        </div>

        <!-- RIGHT: Parameters -->
        <div class="params-col">
          <div class="psection">
            <div class="field">
              <label>{$_('props.name')}</label>
              <input type="text" bind:value={name} />
            </div>
            <div class="field-row">
              <div class="field">
                <label>{$_('shapeManager.kvtType')}</label>
                <select bind:value={kvtType} onchange={applyKvtDefaults}>
                  <option value="A">A — {$_('shapeManager.kvt.stijl')}</option>
                  <option value="A1">A1 — {$_('shapeManager.kvt.stijlWaterhol')}</option>
                  <option value="B">B — {$_('shapeManager.kvt.tussenstijl')}</option>
                  <option value="C">C — {$_('shapeManager.kvt.onderdorpel')}</option>
                  <option value="D">D — {$_('shapeManager.kvt.tussendorpel')}</option>
                  <option value="R">R — {$_('shapeManager.kvt.raamhout')}</option>
                  <option value="Custom">{$_('shapeManager.kvt.vrij')}</option>
                </select>
              </div>
              <div class="field">
                <label>{$_('shapeManager.series')}</label>
                <select bind:value={profileSeries} onchange={applySeriesDefaults}>
                  <option value="54">54mm</option>
                  <option value="67">67mm</option>
                  <option value="78">78mm</option>
                  <option value="90">90mm</option>
                </select>
              </div>
            </div>
            <div class="field">
              <label>{$_('props.material')}</label>
              <select bind:value={materialSubtype}>
                <option value="meranti">{$_('material.meranti')}</option>
                <option value="accoya">{$_('material.accoya')}</option>
                <option value="vuren">{$_('material.vuren')}</option>
                <option value="eiken">{$_('material.eiken')}</option>
              </select>
            </div>
          </div>

          <div class="psection">
            <h4>{$_('profileEditor.dimensions')}</h4>
            <div class="field-row">
              <div class="field">
                <label>{$_('production.width')}</label>
                <input type="number" bind:value={width} min="30" max="150" step="1" />
              </div>
              <div class="field">
                <label>{$_('props.depth')}</label>
                <input type="number" bind:value={depth} min="50" max="250" step="1" />
              </div>
            </div>
          </div>

          <div class="psection">
            <h4>{$_('shapeManager.sponningDagzijde')}</h4>
            <div class="field">
              <label>{$_('shapeManager.type')}</label>
              <select bind:value={sponningType}>
                <option value="binnensponning">{$_('shapeManager.sponning.binnen')}</option>
                <option value="buitensponning">{$_('shapeManager.sponning.buiten')}</option>
                <option value="dubbele_sponning">{$_('shapeManager.sponning.dubbel')}</option>
                <option value="draaikiep">{$_('shapeManager.sponning.draaikiep')}</option>
                <option value="geen">{$_('shapeManager.sponning.geen')}</option>
              </select>
            </div>
            {#if sponningType !== "geen"}
              <div class="field-row">
                <div class="field">
                  <label>{$_('shapeManager.heightMm')}</label>
                  <input type="number" bind:value={sponningDepth} min="10" max="50" step="1" />
                </div>
                <div class="field">
                  <label>{$_('props.width')}</label>
                  <input type="number" bind:value={sponningWidth} min="5" max="40" step="1" />
                </div>
              </div>
            {/if}
            {#if sponningType === "draaikiep"}
              <div class="field-row">
                <div class="field">
                  <label>{$_('shapeManager.opdekMm')}</label>
                  <input type="number" bind:value={opdekWidth} min="5" max="25" step="1" />
                </div>
                <div class="field">
                  <label>{$_('shapeManager.rubberGrooves')}</label>
                  <select bind:value={rubberCount}>
                    <option value={2}>2 {$_('shapeManager.grooves')}</option>
                    <option value={3}>3 {$_('shapeManager.grooves')}</option>
                  </select>
                </div>
              </div>
            {/if}
            {#if sponningType === "dubbele_sponning"}
              <div class="field-row">
                <div class="field">
                  <label>{$_('shapeManager.secondHeight')}</label>
                  <input type="number" bind:value={secondDepth} min="10" max="50" step="1" />
                </div>
                <div class="field">
                  <label>{$_('shapeManager.secondWidth')}</label>
                  <input type="number" bind:value={secondWidth} min="5" max="40" step="1" />
                </div>
              </div>
            {/if}
          </div>

          {#if kvtType === "C"}
            <div class="psection">
              <h4>{$_('profileEditor.sill')}</h4>
              <div class="field">
                <label>{$_('shapeManager.slopeDegrees')}</label>
                <input type="number" bind:value={slopeDegrees} min="0" max="30" step="1" />
              </div>
            </div>
          {/if}

          <div class="psection">
            <h4>{$_('shapeManager.glazingThermal')}</h4>
            <div class="field-row">
              <div class="field">
                <label>{$_('shapeManager.glazingBeadW')}</label>
                <input type="number" bind:value={glaslatWidth} min="8" max="40" step="1" />
              </div>
              <div class="field">
                <label>{$_('shapeManager.glazingBeadH')}</label>
                <input type="number" bind:value={glaslatHeight} min="10" max="30" step="1" />
              </div>
              <div class="field">
                <label>{$_('shapeManager.uf')}</label>
                <input type="number" bind:value={ufValue} min="0.5" max="5" step="0.1" />
              </div>
            </div>
          </div>

          <div class="psection">
            <h4>{$_('profileEditor.application')}</h4>
            <div class="checkbox-row">
              <label class="cb"><input type="checkbox" bind:checked={applicableAs.frame}/> {$_('profileEditor.frame')}</label>
              <label class="cb"><input type="checkbox" bind:checked={applicableAs.sash}/> {$_('shapeManager.window')}</label>
              <label class="cb"><input type="checkbox" bind:checked={applicableAs.divider}/> {$_('profileEditor.divider')}</label>
              <label class="cb"><input type="checkbox" bind:checked={applicableAs.sill}/> {$_('profileEditor.sill')}</label>
            </div>
          </div>

          <!-- KVT Validation -->
          <div class="psection validation">
            <h4>{$_('shapeManager.validation.title')}</h4>
            {#each validations as v}
              <div class="vrow" class:vok={v.ok} class:vfail={!v.ok}>
                <span class="vicon">{v.ok ? "\u2713" : "\u2717"}</span>
                <span>{v.label}</span>
                <span class="vval">({v.value}mm)</span>
              </div>
            {/each}
          </div>

          <div class="actions">
            <button class="btn-primary" onclick={handleSave} disabled={!allValid}>
              {allValid ? $_('profileEditor.save') : $_('shapeManager.kvtInvalid')}
            </button>
            <button class="btn-secondary" onclick={applyKvtDefaults}>{$_('shapeManager.kvtDefault')}</button>
            <button class="btn-secondary" onclick={handleClose}>{$_('profileEditor.cancel')}</button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .modal {
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 720px;
    max-height: 92vh;
    overflow-y: auto;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-3) var(--sp-5);
    border-bottom: var(--border-featured);
  }
  .header h2 {
    font-family: var(--font-heading);
    font-size: 15px;
    font-weight: 700;
    color: var(--amber);
    margin: 0;
  }
  .close-btn {
    color: var(--text-muted);
    padding: 4px;
    border-radius: var(--radius-sm);
  }
  .close-btn:hover { color: var(--text-primary); background: var(--bg-surface-alt); }

  .body {
    display: flex;
    gap: var(--sp-5);
    padding: var(--sp-4) var(--sp-5);
  }
  .preview-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-2);
    min-width: 300px;
    padding: var(--sp-3);
    background: var(--bg-surface-alt);
    border-radius: var(--radius-md);
    border: var(--border-default);
  }
  .preview-label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--scaffold-gray);
  }
  .profile-svg {
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
  }
  .point {
    cursor: default;
    transition: r 0.1s;
  }
  .point:hover { r: 3.5; }
  .point-info {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-code);
    padding: 2px 6px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
  }
  .computed-row {
    display: flex;
    gap: var(--sp-3);
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .params-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    overflow-y: auto;
    max-height: 75vh;
  }
  .psection h4 {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--amber);
    margin-bottom: var(--sp-1);
    padding-bottom: 2px;
    border-bottom: var(--border-default);
  }
  .field { margin-bottom: var(--sp-1); }
  .field label {
    display: block;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 1px;
  }
  .field input, .field select {
    width: 100%;
    padding: 4px 6px;
    background: var(--bg-surface-alt);
    border: var(--border-default);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 12px;
  }
  .field input:focus, .field select:focus {
    outline: none;
    border-color: var(--amber);
    box-shadow: 0 0 0 2px rgba(217,119,6,0.2);
  }
  .field-row { display: flex; gap: var(--sp-2); }
  .field-row .field { flex: 1; }
  .checkbox-row { display: flex; flex-wrap: wrap; gap: var(--sp-3); }
  .cb {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    color: var(--text-primary);
    cursor: default;
    text-transform: none;
    letter-spacing: normal;
    font-weight: 500;
  }
  .cb input { width: auto; accent-color: var(--amber); }

  .validation { padding: var(--sp-2); background: var(--bg-surface-alt); border-radius: var(--radius-sm); border: var(--border-default); }
  .vrow { display: flex; align-items: center; gap: var(--sp-1); font-size: 11px; padding: 1px 0; }
  .vicon { font-weight: 700; width: 14px; text-align: center; }
  .vok .vicon { color: var(--success); }
  .vfail .vicon { color: var(--error); }
  .vfail { color: var(--error); font-weight: 500; }
  .vval { color: var(--text-muted); font-size: 10px; }

  .actions {
    display: flex;
    gap: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: var(--border-default);
  }
  .btn-primary {
    flex: 2;
    padding: var(--sp-2) var(--sp-3);
    background: var(--amber);
    color: white;
    font-weight: 600;
    font-size: 12px;
    border-radius: var(--radius-sm);
  }
  .btn-primary:hover:not(:disabled) { background: var(--signal-orange); }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }
  .btn-secondary {
    flex: 1;
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface-alt);
    color: var(--text-primary);
    font-weight: 500;
    font-size: 12px;
    border: var(--border-default);
    border-radius: var(--radius-sm);
  }
  .btn-secondary:hover { background: var(--bg-surface); }
</style>
