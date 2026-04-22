# Activity Indicator Feature

## Overview

Floating visual indicator that appears during AH actions (dictation, grammar, translation, improve) to provide feedback when the app runs in the background.

**Location**: Bottom-center of screen, above taskbar

## Architecture

```
┌─────────────────┐                    ┌─────────────────┐
│   Main Window   │                    │ Indicator Window │
│                 │                    │                 │
│  Feature calls  │                    │  ┌───────────┐  │
│  startActivity  │───emit event──────►│  │  ● ● ●    │  │
│  updateActivity │  'indicator-update'│  │ Recording │  │
│  endActivity    │                    │  └───────────┘  │
│                 │                    │                 │
│  activity.svelte.ts                  │  +page.svelte   │
└─────────────────┘                    └─────────────────┘
        │
        │ invoke()
        ▼
┌─────────────────┐
│   Rust Backend  │
│  indicator/     │
│                 │
│  mod.rs         │
│  topology.rs    │
│  positioning.rs │
│  lifecycle.rs   │
└─────────────────┘
```

## Activity Types

| Type | Color | Label | Use Case |
|------|-------|-------|----------|
| `dictation` | Red (#ef4444) | Recording | Voice recording active |
| `grammar` | Blue (#3b82f6) | Fixing | Grammar fix in progress |
| `translate` | Purple (#8b5cf6) | Translating | Translation in progress |
| `improve` | Teal (#10b981) | Improving | Text improvement via AI |
| `processing` | Gray (#6b7280) | Processing | Generic operation |

## Activity States

| State | Display | Auto-hide |
|-------|---------|-----------|
| `idle` | Hidden | N/A |
| `active` | Animated dots | No |
| `success` | Checkmark | After 1s |
| `error` | X mark | After 2s |

## Usage

### Simple Pattern (withIndicator)

```typescript
import { withIndicator } from '$lib/features/indicator';

await withIndicator('grammar', async () => {
  const text = await getSelectedText();
  const fixed = await fixGrammar(text);
  await pasteText(fixed);
}, { successMessage: 'Fixed!' });
```

### Manual Control

```typescript
import { startActivity, updateActivity, endActivity } from '$lib/state/activity.svelte';

await startActivity('dictation');
// ... do work ...
await updateActivity('success', 'Done!');
// auto-hides after delay
```

## File Structure

```
src/lib/
├── features/indicator/
│   ├── index.ts              # Exports
│   ├── types.ts              # ActivityType, ActivityState, ActivityInfo
│   ├── constants.ts          # Colors, animation timing, labels
│   └── helpers.ts            # withIndicator(), startIndicator(), etc.
│
├── components/indicator/
│   ├── index.ts
│   ├── IndicatorDots.svelte  # Animated dot display
│   └── IndicatorWindow.svelte
│
└── state/
    └── activity.svelte.ts    # startActivity, updateActivity, endActivity

src/routes/indicator/
├── +page.svelte              # Indicator window content
└── +layout.ts                # SSR disabled

src-tauri/src/
├── indicator/                # Split module (PHASE 3B)
│   ├── mod.rs                #   Tauri commands: show_indicator, hide_indicator, reset_indicator
│   ├── topology.rs           #   Display topology tracking; poison-recoverable mutex
│   ├── positioning.rs        #   Bottom-center clamp + monitor math
│   └── lifecycle.rs          #   Create / recreate / validate window handle
└── window_style.rs           # Shared overlay helpers: is_window_healthy, OverlayConfig, build_overlay_window
```

## Window Configuration

```json
{
  "label": "indicator",
  "url": "/indicator",
  "width": 120,
  "height": 44,
  "visible": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "focus": false
}
```

## Dependencies

**Cargo.toml:**
```toml
tauri-plugin-positioner = "2"
```

**package.json:**
```json
"@tauri-apps/plugin-positioner": "^2"
```

**Capabilities:**
```json
"positioner:default",
"core:window:allow-show",
"core:window:allow-hide",
"core:window:allow-set-position"
```

## Animation Constants

```typescript
const ANIMATION = {
  DOT_PULSE_DURATION: 1500,   // ms per pulse cycle
  DOT_STAGGER_DELAY: 200,     // ms between dots
  FADE_IN_DURATION: 200,
  FADE_OUT_DURATION: 150,
  SUCCESS_DISPLAY_TIME: 1000, // ms before auto-hide
  ERROR_DISPLAY_TIME: 2000,
};
```

## Event Communication

Main window broadcasts state to indicator via Tauri events:

```typescript
// In activity.svelte.ts
await emit('indicator-update', {
  type: activityState.type,
  state: activityState.state,
  message: activityState.message,
});
```

Indicator window listens:

```typescript
// In indicator/+page.svelte
await listen<ActivityInfo>('indicator-update', (event) => {
  activityType = event.payload.type;
  activityState = event.payload.state;
  message = event.payload.message || '';
});
```

## Display Resilience

The indicator window is protected against display topology changes (monitor disconnect, lid close/reopen, sleep/wake) with three recovery layers:

### Layer 1: Validate on Show
Every `show_indicator()` call validates the window handle via the shared `window_style::is_window_healthy` helper (`is_visible()` + `scale_factor()`). If the handle is stale, the window is destroyed and recreated automatically via `window_style::build_overlay_window`.

### Layer 2: System Event Response
- `RunEvent::Resumed` (sleep/wake) triggers indicator validation
- `WindowEvent::ScaleFactorChanged` on main window (monitor topology change) triggers validation

### Layer 3: Manual Reset
Dashboard has a "Reset display indicator" link that forces window recreation. Use when automatic recovery fails.

### Recreation Flow
1. Destroy existing window via `window.destroy()`
2. Wait 100ms for OS to release handle
3. Build new window with `WebviewWindowBuilder` (same config as tauri.conf.json)
4. Re-apply `WS_EX_NOACTIVATE` / macOS collection behavior
5. Wait 200ms for webview to load Svelte app

## Implementation Status

✅ **Fully Implemented**

- Backend: `indicator/` split module (mod/topology/positioning/lifecycle) with show/hide/position/recreate commands; shared overlay scaffolding in `window_style.rs`
- Frontend: `features/indicator/` with types, constants, helpers
- State: `activity.svelte.ts` with reactive state management
- UI: `components/indicator/` with IndicatorWindow and IndicatorDots
- Integration: Grammar, translate, improve, and dictation features use `withIndicator`
- Resilience: 3-layer recovery (validate-on-show, system events, manual reset)
