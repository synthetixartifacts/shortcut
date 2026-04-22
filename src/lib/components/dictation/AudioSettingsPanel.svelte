<script lang="ts">
  /**
   * AudioSettingsPanel - Configure microphone and audio processing
   *
   * Includes:
   * - Microphone selection with permission handling
   * - Audio processing options
   * - Microphone test functionality
   *
   * Accepts optional `microphoneSaveStatus` / `audioSettingsSaveStatus` props
   * so per-field save feedback renders adjacent to the control it describes.
   */
  import type { AudioSettings } from '$lib/features/dictation/types';
  import MicrophoneSelector from './MicrophoneSelector.svelte';
  import MicrophoneTest from './MicrophoneTest.svelte';
  import { SaveIndicator } from '$lib/components/ui/patterns';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    microphoneId: string | null;
    settings: AudioSettings;
    onMicrophoneChange: (id: string | null) => void;
    onSettingsChange: (settings: AudioSettings) => void;
    microphoneSaveStatus?: SaveStatus;
    audioSettingsSaveStatus?: SaveStatus;
  }

  let {
    microphoneId,
    settings,
    onMicrophoneChange,
    onSettingsChange,
    microphoneSaveStatus,
    audioSettingsSaveStatus,
  }: Props = $props();

  function toggle(key: keyof AudioSettings): void {
    onSettingsChange({
      ...settings,
      [key]: !settings[key],
    });
  }
</script>

<div class="audio-settings">
  <div class="mic-block">
    <MicrophoneSelector
      selectedDeviceId={microphoneId}
      onSelect={onMicrophoneChange}
    />
    {#if microphoneSaveStatus}
      <SaveIndicator status={microphoneSaveStatus} />
    {/if}
  </div>

  <MicrophoneTest />

  <div class="audio-options">
    <div class="options-label label-caps">{t('audio.processing')}</div>
    <label class="checkbox-option">
      <input
        type="checkbox"
        checked={settings.noiseSuppression}
        onchange={() => toggle('noiseSuppression')}
      />
      <span>{t('audio.noise_suppression')}</span>
    </label>
    <label class="checkbox-option">
      <input
        type="checkbox"
        checked={settings.echoCancellation}
        onchange={() => toggle('echoCancellation')}
      />
      <span>{t('audio.echo_cancellation')}</span>
    </label>
    <label class="checkbox-option">
      <input
        type="checkbox"
        checked={settings.autoGainControl}
        onchange={() => toggle('autoGainControl')}
      />
      <span>{t('audio.auto_gain')}</span>
    </label>
    {#if audioSettingsSaveStatus}
      <SaveIndicator status={audioSettingsSaveStatus} />
    {/if}
  </div>
</div>

<style>
  .audio-settings {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .mic-block {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .audio-options {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .options-label {
    margin-bottom: var(--spacing-xs);
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
  }

  .checkbox-option input {
    margin: 0;
  }

  .checkbox-option span {
    font-size: 0.9rem;
  }
</style>
