# Dictation Feature - Audio Recording System

## Overview

Voice-to-text dictation using hold-to-talk pattern. Records audio locally via MediaRecorder API and sends to the active transcription engine for processing.

**Pattern**: Hold shortcut to record, release to transcribe and paste

**Engines**: ShortCut supports pluggable transcription engines. The frontend audio capture path is identical regardless of engine — the backend dispatches to the active provider. See [LOCAL_STT.md](./LOCAL_STT.md) for the local engine details.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                                USER INTERACTION                                  │
│  Press Alt+D ──► Recording starts ──► Release Alt+D ──► Text pasted  │
└───────────┬──────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           RUST BACKEND (Tauri)                             │
│                                                                            │
│  hotkeys/registration.rs                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  Global Shortcut Handler                                            │  │
│  │  - Listens for ShortcutState::Pressed  → emit "dictation_start"     │  │
│  │  - Listens for ShortcutState::Released → emit "dictation_stop"      │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└───────────┬────────────────────────────────────────────────────────────────┘
            │ Tauri Event: "shortcut-triggered"
            ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                         FRONTEND (Svelte/TypeScript)                       │
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  ShortcutDispatcher (shortcut-dispatcher.ts)                        │  │
│  │  - Routes "dictation_start" → DictationController.startRecording()  │  │
│  │  - Routes "dictation_stop"  → DictationController.stopRecording()   │  │
│  └──────────────────────────────┬──────────────────────────────────────┘  │
│                                 │                                          │
│  ┌──────────────────────────────▼──────────────────────────────────────┐  │
│  │  DictationController (dictation-controller.ts)                      │  │
│  │  - Manages recording lifecycle                                      │  │
│  │  - Updates app state and activity indicator                         │  │
│  │  - Sends audio to active STT engine for transcription               │  │
│  └──────────────────────────────┬──────────────────────────────────────┘  │
│                                 │                                          │
│  ┌──────────────────────────────▼──────────────────────────────────────┐  │
│  │  AudioRecorder (audio-recorder.ts)                                  │  │
│  │  - State machine: idle → starting → recording → stopping → idle     │  │
│  │  - Uses MediaRecorder API for audio capture                         │  │
│  │  - Manages microphone stream acquisition                            │  │
│  │  - Fixes WebM duration metadata                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────┘
            │
            │ Audio blob (base64)
            ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           RUST BACKEND (Tauri)                             │
│                                                                            │
│  transcription/mod.rs: transcribe_audio()                                 │
│  - Reads active_engine from config                                        │
│  - Dispatches to active provider:                                         │
│                                                                            │
│  ┌─ "soniox" ──► soniox_provider.rs ──────────────────────────────────┐   │
│  │   Reads Soniox API key + base URL from config                      │   │
│  │   5-step: upload → create → poll → fetch → cleanup                 │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌─ "local-windows" ──► local_provider.rs (local-stt feature) ───────┐   │
│  │   Decodes WebM/Opus → 16kHz PCM                                   │   │
│  │   Runs Parakeet ONNX model via transcribe-rs                      │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Returns: TranscriptionData { text, duration_ms, language, engine }       │
└───────────────────────────────────────────────────────────────────────────┘
```

## Transcription Engine Selection

ShortCut supports multiple transcription engines behind a provider abstraction. The frontend is engine-agnostic — it calls the same `transcribe_audio` command regardless of which engine is active. The Rust backend dispatches to the appropriate provider.

### What Stays the Same (All Engines)
- Frontend audio capture (MediaRecorder API)
- Activity indicator flow (preparing, recording, processing)
- Clipboard paste of transcribed text
- History entry creation (includes `engine` field)

### What Differs Per Engine
- **Soniox (cloud)**: Sends audio to Soniox API directly (5-step flow), supports custom terms, translation, background text. The dictation page only shows Custom Vocabulary, Context, and Languages when Soniox is the active engine. Requires Soniox API key.
- **Local (Windows)**: Processes audio on-device via Parakeet ONNX, no network required, fewer features. No API key needed.

### Engine State
Engine selection is managed by `state/engine.svelte.ts` with active engine stored in `config.json` under `transcription.active_engine`. Engine IDs: `"soniox"` (default), `"local-windows"`. See [LOCAL_STT.md](./LOCAL_STT.md) for full details.

## State Management

### Recording State Machine (AudioRecorder)

```
     ┌─────────┐
     │  idle   │◄────────────────────────────┐
     └────┬────┘                             │
          │ start()                          │
          ▼                                  │
     ┌─────────┐                             │
     │starting │ (waiting for MediaRecorder  │
     └────┬────┘  onstart event)             │
          │                                  │
          │ onstart fires                    │
          ▼                                  │
     ┌─────────┐                             │
     │recording│                             │
     └────┬────┘                             │
          │ stop()                           │
          ▼                                  │
     ┌─────────┐                             │
     │stopping │ (waiting for onstop event)  │
     └────┬────┘                             │
          │ onstop fires + cleanup           │
          └──────────────────────────────────┘
