<script>
  let { name, size = 16 } = $props();
  const sprite = import.meta.env.BASE_URL + "blueprint/frame.svg";
</script>

<svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true" focusable="false">
  <use href={`${sprite}#i${size}-${name}`} />
</svg>

<style>
  svg {
    flex: none;
    --icon-accent: var(--oa-accent-text);
  }

  :global(button:disabled) svg { --icon-accent: currentColor; }
</style>
