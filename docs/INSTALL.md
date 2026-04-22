# ShortCut Installation Guide

Step-by-step guide to install ShortCut on macOS and Windows. Both platforms block unsigned apps by default — this guide walks you through every security prompt you will see.

---

## macOS

### Step 1: Download the right file

Go to [Releases](../../releases) and download one DMG:

| Your Mac | How to check | File to download |
|----------|-------------|-----------------|
| **Intel** — MacBook Air/Pro 2020 or older, iMac 2020 or older, Mac Mini 2020 or older, Mac Pro | Apple menu → About This Mac → "Processor: **Intel**..." | `ShortCut_x.x.x_x64.dmg` |
| **Apple Silicon** — MacBook Air/Pro 2021+, iMac 2021+, Mac Mini 2021+, Mac Studio | Apple menu → About This Mac → "Chip: **Apple M1/M2/M3/M4**" | `ShortCut_x.x.x_aarch64.dmg` |
| **Not sure** — works on any Mac, larger download | — | `ShortCut_x.x.x_universal.dmg` |

### Step 2: Install

1. Open the downloaded `.dmg` file
2. Drag **ShortCut** into the **Applications** folder
3. Close the DMG window and eject it (right-click the desktop icon → Eject)

### Step 3: First launch (Gatekeeper)

macOS blocks apps that aren't signed with an Apple Developer certificate. You will need to bypass this on first launch.

1. Open **Finder** → **Applications**
2. **Double-click** ShortCut — macOS will show a warning: *"ShortCut can't be opened because Apple cannot check it for malicious software"*
3. Click **OK** (do not click Move to Trash)
4. Open **System Settings** → **Privacy & Security**
5. Scroll down — you will see: *"ShortCut was blocked from use because it is not from an identified developer"*
6. Click **Open Anyway**
7. Enter your password when prompted
8. ShortCut will launch

> **Alternative method**: Instead of steps 2-7, you can **right-click** ShortCut in Applications → select **Open** → click **Open** in the dialog. This works on all macOS versions.

> **"ShortCut is damaged and can't be opened"**: This happens if macOS quarantine flags are set. Open Terminal and run:
> ```bash
> xattr -cr /Applications/ShortCut.app
> ```
> Then try opening again.

### Step 4: Grant Accessibility permission (required for shortcuts)

Without this permission, global shortcuts will not work. ShortCut needs Accessibility to simulate copy/paste in other apps.

1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Click the lock icon (bottom-left) and enter your password
3. Click the **+** button
4. Navigate to **Applications** → select **ShortCut** → click **Open**
5. Make sure ShortCut is toggled **ON** in the list
6. **Quit and relaunch ShortCut** (the permission takes effect on next launch)

### Step 5: Grant Microphone permission (required for dictation)

1. Open **System Settings** → **Privacy & Security** → **Microphone**
2. Find **ShortCut** in the list and toggle it **ON**

> If ShortCut is not in the list: launch ShortCut, try using dictation (`Cmd+Shift+D`), and macOS will prompt you to allow microphone access. Click **OK**, then verify it appears in the list.

### Step 6: Grant Screen Recording permission (required for Screen Question)

This is only needed if you want to use the Screen Question feature (`Cmd+Shift+S`).

1. Open **System Settings** → **Privacy & Security** → **Screen Recording**
2. Click the **+** button and add **ShortCut**
3. Toggle it **ON**
4. **Restart ShortCut** (screen recording permission requires a restart)

> Without this permission, Screen Question will capture a black image. macOS silently returns a blank frame when the app lacks screen recording access.

### Step 7: Configure AI providers

