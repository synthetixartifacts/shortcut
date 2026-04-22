/** Dictation Controller — Records audio and sends to active transcription engine. */

import { log, logError, logWarn } from '$lib/utils/logger';
import { extractErrorMessage, truncateMessage, isNetworkError } from '$lib/utils/error';
import { AudioRecorder, getAudioRecorder } from './audio-recorder';
import { MIN_AUDIO_BLOB_SIZE, ERROR_TRUNCATION_LENGTH } from './constants';
import { convertToWav } from './wav-encoder';
import { attachTranscriptionListeners } from './transcription-events';
import { transcribeAudio, pasteText, addHistoryEntry, getModelStatus } from '$lib/api/tauri';
import { refreshHistory } from '$lib/state/history.svelte';
import { appState, setRecording, resetRecordingState } from '$lib/state/app.svelte';
import { dictationConfigState } from '$lib/state/dictation-config.svelte';
import { engineState, showSlownessWarning } from '$lib/state/engine.svelte';
import { startIndicator, indicatorSuccess, indicatorError } from '$lib/features/indicator';
import { updateActivity, endActivity, activityState } from '$lib/state/activity.svelte';

class DictationController {
  private audioRecorder: AudioRecorder;
  private initialized = false;
  private cancellingStartup = false; // Track intentional cancellation during startup

  constructor() {
    // Start with default options; recorder is re-initialized per start()
    // using the current config — avoids stale constructor-time config reads.
    this.audioRecorder = getAudioRecorder();
  }

  get isReady(): boolean {
    return this.initialized && AudioRecorder.isSupported();
  }

  get isRecording(): boolean {
    return appState.isRecording || this.audioRecorder.state !== 'idle';
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;
    try {
      await log('[DictationController] Initializing...');
      if (!AudioRecorder.isSupported()) {
        await logError('[DictationController] Audio recording not supported');
        return;
      }
      // Pre-grant mic permission while window is visible (important for macOS WKWebView)
      if (document.visibilityState === 'visible') {
        try {
          const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
          stream.getTracks().forEach(track => track.stop());
          await log('[DictationController] Microphone permission pre-granted');
        } catch {
          await logWarn('[DictationController] Mic pre-grant failed, will prompt on first use');
        }
      }
      this.initialized = true;
      await log('[DictationController] Initialized successfully');
    } catch (e) {
      await logError('[DictationController] Failed to initialize', e);
    }
  }

  /** Gate startRecording on local-engine model availability. */
  private async ensureEngineReady(): Promise<boolean> {
    if (engineState.activeEngine !== 'local-windows') return true;
    try {
      const modelStatus = await getModelStatus();
      if (modelStatus.state !== 'ready') {
        await indicatorError('Model not downloaded');
        await log('[DictationController] Local model not ready: ' + modelStatus.state);
        return false;
      }
      return true;
    } catch {
      await indicatorError('Local engine error');
      return false;
    }
  }

  async startRecording(): Promise<void> {
    await log('[DictationController] startRecording called');
    if (!this.isReady) {
      await log('[DictationController] ERROR: Not initialized');
      return;
    }
    if (appState.isRecording) {
      await log('[DictationController] Already recording, ignoring');
      return;
    }
    if (!(await this.ensureEngineReady())) return;

    try {
      setRecording(true);
      await startIndicator('dictation', 'Preparing...');
      await updateActivity('preparing', 'Preparing...');

      const config = dictationConfigState.config;
      this.audioRecorder = getAudioRecorder({
        deviceId: config.selectedMicrophoneId || undefined,
        noiseSuppression: config.audioSettings.noiseSuppression,
        echoCancellation: config.audioSettings.echoCancellation,
        autoGainControl: config.audioSettings.autoGainControl,
      });
      // Waits for MediaRecorder to confirm it's actually capturing
      await this.audioRecorder.start();
      await updateActivity('active', 'Recording...');
      await log('[DictationController] Recording started - microphone active');
    } catch (e) {
      if (this.cancellingStartup) {
        await log('[DictationController] Start cancelled by stop request, deferring to stopRecording');
        return;
      }
      await logError('[DictationController] Failed to start recording', e);
      const message = extractErrorMessage(e);
      await indicatorError(truncateMessage(message, ERROR_TRUNCATION_LENGTH) || 'Microphone error');
      await this.resetState();
    }
  }

