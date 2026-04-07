<script>
  import { _ } from "svelte-i18n";
  import { updateCellType } from "../../stores/kozijn.js";

  let { visible = false, cellIndex = 0, screenX = 0, screenY = 0, onclose = () => {} } = $props();

  const PANEL_TYPES = [
    { type: "fixed_glass", label: "Vast glas", icon: "VG", color: "#3B82F6" },
    { type: "turn_tilt", label: "Draaikiep", icon: "DK", color: "#3B82F6", dir: "left" },
    { type: "turn", label: "Draairaam", icon: "D", color: "#60A5FA", dir: "left" },
    { type: "tilt", label: "Kiepraam", icon: "K", color: "#818CF8" },
    { type: "sliding", label: "Schuifraam", icon: "S", color: "#34D399" },
    { type: "door", label: "Deur", icon: "DR", color: "#F97316", dir: "inward" },
    { type: "panel", label: "Paneel", icon: "P", color: "#A8A29E" },
    { type: "ventilation", label: "Ventilatie", icon: "V", color: "#F59E0B" },
  ];

  async function selectType(pt) {
    await updateCellType(cellIndex, pt.type, pt.dir || null);
    onclose();
  }

  function handleClickOutside(e) {
    if (visible) onclose();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="overlay" onclick={handleClickOutside}>
    <div class="context-menu" style="left:{screenX}px; top:{screenY}px" onclick={(e) => e.stopPropagation()}>
      <div class="menu-header">Vaktype</div>
      {#each PANEL_TYPES as pt}
        <button class="menu-item" onclick={() => selectType(pt)}>
          <span class="menu-icon" style="background:{pt.color}20; color:{pt.color}">{pt.icon}</span>
          <span class="menu-label">{pt.label}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
  }

  .context-menu {
    position: fixed;
    background: var(--bg-surface);
    border: var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 4px;
    min-width: 160px;
    z-index: 101;
  }

  .menu-header {
    padding: 4px 8px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--text-primary);
    text-align: left;
    transition: background 0.1s;
  }

  .menu-item:hover {
    background: var(--bg-surface-alt);
  }

  .menu-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 18px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    font-family: var(--font-heading);
  }

  .menu-label {
    flex: 1;
  }
</style>
