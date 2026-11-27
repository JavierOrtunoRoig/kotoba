# Architecture

## Overview

```
┌─────────────────────────────┐
│  WebView2 (React + TS)      │
│  main / popup               │
└──────────────┬──────────────┘
               │ invoke / events
┌──────────────▼──────────────┐
│  Tauri 2 (Rust)             │
│  commands · tray · scheduler│
│  import · text · media      │
└──────────────┬──────────────┘
               │ rusqlite
┌──────────────▼──────────────┐
│  SQLite + media/            │
│  %APPDATA%\com.kotoba.app   │
└─────────────────────────────┘
```

No network. The frontend only talks to Rust through commands; Rust is the only one that touches
SQLite and files.

## Rust modules (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `main.rs` | Binary entry point; calls `kotoba_lib::run()` |
| `lib.rs` | App state, Tauri commands, setup, windows, plugins |
| `models.rs` | Serde structs shared with the frontend |
| `db.rs` | SQLite connection, schema, migrations, settings, items, reviews, SM-2 SRS, statistics (global, per lesson and difficult) and media cleanup |
| `import.rs` | `.apkg` / `.txt` importer, deduplication and audio extraction |
| `media.rs` | Media map parser (Anki protobuf + JSON fallback) |
| `text.rs` | HTML cleanup, normalization, kana→romaji and answer validation |
| `scheduler.rs` | Passive scheduler loop, appearance modes and DND |
| `shell.rs` | Show/hide popup and widget, toast, focus, positioning and pause |
| `tray.rs` | Icon, menu (including “Mode”) and tray events |

