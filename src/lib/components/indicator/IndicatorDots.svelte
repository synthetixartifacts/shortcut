<script lang="ts">
  /**
   * Animated Dots Indicator
   *
   * Three pulsing dots with staggered animation.
   * Color changes based on activity type.
   */

  import type { ActivityType } from '$lib/features/indicator';
  import { ACTIVITY_COLORS } from '$lib/features/indicator';

  interface Props {
    activityType?: ActivityType;
  }

  let { activityType = 'processing' }: Props = $props();

  const colors = $derived(ACTIVITY_COLORS[activityType]);
</script>

<div class="dots" style="--dot-color: {colors.primary}">
  <span class="dot"></span>
  <span class="dot"></span>
  <span class="dot"></span>
</div>

<style>
  .dots {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dot {
    width: 10px;
    height: 10px;
    background: var(--dot-color);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .dot:nth-child(2) {
    animation-delay: 0.2s;
  }

  .dot:nth-child(3) {
    animation-delay: 0.4s;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 0.4;
      transform: scale(0.8);
    }
    50% {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
