import { writeFile } from '@tauri-apps/plugin-fs';
import { join, tempDir } from '@tauri-apps/api/path';
import { invokeWithErrorHandling } from './core';
import { log } from '$lib/utils/logger';
import type { TranscriptionConfig, TranscriptionResult, ModelStatus } from '$lib/types';

/**
 * Transcribe audio via the active transcription engine.
 * Persists blob to a temp file to avoid IPC size limits.
 */
export async function transcribeAudio(
  audioBlob: Blob,
  mimeType: string,
  languageHints?: string[],
  contextTerms?: string[],
  contextText?: string
): Promise<TranscriptionResult> {
  const arrayBuffer = await audioBlob.arrayBuffer();
  const bytes = new Uint8Array(arrayBuffer);

  const tmpDir = await tempDir();
  const extension = mimeType.includes('wav')
    ? 'wav'
    : mimeType.includes('mp3')
      ? 'mp3'
      : mimeType.includes('ogg') || mimeType.includes('opus')
        ? 'ogg'
        : mimeType.includes('m4a') || mimeType.includes('mp4')
          ? 'm4a'
          : 'webm';

  const fileName = `dictation-${crypto.randomUUID ? crypto.randomUUID() : Date.now()}-${Math.floor(Math.random() * 1e6)}.${extension}`;
  const filePath = await join(tmpDir, fileName);

  await writeFile(filePath, bytes);
  await log(`[transcribeAudio] Wrote ${bytes.length} bytes to ${filePath}`);

  await log('[transcribeAudio] Invoking transcribe_audio...');
  const result = await invokeWithErrorHandling<TranscriptionResult>('transcribe_audio', {
    audioPath: filePath,
    mimeType,
    languageHints: languageHints || [],
    contextTerms: contextTerms || [],
    contextText: contextText || null,
  });
  await log(`[transcribeAudio] Result: ${result.text.length} chars, engine=${result.engine}`);
  return result;
}

export async function getActiveEngine(): Promise<string> {
  return invokeWithErrorHandling<string>('get_active_engine');
}

export async function setActiveEngine(engine: string): Promise<void> {
  await invokeWithErrorHandling<void>('set_active_engine', { engine });
}

export async function updateTranscriptionConfig(transcription: TranscriptionConfig): Promise<void> {
  await invokeWithErrorHandling<void>('update_transcription_config', { transcription });
}

/**
 * Get the status of the local STT model.
 * Returns { state: "unavailable" } if local-stt feature is not enabled.
 */
export async function getModelStatus(): Promise<ModelStatus> {
  return invokeWithErrorHandling<ModelStatus>('get_model_status');
}

/**
 * Start downloading the local STT model.
 * Listen for 'model-download-progress' events for progress updates.
 */
export async function downloadModel(): Promise<void> {
  await invokeWithErrorHandling<void>('download_model');
}

export async function deleteModel(): Promise<void> {
  await invokeWithErrorHandling<void>('delete_model');
}

export async function cancelModelDownload(): Promise<void> {
  await invokeWithErrorHandling<void>('cancel_model_download');
}