1. In ShortCut, open **Settings** → **AI Providers**
2. Add at least one LLM provider API key:
   - **OpenAI** — [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
   - **Anthropic** — [console.anthropic.com/settings/keys](https://console.anthropic.com/settings/keys)
   - **Gemini** — [aistudio.google.com/apikey](https://aistudio.google.com/apikey)
   - **Grok** — [console.x.ai](https://console.x.ai)
   - **Ollama** — local, no API key needed (just set the URL)
3. For voice dictation: add a **Soniox** API key — [console.soniox.com](https://console.soniox.com)
4. Go to **Settings** → **Task Assignments** and assign each task to a provider + model

You're ready to use ShortCut.

### macOS quick verification

| Test | How | Expected |
|------|-----|----------|
| Shortcuts work | Select text anywhere, press `Cmd+Shift+G` | Text gets grammar-fixed and replaced |
| Dictation works | Hold `Cmd+Shift+D`, speak, release | Speech transcribed and pasted at cursor |
| Screen Question works | Press `Cmd+Shift+S` | Overlay appears with screenshot thumbnail |

If shortcuts don't respond, double-check Accessibility permission and restart ShortCut.

---

## Windows

### Step 1: Download

Go to [Releases](../../releases) and download:

| File | Description |
|------|-------------|
| `shortcut_x.x.x_x64-setup.exe` | **Installer** (recommended) |
| `shortcut.exe` + `WebView2Loader.dll` | Portable — no install, keep both files in the same folder |

### Step 2: Run the installer (Windows Defender / SmartScreen)

Windows blocks unsigned apps with SmartScreen. Here's how to get past it:

**SmartScreen warning ("Windows protected your PC")**:

1. Run `shortcut_x.x.x_x64-setup.exe`
2. A blue SmartScreen window appears: *"Windows protected your PC — Microsoft Defender SmartScreen prevented an unrecognized app from starting"*
3. Click **More info** (small text link under the warning)
4. Click **Run anyway**
5. Follow the installer prompts

**If Windows Defender flags the file as a threat**:

Sometimes Defender quarantines or deletes the installer before you can run it. If the file disappears or you see a threat notification:

1. Open **Windows Security** (search "Windows Security" in Start)
2. Go to **Virus & threat protection**
3. Under "Current threats" or click **Protection history**
4. Find the ShortCut entry — it will show as "Threat blocked" or "Threat quarantined"
5. Click on it → select **Allow on device**
6. Re-download the installer if it was deleted, then run it

**To prevent Defender from blocking it again** (optional):

1. Open **Windows Security** → **Virus & threat protection**
2. Under "Virus & threat protection settings", click **Manage settings**
3. Scroll to **Exclusions** → click **Add or remove exclusions**
4. Click **Add an exclusion** → **Folder**
5. Select the folder where ShortCut is installed (default: `C:\Users\<you>\AppData\Local\ShortCut`)

### Step 3: First launch

After installation, ShortCut launches automatically. If you used the portable version, double-click `shortcut.exe` (make sure `WebView2Loader.dll` is in the same folder).

> **"WebView2Loader.dll not found"**: The DLL must be in the same directory as `shortcut.exe`. If you used the installer, this is handled automatically.

### Step 4: Configure AI providers

1. In ShortCut, open **Settings** → **AI Providers**
2. Add at least one LLM provider API key:
   - **OpenAI** — [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
   - **Anthropic** — [console.anthropic.com/settings/keys](https://console.anthropic.com/settings/keys)
   - **Gemini** — [aistudio.google.com/apikey](https://aistudio.google.com/apikey)
   - **Grok** — [console.x.ai](https://console.x.ai)
   - **Ollama** — local, no API key needed (just set the URL)
3. For voice dictation: add a **Soniox** API key — [console.soniox.com](https://console.soniox.com)
4. Go to **Settings** → **Task Assignments** and assign each task to a provider + model

### Windows quick verification

| Test | How | Expected |
|------|-----|----------|
| Shortcuts work | Select text anywhere, press `Alt+G` | Text gets grammar-fixed and replaced |
| Dictation works | Hold `Alt+D`, speak, release | Speech transcribed and pasted at cursor |
| Screen Question works | Press `Alt+S` | Overlay appears with screenshot thumbnail |

> **Shortcuts not working in some apps?** Elevated (admin) apps block simulated input from non-elevated processes. Right-click ShortCut → **Run as administrator** to send keystrokes to those apps.

---

## Default Shortcuts

| Action | macOS | Windows |
|--------|-------|---------|
| Dictation (hold to record) | `Cmd+Shift+D` | `Alt+D` |
| Grammar Fix | `Cmd+Shift+G` | `Alt+G` |
| Translate | `Cmd+Shift+T` | `Alt+T` |
| Improve | `Cmd+Shift+I` | `Alt+I` |
| Action Wheel | `Cmd+Shift+J` | `Alt+J` |
| Screen Question | `Cmd+Shift+S` | `Alt+S` |

All shortcuts can be customized in **Settings** → **Shortcuts**.

---

## Troubleshooting

### macOS

| Problem | Solution |
|---------|----------|
| "ShortCut can't be opened" | System Settings → Privacy & Security → Open Anyway |
| "ShortCut is damaged" | Terminal: `xattr -cr /Applications/ShortCut.app` |
| Shortcuts don't work | Enable Accessibility permission, restart the app |
| Dictation doesn't record | Enable Microphone permission |
| Screen Question shows black image | Enable Screen Recording permission, restart the app |
| App won't start after macOS update | Re-grant Accessibility permission (macOS resets it on updates) |

### Windows

| Problem | Solution |
|---------|----------|
| SmartScreen blocks installer | Click More info → Run anyway |
| Defender deletes/quarantines file | Windows Security → Protection history → Allow on device |
| Defender keeps blocking | Add exclusion for ShortCut install folder |
| "WebView2Loader.dll not found" | Keep the DLL in the same folder as the exe |
| Shortcuts don't work in admin apps | Run ShortCut as administrator |
