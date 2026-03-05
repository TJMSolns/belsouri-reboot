<script lang="ts">
  let {
    practiceComplete,
    officesComplete,
    providersComplete,
    proceduresComplete,
    onGoTo,
  }: {
    practiceComplete: boolean;
    officesComplete: boolean;
    providersComplete: boolean;
    proceduresComplete: boolean;
    onGoTo: (tab: string) => void;
  } = $props();

  const steps = $derived([
    { label: "Practice details", complete: practiceComplete, tab: "practice" },
    { label: "Add an office", complete: officesComplete, tab: "offices" },
    { label: "Register a provider", complete: providersComplete, tab: "providers" },
    { label: "Define procedure types", complete: proceduresComplete, tab: "procedures" },
  ]);

  const completedCount = $derived(steps.filter((s) => s.complete).length);
  const allComplete = $derived(completedCount === steps.length);

  // Index of first incomplete step (for "current" step indicator)
  const currentStepIndex = $derived(steps.findIndex((s) => !s.complete));
</script>

{#if allComplete}
  <div class="complete-chip" role="status" aria-label="Practice setup complete">
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <polyline points="20 6 9 11 4 16" />
    </svg>
    Practice setup complete
  </div>
{:else}
  <div class="checklist-panel" role="region" aria-label="Practice setup checklist">
    <div class="checklist-header">
      <span class="checklist-title">Practice Setup</span>
      <span class="checklist-count" aria-live="polite">
        {completedCount} of {steps.length} complete
      </span>
    </div>

    <div
      class="progress-bar-track"
      role="progressbar"
      aria-valuenow={completedCount}
      aria-valuemin={0}
      aria-valuemax={steps.length}
      aria-label="Setup progress: {completedCount} of {steps.length} steps complete"
    >
      <div
        class="progress-bar-fill"
        style="width: {(completedCount / steps.length) * 100}%"
      ></div>
    </div>

    <ol class="step-list" aria-label="Setup steps">
      {#each steps as step, i}
        <li
          class="step-row"
          class:step-complete={step.complete}
          class:step-current={!step.complete && i === currentStepIndex}
          class:step-future={!step.complete && i !== currentStepIndex}
        >
          <span class="step-icon" aria-hidden="true">
            {#if step.complete}
              <!-- Checkmark icon -->
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="20 6 9 11 4 16" />
              </svg>
            {:else if i === currentStepIndex}
              <!-- Arrow indicator for current step -->
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            {:else}
              <!-- Empty spacer for future steps -->
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                aria-hidden="true"
              ></svg>
            {/if}
          </span>

          <span class="step-label">
            {step.label}
          </span>

          <button
            class="btn btn-sm btn-secondary go-btn"
            onclick={() => onGoTo(step.tab)}
            aria-label="Go to {step.label}"
          >
            Go
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <polyline points="9 18 15 12 9 6" />
            </svg>
          </button>
        </li>
      {/each}
    </ol>
  </div>
{/if}

<style>
  /* ── Complete chip ───────────────────────────────────────────────── */
  .complete-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-pill);
    background: var(--caribbean-teal-lt);
    color: var(--caribbean-teal);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: 600;
    margin-bottom: var(--space-5);
  }

  /* ── Checklist panel ─────────────────────────────────────────────── */
  .checklist-panel {
    background: #fff;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    padding: var(--space-5) var(--space-6);
    margin-bottom: var(--space-6);
  }

  .checklist-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .checklist-title {
    font-family: var(--font-heading);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--abyss-navy);
  }

  .checklist-count {
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--slate-fog);
  }

  /* ── Progress bar ────────────────────────────────────────────────── */
  .progress-bar-track {
    height: 6px;
    background: var(--pearl-mist-dk);
    border-radius: var(--radius-pill);
    margin-bottom: var(--space-4);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--caribbean-teal);
    border-radius: var(--radius-pill);
    transition: width var(--transition-slow);
  }

  /* ── Step list ───────────────────────────────────────────────────── */
  .step-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .step-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) 0;
  }

  .step-icon {
    flex-shrink: 0;
    width: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .step-label {
    flex: 1;
    font-family: var(--font-body);
    font-size: var(--text-sm);
  }

  /* Complete */
  .step-complete .step-icon {
    color: var(--caribbean-teal);
  }
  .step-complete .step-label {
    color: var(--slate-fog);
    text-decoration: line-through;
    text-decoration-color: var(--pearl-mist-dk);
  }

  /* Current (first incomplete) */
  .step-current .step-icon {
    color: var(--caribbean-teal);
  }
  .step-current .step-label {
    color: var(--abyss-navy);
    font-weight: 600;
  }

  /* Future (not yet reached) */
  .step-future .step-icon {
    color: var(--pearl-mist-dk);
  }
  .step-future .step-label {
    color: var(--slate-fog);
  }

  /* Go button */
  .go-btn {
    flex-shrink: 0;
    min-height: 44px;
    padding: 0 var(--space-3);
    font-size: var(--text-xs);
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  /* Completed rows hide the Go button */
  .step-complete .go-btn {
    visibility: hidden;
    pointer-events: none;
  }
</style>
