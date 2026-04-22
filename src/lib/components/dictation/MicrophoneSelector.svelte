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
  import { Button, Select } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';

  interface Props {
    selectedDeviceId: string | null;
    onSelect?: (deviceId: string | null) => void;
  }

  let { selectedDeviceId, onSelect }: Props = $props();

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

<div class="microphone-selector">
  <label for="mic-select" class="label-caps">{t('mic.label')}</label>

  {#if isLoading}
    <div class="state-box">
      <span class="muted">{t('mic.loading')}</span>
    </div>
  {:else if showPermissionPrompt}
    <div class="state-box">
      <p>{t('mic.permission_required')}</p>
      <Button onclick={handleRequestPermission} disabled={isRequesting}>
        {isRequesting ? t('mic.button_requesting') : t('mic.button_allow')}
      </Button>
    </div>
  {:else if showDeniedMessage}
    <div class="state-box denied">
      <p>{t('mic.permission_denied')}</p>
      <Button variant="secondary" onclick={() => loadDevices(true)}>
        {t('mic.button_check_again')}
      </Button>
    </div>
  {:else if showDropdown}
    <Select
      id="mic-select"
      value={dropdownValue}
      options={dropdownOptions}
      onchange={handleChange}
    />
  {:else if error}
    <div class="state-box warning">
      <p>{error}</p>
      <Button variant="secondary" onclick={() => loadDevices(true)}>
        {t('mic.button_check_again')}
      </Button>
    </div>
  {:else}
    <div class="state-box">
      <p>{t('mic.no_microphones')}</p>
      <Button variant="secondary" onclick={() => loadDevices(true)}>
        {t('common.refresh')}
      </Button>
    </div>
  {/if}
</div>

<style>
  .microphone-selector {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-lg);
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
