# Project status

- **Word achievements (#25):** progress and new unlocks count words at intervals of 21 days or more; earlier unlocks remain recorded.
- **Lesson maturity (#24):** Progress shows five maturity counts and lesson dominance;
  patterns follow the review setting, and lesson achievements use the same dominance rule.
  New has no SRS state; learning and relearning are below one day, distinguished by whether
  the word previously reached one day; young is from one day to below 21 days and mature starts
  at 21 days. Dominance is mature items divided by reviewable items; a nonempty lesson is
  dominated only with no new items and at or above the threshold. New installations default
  to 80%; stored thresholds are retained.

Last verification: **2026-11-25**.

**Public repository:** <https://github.com/JavierOrtunoRoig/kotoba> (code under MIT).

- README oriented to public release, with screenshots, privacy, roadmap and English summary.
- Versioned agent configuration (`AGENTS.md`, `opencode.jsonc`, `.mcp.json`,
  `skills-lock.json`); the local `.codegraph/` index stays ignored.
- CI and releases in `.github/workflows/`: `ci.yml` validates TypeScript and Rust on every push/PR, and
  `release.yml` publishes MSI and NSIS when pushing a `v*` tag. Release `v0.2.0` published (MSI + NSIS signed with `.sig` and `latest.json` in the release).
- **Dependabot up to date (2026-11-21):** the 7 PRs from the first round are merged. `rand` 0.10
  forced migrating the scheduler to `rand::rng().random_range()` with the `RngExt` trait; `zip` 9,
  `zstd` 0.14, `sha1` 0.11, `windows-sys` 0.61, TypeScript 7 and the updated actions without
  code changes. Automated process as a local skill: `skills/dependabot-prs/`.
- **Automatic updates (2026-11-22):** `tauri-plugin-updater` + `tauri-plugin-process`, with
  `latest.json` in GitHub Releases and artifacts signed with minisign. Notice on startup and manual
  button in Settings → System. Private key outside the repo and in GitHub secrets.
- **Autostart only in release (2026-11-22):** the debug binary uses `devUrl` and opens a console. The UI
  disables “Start with Windows” when `Bootstrap.dev_build` is true so as not to register the
  `target/debug` exe in the Windows startup.
- **Workflows (2026-11-25):** engineering skills from `mattpocock/skills` versioned in
  `.agents/skills/`, with `.opencode/workflows.md` as the playbook (issue → branch → PR, spec only if
  multi-session, review only on large PRs) and `.opencode/inbox.md` for loose ideas.
- **Backlog in issues (2026-11-25):** the tracker was seeded from `requirements.md`: #9 third-party
  decks, #10 audio in `.txt`, #11 history in the app, #12 real autostart, #13 real full screen
  and #14 installer signing. AnkiConnect is left for a `grill-with-docs` session.
- **Third-party Anki decks (#9):** modern Anki export fixtures cover Basic, reversed cards,
  text Cloze, multiple decks and a dedicated audio field. The importer flattens cloze text,
  accepts empty media maps and preserves per-note deck metadata and source IDs on reimport.
- **History in the app (2026-11-25):** #11 adds the paginated `review_history` command with direction,
  result and date filters, and a table in Progress with “load more” and an empty state; CSV
  export does not change and there is no new migration.
- **Audio in `.txt` (2026-11-25):** #10 reads `[sound:…]` (also inside HTML) from the audio column
  of the mapping, copies from a media folder chosen in the dialog (by default, the sibling
  with the `.txt` name) and reuses `media::store_bytes`: SHA-1, name by hash and deduplication.
  A missing file adds `audio_missing` and a warning, without aborting the import.
- **Single instance (2026-11-25):** #18 registers `tauri-plugin-single-instance` before the rest of
  the plugins. The second run delivers its arguments and exits without opening SQLite or the scheduler; the
  first shows and focuses the main window. Verified by launching the exe twice: a single
  process, the second exits in 0.05 s and the window stays in the foreground.
- **Windows desktop E2E (#34):** `npm run test:e2e:windows` tests synthetic deck import,
  practice and review history after restarting with an isolated temporary profile. WebdriverIO
  supplies the file path at the picker boundary; the native file picker is outside this test.
  The test keeps its Tauri window hidden and verifies its visibility in both processes.

## Current phase

**SRS maximum review interval (#23):** migration v5 backfills each existing word's maximum
interval from its current interval. Reviews persist the highest interval reached, including
after a lapse, without changing scheduling or the interface.

**Phase 2 — Polish: completed.**

Completed in the phase:

- **Full SM-2** with quality 1-5, `ease`, learning steps and tests.
- **Three appearance modes**: popup, persistent widget and native toast.
- **Mode selector** in Settings and “Mode” submenu in the tray.
- **No focus stealing**: show + returning focus to the previous window.
- **Full-screen/presentation detection** with `SHQueryUserNotificationState`.
- **Versioned migrations** with `PRAGMA user_version` (added `review_log.quality`).
- **Normalization validated** against 2,629 real deck answers, including romaji.
- **Romaji input** (Hepburn, kunrei, long vowels and は→wa).
- **Own icon** generated with `tauri icon` (言 seal).
- **Distribution:** MSI and NSIS installers generated with `npm run tauri build`.
- **Retry** in the practice empty state and on import errors.
- **Extended statistics** with progress per lesson and difficult words.
- **Data management**: orphan audio cleanup and backup to folder.

Phase 1 still works: import, bidirectional review, tray, scheduler, popup,
progress and settings.

**Phase 3 — Extras: in progress.** R3-04a (gamification) completed:

- `profile` record with XP, level and best streak (migration v2).
- XP only for correct answers weighted by `quality`; levels by thresholds.
- Flexible streak and configurable weekly goal (default 30).
- Opt-in weekly goal notification, subject to pause/DND/full screen.

## Verified metrics

| Metric | Value |
|---|---|
| Notes in `Genki 1.apkg` | 1,076 |
| Unique items imported | 1,012 |
| Deduplicated notes | 64 |
| Audio files extracted | 827 |
| Reused audio references | 47 |
| Templates (`is_pattern`) | 66 |
| Import warnings | 0 |
| RAM at rest | ~33 MB |
| Rust unit tests | 94 OK (+4 ignored: import, normalization, progress and real note types) |
| Real answers validated | 2,629 |
| Backup | SQLite + 827 audio files |
| Typecheck / frontend build | OK |

## Functional verification performed

**Phase 1**

- Real import from the app: 1,076 → 1,012 items, 827 audio files, 0 warnings.
- ES → JP challenge: `estudiante de tercer año` → `三年生` accepted, with reading `さんねんせい`, gloss
  `Tres - Año - Vida` and audio.
- Progress and settings screen.
- Popup launched by the scheduler.
- App shutdown and process release.

**Phase 2**

- SM-2: tests for learning step, growth, failure and `ease` floor.
- Mode switch popup → widget → toast → popup from Settings, applied live.
- Persistent widget rendered at 340×248 with an active challenge and “Hide” button.
- Popup rendered at 392×280 with an active challenge.
- `practice_now` respects the selected mode.
- Toast launched without error; the plugin records the click (`tap`) to open the popup.
- **Focus:** `GetForegroundWindow` before and after showing the popup returns the same handle;
  the popup does not steal focus.
- Migration of the existing database: `user_version = 1` and `quality` column present.
- **Normalization:** test against the real deck with **2,629 answers** (kanji, readings, romaji and
  meanings) with no failures.
- **Statistics:** progress per lesson and difficult words rendered with real data.
- **Backup:** export to folder with `kotoba.sqlite3` and 827 audio files.
- **Restore:** full library wipe and restore from the backup; back to 1,012
  items and 827 audio files.
- **History:** CSV export of `review_log` with local date, reading, meaning, direction,
  result, quality and latency.
- **Import progress:** `import:progress` event every 25 notes and callback test.
- **Logs:** `tauri-plugin-log` writes to `%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`.
- **Icon:** 言 seal generated in all desktop sizes with `tauri icon`.
- **Installer:** `npm run tauri build` generates MSI (3.74 MiB) and NSIS (2.70 MiB).
- **Retry:** button in the practice empty state and on import errors.
- **Media cleanup:** unit test that keeps the referenced audio and deletes the orphan.

**Phase 3**

- **Gamification R3-04a:** profile, XP by quality, level, streak/best streak and weekly goal
  rendered in Progress.
- 3 consecutive correct answers → **9 XP**; the level stays at 1 (threshold 100).
- **Weekly goal notification:** suppressed with DND active; emitted with DND disabled
  (verified in `kotoba.log`: `objetivo semanal completado: 12 repasos`).
- **Gamification R3-04b:** catalog of 16 achievements, achievements panel with progress and unlock
  date, dominance per lesson with configurable threshold and `✦ Achievement` feedback on unlock.
- **Migration v3:** `achievements` table and `profile.weekly_goals_done` column; verified on
  an existing v2 database with a dedicated test.
- **Achievements measured in the app:** `first_step` on the first review and `reviews_25` on reaching 25;
  notification emitted with `milestone_notifications` and recorded in the log.
- **Gamification R3-04c:** streak savers (max. 2) earned on completing the weekly goal,
  automatic repair of a gap (can be disabled), level titles and a curve from 0 to 5,800 XP.
- **Migration v4:** `profile.streak_savers` and `streak_repairs` table; tested on a v2 database.
- **Saver measured in the app:** goal 26 → review 26, `saver_earned = true`, 1 saver,
  `weekly_goal_1` achievements and notice `objetivo semanal completado: 26 repasos`.
- **RF2-07 UX and accessibility:** dark theme `auto`/light/dark following Windows, language
  Spanish/English/Japanese in interface, tray and notifications, and interface scale 90-125 %.
- **Keyboard:** “skip to content” link, `aria-current`, `Enter` to check/advance and `Esc`
  to close the popup; visible focus and `prefers-reduced-motion` respected.
- **Verified in the app:** live light/dark theme, English and Japanese texts, 110 % scale in
  Progress and popup, and streak bar with correct Japanese formatting.
- **RF2-09 new words ordered:** new words follow `lesson → category → id`; verified with the real deck:
  `L00 / 01 Saludos` complete and jump to `L00 / 02 Numeros 0 100`.
- **Limit and priority:** `new_per_day = 10` and `new_mix_percent = 0` by default; separate
  counters for due reviews and new words; the scheduler does not open challenges if nothing is available.
- **RF-04 / RF2-06 manual mapping:** prior inspection of `.txt` and `.apkg`, mapping by columns or by
  note type (reading, kanji, meaning, gloss, audio, UID, tags, lesson and subsection),
  row preview and adjustable automatic detection.
- **Verified with the real deck:** type `Genki Model ES` with fields `kana`, `kanjis`, `tipo`,
  `español`, `significado`, `sonido`, `id_orden` and `uid`; reimport correct while keeping 1,012
  items. Anki's `unicase` collation was registered to read the field names.
- **Single instance:** with kotoba already running, a second run of the exe exits with code 0 in
  0.05 s, leaves a single `kotoba` process and brings the main window to the front
  (`GetForegroundWindow` = “kotoba”).

## Test data on the machine

The app has the deck imported in `%APPDATA%\com.kotoba.app` and **3 test reviews**, **9 XP**,
level 1 “Seed”, streak/best 1, **1 achievement** (`first_step`), **946 new available**,
**3 new today out of 10**, 0 due reviews and 0 savers. Weekly goal 30, dominance per
lesson 90 % and milestone notifications disabled. The mode was left at `popup`, DND at
22:00–08:00 and appearance at `auto`/Spanish/100 %.

To leave it clean:

- **Settings → Reset progress**: keeps the words, deletes history and intervals.
- **Settings → Delete library**: also deletes the words and forces a reimport.
- Full manual wipe: close the app and delete `%APPDATA%\com.kotoba.app`.

## Observed development environment

| Item | Value |
|---|---|
| Windows | 10/11, DPI 250 % |
| Node.js | 26.13.1 (Scoop) |
| npm | 11.20.0 |
| Rust | 1.100.0 |
| Target | `x86_64-pc-windows-msvc` |
| Visual Studio | 2022 |
| WebView2 | installed |

Note: in new sessions you may need to add Node to `PATH`:

```powershell
$env:Path = "C:\Users\Javier\scoop\apps\nodejs-lts\current;$env:Path"
```

## Technical debt and risks

| Priority | Topic | Detail |
|---|---|---|
| Medium | Real full screen | Implemented, but not tested against a real game/presentation |
| Low | Accessibility | Focus and keyboard navigation can be improved |
| Low | Gamification | Achievements, dominance and savers implemented; retention metrics outside the MVP |
| Low | Code signing | The CI pipeline and releases already work; the MSI/NSIS still needs signing |

## Recommended next steps

1. **Close Phase 1-2 loose ends:** #12 real autostart and #13 real full screen (manual
   verification, no code).
2. **Phase 3:** #9 third-party decks; then a grill session for
   AnkiConnect.
3. **Distribution:** #14 installer signing.

## How to update this document

- When closing a phase or a block of work, update metrics and technical debt.
- Add a row to “Functional verification” only if it was actually tested.
- Move completed requirements from `requirements.md` and reflect them here.
