<script lang="ts">
  import { toast, type Toast } from '$lib/stores/toast';

  const icons: Record<Toast['type'], string> = {
    success: '✓',
    error:   '✕',
    info:    'i',
    warning: '!',
  };
</script>

<div class="toast-region" aria-live="polite" aria-atomic="false">
  {#each $toast as t (t.id)}
    <div class="toast toast-{t.type}" role="status" aria-label={t.message}>
      <span class="toast-icon" aria-hidden="true">{icons[t.type]}</span>
      <span class="toast-message">{t.message}</span>
      <button
        class="toast-close"
        aria-label="Dismiss"
        onclick={() => toast.dismiss(t.id)}
      >×</button>
    </div>
  {/each}
</div>

<style>
  .toast-region {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 9000;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    min-width: 280px;
    max-width: 420px;
    padding: 14px 16px;
    border-radius: var(--radius-lg, 12px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(26,45,51,0.16));
    font-family: var(--font-body, 'Inter', sans-serif);
    font-size: var(--text-sm, 0.875rem);
    line-height: 1.4;
    pointer-events: all;
    animation: slide-up 200ms ease;
    color: #fff;
  }

  .toast-success { background: var(--island-palm, #27AE60); }
  .toast-error   { background: var(--healthy-coral-dk, #E56554); }
  .toast-info    { background: var(--caribbean-teal, #008B99); }
  .toast-warning { background: #D4820A; }

  .toast-icon {
    font-size: 0.875rem;
    font-weight: 700;
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: rgba(255,255,255,0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    font-style: normal;
    line-height: 1;
  }

  .toast-message { flex: 1; }

  .toast-close {
    background: none;
    border: none;
    color: rgba(255,255,255,0.75);
    cursor: pointer;
    font-size: 1.25rem;
    line-height: 1;
    padding: 0;
    margin: -2px -4px 0 0;
    flex-shrink: 0;
  }
  .toast-close:hover { color: #fff; }

  @keyframes slide-up {
    from { transform: translateY(12px); opacity: 0; }
    to   { transform: translateY(0);    opacity: 1; }
  }
</style>
