/**
 * Audio Recording Helpers
 *
 * Utility functions for microphone readiness detection and MIME type support.
 * Extracted from AudioRecorder to keep files under 300 lines.
 */

import { logError } from '$lib/utils/logger';

/**
 * Configuration for microphone readiness detection
 */
const MIC_READY_CONFIG = {
  /** Maximum time to wait for microphone to become ready (ms) */
  timeoutMs: 3000,
  /** How often to check audio levels (ms) */
  checkIntervalMs: 50,
  /** Minimum signal variation to consider mic "ready" (even silence has some noise) */
  minSignalThreshold: 0.5,
};

/**
 * Wait for microphone to actually produce audio signal.
 *
 * Some devices (especially Bluetooth on macOS) need time after getUserMedia()
 * returns before they actually start capturing audio. This function monitors
 * the audio stream and waits until real samples are detected.
 *
 * Even in a silent room, a working microphone produces some noise floor.
 * If we're getting perfect zeros or no variation, the device isn't ready yet.
 *
 * @param stream - The MediaStream from getUserMedia
 * @param timeoutMs - Maximum time to wait before proceeding anyway
 * @returns Object with ready status and time taken
 */
export async function waitForMicrophoneReady(
  stream: MediaStream,
  timeoutMs: number = MIC_READY_CONFIG.timeoutMs
): Promise<{ ready: boolean; waitTimeMs: number }> {
  const startTime = Date.now();

  let audioContext: AudioContext | null = null;

  try {
    audioContext = new AudioContext();
    const source = audioContext.createMediaStreamSource(stream);
    const analyser = audioContext.createAnalyser();
    analyser.fftSize = 256;
    source.connect(analyser);

    const dataArray = new Uint8Array(analyser.frequencyBinCount);

    return await new Promise((resolve) => {
      const checkAudioLevel = () => {
        analyser.getByteTimeDomainData(dataArray);

        // Calculate variation from center (128 = silence in unsigned byte format)
        let totalDeviation = 0;
        for (let i = 0; i < dataArray.length; i++) {
          totalDeviation += Math.abs(dataArray[i] - 128);
        }
        const avgDeviation = totalDeviation / dataArray.length;

        const elapsed = Date.now() - startTime;

        if (avgDeviation > MIC_READY_CONFIG.minSignalThreshold) {
          audioContext?.close();
          resolve({ ready: true, waitTimeMs: elapsed });
          return;
        }

        if (elapsed >= timeoutMs) {
          audioContext?.close();
          resolve({ ready: false, waitTimeMs: elapsed });
          return;
        }

        setTimeout(checkAudioLevel, MIC_READY_CONFIG.checkIntervalMs);
      };

      checkAudioLevel();
    });
  } catch (error) {
    await logError('[AudioRecorder] Mic readiness check failed', error);
    audioContext?.close();
    return { ready: false, waitTimeMs: Date.now() - startTime };
  }
}

/**
 * Get the best supported MIME type for audio recording.
 *
 * Tries preferred formats in order, falling back to browser default.
 */
export function getSupportedMimeType(): string {
  const types = [
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/ogg;codecs=opus',
    'audio/mp4',
  ];

  for (const type of types) {
    if (MediaRecorder.isTypeSupported(type)) {
      return type;
    }
  }

  return '';
}
