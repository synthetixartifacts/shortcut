# ShortCut Documentation

## What is ShortCut?

**ShortCut** is an open-source, multi-provider voice-to-text desktop app with AI text transformation. It calls LLM and STT providers directly — no proxy, no token gate.

**Features:**
- **Voice Dictation**: Hold a hotkey, speak, release to paste transcribed text anywhere
- **Grammar Fix**: Select text + hotkey to correct grammar via AI
- **Translation**: Select text + hotkey to translate via AI
- **Improve**: Select text + hotkey to improve with AI
- **Screen Question**: Capture your screen and ask AI about it via streaming chat
- **Radial Action Wheel**: Quick access to all features from any app
- **Text Transform History**: Browse, search, filter, and copy past Grammar Fix / Translate / Improve results

## Documentation Index

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System design, provider flow, data flows |
| [BACKEND.md](./BACKEND.md) | Rust modules, providers/ layer, Tauri commands |
| [FRONTEND.md](./FRONTEND.md) | Frontend structure, Svelte 5 patterns, state |
| [SETUP.md](./SETUP.md) | Development environment setup |
| [BUILD.md](./BUILD.md) | Build commands, Docker |
| [BUILD_MACOS.md](./BUILD_MACOS.md) | macOS build guide |
| [EXTENDING.md](./EXTENDING.md) | Adding features, shortcuts, LLM/STT providers |
| [PRODUCTION.md](./PRODUCTION.md) | Distribution, signing, installers |
| [PROVIDERS.md](./PROVIDERS.md) | Provider configuration (5 LLM + 2 STT providers) |
| [USER_GUIDE_MACOS.md](./USER_GUIDE_MACOS.md) | End-user installation guide for macOS |
| [features/DICTATION.md](./features/DICTATION.md) | Dictation feature details |
| [features/INDICATOR.md](./features/INDICATOR.md) | Activity indicator feature details |
| [features/ACTION_WHEEL.md](./features/ACTION_WHEEL.md) | Action wheel feature details |
| [features/SCREEN_QUESTION.md](./features/SCREEN_QUESTION.md) | Screen question feature details |
| [features/LOCAL_STT.md](./features/LOCAL_STT.md) | Local speech-to-text engine |
| [features/TEXT_TRANSFORM_HISTORY.md](./features/TEXT_TRANSFORM_HISTORY.md) | Text transform history feature details |