## Shared state

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub db_path: PathBuf,
    pub media_dir: PathBuf,
    pub popup_visible: AtomicBool,
    pub widget_visible: AtomicBool,
}
```

It is registered with `app.manage(AppState { ... })` in `setup`. Commands receive it as
`State<AppState>`.

## SQLite schema

### `items`

| Column | Type | Note |
|---|---|---|
| `id` | INTEGER PK | |
| `source_uid` | TEXT UNIQUE | Anki `uid`, idempotency key |
| `kanji` | TEXT NULL | kanji form |
| `reading` | TEXT | kana |
| `meaning` | TEXT | main meaning in Spanish (clean) |
| `meaning_plain` | TEXT | normalized meaning for keys |
| `kanji_gloss` | TEXT NULL | kanji-to-kanji gloss (`significado`) |
| `audio_file` | TEXT NULL | `<sha1>.mp3` name inside `media/` |
| `lesson` | TEXT | `L00`…`L12` |
| `category` | TEXT | `01 Vocabulario`, `03 Partes Del Cuerpo`, … |
| `tags` | TEXT | space-separated Anki tags |
| `is_pattern` | INTEGER | `〜`, `+`, `...` |
| `irregular` | INTEGER | `*` in the reading |
| `dedup_key` | TEXT UNIQUE | `normalize_key(reading) + "|" + normalize_key(meaning)` |
| `created_at` | INTEGER | Unix UTC |

### `item_sources`

`item_id`, `source_uid`, `lesson`, `category`. Lets a deduplicated item remember every Anki note
that originated it.

### `review_log`

`id`, `item_id`, `shown_at`, `answered_at`, `result` (1/0), `quality` (1-5), `latency_ms`,
`direction` (`jp_to_es` / `es_to_jp`).

### `srs_state`

`item_id` PK, `interval_minutes`, `max_interval_minutes`, `ease`, `due_at`, `reps`, `lapses`.
Migration v5 initializes the maximum from the current interval; subsequent reviews retain the
highest interval, even after a lapse. Earlier lapses cannot be reconstructed.

### Maturity and lesson statistics

Maturity is derived from SRS state, not an input to scheduling: new has no state; learning is
below one day without a previous interval of one day; relearning is below one day after reaching
one day; young is at least one day and below 21 days; mature is at least 21 days. The global
mature count and word achievements use the same 21-day boundary; existing achievement unlocks
remain permanent.

Lesson counts include patterns only when `include_patterns` is enabled. Dominance is mature
items divided by all reviewable lesson items. A nonempty lesson is dominated when it has no new
items and reaches the configured threshold. The badge and lesson achievements share this rule.
The threshold ranges from 50% to 100%, defaults to 80% on new installs, and preserves stored
settings. Lesson statistics retain review counts and accuracy alongside the five maturity counts.

### `settings`

`key` / `value`. All configuration is stored as JSON under the `settings` key.

### Indexes

`idx_srs_due`, `idx_review_answered`, `idx_items_pattern`.

## `.apkg` import flow

1. Open the ZIP.
2. Look for `collection.anki21b` → decompress zstd → write a temporary SQLite database → open it
   with `rusqlite`. Fallback to `collection.anki21` / `collection.anki2`.
3. Read `fields` (by `mid`), `cards` + `decks` (for lesson/category) and `notes` (`flds`, `tags`).
4. Read `media` → zstd → protobuf (`MediaEntry { name, size, sha1 }`) or legacy JSON.
5. For each note:
   - split `flds` by `\x1f`;
   - map fields by name (positional fallback);
   - clean HTML/entities and normalize;
   - compute `is_pattern`, `irregular` and `dedup_key`;
   - extract the referenced audio (`[sound:...]`), verify SHA-1 and write it to
     `media/<sha1>.<ext>`;
   - insert or reuse the item and record `item_sources`;
   - emit `import:progress` every 25 notes.
6. Commit and return `ImportSummary`.

Audio is extracted only once per name; repeated references count as `audio_reused`.

## Review flow

1. The scheduler (its own thread) wakes up every 10 s.
2. It checks pause, DND, the daily maximum, fullscreen and whether the popup is already visible.
3. Depending on the mode: `show_popup`, `show_widget` or `show_toast`.
4. The popup/widget window receives the `challenge:show` event and requests `next_challenge`.
5. `db::next_item` picks a due item (or any item if none are due) excluding the current one and
   filtering templates.
6. The user answers → `submit_answer` validates, computes the quality (1-5), logs to `review_log`,
   updates `srs_state` with SM-2 and returns `AnswerResult`.
7. The frontend shows the feedback, the audio and the next interval.
8. “Next” requests another challenge; “Not now”/“Hide” hides the window without recording anything.

## Answer normalization

- NFKC, lowercase, no diacritics, no spaces or punctuation.
- ES → JP: kanji, kana and romaji (Hepburn/kunrei, long vowels and final は → wa).
- JP → ES: senses separated by `;`, without parentheses or `...`.
- Romaji is generated from the kana variants; kanji are not romanized.

## IPC commands

| Command | Input | Output |
|---|---|---|
| `bootstrap` | — | `Bootstrap { item_count, settings, stats }` |
| `get_settings` | — | `Settings` |
| `update_settings` | `settings` | Sanitized `Settings` |
| `get_stats` | — | `Stats` with 14-day activity, per-lesson progress and difficult words |
| `import_deck` | `path` | `ImportSummary` (emits `import:progress`) |
| `next_challenge` | `excludeId?` | `Challenge \| null` |
| `submit_answer` | `itemId`, `direction`, `answer`, `latencyMs` | `AnswerResult` |
| `get_audio` | `itemId` | `number[] \| null` (mp3 bytes) |
| `reset_progress` | — | `Stats` |
| `clear_library` | — | `void` (deletes items, progress and orphan audio) |
| `export_backup` | `destination` | path of the created folder |
| `restore_backup` | `source` | restored `Settings` (replaces DB and audio) |
| `export_history` | `destination` | path of the `review_log` CSV |
| `practice_now` | — | `void` (opens the current mode: popup/widget/toast) |
| `close_popup` | — | `void` |
| `hide_widget` | — | `void` |
| `show_main_window` | — | `void` |

Tauri converts `snake_case` arguments to `camelCase` in JS (`itemId`, `latencyMs`, `excludeId`).

Main events: `challenge:show` (to popup/widget), `settings:changed`, `import:progress` and
`library:changed`.

## Frontend (`src/`)

| File | Responsibility |
|---|---|
| `App.tsx` | Detects whether the window is `popup` via `getCurrentWindow().label` |
| `MainApp.tsx` | Bootstrap, navigation rail, views and global state |
| `PracticePanel.tsx` | Review card, input, feedback, audio and shortcuts |
| `PopupApp.tsx` | Compact version for the popup window; listens for `challenge:show` |
| `ImportPanel.tsx` | `.apkg` picker, summary and notices |
| `StatsPanel.tsx` | Metrics and activity chart |
| `SettingsPanel.tsx` | Frequency, DND, content, audio, autostart and data |
| `api.ts` | Types and `invoke` wrappers |
| `audio.ts` | Playback of local bytes with a Blob URL |
| `Icons.tsx` | Inline SVG icons |
| `styles.css` | “washi” visual system and styles for all views |

The popup and the main window share the same bundle; they are told apart by the window label.

## Scheduler

- `std::thread` thread that sleeps 10 s between checks.
- `next_run` is an `Instant`; the first challenge is scheduled at ~75 s.
- Before showing: check that there are items, that it is not paused, that it is not DND, that
  `max_per_day` has not been reached and that there is no fullscreen/presentation if
  `suppress_fullscreen` is active.
- If the mode is `popup` and the popup is already visible, it waits 45 s.
- Dispatch by mode: `popup` → `show_popup`; `widget` → `show_widget` (re-emits the challenge);
  `toast` → `show_toast` (native notification).
- After showing: `next_run = now + rand(min..=max) minutes`.

## SRS SM-2

- Quality: 5 if correct in ≤ 5 s, 4 if correct but slower, 2 if wrong, 1 if you don't know.
- `ease += 0.1 - (5 - q) * (0.08 + (5 - q) * 0.02)`, with a floor of 1.3.
- Learning: first correct → 10 min; second → 1 day; afterwards interval × `ease`.
- Failure: `reps = 0`, interval 10 min, `lapses += 1`.
- `due_at` is stored in Unix UTC.

## Default settings

| Key | Value |
|---|---|
| `interval_min_minutes` | 10 |
| `interval_max_minutes` | 25 |
| `max_per_day` | 40 (0 = no limit) |
| `paused` | `false` |
| `dnd_enabled` | `true` |
| `dnd_start` / `dnd_end` | `22:00` / `08:00` |
| `audio_autoplay` | `true` |
| `mode` | `popup` (`popup` \| `widget` \| `toast`) |
| `suppress_fullscreen` | `true` |
| `direction_mode` | `random` (`random` \| `alternate`) |
| `include_patterns` | `false` |
| `launch_at_startup` | `false` |

## Local data

```
%APPDATA%\com.kotoba.app\
  kotoba.sqlite3
  kotoba.sqlite3-wal
  kotoba.sqlite3-shm
  media\
    <sha1>.mp3
