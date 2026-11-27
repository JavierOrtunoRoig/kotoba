# Decisions

Lightweight ADR format: context → decision → consequences. Decisions are not deleted: if they
change, a new entry is added that supersedes the previous one.

---

## D-001 — Tauri 2 + React 19 + TypeScript + Vite + SQLite stack

**Context:** Windows desktop app, always on, with a low-RAM requirement.
**Decision:** Tauri 2 (Rust + WebView2) with a React 19/TS/Vite frontend and embedded SQLite via
`rusqlite` (`bundled`).
**Consequences:** smaller binary and lower consumption than Electron; paid for with Rust and
WebView2 complexity. `rusqlite` gives full control over the schema and the importer.

## D-002 — Windows only

**Context:** the target user runs Windows and the original requirement fixes it.
**Decision:** no macOS/Linux branches are maintained in Phase 1. The tray, autostart and windows are
implemented with Windows APIs.
**Consequences:** less test surface; if it is ported, the tray, DND and fullscreen detection will
need review.

## D-003 — Primary import from `.apkg`

**Context:** the `Genki 1` deck includes audio; Anki's `.txt` only stores `[sound:...]` references.
**Decision:** `.apkg` is the primary format. `.txt`/`.tsv`/`.csv` are accepted as a fallback without
audio.
**Consequences:** the importer must read ZIP + zstd + SQLite schema 18 + protobuf. `.txt` is limited
to vocabulary without audio and assumes Genki's field order.

## D-004 — Read the new Anki format

**Context:** current `.apkg` files ship `collection.anki2` as a placeholder and the real database in
`collection.anki21b` compressed with zstd.
**Decision:** look in the order `collection.anki21b` → `collection.anki21` → `collection.anki2`,
decompress zstd if needed and validate the `SQLite format 3\0` header.
**Consequences:** works with modern Anki; old `.apkg` files keep working if their `collection.anki2`
is real SQLite. The placeholder's “update Anki” message is ignored.

## D-005 — Map fields by name, with positional fallback

**Context:** the `Genki Model ES` notetype has 8 fields, but other decks may vary.
**Decision:** map by name (`kana`, `kanjis`, `español`, `significado`, `sonido`, `uid`) and, if
there is no match, use Genki's order: 0 reading, 1 kanji, 3 meaning, 4 gloss, 5 audio, 7 uid.
**Consequences:** the importer is reusable; the positional fallback may be wrong for very different
decks and a mapping screen will be needed if it is generalized.

## D-006 — Deduplicate by reading + meaning

**Context:** 1,076 notes produce 1,012 items because vocabulary repeats across subdecks.
**Decision:** deduplication key = `normalize_key(reading) + "|" + normalize_key(meaning)`.
The first item is kept and all sources go in `item_sources`.
**Consequences:** fewer duplicate cards; the notion of “one card per lesson” is lost except through
`item_sources`. Reimporting is idempotent by `source_uid`.

## D-007 — Tagged templates and grammar, excluded by default

**Context:** 66 notes use `〜`, `+ negative` or `...` and are not clean typeable vocabulary.
**Decision:** mark them with `is_pattern = true`; the scheduler excludes them unless
`settings.include_patterns = true`.
**Consequences:** the MVP shows “pure” vocabulary; the user can enable them in Settings.

## D-008 — Automatic audio only in JP → ES

**Context:** audio reveals the answer in ES → JP.
**Decision:** in JP → ES it plays when the card is shown if `audio_autoplay` is active; in
ES → JP it plays after answering.
**Consequences:** UX aligned with the Anki templates. Audio is optional: if it is missing or fails,
the card keeps working.

## D-009 — Provisional lightweight SRS

