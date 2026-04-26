<script lang="ts">
  /**
   * MicrophoneSelector - Device selection with permission awareness
   *
   * Presentational component that enumerates audio devices and shows
   * appropriate UI based on permission state. Accepts selectedDeviceId
   * as a prop for unidirectional data flow.
   */
  import { onMount, onDestroy } from 'svelte';
  import {
    getAudioInputDevices,
    requestAndEnumerateDevices,
    type AudioDevice,
    type DeviceEnumerationResult,
  } from '$lib/services/microphone';
  import { type PermissionState } from '$lib/services/microphone-permission';
  import { Button, Select, Icon } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    selectedDeviceId: string | null;
    onSelect?: (deviceId: string | null) => void;
    /** Compact inline rendering (no label, smaller chip-sized controls). */
    compact?: boolean;
  }

  let { selectedDeviceId, onSelect, compact = false }: Props = $props();

  // State
  let devices = $state<AudioDevice[]>([]);
  let permissionState = $state<PermissionState>('unknown');
  let isLoading = $state(true);
  let isRequesting = $state(false);
  let error = $state<string | null>(null);

  onMount(() => {
    loadDevices(true);
    if (navigator.mediaDevices) {
      navigator.mediaDevices.addEventListener('devicechange', refreshDevices);
    }
  });

  onDestroy(() => {
    if (navigator.mediaDevices) {
      navigator.mediaDevices.removeEventListener('devicechange', refreshDevices);
    }
  });

  async function loadDevices(showLoading = false) {
    if (showLoading) isLoading = true;
    error = null;

    const result: DeviceEnumerationResult = await getAudioInputDevices();
    devices = result.devices;
    permissionState = result.permissionState;
    error = result.error || null;

    if (result.devices.length > 0 && selectedDeviceId) {
      const exists = result.devices.some(d => d.deviceId === selectedDeviceId);
      if (!exists) {
        onSelect?.(null);
      }
    }

    if (showLoading) isLoading = false;
  }

  function refreshDevices() {
    loadDevices(false);
  }

  async function handleRequestPermission() {
    isRequesting = true;
    error = null;

    const result = await requestAndEnumerateDevices();
    devices = result.devices;
    permissionState = result.permissionState;
    error = result.error || null;
    isRequesting = false;
  }

  function handleChange(value: string) {
    onSelect?.(value === 'default' ? null : value);
  }

  const showDropdown = $derived(permissionState === 'granted' && devices.length > 0);
  const showPermissionPrompt = $derived(permissionState === 'prompt' || permissionState === 'unknown');
  const showDeniedMessage = $derived(permissionState === 'denied');
  const dropdownValue = $derived(selectedDeviceId || 'default');
  const dropdownOptions = $derived([
    { value: 'default', label: t('mic.system_default') },
    ...devices
      .filter(d => d.deviceId !== 'default')
      .map(d => ({ value: d.deviceId, label: d.label })),
  ]);
</script>

<div class="microphone-selector" class:compact>
  {#if !compact}
    <label for="mic-select" class="label-caps">{t('mic.label')}</label>
  {/if}

  {#if isLoading}
    <div class="state-box">
      <span class="muted">{t('mic.loading')}</span>
    </div>
  {:else if showPermissionPrompt}
    {#if compact}
      <a class="mic-chip" href="/actions/dictation">{t('mic.permission_required')}</a>
    {:else}
      <div class="state-box">
        <p>{t('mic.permission_required')}</p>
        <Button onclick={handleRequestPermission} disabled={isRequesting}>
          {isRequesting ? t('mic.button_requesting') : t('mic.button_allow')}
        </Button>
      </div>
    {/if}
  {:else if showDeniedMessage}
    {#if compact}
      <a class="mic-chip mic-chip--danger" href="/actions/dictation">{t('mic.permission_denied')}</a>
    {:else}
      <div class="state-box denied">
        <p>{t('mic.permission_denied')}</p>
        <Button variant="secondary" onclick={() => loadDevices(true)}>
          {t('mic.button_check_again')}
        </Button>
      </div>
    {/if}
  {:else if showDropdown}
    {#if compact}
      <div class="mic-pill">
        <span class="mic-pill-icon" aria-hidden="true">
          <Icon name="mic" size={12} />
        </span>
        <Select
          id="mic-select"
          value={dropdownValue}
          options={dropdownOptions}
          onchange={handleChange}
          ariaLabel={t('mic.label')}
        />
      </div>
    {:else}
      <Select
        id="mic-select"
        value={dropdownValue}
        options={dropdownOptions}
        onchange={handleChange}
        ariaLabel={t('mic.label')}
      />
    {/if}
  {:else if error}
    {#if compact}
      <a class="mic-chip mic-chip--warning" href="/actions/dictation">{error}</a>
    {:else}
      <div class="state-box warning">
        <p>{error}</p>
        <Button variant="secondary" onclick={() => loadDevices(true)}>
          {t('mic.button_check_again')}
        </Button>
      </div>
    {/if}
  {:else}
    {#if compact}
      <a class="mic-chip" href="/actions/dictation">{t('mic.no_microphones')}</a>
    {:else}
      <div class="state-box">
        <p>{t('mic.no_microphones')}</p>
        <Button variant="secondary" onclick={() => loadDevices(true)}>
          {t('common.refresh')}
        </Button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .microphone-selector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-lg);
  }

  .microphone-selector.compact {
    margin-bottom: 0;
    gap: 0;
    flex-direction: row;
    align-items: center;
  }

  .mic-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    height: 28px;
    padding: 0 var(--spacing-sm);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    background: var(--color-card-bg);
    box-sizing: border-box;
  }

  .mic-pill-icon {
    display: inline-flex;
    align-items: center;
    color: var(--color-text-muted);
    line-height: 1;
  }

  .microphone-selector.compact :global(.select) {
    height: auto;
    padding: 0;
    border: none;
    background: var(--color-card-bg);
    color: var(--color-text);
    font-size: 0.8rem;
    line-height: 1;
    border-radius: 0;
    max-width: 110px;
    box-shadow: none;
  }

  .microphone-selector.compact :global(.select:focus) {
    box-shadow: none;
  }

  .mic-chip {
    display: inline-flex;
    align-items: center;
    height: 28px;
    padding: 0 var(--spacing-sm);
    font-size: 0.8rem;
    font-weight: 500;
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-kbd-border);
    background: var(--color-card-bg);
    color: var(--color-text);
    text-decoration: none;
    white-space: nowrap;
    box-sizing: border-box;
  }

  .mic-chip:hover {
    filter: brightness(1.05);
  }

  .mic-chip--danger {
    border-color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, var(--color-card-bg));
    color: var(--color-danger);
  }

  .mic-chip--warning {
    border-color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 8%, var(--color-card-bg));
    color: var(--color-warning);
  }

  .state-box {
    padding: var(--spacing-md);
    background: var(--color-surface);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
  }

  .state-box p {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.9rem;
    color: var(--color-text);
  }

  .state-box .muted {
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }

  .state-box.denied {
    border-color: var(--color-danger);
    background: var(--color-danger-light);
  }

  .state-box.denied p {
    color: var(--color-danger);
  }

  .state-box.warning {
    border-color: var(--color-warning);
    background: var(--color-warning-light);
  }
</style>
