<script>
  import { _ } from "svelte-i18n";
  let { leaves = [] } = $props();

  function quantities(components) {
    const items = new Map();
    for (const { kind } of components) items.set(kind, (items.get(kind) ?? 0) + 1);
    return items;
  }
</script>

{#each leaves.filter(l => l.issue || l.components.length > 0) as leaf}
  <section>
    <h4>{$_(leaf.leaf === "left" ? "sashFit.left" : leaf.leaf === "right" ? "sashFit.right" : "system.fittings")}</h4>
    {#if leaf.issue}
      <p>{$_("system." + leaf.issue)}</p>
    {:else}
      <dl>
        {#each quantities(leaf.components) as [kind, quantity]}
          <dt>{$_("fitting." + kind)}</dt><dd>{quantity}</dd>
        {/each}
      </dl>
    {/if}
  </section>
{/each}

<style>
  section { margin: var(--sp-3) 0; }
  h4 { margin: 0 0 var(--sp-2); font-size: 12px; }
  dl { display: grid; grid-template-columns: 1fr auto; gap: 3px 8px; margin: 0; font-size: 12px; }
  dt { color: var(--text-secondary); }
  dd { margin: 0; font-variant-numeric: tabular-nums; }
  p { color: var(--amber); font-size: 12px; margin: 0; }
</style>
