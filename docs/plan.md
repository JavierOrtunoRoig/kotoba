# Plan: kotoba — passive Japanese review (Genki 1)

## Summary

Desktop application for **Windows** that lives in the system tray and launches **vocabulary
micro-challenges** (5-10 s) at random moments. The vocabulary comes from the **Anki deck "Genki 1"**
(already in Spanish). **Passive** study: you don't open an app to study, the app brings it to you.
"Invisible" spaced repetition behind the scenes so it actually sticks.

## Settled decisions

- **Stack**: Tauri 2 (Rust) + WebView2
- **OS**: Windows only
- **MVP**: vocabulary only (bidirectional ES↔JP)
- **Appearances**: 3 configurable modes (widget / popup / toast)
- **Content**: export of the Anki deck "Genki 1" (definitions already in Spanish)
- **Extraction**: import from a `.txt`/`.apkg` file exported from Anki

## Stack

- **Tauri 2** (Rust) + Windows **WebView2**.
- Frontend: **React + TypeScript + Vite**.
- Rust backend: tray, scheduler, window management.
- Storage: **SQLite** (`tauri-plugin-sql` or `rusqlite`).
- Native notifications: `tauri-plugin-notification`.

## Functional requirements

1. **Residence**: tray icon with a menu (Pause/Resume · Mode · Stats · Config · Quit) and
   start with Windows.
2. **Content import**: import the vocabulary from a **`.txt` export** (tab-separated) or
   **`.apkg`** of the "Genki 1" deck. Field mapping configurable on first import.
3. **Bidirectional exercise**: show a word in one language → type it in the other (with Japanese IME).
4. **Answer normalization**: accept kana and kanji, tolerate reading variants.
5. **Immediate feedback** + kana reading of the word after answering.
6. **3 display modes** (configurable): floating widget / popup / system toast.
7. **Do not disturb**: hours, fullscreen app detection, daily maximum.
8. **Passive SRS** (light SM-2) per item, invisible to the user.
9. **Simple stats**: hit %, streak, dominated words.

## Non-functional requirements

- RAM < ~120 MB idle (it is an always-on app).
- Do not interrupt fullscreen apps or steal keyboard focus on appearance.
- Full local persistence in SQLite (no cloud).

## Data model

```
items       → id, word (kanji), reading (kana), meaning (ES), lesson, tags
review_log  → id, item_id, shown_at, result, latency_ms, direction
srs_state   → item_id, interval, ease, due_at
settings    → mode, interval_range, do_not_disturb, max_per_day
```

## Import flow from Anki

1. In Anki: *File → Export → Plain text notes (.txt)* (or `.apkg`) of the "Genki 1" deck.
2. The app reads the file and shows a **preview of the field mapping** (`word`/`reading`/
   `meaning` detected from the headers) for confirmation.
3. It saves a copy in its own SQLite with an independent SRS (Anki = active study; this app = passive
   review; they don't overwrite each other).

## UX of the 3 modes

- **Floating widget**: frameless, always-on-top, transparent window, always visible proposing
  challenges.
- **Popup**: small window next to the tray; answer or dismiss, it goes away on its own.
- **Toast**: native Windows notification that you open only if you want.

## Roadmap

1. **Phase 0** — confirm the exact fields of the deck export.
2. **Phase 1 — Technical MVP**: Tauri 2 scaffold, tray, interval scheduler, `.txt` import,
   bidirectional exercise, SQLite, one window mode.
3. **Phase 2 — Polish**: the 3 modes, do-not-disturb, robust normalization, SRS, stats.
4. **Phase 3 — Extras**: AnkiConnect sync, streaks, grammar/particle types, more decks.

## License note

The Anki deck is for **personal use**; the app consumes your own export and does not redistribute content
from the book.
