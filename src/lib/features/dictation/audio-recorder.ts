/**
 * Audio Recorder — MediaRecorder wrapper recording WebM/Opus into a Blob.
 * Browser WebM lacks duration metadata; we post-process with webm-fix-duration.
 */
import { log, logError } from '$lib/utils/logger';
import { webmFixDuration } from 'webm-fix-duration';
import { executeStartup } from './audio-startup';

export interface AudioRecorderOptions {
  mimeType?: string;
  audioBitsPerSecond?: number;
  noiseSuppression?: boolean;
  echoCancellation?: boolean;
  autoGainControl?: boolean;
  deviceId?: string;
}

interface RecordingResult {
  blob: Blob;
  mimeType: string;
  duration: number;
}

const DEFAULT_OPTIONS: AudioRecorderOptions = {
  mimeType: 'audio/webm;codecs=opus',
  audioBitsPerSecond: 128000,
  noiseSuppression: true,
  echoCancellation: true,
  autoGainControl: true,
};

/** Recording lifecycle states */
export type RecordingState = 'idle' | 'starting' | 'recording' | 'stopping';

/** Wraps MediaRecorder, returning a final Blob on stop(). */
export class AudioRecorder {
  private mediaRecorder: MediaRecorder | null = null;
  private chunks: Blob[] = [];
  private stream: MediaStream | null = null;
  private startTime: number = 0;
  private options: AudioRecorderOptions;
  private _state: RecordingState = 'idle';
  private startPromise: Promise<void> | null = null;
  private abortController: AbortController | null = null;
  // Signal resolved when the recorder returns to 'idle' after a stop(),
  // letting a concurrent start() wait instead of sleeping on a timer.
  private idleSignal: Promise<void> | null = null;
  private resolveIdle: (() => void) | null = null;

  constructor(options: AudioRecorderOptions = {}) {
    this.options = { ...DEFAULT_OPTIONS, ...options };
  }

  static isSupported(): boolean {
    if (typeof navigator === 'undefined' || typeof window === 'undefined') {
      return false;
    }
    const hasGetUserMedia = navigator.mediaDevices && 'getUserMedia' in navigator.mediaDevices;
    const hasMediaRecorder = 'MediaRecorder' in window;
    return hasGetUserMedia && hasMediaRecorder;
  }

  updateOptions(options: Partial<AudioRecorderOptions>): void {
    this.options = { ...this.options, ...options };
  }

  get state(): RecordingState {
    return this._state;
  }

  async waitForStart(): Promise<boolean> {
    if (this.startPromise) {
      try {
        await this.startPromise;
        return true;
      } catch {
        return false;
      }
    }
    return false;
  }

  /**
   * Start recording. Throws if another start() is already in flight; waits
   * (no fixed sleep) for an in-flight stop() to drain back to idle.
   */
  async start(): Promise<void> {
    if (this._state !== 'idle') {
      await log(`[AudioRecorder] Cannot start - current state: ${this._state}`);
      if (this._state === 'starting') {
        // Surface re-entry to caller instead of silently no-oping.
        throw new Error('audio-recorder: already starting');
      }
      if (this._state === 'recording') {
        await log('[AudioRecorder] Already recording');
        return;
      }
      if (this._state === 'stopping') {
        // Await the onstop handler to set state back to idle; no timer poll.
        if (!this.idleSignal) {
          this.idleSignal = new Promise<void>((res) => {
            this.resolveIdle = res;
          });
        }
        await this.idleSignal;
        if (this._state as RecordingState !== 'idle') {
          throw new Error('Recorder busy');
        }
      }
    }

    this._state = 'starting';
    this.abortController = new AbortController();
    const signal = this.abortController.signal;

    await log('[AudioRecorder] Starting...');
    await log(`[AudioRecorder] Options: deviceId=${this.options.deviceId || 'default'}, ` +
      `noiseSuppression=${this.options.noiseSuppression}, ` +
      `echoCancellation=${this.options.echoCancellation}, ` +
      `autoGainControl=${this.options.autoGainControl}`);

    this.startPromise = this.doStart(signal);

    try {
      await this.startPromise;
    } finally {
      this.startPromise = null;
      this.abortController = null;
    }
  }