```

### Three State Systems

| System | File | Purpose |
|--------|------|---------|
| App State | `app.svelte.ts` | `isRecording`, `status`, `lastTranscription` |
| Activity State | `activity.svelte.ts` | Indicator: `preparing`, `active`, `success`, `error` |
| Recorder State | `audio-recorder.ts` | Internal: `idle`, `starting`, `recording`, `stopping` |

**Critical**: All three must be kept in sync. The `resetState()` method in DictationController handles this.

## Audio Recording Details

### MediaRecorder Configuration

```typescript
const DEFAULT_OPTIONS = {
  mimeType: 'audio/webm;codecs=opus',
  audioBitsPerSecond: 128000,
  noiseSuppression: true,
  echoCancellation: true,
  autoGainControl: true,
};
```

### MIME Type Priority

Tried in order until one is supported:
1. `audio/webm;codecs=opus` (preferred - best compression)
2. `audio/webm`
3. `audio/ogg;codecs=opus`
4. `audio/mp4`
5. Browser default

### Audio Constraints

When a specific microphone is selected:
```typescript
{
  deviceId: { exact: selectedDeviceId },
  noiseSuppression: true,
  echoCancellation: true,
  autoGainControl: true
}
```

The `exact` constraint ensures the selected device is used or an error is thrown (no silent fallback).

## Platform Differences

### WebView Engines

| Platform | WebView | Engine | MediaRecorder Support |
|----------|---------|--------|----------------------|
| Windows | WebView2 | Chromium/Edge | Full (WebM/Opus) |
| macOS | WKWebView | WebKit/Safari | Full since Safari 14.1 |
| Linux | WebKitGTK | WebKit | Varies by distribution |

### Key Platform-Specific Behaviors

#### Windows (WebView2/Chromium)
- WebM container sets `MuxingApp: Chrome`
- Full `audio/webm;codecs=opus` support
- `getUserMedia` permission prompt handled by OS settings

#### macOS (WKWebView/WebKit)
- WebM container sets `MuxingApp: WebKit`
- Frame duration fixed at 2.5ms (no control available)
- Requires microphone permission in System Settings > Privacy & Security
- Output may differ slightly from Windows
- **getUserMedia visibility requirement**: WebKit requires the WKWebView to be visible for `getUserMedia()` to resolve. The app includes a visibility guard that auto-shows the main window on macOS when needed (see "macOS Considerations" section below)

### WebM Duration Fix

Browser-recorded WebM files lack duration metadata, which causes issues with some transcription services. We use `webm-fix-duration` library to patch the duration:

```typescript
import { webmFixDuration } from 'webm-fix-duration';
blob = await webmFixDuration(rawBlob, durationMs, mimeType);
```

## macOS Considerations

### WebKit Visibility Requirement

WebKit blocks `getUserMedia()` from non-visible WKWebView documents. When AH is hidden to the system tray (the normal state for a tray app), the WKWebView document becomes non-visible, and `getUserMedia()` will hang indefinitely instead of resolving or rejecting.

This is addressed with a three-layer defense:

#### 1. Visibility Guard (`audio-startup.ts`)

Before calling `getUserMedia()` on macOS, the startup sequence checks `document.visibilityState`. If the document is not visible:
- Calls `showMainWindow()` (Tauri command) to show and focus the main window
- Waits up to 3 seconds for the `visibilitychange` event via `waitForVisible()`
- If visibility is confirmed, proceeds normally
- If visibility times out, proceeds anyway (the stream timeout will catch failures)

The guard only runs on macOS (detected via `isMacOS()` user-agent check) and only when the document is not visible. Windows/Linux are unaffected.

#### 2. Stream Acquisition Timeout (`audio-startup.ts`)

`acquireStream()` wraps every `getUserMedia()` call with an 8-second `Promise.race` timeout (`STREAM_ACQUIRE_TIMEOUT_MS`). If `getUserMedia()` hangs (e.g., visibility guard didn't help), the timeout rejects with a clear error message instead of hanging forever.

Both the primary `getUserMedia()` call and the `OverconstrainedError` fallback are wrapped with the same timeout. The timeout timer is cleaned up via `.finally(() => clearTimeout(timeoutId))`.

#### 3. Mic Permission Pre-Grant (`dictation-controller.ts`)

On app startup, `DictationController.initialize()` calls `getUserMedia({ audio: true })` once while the window is naturally visible (the app window is visible when first launched). This pre-grants WebKit-level microphone permission for the session, so subsequent calls from a hidden window are more likely to succeed.

- Only runs when `document.visibilityState === 'visible'`
- Immediately stops the acquired tracks (no persistent mic access)
- Failure is non-fatal (logs a warning, continues initialization)
- Runs on all platforms (safe and beneficial everywhere)

### MIME Type Behavior

On macOS < 15.4, `audio/webm` is not supported by WebKit. The MIME type fallback in `getSupportedMimeType()` (`audio-helpers.ts`) returns an empty string `''` when no preferred type is supported, which lets `MediaRecorder` choose its own default. The `startMediaRecorder()` function handles this by conditionally including `mimeType` in the recorder options only when it's truthy, avoiding an unnecessary throw-and-catch.

Typical results:
- **macOS < 15.4**: `audio/mp4` (from the fallback list)
- **macOS >= 15.4**: `audio/webm;codecs=opus` (WebM support added)
- **Windows**: `audio/webm;codecs=opus`

### Diagnostic Logging

Stream acquisition logs `document.visibilityState` before the attempt and elapsed time after success, aiding debugging of visibility-related issues. Log entries:
- `[AudioRecorder] Acquiring stream, visibilityState=...`
- `[AudioRecorder] Stream acquired in Xms`
- `[AudioRecorder] macOS: document not visible, requesting window focus` (when visibility guard triggers)

### Tauri Command

The `show_main_window` command in `lib.rs` shows and focuses the main window. It is the complement to the existing `hide_main_window` command. Used by the macOS visibility guard to surface the window before `getUserMedia()`.

## Known Issues & Edge Cases

### 1. Race Condition: Quick Press-Release

**Symptom**: "Failed to stop recording: Not recording" error

**Cause**: User releases shortcut before `MediaRecorder.onstart` fires

**Solution**: AudioRecorder now waits for start to complete before stopping:
```typescript
if (this._state === 'starting' && this.startPromise) {
  await this.startPromise; // Wait for start to complete
}
```

### 2. Indicator Stuck on "Preparing"

**Symptom**: Indicator shows "Preparing..." forever after error

**Cause**: `resetRecordingState()` didn't call `endActivity()`

**Solution**: `resetState()` now ends activity if still in preparing/active state:
```typescript
if (activityState.state === 'preparing' || activityState.state === 'active') {
  await endActivity();
}
```

### 3. Stale Audio from Previous Session

**Symptom**: New recording contains audio from previous session

**Cause**: Event handlers not cleared, singleton instance retains state

**Solution**: Cleanup now removes all event handlers:
```typescript
cleanup(): void {
  if (this.mediaRecorder) {
    this.mediaRecorder.ondataavailable = null;
    this.mediaRecorder.onstop = null;
    this.mediaRecorder.onerror = null;
    this.mediaRecorder.onstart = null;
  }
  // ... stop streams and reset state
}
```

### 4. Microphone Selection Issues

**Symptom**: Wrong microphone used or silent fallback to default

**Cause**: Browser may silently select different device if constraints can't be met

**Solution**: Use `exact` constraint and validate device exists before requesting:
```typescript
if (this.options.deviceId) {
  const devices = await navigator.mediaDevices.enumerateDevices();
  const targetDevice = audioInputs.find(d => d.deviceId === this.options.deviceId);
  if (!targetDevice) {
    throw new Error('Selected microphone not found');
  }
  audioConstraints.deviceId = { exact: this.options.deviceId };
}
```

### 5. Very Short Recordings (No Audio)

**Symptom**: "No speech detected" with very small blob size

**Cause**: User released shortcut immediately or microphone issue

**Detection**: Blob size < 1000 bytes
```typescript
if (result.blob.size < 1000) {
  await indicatorError('No speech');
  return;
}
```

### 6. Permission Denied

**Symptom**: "Could not access the selected microphone" error

**Platform-specific recovery**:
- **Windows**: Settings > Privacy > Microphone
- **macOS**: System Settings > Privacy & Security > Microphone

### 7. macOS: Stuck on "Preparing..." (WebKit Visibility Requirement)

**Symptom**: Dictation stuck on "Preparing..." forever on macOS when AH window is hidden

**Cause**: WebKit blocks `getUserMedia()` from non-visible WKWebView documents. The promise never resolves or rejects.

**Solution**: Three-layer defense (see "macOS Considerations" section above):
1. **Visibility guard**: Auto-shows main window before mic access
2. **8-second timeout**: Fails with clear error instead of hanging forever
3. **Permission pre-grant**: Requests mic at startup while window is visible

**Platform note**: Windows/Linux are unaffected -- `getUserMedia()` works regardless of window visibility.

### 8. macOS: Mic Permission Not Persisted Across Restarts

**Symptom**: First dictation after app launch may be slower or require the visibility guard

**Cause**: WebKit does not persist WKWebView-level microphone permission across app restarts. Each launch requires a new `getUserMedia()` grant.

**Mitigation**: The mic pre-grant in `DictationController.initialize()` handles this at startup. If the pre-grant fails (e.g., window hidden at startup time), the visibility guard handles it on first dictation attempt.

## Potential Issues to Watch

### 1. Concurrent Recording Attempts

While we have state machine protection, rapid shortcut pressing could theoretically cause issues. The `_state !== 'idle'` check should prevent this.

### 2. Memory Leaks

The singleton pattern means the AudioRecorder persists for the app lifetime. Ensure:
- Streams are always stopped
- Event handlers are always cleared
- Chunks array is always emptied

### 3. Long Recording Sessions

For very long recordings:
- Memory usage grows (chunks array)
- WebM duration fix may take longer
- Consider implementing max recording duration

### 4. Device Hot-Plugging

If microphone is disconnected during recording:
- Stream may become inactive
- MediaRecorder may throw error
- Current behavior: error shown, cleanup performed

### 5. Browser Audio Context Limits

Some systems limit the number of AudioContext instances. The microphone test creates its own AudioContext - ensure it's always closed.

### 6. Base64 Encoding Size

Large audio files → large base64 strings → memory pressure:
- 1 minute of audio ≈ 1MB raw ≈ 1.33MB base64
- Consider streaming for very long recordings (future enhancement)

## File Structure

```
src/lib/
├── features/dictation/
│   ├── index.ts                 # Exports
│   ├── audio-recorder.ts        # MediaRecorder wrapper, state machine
│   ├── audio-helpers.ts         # Audio utility functions
│   ├── audio-startup.ts         # Recording startup logic
│   ├── device-validation.ts     # Device validation
│   ├── dictation-controller.ts  # Recording lifecycle, transcription
│   ├── microphone-test.ts       # Pre-recording microphone validation
│   ├── wav-encoder.ts           # PCM → WAV header encoder (PHASE 3A extract)
│   ├── transcription-events.ts  # Retry + diagnostic Tauri event listeners (PHASE 3A extract)
│   ├── model-download.svelte.ts # Shared model-download controller (ModelDownload.svelte + EngineCard.svelte)
│   ├── types.ts                 # DictationConfig, AudioSettings
│   └── constants.ts             # Language codes, limits
│
├── state/
│   ├── app.svelte.ts            # isRecording, status, lastTranscription
│   ├── activity.svelte.ts       # Indicator state management
│   └── dictation-config.svelte.ts # Audio settings, language hints, terms
│
├── services/
│   └── microphone-permission.ts # Permission checking and requesting
│
└── api/
    └── tauri.ts                 # transcribeAudio() - sends to backend

