# Requirements

Legend: `[x]` implemented · `[~]` partial · `[ ]` pending.

Functional requirements use `RF-xx`; non-functional ones, `RNF-xx`. Future-phase ones use
`RF2-xx` / `RF3-xx`.

---

# Phase 1 — Technical MVP (implemented)

## RF-01 — Tray residency

- [x] Tray icon with menu: Practice now, Open kotoba, Pause/Resume, Quit.
- [x] Left click opens the main window.
- [x] Closing the main window hides it; it does not terminate the process.
- [x] The pause state is persisted and reflected in the menu.
- [x] The main window is only shown at startup if the library is empty.

## RF-02 — Start with Windows

- [x] `tauri-plugin-autostart` plugin integrated.
- [x] Toggle in Settings → System.
- [~] Still needs verification on a real Windows startup (registry/startup entry) → #12.

## RF-03 — `.apkg` import

- [x] Native file picker.
- [x] Reading of ZIP, `collection.anki21b`, zstd and SQLite schema 18.
- [x] Fallback to `collection.anki21` / `collection.anki2`.
- [x] Mapping by field name with positional fallback.
- [x] Audio extraction with SHA-1 verification and deduplication by hash.
- [x] Import summary: notes, items, audio files, templates and warnings.
- [x] Idempotency by `source_uid`.
- [x] Deduplication by reading + meaning, keeping `item_sources`.

**Verified acceptance:** `Genki 1.apkg` → 1,076 notes, 1,012 items, 827 audio files, 66 templates,
0 warnings.

## RF-04 - `.txt` import (fallback)

- [x] Parsing of `#separator`, `#tags column` and `#columns` headers.
- [x] Genki positional mapping (8 columns) and reduced mode (3 columns).
- [x] Audio in `.txt`: `[sound:…]` (also inside HTML) from the mapped column, copied with
  SHA-1 and deduplicated from the chosen media folder (by default, the `.txt` sibling);
  missing files add warnings without aborting. → #10
- [x] Manual column-mapping screen with row preview and assignment of reading,
  kanji, meaning, gloss, UID, tags, lesson and subsection.

## RF-05 — Bidirectional exercise

- [x] JP → ES: kanji is shown (or kana if there is no kanji) and the meaning is typed.
- [x] ES → JP: the meaning is shown and the answer is typed in Japanese (kanji or kana).
- [x] Input with the system's native IME (`lang="ja"`).
- [x] Random or alternating direction (`direction_mode`).
- [x] Enter submits without interfering with IME composition.

## RF-06 — Answer normalization (basic)

- [x] NFKC, lowercase, no diacritics, no spaces or trailing punctuation.
- [x] ES → JP accepts kanji and reading variants separated by `／`.
- [x] Accepts forms with and without optional parentheses (`おやすみ（なさい）`).
- [x] Strips `*` and `〜` for comparison.
- [x] JP → ES accepts senses separated by `;`.
- [x] JP → ES accepts the form without parentheses or `...`.
- [x] Edge cases (particles in the meaning, IME variants, romaji) closed in RF2-04.

## RF-07 — Immediate feedback

- [x] Correct/incorrect indicator.
- [x] Primary expected answer and accepted variants.
- [x] Kana reading.
- [x] Meaning.
- [x] Kanji gloss (`significado`) when present.
- [x] Audio on demand.
- [x] Next interval (“Come back in …”).

## RF-08 — Audio

- [x] Automatic audio in JP → ES when enabled.
- [x] Audio after answering in ES → JP.
- [x] Playback from local bytes (no server or asset protocol).
- [x] If the audio is missing, the card still works.

## RF-09 — Popup mode

- [x] Frameless, transparent, always-on-top window with no taskbar.
- [x] Created on demand and reused.
- [x] Positioned in the bottom-right corner of the monitor.
- [x] Draggable from the header.
- [x] Closing with Escape or the X hides the window without recording a review.
- [x] Does not steal focus when appearing: created with `focused(false)`, shown without activating and
  focus is returned (verified with `GetForegroundWindow`). `WS_EX_NOACTIVATE` is discarded: it would
  prevent typing in the popup, which needs keyboard focus.

## RF-10 — Scheduler

- [x] Random interval between minimum and maximum.
- [x] First challenge ~75 s after startup.
- [x] Check every 10 s.
- [x] Does not launch another challenge if the popup is still visible.
- [x] Stops if the app is paused, in DND or the daily maximum has been reached.

## RF-11 — Do not disturb

- [x] Configurable schedule with start/end time.
- [x] Supports ranges that cross midnight.
- [x] Enable toggle.

## RF-12 — Daily limit

- [x] Configurable `max_per_day` (0 = no limit).
- [x] Only answered reviews count; closing the popup does not count.

## RF-13 — SRS SM-2

- [x] Per-item state in `srs_state`.
- [x] Quality 1-5 derived from correctness and latency.
- [x] Learning steps (10 min, 1 day) and growth by `ease`.
- [x] A failure resets `reps`, returns to 10 min and increments `lapses`.
- [x] `ease` is updated with the SM-2 formula and a floor of 1.3.
- [x] Tests for learning, growth, failure and the `ease` floor.

