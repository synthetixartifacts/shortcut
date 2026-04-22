/**
 * WAV encoder — convert a browser-encoded Opus/WebM blob into 16 kHz mono WAV.
 *
 * The pure-Rust Opus decoder in our local STT path cannot decode browser-encoded
 * Opus frames (it produces silence). The Web Audio API decodes them correctly,
 * and the backend's WAV path works reliably — so we resample + rewrap on the
 * frontend before hand-off.
 *
 * Extracted from `dictation-controller.ts` in PHASE 3A. Consumers migrate to
 * this module in PHASE 3B.
 */

/** Parakeet and most local STT engines expect 16 kHz mono audio. */
const TARGET_SAMPLE_RATE = 16_000;

/**
 * Convert an audio blob (WebM/Opus or similar) into a 16 kHz mono 16-bit PCM
 * WAV blob suitable for the backend's local STT engine.
 *
 * @param blob       The source audio (any format the browser can decode).
 * @param sampleRate Target sample rate in Hz. Defaults to 16 000.
 */
export async function convertToWav(
  blob: Blob,
  sampleRate: number = TARGET_SAMPLE_RATE
): Promise<Blob> {
  const audioContext = new AudioContext();
  try {
    const arrayBuffer = await blob.arrayBuffer();
    const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);

    // Resample to the target rate as mono using OfflineAudioContext.
    const duration = audioBuffer.duration;
    const offlineCtx = new OfflineAudioContext(
      1,
      Math.ceil(duration * sampleRate),
      sampleRate
    );
    const source = offlineCtx.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(offlineCtx.destination);
    source.start(0);
    const resampled = await offlineCtx.startRendering();

    const samples = resampled.getChannelData(0);
    return encodePcmWav(samples, sampleRate);
  } finally {
    await audioContext.close();
  }
}

/**
 * Encode a Float32 PCM buffer as a 16-bit mono WAV blob.
 * Exposed for tests; call sites should prefer {@link convertToWav}.
 */
export function encodePcmWav(samples: Float32Array, sampleRate: number): Blob {
  const numSamples = samples.length;
  const buffer = new ArrayBuffer(44 + numSamples * 2);
  const view = new DataView(buffer);

  // RIFF header
  writeAscii(view, 0, 'RIFF');
  view.setUint32(4, 36 + numSamples * 2, true);
  writeAscii(view, 8, 'WAVE');
  // fmt chunk
  writeAscii(view, 12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  // data chunk
  writeAscii(view, 36, 'data');
  view.setUint32(40, numSamples * 2, true);

  for (let i = 0; i < numSamples; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(44 + i * 2, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }

  return new Blob([buffer], { type: 'audio/wav' });
}

function writeAscii(view: DataView, offset: number, str: string): void {
  for (let i = 0; i < str.length; i++) {
    view.setUint8(offset + i, str.charCodeAt(i));
  }
}
