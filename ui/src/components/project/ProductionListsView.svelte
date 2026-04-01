<script>
  import { onMount } from "svelte";
  import { productionDataProject, loadProductionDataProject } from "../../stores/production.js";

  let activeTab = "kortlijst";

  const tabs = [
    { id: "kortlijst", label: "Kortlijst" },
    { id: "glaslijst", label: "Glaslijst" },
    { id: "beslaglijst", label: "Beslaglijst" },
    { id: "rubberlijst", label: "Rubberlijst" },
    { id: "paneellijst", label: "Paneellijst" },
    { id: "stuklijst", label: "Stuklijst" },
  ];

  const MEMBER_LABELS = {
    frame_top: "Bovendorpel", frame_bottom: "Onderdorpel",
    frame_left: "Stijl links", frame_right: "Stijl rechts",
    divider_h: "Tussendorpel", divider_v: "Tussenstijl",
    sash_top: "Raamhout boven", sash_bottom: "Raamhout onder",
    sash_left: "Raamhout links", sash_right: "Raamhout rechts",
  };

  const GASKET_LABELS = {
    glazing_inner: "Binnenrubber", glazing_outer: "Buitenrubber",
    sash_seal: "Vleugelafdichting", frame_seal: "Kozijnafdichting",
  };

  onMount(loadProductionDataProject);

  // Flatten data per list type
  $: allCut = ($productionDataProject || []).flatMap(p =>
    (p.cutList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allGlass = ($productionDataProject || []).flatMap(p =>
    (p.glassList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allHw = ($productionDataProject || []).flatMap(p =>
    (p.hardwareList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allGasket = ($productionDataProject || []).flatMap(p =>
    (p.gasketList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allPanel = ($productionDataProject || []).flatMap(p =>
    (p.panelList || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );
  $: allBom = ($productionDataProject || []).flatMap(p =>
    (p.bom || []).map(i => ({ ...i, mark: p.kozijnMark }))
  );

  async function refresh() {
    await loadProductionDataProject();
  }
</script>

<div class="production-view">
  <div class="toolbar">
    <h2>Productiestaten</h2>
    <button class="refresh-btn" on:click={refresh}>Herbereken</button>
  </div>

  <div class="tab-bar">
    {#each tabs as tab}
      <button
        class="tab"
        class:active={activeTab === tab.id}
        on:click={() => (activeTab = tab.id)}
      >
        {tab.label}
        <span class="count">
          {#if tab.id === "kortlijst"}{allCut.length}
          {:else if tab.id === "glaslijst"}{allGlass.length}
          {:else if tab.id === "beslaglijst"}{allHw.length}
          {:else if tab.id === "rubberlijst"}{allGasket.length}
          {:else if tab.id === "paneellijst"}{allPanel.length}
          {:else if tab.id === "stuklijst"}{allBom.length}
          {/if}
        </span>
      </button>
    {/each}
  </div>

  <div class="table-container">
    {#if activeTab === "kortlijst"}
      <table>
        <thead><tr>
          <th>Kozijn</th><th>Pos.</th><th>Onderdeel</th><th>Profiel</th>
          <th>Materiaal</th><th>Netto</th><th>Bruto</th><th>Hoek L</th><th>Hoek R</th><th>Aantal</th>
        </tr></thead>
        <tbody>
          {#each allCut as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.pieceId}</td>
              <td>{MEMBER_LABELS[item.memberType] || item.memberType}</td>
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
        <thead><tr>
          <th>Kozijn</th><th>Pos.</th><th>Glastype</th><th>Breedte</th>
          <th>Hoogte</th><th>Dikte</th><th>Ug</th><th>Opp.</th><th>Aantal</th>
        </tr></thead>
        <tbody>
          {#each allGlass as item}
            <tr>
              <td>{item.mark}</td>
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

    {:else if activeTab === "beslaglijst"}
      <table>
        <thead><tr>
          <th>Kozijn</th><th>Cel</th><th>Component</th><th>Omschrijving</th><th>Aantal</th>
        </tr></thead>
        <tbody>
          {#each allHw as item}
            <tr>
              <td>{item.mark}</td>
              <td>{item.cellIndex + 1}</td>
              <td>{item.component}</td>
              <td>{item.description}</td>
              <td class="num">{item.quantity}</td>
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "rubberlijst"}
      <table>
        <thead><tr>
          <th>Kozijn</th><th>Type</th><th>Lengte (mm)</th><th>Aantal</th>
        </tr></thead>
        <tbody>
          {#each allGasket as item}
            <tr>
              <td>{item.mark}</td>
              <td>{GASKET_LABELS[item.gasketType] || item.gasketType}</td>
              <td class="num">{Math.round(item.lengthMm)}</td>
              <td class="num">{item.quantity}</td>
            </tr>
          {/each}
        </tbody>
      </table>

    {:else if activeTab === "paneellijst"}
      <table>
        <thead><tr>
          <th>Kozijn</th><th>Pos.</th><th>Breedte</th><th>Hoogte</th><th>Type</th><th>Aantal</th>
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

    {:else if activeTab === "stuklijst"}
      <table>
        <thead><tr>
          <th>Kozijn</th><th>Categorie</th><th>Omschrijving</th><th>Eenheid</th><th>Hoeveelheid</th>
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
    cursor: pointer;
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
    cursor: pointer;
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

  table {
    width: 100%;
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
  }

  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
