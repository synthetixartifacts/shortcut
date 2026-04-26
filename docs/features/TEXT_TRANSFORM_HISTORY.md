# Text Transform History

## Overview

Persistent log of successful Grammar Fix / Translate / Improve outputs, with browse / search / filter / copy / delete affordances. Stored separately from Dictation History — different storage file, separate Rust module, separate state, separate route, separate components. Only the **result** text is recorded; no source text, provider id, model id, or latency metadata.

**Sidebar location**: between "Dictation History" and "Settings" (decision D9).
**Route**: `/text-transform-history`.
**Storage file**: `text_transform_history.json` in the app data directory (sibling of `history.json`).
**Retention**: 10,000 most-recent entries, oldest dropped first.

## How It Works

```
Direct shortcut OR Action Wheel
        │
        ▼
features/shortcuts/shortcut-dispatcher.ts
        │
        ▼
grammar-controller.ts | translation-controller.ts | improve-controller.ts
        │
        ▼
features/text-transform/base-controller.ts        <-- SINGLE save hook (D6)
        │  (after pasteFormatted; isolated try/catch — D5)
        ▼
api/text-transform-history.ts → invokeWithErrorHandling
        │
        ▼ (Tauri IPC)
src-tauri/src/text_transform_history.rs
        │
        ▼
text_transform_history.json (atomic tmp→rename)
```

### Read path (the page)

```
/text-transform-history
  └─ uses state/text-transform-history.svelte.ts (loaded flag, actionFilter)
     └─ calls api/text-transform-history.ts
        └─ Tauri get_text_transform_history → text_transform_history.rs::load
           └─ renders TextTransformHistoryList → TextTransformHistoryItem (action badge + click-to-copy + delete)
```

### Single integration hook (D6)

`base-controller.ts::createTextTransformHandler` is the factory used by all three transform controllers. Each controller passes a `name` (`"Grammar"`, `"Translation"`, `"Improve"`) which a small `nameToAction` helper maps to the lowercase action discriminator (`"grammar" | "translate" | "improve"`). Both direct shortcuts and the Action Wheel reach the same factory through `shortcut-dispatcher.ts`, so one hook covers all six entry points (3 actions × 2 entry types). `screen_question` runs through a different streaming pipeline (`stream_screen_question`) and is **not** recorded — by scope.

The save block:
- runs **after** `pasteFormatted` succeeds (so we never persist an unpasted result),
- is wrapped in its own `try / catch` and `logError`-then-swallow (D5 — paste must not fail just because history failed),
- is gated on a non-empty `result.trim()` (D13 — no empty entries),
- best-effort fires `void refreshTextTransformHistory()` so an open `/text-transform-history` page reflects the new entry.

### The `loaded` flag (D7)

The dictation module's `refreshHistory()` only fires when `entries.length > 0 || total > 0` — silently broken when the user opens `/history` while empty and then runs a dictation. The new state module fixes this with an explicit `loaded: boolean` field that flips to `true` after the **first** `loadTextTransformHistory()` resolution (success or error). `refreshTextTransformHistory()` checks `loaded === true`, which fires correctly even after an empty initial load.

## Data Shape

```ts
TextTransformHistoryEntry {
  id: string;                                  // uuid v4
  timestamp: number;                           // unix ms
  action: 'grammar' | 'translate' | 'improve'; // discriminator
  result: string;                              // the transformed output
}

TextTransformHistoryPage {
  entries: TextTransformHistoryEntry[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}
```

No `original`, no `provider`, no `model`, no `latency`, no `language` — by scope (decisions D2, D3).

## Tauri Commands

Defined in `src-tauri/src/text_transform_history.rs`, registered in `src-tauri/src/lib.rs::invoke_handler`:

| Command | Args | Returns | Notes |
|---------|------|---------|-------|
| `get_text_transform_history` | `page, pageSize, query?, action?` | `TextTransformHistoryPage` | Newest-first sort. `query` is a case-insensitive substring on `result`. `action` accepts `"grammar"`, `"translate"`, `"improve"`, or the no-op sentinel `"all"`. `page = 0 → 1`; `page_size = 0 → 20`. |
| `add_text_transform_history_entry` | `action, result` | `TextTransformHistoryEntry` | Validates `action` against the allowed set. Rejects empty/whitespace `result`. Enforces 10k cap (drops oldest). |
| `delete_text_transform_history_entry` | `id: String` | `()` | Errors if id not found. |
| `clear_text_transform_history` | — | `()` | Wipes every entry. |

## Frontend Pieces