src-tauri/src/
├── transcription/
│   ├── mod.rs                   # Dispatch command, engine routing
│   ├── soniox_provider.rs       # Soniox direct API (5-step flow)
│   ├── soniox_api/              # Soniox HTTP operations (split: mod.rs + types.rs)
│   ├── utils.rs                 # Audio decode helpers
│   ├── local_provider/          # Local transcription (local-stt feature, split: mod/audio/ort_init/transcribe)
│   └── model_manager.rs         # Model download (streaming SHA256) / verify / delete (local-stt feature)
├── hotkeys/
│   ├── mod.rs                   # Shortcut definitions
│   └── registration.rs          # dictation_start/stop event emission
└── clipboard.rs                 # paste_text for final transcription
```

## Configuration

### Dictation Settings (config.json)

```typescript
interface DictationConfig {
  // Audio capture
  selectedMicrophoneId: string | null;
  audioSettings: {
    noiseSuppression: boolean;
    echoCancellation: boolean;
    autoGainControl: boolean;
  };

  // Recognition context
  topic: string;
  names: string[];
  backgroundText: string;
  customTerms: string[];

  // Language
  languageHints: string[];  // empty by default; empty means no language hints
  enableLanguageIdentification: boolean;

  // Translation (optional)
  translationMode: 'off' | 'one_way';
  translationTargetLanguage: string;
}
```

## Debugging

### Log Prefixes

| Prefix | Component |
|--------|-----------|
| `[AudioRecorder]` | MediaRecorder operations |
| `[DictationController]` | Recording lifecycle |
| `[Activity]` | Indicator state changes |
| `[MicrophoneTest]` | Microphone validation |

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Not recording" | Stop called before start completed | Fixed by race condition handling |
| "MediaRecorder start timeout" | Microphone didn't respond in 5s | Check device, permissions |
| "No microphones found" | No audio input devices | Connect microphone |
| "Selected microphone not found" | Device disconnected | Select new device in settings |
| "Could not access the selected microphone" | Permission denied or device busy | Check system permissions |
| "Microphone access timed out..." | `getUserMedia()` hung for 8+ seconds | On macOS, open AH window first; check permissions |
| "OverconstrainedError" | Requested constraints not supported | Falls back to minimal constraints |

## Testing Checklist

### Basic Flow
- [ ] Press and hold shortcut → shows "Preparing..." then "Recording..."
- [ ] Release shortcut → shows "Processing..." then "Transcribing..."
- [ ] Text appears at cursor position
- [ ] Indicator shows success and auto-hides

### Edge Cases
- [ ] Quick press-release (< 500ms) → handles gracefully
- [ ] Very long recording (> 60s) → still works
- [ ] Microphone disconnected during recording → error shown
- [ ] No microphone connected → clear error message
- [ ] Permission denied → platform-specific guidance

### Platform Testing
- [ ] Windows: WebView2 with default mic
- [ ] Windows: WebView2 with selected mic
- [ ] macOS: WKWebView with default mic
- [ ] macOS: WKWebView with selected mic

## References

- [MediaRecorder API - MDN](https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder)
- [Tauri Webview Versions](https://v2.tauri.app/reference/webview-versions/)
- [Cross-browser MediaRecorder](https://media-codings.com/articles/recording-cross-browser-compatible-media)
