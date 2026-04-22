/**
 * Audio Recorder Startup Logic
 *
 * Handles the complex startup sequence: device validation, stream acquisition,
 * microphone readiness check, and MediaRecorder initialization.
 * Extracted from AudioRecorder to keep file sizes manageable.
 */

import { log, logError } from '$lib/utils/logger';
import { waitForMicrophoneReady, getSupportedMimeType } from './audio-helpers';
import { validateMicrophoneDevice } from './device-validation';
import { RECORDER_CHUNK_INTERVAL_MS } from './constants';
import type { AudioRecorderOptions, RecordingState } from './audio-recorder';

/** Timeout for getUserMedia() — prevents infinite hang when WebKit blocks mic access */
const STREAM_ACQUIRE_TIMEOUT_MS = 8000;

/** Internal recorder state needed by startup */
interface RecorderInternals {
  options: AudioRecorderOptions;
  stream: MediaStream | null;
  mediaRecorder: MediaRecorder | null;
  chunks: Blob[];
  startTime: number;
  _state: RecordingState;
  setStream(stream: MediaStream | null): void;
  setMediaRecorder(recorder: MediaRecorder | null): void;
  setChunks(chunks: Blob[]): void;
  setStartTime(time: number): void;
  setState(state: RecordingState): void;
  pushChunk(chunk: Blob): void;
}

/**
 * Acquire microphone stream with the given constraints.
 * Handles OverconstrainedError with fallback for system default.
 */
async function acquireStream(
  audioConstraints: MediaTrackConstraints,
  hasDeviceId: boolean,
): Promise<MediaStream> {
  const withTimeout = <T>(promise: Promise<T>): Promise<T> => {
    let timeoutId: ReturnType<typeof setTimeout>;
    const timeoutPromise = new Promise<never>((_, reject) => {
      timeoutId = setTimeout(() => {
        reject(new Error(
          'Microphone access timed out. On macOS, try opening the ShortCut window first, then retry.'
        ));
      }, STREAM_ACQUIRE_TIMEOUT_MS);
    });
    return Promise.race([promise, timeoutPromise]).finally(() => clearTimeout(timeoutId));
  };

  try {
    return await withTimeout(
      navigator.mediaDevices.getUserMedia({ audio: audioConstraints })
    );
  } catch (err) {
    const error = err instanceof Error ? err : new Error(String(err));

    if (error.name === 'OverconstrainedError') {
      if (hasDeviceId) {
        throw new Error(
          'Could not access the selected microphone. ' +
          'Please check that it is connected and select it in Dictation settings.'
        );
      }
      await log('[AudioRecorder] Constraints failed for default device, trying minimal...');
      return await withTimeout(
        navigator.mediaDevices.getUserMedia({ audio: true })
      );
    }
    throw error;
  }
}

/**
 * Log device mismatch warnings and actual device info.
 */
async function logDeviceInfo(stream: MediaStream, requestedDeviceId?: string): Promise<void> {
  const audioTrack = stream.getAudioTracks()[0];
  if (!audioTrack) return;

  const settings = audioTrack.getSettings();
  const actualDeviceId = settings.deviceId || 'unknown';

  if (requestedDeviceId && actualDeviceId !== requestedDeviceId) {
    await logError(
      `[AudioRecorder] UNEXPECTED: Requested ${requestedDeviceId.slice(0, 8)}... but got ${actualDeviceId.slice(0, 8)}...`
    );
  }
  await log(`[AudioRecorder] Recording with: ${audioTrack.label || actualDeviceId.slice(0, 8)}`);
}

/**
 * Create and start a MediaRecorder, returning when it confirms active recording.
 */