## RF-14 — Statistics

- [x] Total words, total reviews and reviews today.
- [x] Today's accuracy.
- [x] Day streak.
- [x] Mature words (interval ≥ 21 days), also used by word achievements.
- [x] Pending (respecting `include_patterns`).
- [x] Activity over the last 14 days.

## RF-15 — Local persistence

- [x] SQLite with WAL and foreign keys.
- [x] `items`, `item_sources`, `review_log`, `srs_state`, `settings` tables.
- [x] Audio in `%APPDATA%\com.kotoba.app\media`.
- [x] Settings persisted as JSON.

## RF-16 — Data management

- [x] Reimport a deck without losing the progress of existing words.
- [x] Reset progress (deletes history and intervals).
- [x] Delete library (deletes words, sources and progress).

---

# Non-functional requirements

## RNF-01 — RAM

- [x] Target < ~120 MB at rest.
- [x] Measured in the Phase 1 verification: **~33 MB** with the app in the tray.

## RNF-02 — Privacy

- [x] No network, no telemetry, no accounts.
- [x] All content lives on the user's machine.

## RNF-03 — Do not interrupt

- [x] Do not steal focus (see RF-09).
- [x] Full-screen app detection (RF2-02).

## RNF-04 — Platform

- [x] Windows 10/11 with WebView2.
- [x] MSVC build.
- [x] CI with build and tests on Windows.
- [x] Automatic release with MSI and NSIS when pushing a `v*` tag.
- [ ] Signed installer → #14.

## RNF-05 — Maintainability

- [x] Separate Rust modules (`db`, `import`, `media`, `text`, `scheduler`, `shell`, `tray`).
- [x] Unit tests for normalization, media parser and importer.
- [x] Versioned migrations (RF2-08: `PRAGMA user_version` + `ensure_column`).
- [x] Structured logging to file and visible errors with retry (RF2-08).

---

# Phase 2 — Polish

> Status: completed. The remaining `[ ]` are optional or distribution-related (signing/CI).

## RF2-01 — The 3 appearance modes

- [x] Popup (Phase 1).
- [x] Persistent floating widget, always-on-top, draggable and with live challenges.
- [x] Native Windows toast (`tauri-plugin-notification`); clicking it opens the popup.
- [x] Mode selector in Settings and “Mode” submenu in the tray.
- [x] The mode is applied on save and on startup (if it is the widget, it appears at startup).

## RF2-02 — Real do not disturb

- [x] Full-screen, presentation, game and silent-mode detection with
  `SHQueryUserNotificationState`.
- [x] The popup/widget is shown and focus is returned to the previous window.
- [ ] Automatic pause during calls (optional). Suppression during full screen already
  exists; the “call” case does not deserve a ticket yet.

## RF2-03 — Full SRS SM-2

- [x] Use `ease` and answer quality (not just correct/incorrect).
- [x] Intervals with ease factor.
- [x] `due_at` is stored in UTC; statistics use local time.
- [x] Progression tests.

## RF2-04 — Robust normalization

- [x] Handle particles, annotations and meaning placeholders.
- [x] Accept romaji (Hepburn, kunrei, long vowels and は→wa).
- [x] Normalize IME variants (NFKC, half-widths, ideographic spaces).
- [x] Tests with the 1,012 real entries: **2,629 validated answers** (includes romaji).

## RF2-05 — Extended statistics

- [x] Progress per lesson (`L00`…`L12`).
- [x] Five maturity counts (new, learning, relearning, young, mature), including zero counts;
  mature starts at 21 days and relearning requires a previous interval of at least one day.
- [x] Dominance is mature words divided by reviewable lesson items, with patterns controlled by
  `include_patterns`. A nonempty lesson is dominated only with no new words and at least the
  configured threshold; the default is 80% for new installs and existing settings are retained.
- [x] Difficult / most-failed words.
- [x] History exportable to CSV (RF2-08).
- [x] Detailed history inside the app → #11.
- [ ] Configurable streaks and goals (the weekly goal is already configurable; what
  “streak” means still needs to be defined).

## RF2-06 - More convenient import

- [x] Import progress via events (`import:progress` every 25 notes).
- [x] Prior inspection of the `.apkg` by note type with field names and sample rows.
- [x] Manual field mapping per note type: reading, kanji, meaning, gloss, audio, UID,
  tags, lesson and subsection.
- [x] Manual column mapping in `.txt` (includes `#columns` when present).
- [x] Automatic field detection by name with positional fallback; the user can adjust it.
- [x] Registration of Anki's `unicase` collation to read `fields.name` and `notetypes.name`.
- [x] Validation with Anki-exported fixtures for Basic, reversed, Cloze, multiple decks, and
  dedicated audio fields → #9.
- [x] Orphan audio cleanup when deleting or reimporting.
- [x] Import of third-party decks with supported note types → #9.

## RF2-07 - UX and accessibility

