<script lang="ts">
  import { confirmState, _resolve } from '$lib/stores/confirm';

  let inputValue = $state('');

  $effect(() => {
    if ($confirmState.open) inputValue = '';
  });

  function onKeydown(e: KeyboardEvent) {
    if (!$confirmState.open) return;
    if (e.key === 'Escape') { e.preventDefault(); _resolve(false); }
    if (e.key === 'Enter' && canConfirm) { e.preventDefault(); doConfirm(); }
  }

  function doConfirm() {
    // If requiredInput is set, only confirm when they match
    _resolve(true);
  }

  let canConfirm = $derived(
    !$confirmState.options.requiredInput ||
    inputValue.trim() === $confirmState.options.requiredInput
  );
</script>

<svelte:window onkeydown={onKeydown} />

{#if $confirmState.open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => _resolve(false)} aria-hidden="true"></div>

  <dialog class="dialog" open aria-modal="true" aria-labelledby="dlg-title">
    <div class="dialog-header">
      <h2 id="dlg-title" class="dialog-title">{$confirmState.options.title}</h2>
    </div>

    <div class="dialog-body">
      <p class="dialog-message">{$confirmState.options.message}</p>

      {#if $confirmState.options.requiredInput}
        <div class="form-field" style="margin-top: 1rem;">
          <label class="field-label" for="dlg-input">
            Type <strong>{$confirmState.options.requiredInput}</strong> to confirm:
          </label>
          <input
            id="dlg-input"
            type="text"
            bind:value={inputValue}
            autocomplete="off"
            spellcheck="false"
          />
        </div>
      {/if}
    </div>

    <div class="dialog-footer">
      <button
        class="btn btn-ghost"
        onclick={() => _resolve(false)}
      >
        {$confirmState.options.cancelLabel ?? 'Cancel'}
      </button>
      <button
        class="btn {$confirmState.options.destructive ? 'btn-destructive' : 'btn-primary'}"
        onclick={doConfirm}
        disabled={!canConfirm}
      >
        {$confirmState.options.confirmLabel ?? 'Confirm'}
      </button>
    </div>
  </dialog>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(26, 45, 51, 0.45);
    z-index: 8000;
  }

  .dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 8001;
    width: min(480px, calc(100vw - 48px));
    background: #fff;
    border: none;
    border-radius: var(--radius-xl, 16px);
    box-shadow: var(--shadow-xl, 0 16px 48px rgba(26,45,51,0.20));
    padding: 0;
    animation: scale-in 150ms ease;
  }

  .dialog-header {
    padding: var(--space-6, 24px) var(--space-6, 24px) 0;
  }

  .dialog-title {
    font-family: var(--font-heading, 'Lexend', sans-serif);
    font-size: var(--text-xl, 1.25rem);
    font-weight: 600;
    color: var(--abyss-navy, #1A2D33);
    margin: 0;
  }

  .dialog-body {
    padding: var(--space-4, 16px) var(--space-6, 24px);
  }

  .dialog-message {
    color: var(--slate-fog, #6B7C82);
    font-size: var(--text-sm, 0.875rem);
    line-height: 1.6;
    margin: 0;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3, 12px);
    padding: var(--space-4, 16px) var(--space-6, 24px) var(--space-6, 24px);
  }

  @keyframes scale-in {
    from { transform: translate(-50%, -50%) scale(0.95); opacity: 0; }
    to   { transform: translate(-50%, -50%) scale(1);    opacity: 1; }
  }
</style>
