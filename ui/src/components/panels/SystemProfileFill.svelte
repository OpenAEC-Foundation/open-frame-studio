<script>
  import { _ } from "svelte-i18n";
  let { solids } = $props();
  const colors = { pvc: "#b9bec2", aluminum: "#8e9fa9", reinforcement: "#576a78", insulator: "#42474b" };

  let paths = $derived.by(() => {
    const groups = new Map();
    for (const { material, points } of solids) {
      if (!groups.has(material)) groups.set(material, []);
      groups.get(material).push("M " + points.map(p => p.join(" ")).join(" L ") + " Z");
    }
    return [...groups].map(([material, segments]) => ({ color: colors[material], d: segments.join(" ") }));
  });
</script>

<g pointer-events="none">
  <title>{$_("crossSection.schematic")}</title>
  {#each paths as path}
    <path d={path.d} fill={path.color} />
  {/each}
</g>
