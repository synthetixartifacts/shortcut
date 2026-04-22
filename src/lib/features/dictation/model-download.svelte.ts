/**
 * Shared local-STT model download controller.
 *
 * Consolidates the identical download state machine previously duplicated in
 * `ModelDownload.svelte` and `EngineCard.svelte`. Exposes reactive `$state`
 * runes for status, in-flight flag, error, and progress percent, plus three
 * action methods (`start`, `cancel`, `refresh`).
 *
 * Wires the three Tauri events emitted by the Rust model manager:
 * - `model-download-progress`: { progress: number (0..1) }
 * - `model-download-complete`: ()
 * - `model-download-error`   : { error: string }
 *
 * Consumers migrate in PHASE 3B. The module is side-effect-free until
 * `attach()` is called.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  downloadModel,
  cancelModelDownload,
  getModelStatus,
} from '$lib/api/tauri';
import type { ModelStatus } from '$lib/types';

export interface ModelDownloadController {
  /** Reactive accessors (read inside templates — Svelte picks up runes). */
  readonly status: ModelStatus;
  readonly isDownloading: boolean;
  readonly error: string | null;
  readonly progressPercent: number;

  /** Attach Tauri listeners + fetch initial status. Returns cleanup callback. */
  attach(): Promise<UnlistenFn>;

  /** Trigger a fresh download attempt. */
  start(): Promise<void>;

  /** Ask the backend to cancel the in-flight download, if any. */
  cancel(): Promise<void>;

  /** Re-query backend for the current status (useful after external changes). */
  refresh(): Promise<void>;
}

/**
 * Create a fresh model-download controller. Each caller gets its own
 * reactive state — callers typically instantiate one per component mount.
 */
export function useModelDownload(): ModelDownloadController {
  let status = $state<ModelStatus>({ state: 'not_downloaded' });
  let isDownloading = $state(false);
  let error = $state<string | null>(null);

  const progressPercent = $derived(
    status.progress != null ? Math.round(status.progress * 100) : 0
  );

  async function refresh(): Promise<void> {
    try {
      status = await getModelStatus();
    } catch {
      /* local-stt unavailable on this platform — leave defaults */
    }
  }

  async function attach(): Promise<UnlistenFn> {
    await refresh();

    const unlistenFns: UnlistenFn[] = [];

    unlistenFns.push(
      await listen<{ progress: number }>('model-download-progress', (event) => {
        status = {
          ...status,
          state: 'downloading',
          progress: event.payload.progress,
        };
        isDownloading = true;
      })
    );

    unlistenFns.push(
      await listen('model-download-complete', async () => {
        try {
          status = await getModelStatus();
        } catch {
          status = { state: 'ready' };
        }
        isDownloading = false;
        error = null;
      })
    );

    unlistenFns.push(
      await listen<{ error: string }>('model-download-error', (event) => {
        error = event.payload.error;
        isDownloading = false;
      })
    );

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

  async function start(): Promise<void> {
    error = null;
    isDownloading = true;
    try {
      await downloadModel();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      isDownloading = false;
    }
  }

  async function cancel(): Promise<void> {
    try {
      await cancelModelDownload();
    } catch {
      /* ignore cancel errors */
    }
    isDownloading = false;
    status = { state: 'not_downloaded' };
  }

  return {
    get status() {
      return status;
    },
    get isDownloading() {
      return isDownloading;
    },
    get error() {
      return error;
    },
    get progressPercent() {
      return progressPercent;
    },
    attach,
    start,
    cancel,
    refresh,
  };
}