  /**
   * Prepare the blob for transmission: local engines need WAV (the pure-Rust
   * Opus decoder can't decode browser-encoded Opus; Web Audio API can).
   */
  private async prepareBlob(
    blob: Blob,
    mimeType: string
  ): Promise<{ blob: Blob; mime: string }> {
    if (!engineState.activeEngine.startsWith('local') || mimeType.includes('wav')) {
      return { blob, mime: mimeType };
    }
    try {
      const converted = await convertToWav(blob);
      await log(`[DictationController] Converted to WAV: ${converted.size} bytes`);
      return { blob: converted, mime: 'audio/wav' };
    } catch (e) {
      await logWarn(`[DictationController] WAV conversion failed, sending original: ${e}`);
      return { blob, mime: mimeType };
    }
  }

  private async handleStopError(e: unknown): Promise<void> {
    if (this.cancellingStartup) {
      await log('[DictationController] Recording cancelled during startup');
      await endActivity();
      return;
    }

    await logError('[DictationController] Failed to stop recording', e);
    const message = extractErrorMessage(e);

    // If Soniox (cloud) failed with a network error, suggest local fallback
    if (engineState.activeEngine === 'soniox' && isNetworkError(e)) {
      try {
        const modelStatus = await getModelStatus();
        if (modelStatus.state === 'ready') {
          await indicatorError('Network error');
          return;
        }
      } catch { /* model status unavailable, skip fallback suggestion */ }
    }

    await indicatorError(truncateMessage(message, ERROR_TRUNCATION_LENGTH) || 'Transcription failed');
  }

  async stopRecording(): Promise<void> {
    await log('[DictationController] stopRecording called');

    if (!appState.isRecording) {
      await log('[DictationController] Not recording, ignoring');
      return;
    }

    if (this.audioRecorder.state === 'starting') {
      await log('[DictationController] Cancelling recording during startup');
      this.cancellingStartup = true;
    }

    const detachListeners = await attachTranscriptionListeners();

    try {
      const result = await this.audioRecorder.stop();
      await log(`[DictationController] Recording stopped: ${result.duration}ms, ${result.blob.size} bytes`);
      await updateActivity('active', 'Processing...');

      if (result.blob.size < MIN_AUDIO_BLOB_SIZE) {
        await log('[DictationController] Audio too short');
        await indicatorError('No speech');
        return;
      }

      await updateActivity('active', 'Transcribing...');

      const config = dictationConfigState.config;
      const { blob: audioBlob, mime: audioMime } = await this.prepareBlob(result.blob, result.mimeType);

      const engineLabel = engineState.activeEngine.startsWith('local') ? 'local' : 'cloud';
      await log(`[DictationController] Sending to ${engineLabel} transcription (${engineState.activeEngine}): ` +
        `blob=${audioBlob.size} bytes, mimeType=${audioMime}, ` +
        `duration=${result.duration}ms, languages=[${config.languageHints.join(',')}]`);

      const transcriptionStartTime = Date.now();
      const transcriptionResult = await transcribeAudio(
        audioBlob,
        audioMime,
        config.languageHints,
        config.customTerms,
        config.backgroundText || undefined
      );

      // Slowness detection for local engine
      if (transcriptionResult.engine?.startsWith('local') && result.duration > 0) {
        const rtf = (Date.now() - transcriptionStartTime) / result.duration;
        if (rtf > 1.0 && !engineState.slownessDismissed) {
          showSlownessWarning(rtf);
          await log(`[DictationController] Local transcription slow: RTF=${rtf.toFixed(2)}`);
        }
      }

      const finalText = transcriptionResult.text;

      if (finalText) {
        await addHistoryEntry(
          finalText,
          result.duration,
          transcriptionResult.language || null,
          transcriptionResult.engine || engineState.activeEngine
        );
        await refreshHistory();

        await log(`[DictationController] Transcription: "${finalText}"`);
        await pasteText(finalText);
        await indicatorSuccess('Pasted!');
      } else {
        await log('[DictationController] No speech detected');
        await indicatorError('No speech');
      }
    } catch (e) {
      await this.handleStopError(e);
    } finally {
      detachListeners();
      this.cancellingStartup = false;
      await this.resetState();
    }
  }

  async toggleRecording(): Promise<void> {
    if (appState.isRecording) {
      await this.stopRecording();
    } else {
      await this.startRecording();
    }
  }

  cleanup(): void {
    this.audioRecorder.cancel();
    this.initialized = false;
  }

  private async resetState(): Promise<void> {
    resetRecordingState();
    // Only end activity if it's still active — don't end when success/error
    // was already shown (those have their own auto-hide).
    if (activityState.state === 'preparing' || activityState.state === 'active') {
      await endActivity();
    }
  }
}

let dictationControllerInstance: DictationController | null = null;

export function getDictationController(): DictationController {
  if (!dictationControllerInstance) {
    dictationControllerInstance = new DictationController();
  }
  return dictationControllerInstance;
}