- [x] Own icon and visual identity (言 seal).
- [x] Keyboard navigation and visible focus: “skip to content” link, `aria-current` in the
  navigation, `Enter` to check and move to the next challenge, `Esc` to close the popup.
- [x] UI texts in Spanish, English and Japanese (interface, tray and notifications).
- [x] Interface size adjustment (90-125 %), with floating windows scaled.
- [x] **Dark theme** with automatic, light and dark options; `auto` follows the Windows theme.
- [x] Respect for `prefers-reduced-motion` and accessible labels on controls and icons.

## RF2-08 - Robustness

- [x] Versioned migrations (`PRAGMA user_version` + `ensure_column`).
- [x] Backup to folder (SQLite + `media/`).
- [x] Restore a backup from the app (replaces database and audio).
- [x] Logging to file (`%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`).
- [x] Error handling with retry (practice and import).
- [x] Export review history (CSV).

## RF2-09 - Order and limit of new words

- [x] A word is new if it has no `srs_state`; there is no separate flag.
- [x] New words follow the deck order: lesson, subsection and import order
  (`ORDER BY lesson, category, id`).
- [x] When importing `.apkg`, notes are read in a stable order (`ORDER BY notes.id`).
- [x] Due reviews before new words by default (`new_mix_percent = 0`).
- [x] Configurable mix 0-100 %: percentage of new words when there are also pending reviews.
- [x] Daily limit of new words (`new_per_day`, default 10; 0 = none) counted by the first
  answer of the day.
- [x] Separate counters: due reviews, available new words and new words introduced today.
- [x] The scheduler does not open challenges if there are no due reviews and no new words within the
  limit.
- [x] Tests for lesson/subsection order, review priority, mix and daily limit.

---

# Phase 3 — Extras

## RF3-01 — AnkiConnect

- [ ] Opt-in sync with local Anki.
- [ ] Deck change detection.
- [ ] Do not overwrite Anki's scheduling.

> Pending a `grill-with-docs` session → spec; it is not a standalone ticket yet.

## RF3-02 — More content

- [ ] Genki 2 content.
- [ ] Grammar and particles.
- [ ] Phrases and listening.

## RF3-03 — Distribution

- [x] CI with build and tests on Windows.
- [x] Automatic release with MSI and NSIS when pushing a `v*` tag.
- [ ] Signed MSI/NSIS installer → #14.
- [x] Auto-update with `tauri-plugin-updater` and `latest.json` (2026-11-22).

## RF3-04 — Gamification

**Character:** hybrid. Silent by default; milestone notifications are opt-in and respect
pause, DND and full screen. Flexible streak. All local, no accounts or social.

**Guardrails:** XP only rewards quality (not volume); the streak does not generate guilt; rewards
cannot incentivize fast/bad answers that break SM-2.

### R3-04a — MVG (implemented)

- [x] `profile` record with `xp_total`, `level` and `streak_best` (migration v2).
- [x] XP only for correct answers: quality 3→+1, 4→+2, 5→+3; failure → 0.
- [x] Levels by XP thresholds (12 levels).
- [x] Flexible streak + best historical streak.
- [x] Configurable weekly goal (default 30; 0 = no goal).
- [x] Progress screen: level, XP, level bar, streak/best streak and weekly goal.
- [x] Feedback: `+X XP` and level-up notice in practice and popup.
- [x] Native notification when completing the weekly goal, opt-in, respects pause/DND/full
  screen and never warns about a “streak in danger”.
- [x] Tests for XP, levels, streak, goal and migration.

### R3-04b — Achievements and dominance (implemented)

- [x] Catalog of 16 achievements: reviews, mature words, streak, dominated lessons, level 5 and
  weekly goal completed.
- [x] Progress of each achievement visible and permanent unlock.
- [x] Lesson achievements use the same dominance predicate as the lesson badge.
- [x] Achievements screen in Progress and immediate feedback on unlock.
- [x] Opt-in notifications on unlock, subject to pause/DND/full screen.
- [x] Migration v3 (`achievements`, `profile.weekly_goals_done`).
- [x] Tests for permanent unlocks, maturity boundaries and lesson dominance.

### R3-04c — Refinement (implemented)

- [x] Streak saver: each completed weekly goal gives one (maximum 2).
- [x] Automatic repair of a missed day, optional in Settings; never repairs more than one
  gap per review nor spends a saver without recent activity.
- [x] Savers visible in Progress and `+1 saver` feedback when earning them.
- [x] Smoother level curve (80 → 5,800 XP) and titles per tier (Seed, Sprout, Root,
  Branch, Tree, Forest).
- [x] Migration v4 (`profile.streak_savers`, `streak_repairs` table) and tests for repair,
  saver cap and migration.
- [ ] Mid-term retention metrics: outside the local MVP (no telemetry or cloud); the
  Progress screen already shows 14-day activity and progress per lesson. It is not actionable
  while there is no telemetry, so it does not generate an issue.

---

# Out of scope (for now)

- Accounts, cloud, multi-device sync.
- Redistribution of the deck content.
- Card editing inside kotoba.
- Mobile or web version.
