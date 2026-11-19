<script>
  import { _, locale } from "svelte-i18n";
  import { onMount } from "svelte";
  import { invoke } from "../../lib/tauri.js";
  import { productionDataProject, loadProductionDataProject } from "../../stores/production.js";
  import { vliesgevels, loadVliesgevels } from "../../stores/vliesgevel.js";
  import { memberLabel, gasketLabel } from "../../lib/labels.js";
  import DividerEndMeasure from "../panels/DividerEndMeasure.svelte";
  import ArchEndMeasure from "../panels/ArchEndMeasure.svelte";
  import ReceivedMortises from "../panels/ReceivedMortises.svelte";
  import { archJointLabel, cutRows, endLabel } from "../../lib/dividerJoinery.js";
  import { circleDiameter, contourPath } from "../../lib/contour2d.js";

  let activeTab = "kortlijst";

  // Vliesgevel production (one vliesgevel at a time, picked via the selector)
  let selectedVgId = "";
  let vgProduction = null;

  // Tabs for which vliesgevel production data exists
  const vgTabs = ["kortlijst", "glaslijst", "rubberlijst", "stuklijst"];

  $: tabs = [
    { id: "kortlijst", label: $_('production.cutList') },
    { id: "glaslijst", label: $_('production.glassList') },
    { id: "beslaglijst", label: $_('production.hardwareList') },
    { id: "rubberlijst", label: $_('production.gasketList') },
    { id: "paneellijst", label: $_('production.panelList') },
    { id: "glaslatlijst", label: $_('production.glaslatList') },
    { id: "stuklijst", label: $_('production.bomList') },
  ];

  onMount(() => {
    loadProductionDataProject();
    loadVliesgevels();
  });

  // Keep the selector valid: pick the first vliesgevel when none (or a removed one) is selected.
  $: if ($vliesgevels.length > 0 && !$vliesgevels.some((v) => v.id === selectedVgId)) {
    selectedVgId = $vliesgevels[0].id;
  }
  $: if ($vliesgevels.length === 0 && selectedVgId) {
    selectedVgId = "";
    vgProduction = null;
  }
  $: if (selectedVgId) loadVgProduction(selectedVgId);

  $: vgCut = vgProduction
    ? [...(vgProduction.mullionList || []), ...(vgProduction.transomList || [])]
    : [];

  async function loadVgProduction(id) {
    try {
      vgProduction = await invoke("get_vliesgevel_production", { id });
    } catch (e) {
      console.error("Vliesgevel productiedata laden mislukt:", e);
      vgProduction = null;
    }
  }

  // Flatten data per list type
  $: allCut = ($productionDataProject || []).flatMap(cutRows);
  $: hasDividerJoinery = ($productionDataProject || []).some(p => p.dividerJoints?.length);
  $: hasArchJoinery = ($productionDataProject || []).some(p => p.archJoints?.length);
  $: hasRingJoints = ($productionDataProject || []).some(p => p.archJoints?.some(j => j.kind === "ring_seam"));
  $: allGlass = ($productionDataProject || []).flatMap(p =>
    (p.glassList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allHw = ($productionDataProject || []).flatMap(p =>
    (p.hardwareList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allGasket = ($productionDataProject || []).flatMap(p =>
    (p.gasketList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: hasSealReferences = allGasket.some(g => g.reference);
  $: hasHardwarePositions = allHw.some(h => h.pieceId);
  $: allPanel = ($productionDataProject || []).flatMap(p =>
    (p.panelList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allGlaslat = ($productionDataProject || []).flatMap(p =>
    (p.glaslatList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: hasCutShapes = allGlass.some(i => i.cutContour);
  $: hasTimberArcs = allCut.some(i => i.arc || i.startCopeRadiusMm != null || i.endCopeRadiusMm != null);
  $: hasDividerEnds = allCut.some(i => i.dividerStart || i.dividerEnd);
  $: hasBentBeads = allGlaslat.some(i => i.contourRuns?.length);
  $: mm = new Intl.NumberFormat($locale || "nl", { maximumFractionDigits: 1 });
  $: allBom = ($productionDataProject || []).flatMap(p =>
    (p.bom || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );

  async function refresh() {
    await loadProductionDataProject();
    await loadVliesgevels();
    if (selectedVgId) await loadVgProduction(selectedVgId);
  }
</script>

<div class="production-view">
  <div class="toolbar">
    <h2>{$_('production.title')}</h2>
    <button class="refresh-btn" onclick={refresh}>{$_('production.recalculate')}</button>
  </div>

  <div class="tab-bar">
    {#each tabs as tab}
      <button
        class="tab"
        class:active={activeTab === tab.id}
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
        <span class="count">
          {#if tab.id === "kortlijst"}{allCut.length}
          {:else if tab.id === "glaslijst"}{allGlass.length}
          {:else if tab.id === "beslaglijst"}{allHw.length}
          {:else if tab.id === "rubberlijst"}{allGasket.length}
          {:else if tab.id === "paneellijst"}{allPanel.length}
          {:else if tab.id === "glaslatlijst"}{allGlaslat.length}
          {:else if tab.id === "stuklijst"}{allBom.length}
          {/if}
        </span>
      </button>
    {/each}
  </div>

  <div class="table-container">
    {#if activeTab === "kortlijst"}
      {#if hasTimberArcs}
        <p class="cut-reference">{$_("production.arcReference")}</p>
      {/if}
      {#if hasDividerEnds}
        <p class="cut-reference">{$_("divider.cutReference")} {$_("divider.edgeReference")}</p>
      {/if}
      {#if hasDividerJoinery}<p class="cut-reference">{$_("joinery.reference")}</p>{/if}
      {#if hasArchJoinery}
        <p class="cut-reference">{#if hasRingJoints}{$_("ringJoint.endOrder")} {/if}{$_("archJoint.reference")} {$_("production.frameOrigin")}</p>
      {/if}
      <table class:timber-table={hasTimberArcs}>
        {#if !hasTimberArcs}
        <colgroup>
          <col style="width:7%"><col style="width:9%"><col style="width:14%"><col style="width:14%">
          <col style="width:10%"><col style="width:10%"><col style="width:10%"><col style="width:8%"><col style="width:8%"><col style="width:6%">
        </colgroup>
        {/if}
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.position')}</th><th>{$_('production.part')}</th><th>{$_('production.profile')}</th>
          <th>{$_('production.material')}</th><th class="num">{$_('production.net')}</th><th class="num">{$_('production.gross')}</th>
          {#if hasTimberArcs}
            <th class="num">{$_('production.outerRadius')}</th><th class="num">{$_('production.outerArcLength')}</th>
          {/if}
          <th class="num">{$_(hasTimberArcs ? 'production.cutStart' : 'production.angleL')}</th><th class="num">{$_(hasTimberArcs ? 'production.cutEnd' : 'production.angleR')}</th><th class="num">{$_('production.quantity')}</th>
        </tr></thead>
        <tbody>
          {#each allCut as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.pieceId}</td>
              <td>
                {memberLabel($_, item.memberType, item.profileSide)}
                {#if item.mortises.length}
                  <ReceivedMortises entries={item.mortises.map(m => ({ tenon: m.tenon, label: m.id, endLabel: m.corner == null ? endLabel($_, m.vertical, m.side) : archJointLabel($_, m.corner, m.kind) }))} />
                {/if}
                {#if item.masonryNotches?.length}
                  <details class="notch-detail">
                    <summary>{$_("production.masonryNotch")}</summary>
                    <div>{$_("production.frameOrigin")}</div>
                    {#each item.masonryNotches as notch}
                      <div>X {mm.format(notch.x)} · Y {mm.format(notch.y)}<br />{mm.format(notch.width)} × {mm.format(notch.height)} mm</div>
                    {/each}
                  </details>
                {/if}
              </td>
              <td>{item.profileName}</td>
              <td>{item.material}</td>
              <td class="num">{hasTimberArcs ? mm.format(item.netLengthMm) : Math.round(item.netLengthMm)}</td>
              <td class="num">{item.unknownGross ? $_("archJoint.unknownGross") : hasTimberArcs ? mm.format(item.grossLengthMm) : Math.round(item.grossLengthMm)}</td>
              {#if hasTimberArcs}
                <td class="num">{#if item.arc}{mm.format(item.arc.outerRadiusMm)}<small>{$_('production.centerline')} {mm.format(item.arc.centerlineRadiusMm)}</small>{:else}—{/if}</td>
                <td class="num">{#if item.arc}{mm.format(item.arc.outerLengthMm)}<small>{$_('production.centerline')} {mm.format(item.arc.centerlineLengthMm)}</small>{:else}—{/if}</td>
              {/if}
              {#each ["start", "end"] as side}
                {@const joint = item[side + "ArchJoint"]}
                <td class="num">
                  {#if joint}
                    <ArchEndMeasure {joint} received={joint.received} mate={joint.mate} />
                  {:else}
                    <DividerEndMeasure end={side === "start" ? item.dividerStart : item.dividerEnd} vertical={item.memberType === "divider_v"}
                      angleDeg={side === "start" ? item.miterLeftDeg : item.miterRightDeg} radiusMm={side === "start" ? item.startCopeRadiusMm : item.endCopeRadiusMm}
                      receiverName={item[side + "Receiver"]} />
                  {/if}
                </td>
              {/each}
              <td class="num">{item.quantity}</td>
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "glaslijst"}
      <table>
        <colgroup>
          <col style="width:8%"><col style="width:10%"><col style="width:14%"><col style="width:12%">
          <col style="width:12%"><col style="width:10%"><col style="width:10%"><col style="width:12%"><col style="width:6%">
          {#if hasCutShapes}<col style="width:18%">{/if}
        </colgroup>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.position')}</th><th>{$_('production.glassType')}</th><th class="num">{$_('production.width')}</th>
          <th class="num">{$_('production.height')}</th><th class="num">{$_('production.thickness')}</th><th class="num">{$_('production.ug')}</th><th class="num">{$_('production.area')}</th><th class="num">{$_('production.quantity')}</th>
          {#if hasCutShapes}<th class="shape-column">{$_('production.cutShape')}</th>{/if}
        </tr></thead>
        <tbody>
          {#each allGlass as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.pieceId}</td>
              <td>{item.glassType}</td>
              <td class="num">{item.cutContour ? mm.format(item.widthMm) : Math.round(item.widthMm)}</td>
              <td class="num">{item.cutContour ? mm.format(item.heightMm) : Math.round(item.heightMm)}</td>
              <td class="num">{item.thicknessMm}</td>
              <td class="num">{item.ugValue.toFixed(1)}</td>
              <td class="num">{item.areaM2.toFixed(2)} m&sup2;</td>
              <td class="num">{item.quantity}</td>
              {#if hasCutShapes}
                <td class="shape-cell">
                  {#if item.cutContour}
                    {@const diameter = circleDiameter(item.cutContour)}
                    <svg width="50" height="38" viewBox={`0 0 ${item.widthMm} ${item.heightMm}`} aria-hidden="true">
                      <path d={contourPath(item.cutContour)} fill="var(--editor-glass)" stroke="currentColor" vector-effect="non-scaling-stroke" />
                    </svg>
                    {#if diameter != null}
                      <span>Ø {mm.format(diameter)} mm</span>
                    {:else}
                      {@const arcs = item.cutContour.segments.filter(e => e.kind === 'arc')}
                      <details>
                        <summary>{arcs.length ? `R ${mm.format(arcs[0].radius)} mm` : $_('props.polygon')}</summary>
                        <div>{$_('production.paneOrigin')}</div>
                        {#each item.cutContour.segments as edge}
                          {#if edge.kind === 'arc'}<div>R {mm.format(edge.radius)} · M ({mm.format(edge.center[0])}; {mm.format(edge.center[1])}) · {mm.format(edge.sweepAngle * 180 / Math.PI)}°</div>{/if}
                          <div>({mm.format(edge.from[0])}; {mm.format(edge.from[1])}) → ({mm.format(edge.to[0])}; {mm.format(edge.to[1])})</div>
                        {/each}
                      </details>
                    {/if}
                  {:else}
                    {$_('props.rectangular')}
                  {/if}
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "beslaglijst"}
      <table>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.cell')}</th><th>{$_('production.component')}</th><th>{$_('production.description')}</th><th>{$_('production.quantity')}</th>
          {#if hasHardwarePositions}<th>{$_('production.position')}</th>{/if}
        </tr></thead>
        <tbody>
          {#each allHw as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.cellIndex + 1}</td>
              <td>{item.component}</td>
              <td>{item.description}</td>
              <td class="num">{item.quantity || '—'}</td>
              {#if hasHardwarePositions}<td>{item.pieceId ?? '—'}</td>{/if}
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "rubberlijst"}
      {#if hasSealReferences}<p>{$_('production.sealMeasureNote')}</p>{/if}
      <table>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.type')}</th><th>{$_('production.lengthMm')}</th><th>{$_('production.quantity')}</th>
          {#if hasSealReferences}
            <th>{$_('production.position')}</th><th>{$_('production.profile')}</th><th>{$_('production.measureLine')}</th>
          {/if}
        </tr></thead>
        <tbody>
          {#each allGasket as item}
            <tr>
              <td>{item.mark}</td>
              <td>{gasketLabel($_, item.gasketType)}</td>
              <td class="num">{item.reference ? mm.format(item.lengthMm) : Math.round(item.lengthMm)}</td>
              <td class="num">{item.quantity || '—'}</td>
              {#if hasSealReferences}
                <td>{item.reference?.pieceId ?? '—'}</td>
                <td>{item.reference?.profileName ?? '—'}</td>
                <td>{item.reference ? $_('production.sealLine.' + item.reference.line) : '—'}</td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "paneellijst"}
      <table>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.position')}</th><th>{$_('production.width')}</th><th>{$_('production.height')}</th><th>{$_('production.type')}</th><th>{$_('production.quantity')}</th>
        </tr></thead>
        <tbody>
          {#each allPanel as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.pieceId}</td>
              <td class="num">{Math.round(item.widthMm)}</td>
              <td class="num">{Math.round(item.heightMm)}</td>
              <td>{item.panelType}</td>
              <td class="num">{item.quantity}</td>
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "glaslatlijst"}
      <table>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.position')}</th><th>{$_('production.material')}</th><th class="num">{$_('production.width')}</th><th class="num">{$_('production.height')}</th><th class="num">{$_('production.totalLength')}</th><th class="num">{$_('production.mitered')}</th><th class="num">{$_('production.quantity')}</th>
          {#if hasBentBeads}<th class="shape-column">{$_('production.beadEdge')}</th>{/if}
        </tr></thead>
        <tbody>
          {#each allGlaslat as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.pieceId} · {item.position}</td>
              <td>{item.material}</td>
              <td class="num">{Math.round(item.widthMm)}</td>
              <td class="num">{Math.round(item.heightMm)}</td>
              <td class="num">{item.contourRuns?.length ? mm.format(item.totalLengthMm) : Math.round(item.totalLengthMm)}</td>
              <td class="num">{item.contourRuns?.length ? "—" : item.mitered ? "45°" : "90°"}</td>
              <td class="num">{item.quantity}</td>
              {#if hasBentBeads}
                <td class="shape-cell">
                  {#each item.contourRuns || [] as run}
                    <div>{mm.format(run.lengthMm)} mm {run.radiusMm == null ? "" : `(R ${mm.format(run.radiusMm)})`}</div>
                  {/each}
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "stuklijst"}
      <table>
        <thead><tr>
          <th>{$_('production.frame')}</th><th>{$_('production.category')}</th><th>{$_('production.description')}</th><th>{$_('production.unit')}</th><th>{$_('production.amount')}</th>
        </tr></thead>
        <tbody>
          {#each allBom as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.category}</td>
              <td>{item.description}</td>
              <td>{item.unit}</td>
              <td class="num">{item.quantity.toFixed(2)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    {#if $vliesgevels.length > 0 && vgTabs.includes(activeTab)}
      <div class="vg-section">
        <div class="vg-toolbar">
          <h3>{$_('vliesgevel.title')}</h3>
          <select class="vg-select" bind:value={selectedVgId} title={$_('vliesgevel.select')}>
            {#each $vliesgevels as vg (vg.id)}
              <option value={vg.id}>{vg.mark} — {vg.name}</option>
            {/each}
          </select>
        </div>

        {#if vgProduction}
          {#if activeTab === "kortlijst"}
            <table>
              <colgroup>
                <col style="width:7%"><col style="width:9%"><col style="width:14%"><col style="width:14%">
                <col style="width:10%"><col style="width:10%"><col style="width:10%"><col style="width:8%"><col style="width:8%"><col style="width:6%">
              </colgroup>
              <thead><tr>
                <th>{$_('vliesgevel.vliesgevel')}</th><th>{$_('production.position')}</th><th>{$_('production.part')}</th><th>{$_('production.profile')}</th>
                <th>{$_('production.material')}</th><th class="num">{$_('production.net')}</th><th class="num">{$_('production.gross')}</th><th class="num">{$_('production.angleL')}</th><th class="num">{$_('production.angleR')}</th><th class="num">{$_('production.quantity')}</th>
              </tr></thead>
              <tbody>
                {#each vgCut as item}
                  <tr>
                    <td>{vgProduction.mark}</td>
                    <td>{item.pieceId}</td>
                    <td>{memberLabel($_, item.memberType)}</td>
                    <td>{item.profileName}</td>
                    <td>{item.material}</td>
                    <td class="num">{Math.round(item.netLengthMm)}</td>
                    <td class="num">{Math.round(item.grossLengthMm)}</td>
                    <td class="num">{item.miterLeftDeg}&deg;</td>
                    <td class="num">{item.miterRightDeg}&deg;</td>
                    <td class="num">{item.quantity}</td>
                  </tr>
                {/each}
              </tbody>
            </table>

          {:else if activeTab === "glaslijst"}
            <table>
              <colgroup>
                <col style="width:8%"><col style="width:10%"><col style="width:14%"><col style="width:12%">
                <col style="width:12%"><col style="width:10%"><col style="width:10%"><col style="width:12%"><col style="width:6%">
              </colgroup>
              <thead><tr>
                <th>{$_('vliesgevel.vliesgevel')}</th><th>{$_('production.position')}</th><th>{$_('production.glassType')}</th><th class="num">{$_('production.width')}</th>
                <th class="num">{$_('production.height')}</th><th class="num">{$_('production.thickness')}</th><th class="num">{$_('production.ug')}</th><th class="num">{$_('production.area')}</th><th class="num">{$_('production.quantity')}</th>
              </tr></thead>
              <tbody>
                {#each vgProduction.glassList || [] as item}
                  <tr>
                    <td>{vgProduction.mark}</td>
                    <td>{item.pieceId}</td>
                    <td>{item.glassType}</td>
                    <td class="num">{Math.round(item.widthMm)}</td>
                    <td class="num">{Math.round(item.heightMm)}</td>
                    <td class="num">{item.thicknessMm}</td>
                    <td class="num">{item.ugValue.toFixed(1)}</td>
                    <td class="num">{item.areaM2.toFixed(2)} m&sup2;</td>
                    <td class="num">{item.quantity}</td>
                  </tr>
                {/each}
              </tbody>
            </table>

          {:else if activeTab === "rubberlijst"}
            <table>
              <thead><tr>
                <th>{$_('vliesgevel.vliesgevel')}</th><th>{$_('production.type')}</th><th>{$_('production.lengthMm')}</th><th>{$_('production.quantity')}</th>
              </tr></thead>
              <tbody>
                {#each vgProduction.gasketList || [] as item}
                  <tr>
                    <td>{vgProduction.mark}</td>
                    <td>{gasketLabel($_, item.gasketType)}</td>
                    <td class="num">{Math.round(item.lengthMm)}</td>
                    <td class="num">{item.quantity}</td>
                  </tr>
                {/each}
              </tbody>
            </table>

          {:else if activeTab === "stuklijst"}
            <table>
              <thead><tr>
                <th>{$_('vliesgevel.vliesgevel')}</th><th>{$_('production.category')}</th><th>{$_('production.description')}</th><th>{$_('production.unit')}</th><th>{$_('production.amount')}</th>
              </tr></thead>
              <tbody>
                {#each vgProduction.bom || [] as item}
                  <tr>
                    <td>{vgProduction.mark}</td>
                    <td>{item.category}</td>
                    <td>{item.description}</td>
                    <td>{item.unit}</td>
                    <td class="num">{item.quantity.toFixed(2)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .production-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: var(--sp-4);
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--sp-3);
  }

  .toolbar h2 {
    font-size: 16px;
    color: var(--text-primary);
    margin: 0;
  }

  .refresh-btn {
    padding: var(--sp-2) var(--sp-4);
    background: var(--amber);
    color: var(--bg-surface);
    border: none;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    cursor: default;
  }

  .tab-bar {
    display: flex;
    gap: 2px;
    margin-bottom: var(--sp-3);
    border-bottom: 2px solid var(--border-color, #e5e7eb);
  }

  .tab {
    padding: var(--sp-2) var(--sp-4);
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    cursor: default;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .tab.active {
    color: var(--amber);
    border-bottom-color: var(--amber);
  }

  .count {
    font-size: 10px;
    background: var(--bg-surface-alt);
    padding: 1px 6px;
    border-radius: 10px;
  }

  .table-container {
    flex: 1;
    overflow: auto;
  }

  .vg-section {
    margin-top: var(--sp-5);
    border-top: 2px solid var(--border-color, #e5e7eb);
    padding-top: var(--sp-3);
  }

  .vg-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--sp-3);
  }

  .vg-toolbar h3 {
    font-size: 13px;
    color: var(--text-primary);
    margin: 0;
  }

  .vg-select {
    padding: var(--sp-2) var(--sp-3);
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: var(--radius-sm);
    font-size: 12px;
  }

  table {
    width: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font-size: 12px;
  }

  thead th {
    position: sticky;
    top: 0;
    background: var(--bg-surface-alt);
    padding: var(--sp-2) var(--sp-3);
    text-align: left;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    border-bottom: 2px solid var(--border-color, #e5e7eb);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  thead th:last-child {
    width: 60px;
    text-align: right;
  }

  thead th.shape-column {
    width: 18%;
    text-align: left;
    white-space: normal;
  }

  tbody tr {
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  tbody tr:nth-child(even) {
    background: var(--bg-surface-alt);
  }

  td {
    padding: var(--sp-2) var(--sp-3);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  td.num, th.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .cut-reference { color: var(--text-secondary); font-size: 12px; line-height: 1.5; margin: 0 0 var(--sp-3); }
  .timber-table { table-layout: auto; min-width: 980px; }
  .timber-table th { white-space: normal; }
  .timber-table small { display: block; margin-top: 2px; font-size: 10px; color: var(--text-muted); }
  .notch-detail { white-space: normal; font-size: 11px; }
  .notch-detail summary { cursor: pointer; }

  .shape-cell { white-space: normal; }
  .shape-cell svg { display: block; margin-bottom: 3px; }
  .shape-cell summary { cursor: pointer; }
</style>