async function startMediaRecorder(
  stream: MediaStream,
  options: AudioRecorderOptions,
  signal: AbortSignal,
  internals: RecorderInternals,
): Promise<string> {
  const mimeType = getSupportedMimeType();
  await log(`[AudioRecorder] Using MIME type: ${mimeType || '(browser default)'}`);

  let recorder: MediaRecorder;
  try {
    const recorderOptions: MediaRecorderOptions = {
      audioBitsPerSecond: options.audioBitsPerSecond,
    };
    if (mimeType) {
      recorderOptions.mimeType = mimeType;
    }
    recorder = new MediaRecorder(stream, recorderOptions);
  } catch (recorderError) {
    await logError('[AudioRecorder] Failed to create MediaRecorder with MIME type', recorderError);
    await log('[AudioRecorder] Trying default MediaRecorder...');
    recorder = new MediaRecorder(stream);
  }

  internals.setMediaRecorder(recorder);

  recorder.ondataavailable = (event) => {
    if (event.data.size > 0) {
      internals.pushChunk(event.data);
    }
  };

  // Wait for MediaRecorder to actually start before returning
  await new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error('MediaRecorder start timeout'));
    }, 5000);

    const abortHandler = () => {
      clearTimeout(timeoutId);
      reject(new Error('Recording cancelled'));
    };
    signal.addEventListener('abort', abortHandler, { once: true });

    recorder.onstart = () => {
      clearTimeout(timeoutId);
      signal.removeEventListener('abort', abortHandler);
      internals.setState('recording');
      resolve();
    };

    recorder.onerror = (event) => {
      clearTimeout(timeoutId);
      signal.removeEventListener('abort', abortHandler);
      reject(new Error(`MediaRecorder error: ${event.type}`));
    };

    internals.setStartTime(Date.now());
    recorder.start(RECORDER_CHUNK_INTERVAL_MS);
  });

  return mimeType;
}

/**
 * Execute the full recording startup sequence.
 *
 * @param signal - AbortSignal for cancellation
 * @param internals - Mutable access to recorder state
 */
export async function executeStartup(
  signal: AbortSignal,
  internals: RecorderInternals,
): Promise<void> {
  const checkAbort = () => {
    if (signal.aborted) throw new Error('Recording cancelled');
  };

  try {
    // Validate device if specified
    if (internals.options.deviceId) {
      checkAbort();
      await validateMicrophoneDevice(internals.options.deviceId);
    }

    // Build audio constraints
    const audioConstraints: MediaTrackConstraints = {
      noiseSuppression: internals.options.noiseSuppression,
      echoCancellation: internals.options.echoCancellation,
      autoGainControl: internals.options.autoGainControl,
    };

    if (internals.options.deviceId) {
      audioConstraints.deviceId = { exact: internals.options.deviceId };
    }

    checkAbort();

    // Acquire microphone stream
    await log(`[AudioRecorder] Acquiring stream, visibilityState=${document.visibilityState}`);
    const streamStart = Date.now();
    const stream = await acquireStream(audioConstraints, !!internals.options.deviceId);
    await log(`[AudioRecorder] Stream acquired in ${Date.now() - streamStart}ms`);
    internals.setStream(stream);

    checkAbort();

    if (!stream) {
      throw new Error('Failed to get audio stream');
    }

    // Log device info
    await logDeviceInfo(stream, internals.options.deviceId);

    checkAbort();

    // Wait for microphone readiness (critical for Bluetooth on macOS)
    if (!signal.aborted) {
      await log('[AudioRecorder] Waiting for microphone to be ready...');
      const readinessResult = await waitForMicrophoneReady(stream);

      if (readinessResult.ready) {
        await log(`[AudioRecorder] Microphone ready after ${readinessResult.waitTimeMs}ms`);
      } else {
        await log(`[AudioRecorder] Microphone readiness not confirmed after ${readinessResult.waitTimeMs}ms, proceeding anyway`);
      }
    }

    checkAbort();

    internals.setChunks([]);

    // Create and start MediaRecorder
    const mimeType = await startMediaRecorder(stream, internals.options, signal, internals);

    await log(`[AudioRecorder] Recording confirmed active with ${mimeType}`);
  } catch (error) {
    const errorDetails = error instanceof Error
      ? `${error.name}: ${error.message}`
      : String(error);
    await logError('[AudioRecorder] Failed to start', errorDetails);
    throw error;
  }
}
