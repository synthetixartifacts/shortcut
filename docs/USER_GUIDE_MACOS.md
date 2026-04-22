# ShortCut User Guide for macOS

Quick setup guide to install and configure ShortCut on your Mac.

---

## Installation

### Step 1: Install the App

1. Open the **ShortCut_x.x.x_x64.dmg** file
2. Drag **ShortCut** to the **Applications** folder
3. Eject the DMG (right-click → Eject)

### Step 2: First Launch

Since the app may not be signed with an Apple Developer certificate:

1. Open **Finder** → **Applications**
2. **Right-click** on **ShortCut** → Select **Open**
3. Click **Open** in the security dialog

> **Note**: Double-clicking won't work the first time. You must right-click → Open.

If you see "ShortCut is damaged and can't be opened":
1. Open **Terminal**
2. Run: `xattr -cr /Applications/ShortCut.app`
3. Try opening the app again

---

## Required Permissions

ShortCut needs two permissions to function properly.

### Accessibility (Required for Shortcuts)

Accessibility permission allows ShortCut to simulate keyboard shortcuts (copy/paste) in other applications.

1. Open **System Settings**
2. Go to **Privacy & Security** → **Accessibility**
3. Click the lock icon and enter your password
4. Click the **+** button
5. Navigate to **Applications** → **ShortCut**
6. Toggle **ShortCut** to **ON**

> **Important**: Without this permission, shortcuts will not work.

### Microphone (Required for Dictation)

1. Open **System Settings**
2. Go to **Privacy & Security** → **Microphone**
3. Find **ShortCut** in the list
4. Toggle **ShortCut** to **ON**

If ShortCut is not in the list, try using the dictation feature once — macOS will prompt you to allow access.

---

## Initial Setup

### Configure AI Providers

1. Launch ShortCut
2. Open **Settings → AI Providers**
3. Add at least one **LLM provider** (for grammar, translate, improve):
   - OpenAI, Anthropic, Gemini, Grok, or Ollama
4. For **Dictation** (cloud): add a **Soniox API key**
5. Open **Settings → Task Assignments** to assign each task to your preferred provider

See [PROVIDERS.md](./PROVIDERS.md) for provider-specific setup instructions.

---

## Using ShortCut

### Keyboard Shortcuts

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Dictation** | `Cmd+Shift+D` | Hold to record, release to paste |
| **Grammar Fix** | `Cmd+Shift+G` | Select text, press shortcut |
| **Translate** | `Cmd+Shift+T` | Select text, press shortcut |
| **Improve** | `Cmd+Shift+I` | Select text, press shortcut |
| **Action Wheel** | `Cmd+Shift+J` | Open radial menu |
| **Screen Question** | `Cmd+Shift+S` | Capture screen + ask AI |

Shortcuts can be customized in Settings → Shortcuts.

### How to Use Each Feature

#### Dictation (Voice-to-Text)
1. Place your cursor where you want text to appear
2. **Hold** `Cmd+Shift+D`
3. Speak clearly
4. **Release** the keys
5. Your speech will be transcribed and pasted

Requires Soniox API key (cloud) or local engine (Windows only).

#### Grammar Fix
1. Select text in any application
2. Press `Cmd+Shift+G`
3. Wait for the indicator to show success
4. The corrected text replaces your selection

#### Translation
1. Select text in any application
2. Press `Cmd+Shift+T`
3. The translated text replaces your selection

#### Improve
1. Select text in any application
2. Press `Cmd+Shift+I`
3. The improved text replaces your selection

> Customize the improvement prompt in Settings → Actions → Improve

#### Screen Question
1. Press `Cmd+Shift+S`
2. A chat overlay appears with a thumbnail of your screen
3. Type your question and press Enter
4. The AI responds using the configured vision provider

> **First use requires Screen Recording permission.** macOS sandboxes screen capture behind TCC. If the captured screenshot appears black or you see a "Screen Recording permission required" banner, open **System Settings → Privacy & Security → Screen Recording**, enable ShortCut, and restart the app.

---

## Troubleshooting

### Shortcuts Not Working

| Issue | Solution |
|-------|----------|
| Nothing happens when pressing shortcut | Grant Accessibility permission (see above) |
| Shortcut doesn't paste result | Restart the app after granting Accessibility permission |

### Microphone Issues

| Issue | Solution |
|-------|----------|
| Dictation doesn't record | Grant Microphone permission (see above) |
| Low audio quality | Check microphone settings in Settings → Actions → Dictation |
| "Microphone access denied" | System Settings → Privacy → Microphone → Enable ShortCut |

### Provider / API Issues

| Issue | Solution |
|-------|----------|
| "API key not configured" error | Open Settings → AI Providers and add your key |
| "Provider does not support vision" | Assign a vision-capable provider for Screen Question |
| "Screen Recording permission required" (macOS) | System Settings → Privacy & Security → Screen Recording → Enable ShortCut, then restart the app |
| Screen Question captures a black image (macOS) | Same as above — TCC-denied captures silently return a blank frame |
| Slow responses | Normal for vision models; try a faster model in Task Assignments |

### App Won't Open

| Issue | Solution |
|-------|----------|
| "App is damaged" error | Run `xattr -cr /Applications/ShortCut.app` in Terminal |
| "Unidentified developer" | Right-click → Open instead of double-clicking |

---

## Settings

Access settings via the **Settings** icon in the sidebar.

### Available Settings

- **AI Providers**: API keys for OpenAI, Anthropic, Gemini, Grok, Soniox, Ollama
- **Task Assignments**: Which provider handles grammar, translate, improve, screen question
- **Shortcuts**: Customize keyboard shortcuts
- **Dictation**: Configure microphone, language hints, vocabulary terms
- **App Settings**: Theme, language, debug mode

### Changing Shortcuts

1. Open **Settings** → **Shortcuts**
2. Click on a shortcut to edit
3. Press your desired key combination
4. Click **Save**

> **Tip**: On macOS, use `Cmd+Shift+key` combinations to avoid conflicts.

---

## Support

- **History**: View past transcriptions in the History tab
- **Debug**: Enable debug mode in App Settings for detailed logs
- **Issues**: Report problems at the project's issue tracker