> Superseded by [D-022](#d-022--full-sm-2-supersedes-d-009).

**Context:** the plan leaves full SM-2 for Phase 2.
**Decision:** fixed intervals `[10 min, 1 h, 6 h, 1 d, 3 d, 7 d, 14 d, 30 d]`; a correct answer moves
up one step, a failure returns to 10 min. `ease` is stored but not used yet.
**Consequences:** the app already distributes reviews usefully; Phase 2 will replace the logic
without changing the schema.

## D-010 — Fully local, no network

**Context:** privacy and always-on app requirement.
**Decision:** SQLite and audio in `%APPDATA%\com.kotoba.app`; no network calls in Phase 1.
**Consequences:** no sync or telemetry. AnkiConnect is left for Phase 3 and will be opt-in.

## D-011 — A single appearance mode in Phase 1: popup

> Extended by [D-023](#d-023--three-appearance-modes-and-focus-supersedes-d-011).

**Context:** the plan asks for 3 modes (widget, popup, toast) in Phase 2.
**Decision:** Phase 1 implements only the frameless/always-on-top popup, created on demand.
**Consequences:** less initial work; the scheduler and settings are already prepared to add the
other two modes.

## D-012 — “Washi” aesthetic with system fonts

**Context:** a Japanese app, for passive use, that should not look like another generic app.
**Decision:** warm paper, ink, vermilion, 言 seal, serif/Mincho typography; system fonts
(`Yu Mincho`, `Constantia`, `Yu Gothic UI`). No external fonts or network.
**Consequences:** clear, offline visual identity. On machines without those fonts there are
fallbacks; if full consistency is wanted, fonts can be bundled in Phase 2.

## D-013 — Ignore Anki's scheduling

**Context:** the `.apkg` ships 352 cards in review and 1,151 `revlog` entries.
**Decision:** import everything as new; kotoba keeps its own SRS.
**Consequences:** Anki's progress is not overwritten and intervals that do not belong to the app are
not inherited. The user can reset progress without touching Anki.

## D-014 — Conventional Commits

**Context:** the user asked for it explicitly.
**Decision:** all commits use `type(scope): summary` with an optional body.
**Consequences:** readable history, compatible with automatic changelogs.

## D-015 — Version `Cargo.lock`

**Context:** this is a binary application, not a library.
**Decision:** `Cargo.lock` is committed for reproducible builds. `skills-lock.json` and `.agents/`
stay out of version control because they are agent tooling.
**Consequences:** more deterministic builds; the lock must be updated when dependencies change.

## D-016 — `significado` is a kanji gloss; `tipo` is empty

**Context:** the deck uses `español` for the meaning and `significado` for a kanji-to-kanji gloss
(“Diez - Cuatro”). The `tipo` field is empty in all 1,076 notes.
**Decision:** ignore `tipo`; use `español` as the meaning and `significado` as an optional
`kanji_gloss`.
**Consequences:** the feedback can show the kanji breakdown without contaminating the answer.

## D-017 — `uid` as a stable key; `id_orden` is not global

**Context:** `id_orden` only goes up to 61 and repeats per subdeck; `uid` is unique
(`Genki 1::L00::01 Saludos_0`).
**Decision:** use `uid` as `source_uid` for idempotency. `id_orden` is not used as a global order.
**Consequences:** reimporting does not duplicate sources; presentation order is decided by the
scheduler.

## D-018 — Answer normalization rules

**Context:** the deck mixes kanji, kana, variants with `／`, optional `（…）`, `*` and `〜`.
**Decision:** NFKC, lowercase, no diacritics, no spaces or punctuation. In ES → JP, kanji and all
reading variants are accepted. In JP → ES, the senses separated by `;` and the variants without
parentheses or `...` are accepted.
**Consequences:** tolerant with the user; it may accept the odd extra variant. Phase 2 will expand
with more IME and particle cases.

## D-019 — Main window hidden if a library already exists

**Context:** the app should live in the tray and not open on its own.
**Decision:** at startup, show the main window only if `items` is empty (onboarding).
If there is a library, it starts in the tray. Closing the window hides it, it does not destroy it.
**Consequences:** the first launch is discoverable; later ones are discreet. “Quit” from the tray
menu ends the process.

## D-020 — `max_per_day` counts answered reviews

**Context:** the user can close a popup without answering (“Not now”).
**Decision:** a review is only logged to `review_log` when an answer is submitted (correct or
incorrect). Closing without answering does not count toward the daily maximum.
**Consequences:** the limit measures real effort; the scheduler can retry later.

## D-021 — The pending counter respects `include_patterns`

**Context:** the sidebar and the card showed different figures (1,012 vs 949) because one included
templates and the other did not.
**Decision:** `stats.due_now` is filtered by `include_patterns`, same as `next_challenge`.
**Consequences:** both figures match and the user sees the same review universe.

---

## D-022 — Full SM-2 (supersedes D-009)

**Context:** the Phase 1 lightweight SRS did not use `ease` or distinguish answer quality.
**Decision:** quality 1-5 (5 = fast correct, 4 = correct, 2 = failure, 1 = don't know); learning
steps of 10 min and 1 day; afterwards, interval = previous interval × `ease`; `ease` is updated
with the SM-2 formula and a floor of 1.3; a failure resets `reps`, returns to 10 min and increments
`lapses`.
**Consequences:** review adapts to real performance. Quality is stored in
`review_log.quality` for future statistics.

## D-023 — Three appearance modes and focus (supersedes D-011)

**Context:** the plan asks for widget, popup and toast; on top of that, the popup must not steal
focus.
**Decision:** three configurable modes (`popup`, `widget`, `toast`) with a selector in Settings and
a “Mode” submenu in the tray. The widget is persistent; the popup is created on demand; the toast
uses `tauri-plugin-notification` and clicking it opens the popup. To avoid stealing focus, the
window is shown with `window.show()` and focus is returned to the previous window with
`SetForegroundWindow`. Fullscreen, presentations, games and quiet mode are detected with
`SHQueryUserNotificationState`.
**Consequences:** the user chooses the level of intrusion. The popup appears without keeping the
keyboard; the widget requires a click to type.

## D-024 — Migrations with `PRAGMA user_version`

**Context:** Phase 1 only used `CREATE TABLE IF NOT EXISTS`; adding `quality` to `review_log` was
not applied to existing databases.
**Decision:** read `PRAGMA user_version` and apply incremental, idempotent migrations
(`ensure_column`). Version 1 adds `review_log.quality`.
**Consequences:** old databases are upgraded without data loss. Future migrations add an
`if version < N` block.

## D-025 — Romaji as alternative input

**Context:** not every user has a Japanese IME active.
**Decision:** in ES → JP, romaji is accepted in addition to kanji and kana. The converter is
approximate Hepburn, with variants for long vowels (`ou`/`oo` → `o`), kunrei (`shi`/`si`), nasal
(`nb` → `mb`) and final は → wa.
**Consequences:** you can practice without an IME. Some variants may accept a slightly unorthodox
form (for example, を as `o`), but the kana reading remains the primary source.

## D-026 — Silent gamification and XP by quality

**Context:** classic gamification (urgent streaks, XP by volume, “streak at risk” notifications)
clashes with kotoba's passive identity and can corrupt the SM-2.
**Decision:** hybrid gamification. XP is only earned for correct answers weighted by `quality`
(3→+1, 4→+2, 5→+3); the streak is flexible and has no punishment; milestone notifications are
opt-in and respect pause/DND/fullscreen; no accounts or social features. It ships in phases:
**R3-04a** streak + weekly goal + XP/level, **R3-04b** achievements/mastery and **R3-04c** streak
freeze and refinement.
**Consequences:** it can retain without turning the app into a daily obligation. The level curve
and the achievement catalogue are left for b/c; XP does not incentivize answering fast or badly.

## D-027 — Permanent achievements and per-lesson mastery

The per-lesson mastery definition in this decision is superseded by D-036.

**Context:** R3-04b needs milestones that celebrate consistency and retention without introducing
daily pressure or social comparisons.
**Decision:** fixed catalogue of 16 achievements computed from real data (reviews, mastered words,
streak, lessons, level and weekly goals). Unlocks are stored in the `achievements` table and do not
expire. A lesson is considered mastered when the percentage of words with an interval ≥ 7 days
reaches the configurable threshold (default 90%). Unlock notifications are opt-in and respect
pause/DND/fullscreen.
**Consequences:** retroactive progress materializes on the next review; there are no social or speed
achievements. The mastery threshold can be adjusted between 50% and 100%.

## D-028 — Streak lifelines and levels with titles

**Context:** a strict streak punishes a missed day and contradicts the passive character. At the same
time, XP needs a legible long-term progression.
**Decision:** each completed weekly goal grants a lifeline (maximum 2). If the user comes back after
a missed day, one is spent automatically to repair just that gap (can be disabled in Settings) and
it is recorded in `streak_repairs`. The level curve moves to 12 thresholds from 0 to 5,800 XP, with
titles per tier: Seed, Sprout, Root, Branch, Tree and Forest.
**Consequences:** the streak stays flexible and is not punished with warnings. Lifelines are not
spent without recent activity and do not repair several gaps at once. Retention metrics are out of
MVP scope because there is no cloud or telemetry.

## D-029 — Dark theme, language and interface scale

**Context:** RF2-07 asks to complete accessibility. The app is used at night and in short sessions,
and part of the target vocabulary is users who prefer the interface in English or Japanese.
**Decision:** add three local preferences: `theme` (`auto`, `light`, `dark`), `language`
(`es`, `en`, `ja`) and `font_scale` (90-125%). `auto` follows the Windows theme live. The scale is
applied with `zoom` compensated on `#root` and floating windows grow proportionally so content is
not clipped. The whole UI, the tray and notifications use the chosen language; technical errors and
deck content stay in their original language. Keyboard navigation uses native elements,
`Enter`/`Esc`, `aria-current`, visible focus and a skip link.
**Consequences:** preferences are local with no external dependencies. The dark theme introduces no
new animations and respects `prefers-reduced-motion`. Achievement texts are translated by title in
the frontend and by key in the backend notifications.

## D-030 — New words by deck order and daily limit

**Context:** when importing a large deck, shuffling all new words at random breaks the deck's
didactic progression and brings advanced lesson vocabulary forward. Also, new words could compete
with due reviews and delay retention.
**Decision:** a word without `srs_state` is new. New words are picked by `lesson, category, id`,
respecting the deck's lessons and subsections. By default due reviews have priority;
`new_mix_percent` allows reserving a percentage for new words when there are also reviews.
`new_per_day` limits how many new words are introduced per day (default 10, 0 = none), counting the
first answer for each word. The due and new counters are independent and the scheduler does not
interrupt if nothing is available.
**Consequences:** the user advances L01 → subsection 1 → subsection 2 → L02 without jumps. Reviews
keep priority and new words do not flood the day. If the limit is lowered mid-day, an excess already
introduced may appear; that progress is not removed.

## D-031 — Manual field mapping on import

**Context:** the importer assumed Genki's field order (or a reduced 3-column mode).
That forced any deck to have exactly those fields and made it impossible to import decks
with different names/order without touching code.
**Decision:** inspect the deck before importing and show a mapping wizard. For `.txt`,
`#separator`, `#tags column` and `#columns` are read; for `.apkg`, Anki notetypes and their fields
are read (`fields` + `notetypes`), with a sample of rows per type. The mapping assigns indices to
reading, kanji, meaning, gloss, audio, UID, tags, lesson and subsection. Automatic detection uses
known names and falls back to Genki positions; the user can correct it. Anki uses `unicase` collation
for field names, so it is registered when the temporary database is opened.
**Consequences:** decks with other fields can be imported without changing the code, and progress
can also be preserved on reimport. Mappings live only in the import; they are not saved as a
preference. The deck content is still responsible for the quality of the mapped data.

## D-032 — Automatic updates with tauri-plugin-updater

**Context:** the app is distributed as MSI/NSIS on GitHub Releases and did not announce new versions.
Without a notice, every update depends on the user checking the project page.
**Decision:** use `tauri-plugin-updater` with `latest.json` published by `tauri-action` on each
release. The endpoint is `releases/latest/download/latest.json`. Artifacts are signed with minisign
(private key in GitHub secrets; public key embedded in `tauri.conf.json`). The app checks at
startup, notifies with a dialog and, if accepted, downloads, installs and restarts. It can be checked
manually from Settings → System. The automatic notice can be postponed per version (localStorage).
**Consequences:** installs with this version onward update themselves; earlier ones require a manual
download. The updater signature does not replace the Authenticode signature: SmartScreen still warns.
If the private key is lost, existing installs reject future updates and must be reinstalled once. The
key and its password must be kept outside the repo.

## D-033 — Matt Pocock's engineering skills

**Context:** the project had a single local skill (`dependabot-prs`) and no standard
triage → spec → tickets → implementation → review flow.
**Decision:** install the engineering core of [`mattpocock/skills`](https://github.com/mattpocock/skills)
for opencode (17 skills: `triage`, `to-spec`, `to-tickets`, `implement`, `code-review`, `tdd`,
`diagnosing-bugs`, `domain-modeling`, etc.). The files live in `.agents/skills/` (versioned, like
the rest of the repo configuration) and the source is pinned in `skills-lock.json`. The per-repo
configuration is generated by `setup-matt-pocock-skills` in `docs/agents/`:
`issue-tracker.md` (GitHub Issues), `triage-labels.md` (the five canonical labels) and `domain.md`
(single-context; it also points to `.opencode/`). `AGENTS.md` gains an `## Agent skills` block that
points to those three files.
**Consequences:** skills travel with the repo: any clone has them without installing anything and
updates show up as a diff in the PR. To update them: `npx skills@latest update` and commit. Triage
uses GitHub Issues and the labels `needs-triage`, `needs-info`, `ready-for-agent`,
`ready-for-human` and `wontfix`, already created in the repo. Project decisions still live in this
file (`D-0xx`); `docs/adr/` and `GLOSSARY.md` will only be created when domain modeling calls for
them.

## D-034 — Workflows by size and idea capture

**Context:** with 19 skills installed, the decision of “which flow do I use” had become fuzzy. The
project is a one-person effort with short sessions, and it had been working with direct commits to
`main` without issues.
**Decision:** fix a working profile in `.opencode/workflows.md` and an operational block in
`AGENTS.md` with the request → flow routing. Summary: an issue for everything that changes
behaviour; branch + PR per issue; a spec only if the work does not fit in one session; full review
only on PRs that touch several modules, the schema/IPC, `db.rs`, `scheduler.rs` or release signing;
release when closing a feature; artifacts in English and docs/conversation in Spanish; technical
autonomy with consultation on product and UX. Loose ideas are captured in `.opencode/inbox.md` and
turned into issues when planning.
**Consequences:** direct commits to `main` are over, so every change goes through CI in a PR. Small
PRs get no deep review so 30-60 min sessions are not slowed down. The inbox avoids creating
premature issues without losing ideas. `workflows.md` is the document to edit if the pace or
formality changes.

## D-035 — Single instance with `tauri-plugin-single-instance`

**Context:** two processes open the same SQLite database and each starts its own scheduler and tray
icon, with the risk of duplicated challenges and reviews and of WAL contention.
**Decision:** register `tauri-plugin-single-instance` as the first plugin of `tauri::Builder`. The
callback of the first instance shows and focuses the main window; the second hands over its
arguments and exits before opening the database or the scheduler.
**Consequences:** a single process, tray and scheduler per Windows session. Manual launch, the Start
menu launch and autostart converge on the same instance. In `tauri dev` it also focuses the first
instance instead of opening another.

## D-036 — Per-word maturity and per-lesson dominance

**Context:** mastery was binary: a word was mastered on reaching 7 days and a lesson on exceeding a
threshold of mastered words. The inbox idea asked for Anki-like levels and visibility of
intermediate progress. The current definition is in D-027.
**Decision:** five states derived from the interval, with Anki's cut-offs: new (never reviewed),
learning (< 1 day and never reached 1 day), relearning (already reached 1 day and is now below),
young (1-20 days) and mature (≥ 21 days). `srs_state` gains `max_interval_minutes` to tell
relearning from learning; the SRS does not change. Lesson dominance is the percentage of mature
words, and a lesson is dominated when it has no new words and reaches the configurable threshold
(default 80% only on new installs; whoever already has it stored keeps it). The lesson total
respects `include_patterns`, and the badge and achievements share the same predicate. Vocabulary in
`GLOSSARY.md` and the full decision in `docs/adr/0001-word-maturity-and-lesson-dominance.md`.
**Consequences:** dominating a lesson becomes a months-long goal. Already unlocked achievements are
kept. The migration fills `max_interval_minutes` with the current interval, so a word that failed
before this change will show as learning until its next failure. Progress, Settings and the
statistics queries change; the popup and practice do not show the state. It supersedes D-027's
per-lesson mastery definition when implemented.