  /** Internal start; delegates to audio-startup.ts. */
  private async doStart(signal: AbortSignal): Promise<void> {
    try {
      await executeStartup(signal, {
        options: this.options,
        stream: this.stream,
        mediaRecorder: this.mediaRecorder,
        chunks: this.chunks,
        startTime: this.startTime,
        _state: this._state,
        setStream: (s) => { this.stream = s; },
        setMediaRecorder: (r) => { this.mediaRecorder = r; },
        setChunks: (c) => { this.chunks = c; },
        setStartTime: (t) => { this.startTime = t; },
        setState: (s) => { this._state = s; },
        pushChunk: (c) => { this.chunks.push(c); },
      });
    } catch (error) {
      this.cleanup();
      this._state = 'idle';
      this.signalIdle();
      throw error;
    }
  }

  /** Stop recording and return the final audio blob. */
  async stop(): Promise<RecordingResult> {
    await log(`[AudioRecorder] stop() called, current state: ${this._state}`);

    if (this._state === 'starting') {
      await log('[AudioRecorder] Cancelling startup...');
      if (this.abortController) {
        this.abortController.abort();
      }
      if (this.startPromise) {
        try {
          await this.startPromise;
        } catch {
          await log('[AudioRecorder] Startup cancelled');
        }
      }
      this.cleanup();
      this._state = 'idle';
      throw new Error('Recording cancelled during startup');
    }

    if (this._state !== 'recording') {
      await log(`[AudioRecorder] Cannot stop - state is ${this._state}, not 'recording'`);
      throw new Error('Not recording');
    }

    if (!this.mediaRecorder || this.mediaRecorder.state !== 'recording') {
      await log('[AudioRecorder] MediaRecorder not in recording state');
      this._state = 'idle';
      throw new Error('Not recording');
    }

    this._state = 'stopping';
    // Arm idleSignal so a concurrent start() awaits onstop, not a timer.
    this.idleSignal = new Promise<void>((res) => {
      this.resolveIdle = res;
    });

    return new Promise((resolve, reject) => {
      log('[AudioRecorder] Stopping...');

      this.mediaRecorder!.onstop = async () => {
        const duration = Date.now() - this.startTime;
        const mimeType = this.mediaRecorder?.mimeType || 'audio/webm';
        const rawBlob = new Blob(this.chunks, { type: mimeType });

        this.cleanup();
        this._state = 'idle';
        this.signalIdle();

        await log(`[AudioRecorder] Raw blob created. Duration: ${duration}ms, Size: ${rawBlob.size} bytes`);

        let blob = rawBlob;
        try {
          if (mimeType.includes('webm')) {
            await log('[AudioRecorder] Fixing WebM duration metadata...');
            blob = await webmFixDuration(rawBlob, duration, mimeType);
            await log(`[AudioRecorder] WebM duration fixed. New size: ${blob.size} bytes`);
          }
        } catch (fixError) {
          await logError('[AudioRecorder] Failed to fix WebM duration, using raw blob', fixError);
        }

        await log(`[AudioRecorder] Stopped. Duration: ${duration}ms, Size: ${blob.size} bytes`);

        resolve({ blob, mimeType, duration });
      };

      this.mediaRecorder!.onerror = () => {
        this.cleanup();
        this._state = 'idle';
        this.signalIdle();
        reject(new Error('Recording error'));
      };

      this.mediaRecorder!.stop();
    });
  }

  /** Release any pending idle waiters. */
  private signalIdle(): void {
    if (this.resolveIdle) {
      this.resolveIdle();
      this.resolveIdle = null;
      this.idleSignal = null;
    }
  }

  cancel(): void {
    log(`[AudioRecorder] Cancelling... (state: ${this._state})`);

    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }

    if (this.mediaRecorder && this.mediaRecorder.state === 'recording') {
      this.mediaRecorder.stop();
    }

    this.cleanup();
    this._state = 'idle';
    this.startPromise = null;
    this.signalIdle();
  }

  get isRecording(): boolean {
    return this._state === 'recording';
  }

  get duration(): number {
    if (!this.startTime || !this.isRecording) return 0;
    return Date.now() - this.startTime;
  }

  private cleanup(): void {
    if (this.mediaRecorder) {
      this.mediaRecorder.ondataavailable = null;
      this.mediaRecorder.onstop = null;
      this.mediaRecorder.onerror = null;
      this.mediaRecorder.onstart = null;
    }
    if (this.stream) {
      this.stream.getTracks().forEach((track) => track.stop());
      this.stream = null;
    }
    this.mediaRecorder = null;
    this.chunks = [];
    this.startTime = 0;
  }
}

let recorderInstance: AudioRecorder | null = null;

/** Get or create the audio recorder singleton. */
export function getAudioRecorder(options?: AudioRecorderOptions): AudioRecorder {
  if (!recorderInstance) {
    recorderInstance = new AudioRecorder(options);
  } else if (options) {
    recorderInstance.updateOptions(options);
  }
  return recorderInstance;
}
