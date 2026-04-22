/**
 * Microphone Test
 *
 * Provides functionality to test microphone recording
 * before actual dictation use.
 */

import { log, logError } from '$lib/utils/logger';
import { dictationConfigState } from '$lib/state/dictation-config.svelte';
import { validateMicrophoneDevice } from './device-validation';

/** Test recording result */
export interface MicrophoneTestResult {
  success: boolean;
  duration: number;
  peakLevel: number;
  averageLevel: number;
  message: string;
  audioBlob?: Blob;
}

/**
 * Run a microphone test recording
 *
 * Records for the specified duration and analyzes the audio levels.
 *
 * @param durationMs - Test duration in milliseconds (default 3000)
 * @param onProgress - Callback for real-time level updates
 */
export async function runMicrophoneTest(
  durationMs: number = 3000,
  onProgress?: (level: number, elapsed: number) => void
): Promise<MicrophoneTestResult> {
  await log('[MicrophoneTest] Starting test...');

  let stream: MediaStream | null = null;
  let audioContext: AudioContext | null = null;

  try {
    // Build constraints from config
    const config = dictationConfigState.config;

    // If a specific device was selected, validate it exists first
    if (config.selectedMicrophoneId) {
      await validateMicrophoneDevice(config.selectedMicrophoneId);
    }

    const constraints: MediaTrackConstraints = {
      noiseSuppression: config.audioSettings.noiseSuppression,
      echoCancellation: config.audioSettings.echoCancellation,
      autoGainControl: config.audioSettings.autoGainControl,
    };

    // Add device ID if specified - use 'exact' to ensure correct device
    if (config.selectedMicrophoneId) {
      constraints.deviceId = { exact: config.selectedMicrophoneId };
    }

    // Get microphone access
    stream = await navigator.mediaDevices.getUserMedia({ audio: constraints });
    await log('[MicrophoneTest] Got microphone access');

    // Create audio context for level analysis
    audioContext = new AudioContext();
    const source = audioContext.createMediaStreamSource(stream);
    const analyser = audioContext.createAnalyser();
    analyser.fftSize = 256;

    source.connect(analyser);

    const dataArray = new Uint8Array(analyser.frequencyBinCount);
    const levels: number[] = [];
    const startTime = Date.now();

    // Record audio for duration while analyzing levels
    const chunks: Blob[] = [];

    // Get supported MIME type (Safari doesn't support WebM)
    const getMimeType = (): string | undefined => {
      const types = ['audio/webm;codecs=opus', 'audio/webm', 'audio/mp4', 'audio/mpeg'];
      for (const type of types) {
        if (MediaRecorder.isTypeSupported(type)) return type;
      }
      return undefined; // Let browser choose
    };

    const mimeType = getMimeType();
    const mediaRecorder = new MediaRecorder(stream, mimeType ? { mimeType } : {});

    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) {
        chunks.push(e.data);
      }
    };

    // Promise to handle recording completion
    const recordingPromise = new Promise<Blob>((resolve) => {
      mediaRecorder.onstop = () => {
        const blob = new Blob(chunks, { type: mediaRecorder.mimeType });
        resolve(blob);
      };
    });

    mediaRecorder.start(100);

    // Analyze levels during recording
    const analyzeInterval = setInterval(() => {
      analyser.getByteTimeDomainData(dataArray);

      // Calculate RMS level
      let sum = 0;
      for (let i = 0; i < dataArray.length; i++) {
        const normalized = (dataArray[i] - 128) / 128;
        sum += normalized * normalized;
      }
      const rms = Math.sqrt(sum / dataArray.length);
      const level = Math.min(1, rms * 3); // Scale for visibility

      levels.push(level);

      const elapsed = Date.now() - startTime;
      onProgress?.(level, elapsed);
    }, 50);

    // Wait for duration
    await new Promise(resolve => setTimeout(resolve, durationMs));

    // Stop recording and analysis
    clearInterval(analyzeInterval);
    mediaRecorder.stop();

    const audioBlob = await recordingPromise;

    // Calculate statistics
    const peakLevel = Math.max(...levels);
    const averageLevel = levels.reduce((a, b) => a + b, 0) / levels.length;
    const duration = Date.now() - startTime;

    await log(`[MicrophoneTest] Complete. Peak: ${peakLevel.toFixed(2)}, Avg: ${averageLevel.toFixed(2)}`);

    // Determine success based on audio levels
    const success = peakLevel > 0.05 && averageLevel > 0.01;
    const message = success
      ? 'Microphone is working correctly'
      : averageLevel > 0 ? 'Low audio level detected - speak louder or check microphone' : 'No audio detected - check microphone connection';

    return {
      success,
      duration,
      peakLevel,
      averageLevel,
      message,
      audioBlob,
    };
  } catch (error) {
    await logError('[MicrophoneTest] Failed', error);

    const message = error instanceof DOMException
      ? error.name === 'NotAllowedError'
        ? 'Microphone access denied'
        : error.message
      : 'Failed to test microphone';

    return {
      success: false,
      duration: 0,
      peakLevel: 0,
      averageLevel: 0,
      message,
    };
  } finally {
    // Cleanup
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
    }
    if (audioContext) {
      await audioContext.close();
    }
  }
}