```

Settings → Data → **Export backup** creates `kotoba-backup-<timestamp>/` with `kotoba.sqlite3` and
`media/`. Before copying, `PRAGMA wal_checkpoint(TRUNCATE)` is run so the database is consistent.
Media cleanup runs when the library is deleted and at the end of an import.
**Restore backup** validates the SQLite file, closes the live connection, replaces the database and
`media/`, and reopens the connection. **Export history** generates a CSV of `review_log` with local
date and item data.

## Logging

`tauri-plugin-log` writes to `%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log` at Info level.
Settings → System → **Open folder** opens that directory. Imports, backups, restores and scheduler
startup are logged.

## Windows and capabilities

- `main`: 1120×780 logical, minimum 820×600, hidden until it decides whether there is a library.
- `popup`: 392×280, frameless, transparent, always-on-top, skip-taskbar, no initial focus.
- `widget`: 340×248, frameless, transparent, always-on-top, persistent; created at startup if
  the mode is widget or when the mode changes.
- Capabilities: `core:default`, show/hide/close/set-focus/start-dragging, `dialog:default`,
  `autostart:default`, `notification:default`.
- `csp: null` for development convenience; there is no remote content.
- Audio does not use `assetProtocol`: it is served as bytes over IPC and played with a Blob URL.

## Appearance modes and focus

| Mode | Window | Behaviour |
|---|---|---|
| `popup` | `popup` | Appears every interval; hides on answer or close |
| `widget` | `widget` | Stays visible; the scheduler re-emits challenges in the same window |
| `toast` | none | Native notification; clicking it opens the popup |

- **No focus stealing:** `shell::show_without_activation` calls `window.show()` to initialize the
  webview, forces topmost with `SetWindowPos(SWP_NOACTIVATE)` and returns focus to the previous
  window with `SetForegroundWindow`.
- **Fullscreen:** `SHQueryUserNotificationState` detects presentations, games, fullscreen apps and
  quiet mode. The scheduler shows nothing in those states if `suppress_fullscreen` is active.
- **Visibility:** `AppState.popup_visible` and `widget_visible` are the source of truth for the
  scheduler; it does not rely on `is_visible()` because the native show is done through Win32.
