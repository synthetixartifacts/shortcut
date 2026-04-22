# Local Speech-to-Text (Local STT)

## Overview

Local STT adds on-device transcription to ShortCut as an alternative to the cloud-based Soniox engine. Audio never leaves the user's machine — all processing happens locally using the NVIDIA Parakeet TDT 0.6B v3 model via ONNX Runtime.

**Why it exists**: Privacy-sensitive users and offline environments need transcription without uploading audio to external servers.

**Platform support**: Windows only (Phase 1). macOS shows "Coming soon" in the UI.

## Architecture

### Provider Dispatch

The frontend calls a single `transcribe_audio` command. The Rust backend routes to the active engine:

```
Frontend                              Rust Backend
--------                              ------------
invoke('transcribe_audio')
  (same args regardless of engine)
                                      transcription/mod.rs
                                        read active_engine from config
                                        |
                                        +-- "soniox" (default)
                                        |     soniox_provider.rs
                                        |       5-step Soniox API (direct)
                                        |
                                        +-- "local-windows"
                                        |     local_provider.rs
                                        |       WebM/Opus -> PCM -> Parakeet ONNX
                                        |
                                        +-- "local-macos" (stub, returns error)
                                      
                                      <- TranscriptionData (same shape)
```

The frontend audio capture path (MediaRecorder, audio-recorder.ts) is completely unchanged. The provider abstraction lives entirely in the Rust backend.

### Model Lifecycle

```
Not Downloaded --> Downloading --> Ready
     ^                |              |
     |                v              v
     +---- Cancel     |         Delete
     +---- Error <----+
     |
     +---- Corrupt (detected on status check)
```

Model files are stored in the app data directory:
- Windows: `%APPDATA%\com.g-prompter.shortcut\models\parakeet-tdt-0.6b-v3\`

## Model Information

| Property | Value |
|----------|-------|
| Name | NVIDIA Parakeet TDT 0.6B v3 |
| Quantization | INT8 ONNX |
| Total download size | ~670 MB |
| License | CC-BY-4.0 |
| Source | HuggingFace (`istupakov/parakeet-tdt-0.6b-v3-onnx`) |
| Inference crate | `transcribe-rs` (with `onnx` feature) |

### Model Files

| File | Size |
|------|------|
| `encoder-model.int8.onnx` | ~652 MB |
| `decoder_joint-model.int8.onnx` | ~18 MB |
| `nemo128.onnx` | ~140 KB |
| `vocab.txt` | ~94 KB |

### Download Integrity

Each file in `MODEL_FILES` carries an `expected_sha256: &'static str` column. During download the file is streamed to a `.tmp` path while a rolling `Sha256` hash is updated in-place (no full re-read). On completion:
- If `expected_sha256` is non-empty, the computed hex digest is compared; a mismatch cleans up the tmp file and returns `AppError::Io`.
- If `expected_sha256` is empty, the integrity path is a no-op (logged at debug). Hash strings are currently empty for Parakeet files while we wire in an upstream-trusted manifest; the verification plumbing is live so flipping a hash on in the future requires no code changes.

Size sanity (`expected_size_bytes`) is also checked against `Content-Length` when the server provides it.

## User Guide

### Switching to Local Engine

1. Open **Actions > Dictation**
2. In the **Transcription Engine** section, find the "Local (Windows)" card
3. Click **Download model** (if not yet downloaded) -- this downloads ~670 MB
4. Once downloaded, click **Make active**

### Onboarding (New Users)

New users see an onboarding screen on first launch offering:
- A provider setup step for LLM keys
- **Cloud (Soniox)**: Select the Soniox card, enter the API key inside that card, then continue
- **Local (Windows)**: Select the local card, use the inline **Download model** button on the same screen, then continue once the model is ready
- **Decide later**: Defaults to Soniox and skips to the dashboard

### Deleting the Model

1. Switch to a different engine first (e.g., Cloud)
2. In **App Settings**, the model can be deleted to free ~670 MB

## Supported Languages and Capabilities

| Capability | Cloud (Soniox) | Local (Windows) |
|------------|---------------|-----------------|
| Custom terms/vocabulary | Yes | No |
| Background context text | Yes | No |
| Translation mode | Yes | No |
| Requires internet | Yes | No |
| Requires Soniox API key | Yes | No |
| Audio leaves device | Yes | No |
| Model download required | No | Yes (~670 MB) |
| Language coverage | Broad (Soniox) | English-primary (Parakeet) |

When the local engine is active, unsupported features (custom terms, background text, translation) are visually disabled in the dictation settings UI with explanatory messages.

## Privacy Guarantees

When using the local engine:
- Audio is decoded and transcribed entirely on-device
- No network requests are made during transcription
- No telemetry or analytics are sent
- The model runs via ONNX Runtime in a blocking thread pool

## Hardware Recommendations

- **RAM**: 4 GB available (model loads into memory per-transcription)
- **Disk**: ~700 MB free for model storage
- **CPU**: Modern x86_64 processor (ONNX Runtime uses CPU; no GPU required)