| Surface | Path | Purpose |
|---------|------|---------|
| Route | `src/routes/text-transform-history/+page.svelte` | Page chrome: header, search input (300 ms debounce), action filter, list, pagination, empty / loading / error states, clear-all `<Modal>`. |
| State | `src/lib/state/text-transform-history.svelte.ts` | Reactive `$state`. Actions: `loadTextTransformHistory`, `searchTextTransformHistory`, `setTextTransformActionFilter`, `deleteTextTransformEntry`, `clearAllTextTransformHistory`, `refreshTextTransformHistory`. The `loaded` flag gates `refreshTextTransformHistory`. |
| API wrapper | `src/lib/api/text-transform-history.ts` | `getTextTransformHistory`, `addTextTransformHistoryEntry`, `deleteTextTransformHistoryEntry`, `clearTextTransformHistory`. Re-exported from `$lib/api/tauri`. |
| Types | `src/lib/types/index.ts` | `TextTransformHistoryEntry`, `TextTransformHistoryPage`, `TransformAction`, `TransformActionFilter`. |
| Components | `src/lib/components/text-transform-history/*` | `TextTransformHistoryList`, `TextTransformHistoryItem` (action badge + click-to-copy + delete), `EmptyTextTransformHistory` (filtered vs. empty), `ActionFilter` (chip row). |
| Sidebar nav | `src/lib/components/layout/Sidebar.svelte` | Inserts the `/text-transform-history` link between "Dictation History" and "Settings". Uses the existing `wand` icon. |
| Integration hook | `src/lib/features/text-transform/base-controller.ts` | The single save call site. |

## Pagination Generalization

`src/lib/components/history/Pagination.svelte` was generalized as part of this feature so both `/history` and `/text-transform-history` can reuse it. Translation keys moved from `history.pagination_*` to `pagination.*` (domain-neutral). The component file lives under `components/history/` — it has not been physically moved. Locale parity holds: `pagination.previous`, `pagination.next`, `pagination.page`, `pagination.of`, `pagination.aria` exist in `en/fr/es`.

## i18n

All user-visible strings live under the `text_transform_history.*` and `nav.text_transform_history` namespaces in `src/lib/i18n/locales/{en,fr,es}.json`. 25 new keys per locale (1 nav + 24 feature). Full key parity across `en/fr/es`.

## Privacy / Storage

- File: `text_transform_history.json` in the app data directory (same dir as `history.json` and `config.json`).
- Plain JSON, no encryption — use OS-level disk encryption for sensitive deployments.
- Atomic tmp→rename writes: a crash mid-write cannot leave the file truncated.
- 10,000-entry retention cap enforced on insert (`drain(0..excess)`); ~5 MB worst-case file size at 500 bytes/entry.
- No source text, provider id, model id, or latency is recorded — only the result.
- Cleared via the in-app "Clear All" affordance or by deleting the file manually.

## User-Facing Behavior

- Successful Grammar Fix / Translate / Improve runs append a new entry **after** the result has been pasted into the target app. A persistence failure does not block paste — the user only sees the paste, never an error toast about history.
- The `/text-transform-history` page lists entries newest-first, paginated 20 per page.
- The action filter renders as a chip row (`All` / `Grammar Fix` / `Translate` / `Improve`).
- Search is a case-insensitive substring match on the result text, debounced 300 ms client-side and applied server-side.
- Each entry shows an action badge (color-coded), a relative timestamp, the result text (clamped, click-to-copy), and a delete affordance.
- "Clear All" is a `<Modal>` confirmation, not a `confirm()` dialog.
- A failed transform (provider error, paste error, empty result) creates **no** entry.

## Comparison With Dictation History

| Aspect | Dictation History | Text Transform History |
|--------|-------------------|------------------------|
| Rust module | `src-tauri/src/history.rs` | `src-tauri/src/text_transform_history.rs` |
| Storage file | `history.json` | `text_transform_history.json` |
| Frontend state | `state/history.svelte.ts` | `state/text-transform-history.svelte.ts` |
| Frontend route | `/history` | `/text-transform-history` |
| Entry payload | `text` + `duration_ms` + `language` + `engine` | `action` + `result` |
| Refresh guard | `entries.length > 0 \|\| total > 0` (broken on empty initial load) | `loaded` flag (correct on empty initial load) |
| Action filter | n/a (single domain) | `all / grammar / translate / improve` |
| Dashboard recent widget | merged into `<RecentEntries>` on `/` (top 5 across both domains, sorted by timestamp; entries badged by `kind`) | merged into `<RecentEntries>` on `/` (same widget) |

The two domains are intentionally not abstracted into a generic history framework — the entry shapes differ enough that a shared trait or sum type would add friction without simplifying the v1 (decision D1). A future third domain or a stable v1 of this feature is the right time to extract shared persistence helpers.

## Related Docs

- [docs/ARCHITECTURE.md → Transform Flow](../ARCHITECTURE.md#transform-flow-grammar--translate--improve) — where the integration hook fits in the larger flow.
- [docs/BACKEND.md → Text-Transform History Commands](../BACKEND.md#text-transform-history-commands) — Tauri command surface.
- [docs/FRONTEND.md → Text Transform History](../FRONTEND.md#text-transform-history) — frontend file map.
