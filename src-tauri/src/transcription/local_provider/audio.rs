//! Audio decoding helpers for the local STT path.
//!
//! Converts browser-encoded WebM/Opus (or WAV) into 16 kHz mono `f32` PCM — the
//! input format Parakeet expects. Opus always decodes internally at 48 kHz, so
//! the WebM path uses a linear-interp resampler to drop to 16 kHz.

use base64::{engine::general_purpose, Engine as _};
use tauri::AppHandle;

use super::emit_log;

/// Resolve audio bytes from either a base64 payload or an on-disk path.
pub(super) async fn resolve_audio_bytes(
    audio_base64: Option<String>,
    audio_path: Option<String>,
) -> Result<Vec<u8>, String> {
    if let Some(path) = audio_path {
        tokio::fs::read(&path)
            .await
            .map_err(|e| format!("Failed to read audio file: {}", e))
    } else if let Some(b64) = audio_base64 {
        general_purpose::STANDARD
            .decode(&b64)
            .map_err(|e| format!("Failed to decode audio base64: {}", e))
    } else {
        Err("No audio provided".to_string())
    }
}

/// Decode audio bytes to 16 kHz mono f32 samples, dispatching by MIME type.
pub(super) fn decode_audio(
    app: &AppHandle,
    audio_bytes: &[u8],
    mime_type: &str,
) -> Result<Vec<f32>, String> {
    if mime_type.contains("wav") {
        decode_wav(audio_bytes)
    } else if mime_type.contains("webm") || mime_type.contains("ogg") || mime_type.contains("opus")
    {
        decode_webm_opus(app, audio_bytes)
    } else {
        Err(format!(
            "Unsupported audio format for local transcription: {}. \
             Supported: WebM/Opus, OGG/Opus, WAV.",
            mime_type
        ))
    }
}

/// Decode WebM/Opus audio to 16 kHz mono f32 PCM samples.
fn decode_webm_opus(app: &AppHandle, audio_bytes: &[u8]) -> Result<Vec<f32>, String> {
    use matroska_demuxer::{Frame, MatroskaFile, TrackType};
    use opus_rs::OpusDecoder;

    let cursor = std::io::Cursor::new(audio_bytes);
    let mut mkv =
        MatroskaFile::open(cursor).map_err(|e| format!("Failed to open WebM: {}", e))?;

    // Find the audio track
    let audio_info = mkv
        .tracks()
        .iter()
        .find(|t| t.track_type() == TrackType::Audio)
        .and_then(|t| {
            let audio = t.audio()?;
            let container_sr = audio.sampling_frequency() as u32;
            let ch = audio.channels().get() as usize;
            let track_num = t.track_number().get();
            Some((track_num, container_sr, ch))
        })
        .ok_or("No audio track found in WebM file")?;

    let (track_num, container_sr, channels) = audio_info;

    // Opus always decodes at 48kHz internally. The container's sampling_frequency
    // may report a lower value (e.g. 24kHz) but the decoder output is 48kHz.
    // Using the container rate for resampling would produce 2x too many samples,
    // stretching speech to half speed and making it unrecognizable.
    const OPUS_DECODE_RATE: u32 = 48000;
    emit_log(
        app,
        &format!(
            "WebM audio: track={}, container={}Hz, decode={}Hz, {}ch",
            track_num, container_sr, OPUS_DECODE_RATE, channels
        ),
    );

    let mut decoder = OpusDecoder::new(OPUS_DECODE_RATE as i32, channels)
        .map_err(|e| format!("Failed to create Opus decoder: {}", e))?;

    // Max Opus frame: 120 ms at 48 kHz stereo = 5760 * 2.
    let max_frame_samples = 5760 * channels;
    let mut decode_buf = vec![0.0f32; max_frame_samples];
    let mut all_samples: Vec<f32> = Vec::new();

    let mut frame_count: u32 = 0;
    let mut total_decoded: usize = 0;
    let mut frame = Frame::default();
    while mkv
        .next_frame(&mut frame)
        .map_err(|e| format!("Frame read error: {}", e))?
    {
        if frame.track != track_num {
            continue;
        }

        let decoded = decoder
            .decode(&frame.data, max_frame_samples / channels, &mut decode_buf)
            .map_err(|e| format!("Opus decode error: {}", e))?;

        if frame_count < 3 {
            emit_log(
                app,
                &format!(
                    "Opus frame {}: data={} bytes, decoded={} (channels={})",
                    frame_count,
                    frame.data.len(),
                    decoded,
                    channels
                ),
            );
        }
        frame_count += 1;
        total_decoded += decoded;

        if channels > 1 {
            for i in 0..decoded {
                let mut sum = 0.0f32;
                for ch in 0..channels {
                    sum += decode_buf[i * channels + ch];
                }
                all_samples.push(sum / channels as f32);
            }
        } else {
            all_samples.extend_from_slice(&decode_buf[..decoded]);
        }
    }

    emit_log(
        app,
        &format!(
            "WebM decode: {} frames, {} raw samples/ch at {}Hz ({}ch), {} mono samples before resample",
            frame_count, total_decoded, OPUS_DECODE_RATE, channels, all_samples.len()
        ),
    );

    let pre_resample = all_samples.len();
    if OPUS_DECODE_RATE != 16000 && !all_samples.is_empty() {
        all_samples = resample(&all_samples, OPUS_DECODE_RATE, 16000);
    }

    if all_samples.is_empty() {
        return Err("No audio samples decoded from WebM".to_string());
    }

    let rms = (all_samples.iter().map(|s| s * s).sum::<f32>() / all_samples.len() as f32).sqrt();
    emit_log(
        app,
        &format!(
            "Resampled: {} → {} samples (16kHz, {:.1}s), RMS={:.4}",
            pre_resample,
            all_samples.len(),
            all_samples.len() as f32 / 16000.0,
            rms
        ),
    );

    Ok(all_samples)
}

/// Decode WAV audio to 16 kHz mono f32 PCM samples.
fn decode_wav(audio_bytes: &[u8]) -> Result<Vec<f32>, String> {
    let tmp = std::env::temp_dir().join(format!("ah_local_{}.wav", std::process::id()));
    std::fs::write(&tmp, audio_bytes)
        .map_err(|e| format!("Failed to write temp WAV: {}", e))?;
    let samples = transcribe_rs::audio::read_wav_samples(&tmp)
        .map_err(|e| format!("Failed to read WAV: {}", e))?;
    let _ = std::fs::remove_file(&tmp);
    Ok(samples)
}

/// Simple linear-interpolation resampler (sufficient for speech audio).
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = (src_idx - idx as f64) as f32;
        let s0 = samples[idx.min(samples.len() - 1)];
        let s1 = samples[(idx + 1).min(samples.len() - 1)];
        out.push(s0 + frac * (s1 - s0));
    }
    out
}