Slowness detection: if transcription takes longer than the audio duration (RTF > 1.0), the app shows a one-time warning suggesting the user switch to cloud.

## File Structure

### Backend (Rust)

```
src-tauri/src/transcription/
├── mod.rs                    # Dispatch command, engine routing, model command stubs
├── soniox_provider.rs        # Soniox direct API (5-step flow)
├── soniox_api/               # Soniox HTTP operations (split)
│   ├── mod.rs                #   upload/create/poll/fetch/delete
│   └── types.rs              #   request/response types
├── utils.rs                  # Audio decode helpers
├── local_provider/           # Local transcription via transcribe-rs (local-stt feature, split)
│   ├── mod.rs                #   public API (transcribe entry point)
│   ├── audio.rs              #   WebM/Opus decode + 16kHz resample
│   ├── ort_init.rs           #   ORT_DYLIB_PATH one-shot init (unsafe-wrapped via OnceLock)
│   └── transcribe.rs         #   Parakeet ONNX inference
└── model_manager.rs          # Model download (streaming SHA256) / verify / delete (local-stt feature)
```

### Frontend

```
src/lib/state/engine.svelte.ts              # Engine state, capabilities, model status
src/lib/components/dictation/
├── EngineCard.svelte                        # Individual engine card
├── EngineSelector.svelte                    # Engine selection container
└── ModelDownload.svelte                     # Model download progress UI
src/routes/onboarding/+page.svelte           # First-run engine choice
```

## Configuration

### TranscriptionConfig (in `config.json`)

```json
{
  "transcription": {
    "active_engine": "soniox",
    "first_run_completed": false,
    "slowness_dismissed": false
  }
}
```

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `active_engine` | `string` | `"soniox"` | Active engine ID: `"soniox"` or `"local-windows"` |
| `first_run_completed` | `bool` | `false` | Has user completed onboarding |
| `slowness_dismissed` | `bool` | `false` | User dismissed slowness warning |

Uses `#[serde(default)]` for backwards compatibility — unknown engine IDs fall back to `"soniox"`.

### History Engine Field

`HistoryEntry.engine` is `Option<String>`:
- `None` or absent: old entries, displayed without engine badge
- `"soniox"`: cloud transcription (Soniox direct)
- `"local-windows"`: local transcription

Displayed as "Cloud" or "Local" badge on history items.

## Developer Guide

### Building With/Without Local STT

```bash
# Default build (no local STT, smaller binary)
cargo build

# Build with local STT support
cargo build --features local-stt
```

The `local-stt` Cargo feature gates:
- `transcribe-rs` (with `onnx` feature)
- `matroska-demuxer` (WebM container parsing)
- `opus-rs` (Opus audio decoding)

Without the feature, model management commands return `{ "state": "unavailable" }` and the local engine dispatch returns an error message. The frontend handles this gracefully.

### Tauri Commands

| Command | Feature | Description |
|---------|---------|-------------|
| `transcribe_audio` | always | Dispatch to active engine |
| `get_active_engine` | always | Read active engine from config |
| `set_active_engine` | always | Set active engine in config |
| `update_transcription_config` | always | Update transcription config |
| `get_model_status` | stub (always) | Model status (returns `unavailable` without feature) |
| `download_model` | stub (always) | Start model download |
| `delete_model` | stub (always) | Delete model files |
| `cancel_model_download` | stub (always) | Cancel in-progress download |

All model management commands are always compiled as stubs so `generate_handler!` works. When `local-stt` is disabled, they return informative errors.

### Audio Decoding Pipeline (Local Engine)

```
WebM/Opus input
  -> matroska-demuxer: extract Opus frames
  -> opus-rs: decode to f32 PCM (native sample rate)
  -> resample to 16kHz mono (linear interpolation)
  -> transcribe-rs: Parakeet ONNX inference
  -> text output
```

WAV input is also supported via `transcribe-rs::audio::read_wav_samples`.

## Known Limitations

1. **No custom terms**: Parakeet does not support vocabulary biasing
2. **No translation mode**: Local engine produces text in the detected language only
3. **No background context**: No context-aware recognition
4. **English-primary**: Best accuracy for English; other languages depend on Parakeet's training data
5. **Model loaded per-transcription**: No persistent model caching yet (future optimization)
6. **Windows only**: macOS local engine is not yet implemented

## Troubleshooting

| Symptom | Cause | Solution |
|---------|-------|----------|
| "Local model not downloaded" | Model files missing | Open Dictation settings, download model |
| "Model file appears corrupted" | Encoder file < 100 MB | Delete and re-download from Settings |
| "Missing model file: X" | Incomplete download | Delete and re-download |
| Slow transcription | Weak CPU or large audio | Dismiss warning or switch to Cloud |
| "Unsupported audio format" | Not WebM/OGG/WAV | Should not happen with default recording; report bug |
| "Local STT not available in this build" | Built without `local-stt` feature | Rebuild with `--features local-stt` |

## Attribution

The local engine uses the NVIDIA Parakeet TDT 0.6B v3 model, licensed under CC-BY-4.0. Attribution is displayed in **App Settings > Credits** on Windows.
