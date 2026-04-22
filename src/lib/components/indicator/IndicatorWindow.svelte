<script lang="ts">
  /**
   * Indicator Window Content
   *
   * Main component rendered in the indicator window.
   * Shows activity type with animated dots and optional label.
   */

  import type { ActivityType, ActivityState } from '$lib/features/indicator';
  import { ACTIVITY_COLORS, ACTIVITY_LABELS } from '$lib/features/indicator';
  import IndicatorDots from './IndicatorDots.svelte';

  interface Props {
    activityType?: ActivityType;
    activityState?: ActivityState;
    message?: string;
  }

  let {
    activityType = 'processing',
    activityState = 'active',
    message = '',
  }: Props = $props();

  const colors = $derived(ACTIVITY_COLORS[activityType]);
  const label = $derived(message || ACTIVITY_LABELS[activityType]);

  const showDots = $derived(activityState === 'active' || activityState === 'preparing');
  const showPreparing = $derived(activityState === 'preparing');
  const showSuccess = $derived(activityState === 'success');
  const showError = $derived(activityState === 'error');
</script>

<div class="wrapper">
  <div
    class="indicator"
    class:preparing={showPreparing}
    class:success={showSuccess}
    class:error={showError}
    style="--bg-color: {colors.background}; --glow-color: {colors.primary}"
  >
    {#if showDots}
      <IndicatorDots {activityType} />
    {:else if showSuccess}
      <span class="icon success-icon">✓</span>
    {:else if showError}
      <span class="icon error-icon">✕</span>
    {/if}

    {#if label}
      <span class="label">{label}</span>
    {/if}
  </div>
</div>

<style>
  .wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    background: transparent;
  }

  .indicator {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 18px;
    background: var(--bg-color);
    border-radius: 22px;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: var(--shadow-indicator-base);
    animation: fadeIn 0.2s ease-out;
  }

  .indicator.preparing {
    background: var(--color-indicator-preparing-bg);
    box-shadow: var(--shadow-indicator-preparing);
  }

  .indicator.success {
    background: var(--color-indicator-success-bg);
    box-shadow: var(--shadow-indicator-success);
  }

  .indicator.error {
    background: var(--color-indicator-error-bg);
    box-shadow: var(--shadow-indicator-error);
  }

  .icon {
    font-size: 16px;
    font-weight: bold;
    color: white;
  }

  .success-icon {
    color: var(--color-indicator-success-icon);
  }

  .error-icon {
    color: var(--color-indicator-error-icon);
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-indicator-text);
    max-width: 380px;
    text-align: center;
    line-height: 1.3;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
