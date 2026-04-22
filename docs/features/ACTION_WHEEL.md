# Action Wheel Feature

## Overview

Floating radial menu (pie menu) that appears at the cursor position when the user presses `Alt+J` (Windows/Linux) or `Cmd+Shift+J` (macOS). Provides visual, mouse-driven access to all 5 actions (Dictation, Grammar, Translate, Improve, Screen Question) without requiring users to memorize individual shortcuts.

**Key properties**: Non-focusable overlay, transparent background, always-on-top, auto-dismiss after 5 seconds.

## Architecture

```
┌─────────────────┐                    ┌──────────────────┐
│   Main Window   │                    │ Action Menu      │
│                 │                    │ Window           │
│  ShortcutDisp.  │◄──menu-action─────│                  │
│  dispatches     │   -selected        │  ┌────────────┐ │
│  action handler │                    │  │  PieMenu   │ │
│                 │                    │  │  (SVG)     │ │
│  open_menu:     │───invoke──────────►│  │  5 wedges  │ │
│  toggleAction   │  toggle_action_    │  └────────────┘ │
│  Menu()         │  menu              │                  │
│                 │                    │  +page.svelte    │
└─────────────────┘                    └──────────────────┘
        │                                      │
        │ invoke()                             │ emit()
        ▼                                      │
┌─────────────────┐                            │
│   Rust Backend  │◄───────────────────────────┘
│  action_menu.rs │  (hideActionMenu via invoke)
│                 │
│  toggle_action  │
│  _menu()        │
│  hide_action    │
│  _menu()        │
│  position_at    │
│  _cursor()      │
└─────────────────┘
```

**Two-way event communication** (unlike indicator which is one-way):
- **Main -> Menu**: `action-menu-show` event (resets auto-dismiss timer)
- **Menu -> Main**: `menu-action-selected` event (carries selected action)

## Interaction Flow

```
User presses Alt+J
  -> Rust: global shortcut handler emits "shortcut-triggered: open_menu"
  -> Frontend: ShortcutDispatcher.dispatch("open_menu")
  -> invoke("toggle_action_menu")
  -> Rust: position_at_cursor() + show_without_focus_steal()
  -> Emit "action-menu-show" event

User clicks a wedge
  -> PieWedge onclick -> selectAction(action)
  -> emit("menu-action-selected", { action })
  -> invoke("hide_action_menu")
  -> Main window ShortcutDispatcher receives event
  -> 50ms delay (let menu hide)
  -> dispatch existing handler (handleGrammarFix, etc.)
```

## Menu Items

| ID | Label (i18n key) | Icon | Color | Action |
|----|------------------|------|-------|--------|
| `dictation` | `action_menu.item_dictation` | Microphone | `#ef4444` (red) | `dictation_start` |
| `grammar` | `action_menu.item_grammar` | Notepad | `#3b82f6` (blue) | `grammar` |
| `translate` | `action_menu.item_translate` | Globe | `#8b5cf6` (purple) | `translate` |
| `improve` | `action_menu.item_improve` | Sparkles | `#10b981` (teal) | `improve` |
| `screen_question` | `action_menu.item_screen_question` | Camera | `#f59e0b` (amber) | `screen_question` |

Colors match the indicator activity colors for visual consistency.

## Dismiss Behaviors

| Trigger | What Happens |
|---------|-------------|
| Press `Alt+J` again | Toggle: hides the menu |
| Click background area | `selectAction('')` hides without executing |
| Auto-dismiss (5s) | `selectAction('')` hides without executing |
| Mouse movement | Resets the 5s auto-dismiss timer |
| Direct shortcut (e.g. `Alt+G`) | `ShortcutDispatcher` calls `hideActionMenu()` before dispatching |

## File Structure

