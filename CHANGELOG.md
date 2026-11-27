# Changelog

All notable changes to kotoba are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0 - 2026-11-27

### Features

- Import third-party Anki packages, including Basic, reversed and Cloze notes,
  multiple decks and bundled audio.
- Track the maximum review interval reached by each word, including across lapses.
- Show word and lesson maturity, lesson dominance and mature-word achievement progress.

## 0.2.0 - 2026-11-25

### Features

- Review history in the stats view: every review with its result, date, latency
  and SM-2 quality, filterable by direction, hit/miss and date range, loaded 25
  entries at a time.
- `.txt` imports can now attach audio. The field mapping dialog accepts a media
  folder, so audio is no longer limited to `.apkg` packages.

### Fixes

- Words are no longer served before their review is due. The practice view
  shows "Nothing to review yet" with the time until the next word instead.
- Only one kotoba instance runs at a time. Launching the app again brings the
  existing window to the front instead of starting a second scheduler and
  database connection.