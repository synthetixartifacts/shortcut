/**
 * Tauri event listeners fired during a transcription round-trip.
 *
 * Wires the three progress/diagnostic event channels emitted by the Rust
 * transcription pipeline and returns a single cleanup function. Extracted from
 * `dictation-controller.ts` in PHASE 3A so the controller can shed ~40 LOC of
 * listener boilerplate in PHASE 3B.
 *
 * Event contract (must stay in sync with `src-tauri/src/transcription/`):
 * - `transcribe-retry`      — backend retried a provider call (attempt count).
 * - `transcribe-diagnostic` — byte-level progress for troubleshooting.
 * - `transcribe-log`        — free-form progress strings from local provider.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { log, logWarn } from '$lib/utils/logger';

export interface TranscribeRetryPayload {
  attempt: number;
  maxAttempts: number;
  delayMs?: number;
  error?: string;
  success?: boolean;
}

export interface TranscribeDiagnosticPayload {
  stage: string;
  base64Bytes?: number;
  decodedBytes?: number;
  expectedBytes?: number;
  fileBytes?: number;
  audioBytes?: number;
  contentLength?: number | null;
}

/**
 * Attach the three listener channels and return a cleanup callback.
 *
 * The cleanup is idempotent — calling it twice is safe. If any individual
 * `listen()` call fails, the helper logs at debug and still returns a valid
 * cleanup for whichever subscriptions succeeded.
 */
export async function attachTranscriptionListeners(): Promise<UnlistenFn> {
  const unlistenFns: UnlistenFn[] = [];

  try {
    unlistenFns.push(
      await listen<TranscribeRetryPayload>('transcribe-retry', async (event) => {
        const { attempt, maxAttempts, delayMs, error, success } = event.payload;
        if (success) {
          await log(
            `[DictationController] Transcription retry ${attempt}/${maxAttempts} succeeded`
          );
        } else {
          await logWarn(
            `[DictationController] Transcription attempt ${attempt}/${maxAttempts} retrying in ${delayMs}ms - ${error}`
          );
        }
      })
    );

    unlistenFns.push(
      await listen<TranscribeDiagnosticPayload>('transcribe-diagnostic', async (event) => {
        const d = event.payload;
        if (d.stage === 'file_read') {
          await log(`[DictationController] Rust read file: ${d.fileBytes} bytes`);
        } else if (d.stage === 'request') {
          await log(
            `[DictationController] Multipart body: audio=${d.audioBytes} bytes, Content-Length=${d.contentLength ?? 'unknown'}`
          );
        } else if (d.stage === 'received') {
          await log(`[DictationController] Rust received base64: ${d.base64Bytes} bytes`);
        } else if (d.stage === 'decoded') {
          await log(
            `[DictationController] Rust decoded audio: ${d.decodedBytes} bytes (expected ~${d.expectedBytes})`
          );
        }
      })
    );

    unlistenFns.push(
      await listen<string>('transcribe-log', async (event) => {
        await log(`[Rust] ${event.payload}`);
      })
    );
  } catch {
    // Non-critical — continue without the subscriptions that failed.
  }

  return () => {
    for (const fn of unlistenFns) {
      try {
        fn();
      } catch {
        /* ignore */
      }
    }
    unlistenFns.length = 0;
  };
}