```
src/lib/
├── features/action-menu/
│   ├── index.ts              # Public exports
│   ├── types.ts              # MenuItem, MenuVisibility
│   ├── constants.ts          # MENU_ITEMS, sizes, AUTO_DISMISS_MS
│   └── menu-controller.ts    # selectAction() — emit event + hide
│
├── components/action-menu/
│   ├── index.ts              # Component exports
│   ├── PieMenu.svelte        # SVG container, renders wedges (55 lines)
│   └── PieWedge.svelte       # Individual SVG arc wedge (157 lines)
│
└── state/
    └── action-menu.svelte.ts # Reactive state (visibility, hoveredItem)

src/routes/action-menu/
├── +page.svelte              # Overlay page: event listeners, auto-dismiss
└── +layout.ts                # SSR disabled

src-tauri/src/
└── action_menu.rs            # Rust: toggle, hide, position, health, recreate (226 lines)
```

## Window Configuration

From `tauri.conf.json`:

```json
{
  "label": "action-menu",
  "title": "AH Action Menu",
  "url": "/action-menu",
  "width": 280,
  "height": 280,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "shadow": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "visible": false,
  "center": false,
  "focus": false
}
```

## Focus Preservation

The action menu must not steal focus from the user's active application.

**Windows**:
- `WS_EX_NOACTIVATE` extended style via `apply_non_focusable()`
- `ShowWindow(SW_SHOWNOACTIVATE)` via `show_without_focus_steal()`
- `WM_MOUSEACTIVATE` -> `MA_NOACTIVATE` subclass via `apply_mouse_no_activate()` (WebView2 first-click fix)

**macOS**:
- `NSWindowCollectionBehavior` set to transient + ignores-cycle
- Partial mitigation only (Tauri #14102)

## Edge Detection

The menu is centered on the cursor position, clamped to screen edges with 4px padding. Uses `get_cursor_monitor()` and `is_position_valid()` from `indicator/positioning.rs` (shared helpers made `pub` after the PHASE 3B split).

## Display Resilience

Same pattern as the indicator window:
1. **Health check on show**: `ensure_menu_window()` validates the window handle before every show
2. **System event response**: `handle_display_change_menu()` called on `RunEvent::Resumed` and `ScaleFactorChanged`
3. **Window recreation**: Destroys stale window, waits 100ms, rebuilds via `window_style::build_overlay_window`, reapplies styles

## Event Communication

### Events Emitted

| Event | Direction | Payload | Purpose |
|-------|-----------|---------|---------|
| `action-menu-show` | Rust -> Menu window | `()` | Resets auto-dismiss timer |
| `menu-action-selected` | Menu window -> Main window | `{ action: ShortcutAction }` | User selected an action |

### Tauri Commands

| Command | Parameters | Returns | Description |
|---------|------------|---------|-------------|
| `toggle_action_menu` | - | - | Show at cursor if hidden, hide if visible |
| `hide_action_menu` | - | - | Hide the menu window |
| `is_action_menu_visible` | - | `bool` | Check visibility |

## Known Limitations

1. **macOS `focusable:false`** (Tauri #14102): On macOS, the `focusable:false` config does not fully prevent activation. A proper fix requires `tauri-nspanel` or native NSPanel wrapper. The `NSWindowCollectionBehavior` provides partial mitigation.

2. **WebView2 hover events**: WebView2 may not deliver hover events to a non-focusable window on some Windows configurations. Mitigation: wedge labels are always visible (not hover-only). CSS hover opacity is best-effort.

3. **WebView2 first-click activation**: Even with `WS_EX_NOACTIVATE`, WebView2 can steal focus on the first click. Mitigated by `WM_MOUSEACTIVATE` subclassing in `window_style.rs`.

## Implementation Status

Phase 1 complete: flat 5-item radial menu with SVG rendering.

**Phase 2 TODOs** (not yet planned):
- Nested menu (sub-actions, e.g., dictation languages)
- Keyboard quick-select (press number keys)
- Hover animations
- Custom action ordering
