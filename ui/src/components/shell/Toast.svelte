<script>
  import { toasts, dismiss } from "../../stores/toast.js";
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as t (t.id)}
      <div class="toast toast-{t.type}" onclick={() => dismiss(t.id)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if t.type === "success"}
            <path d="M20 6L9 17l-5-5"/>
          {:else if t.type === "error"}
            <circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6m0-6l6 6"/>
          {:else if t.type === "warning"}
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><path d="M12 9v4m0 4h.01"/>
          {:else}
            <circle cx="12" cy="12" r="10"/><path d="M12 16v-4m0-4h.01"/>
          {/if}
        </svg>
        <span>{t.message}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 36px;
    right: 12px;
    z-index: 200;
    display: flex;
    flex-direction: column-reverse;
    gap: 6px;
    max-width: 380px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-radius: 4px;
    font-size: 12px;
    color: #fff;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    animation: slideIn 0.2s ease-out;
  }

  .toast-success { background: #16a34a; }
  .toast-error { background: #dc2626; }
  .toast-warning { background: #d97706; }
  .toast-info { background: #2563eb; }

  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
