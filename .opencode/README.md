# kotoba knowledge base

This folder holds the project's important context so it is not lost between sessions, people or
agents. **It is not OpenCode configuration**: it is versioned project documentation.

The product vision still lives in [`../docs/plan.md`](../docs/plan.md).

## Index

| Document | What it is for |
|---|---|
| [`commands.md`](commands.md) | Development, build, test, data inspection and diagnostics commands |
| [`workflows.md`](workflows.md) | Which flow to use depending on the request, definition of done and branch conventions |
| [`inbox.md`](inbox.md) | Loose ideas waiting to become an issue |
| [`decisions.md`](decisions.md) | Architecture and product decisions, with their rationale and consequences |
| [`requirements.md`](requirements.md) | Implemented requirements, acceptance criteria and future backlog |
| [`architecture.md`](architecture.md) | Modules, SQLite schema, import flow, IPC commands and UI |
| [`state.md`](state.md) | Current state, verified metrics, test data and technical debt |
| [`skills/dependabot-prs/`](skills/dependabot-prs/SKILL.md) | OpenCode skill for managing Dependabot PRs (list, prepare, verify and merge) |
| [`../docs/agents/`](../docs/agents/) | Configuration of the engineering skills: issue tracker, triage labels and domain docs |
| [`../.agents/skills/`](../.agents/skills/) | Engineering skills from `mattpocock/skills`, versioned and pinned in `skills-lock.json` |

## Maintenance rules

1. **Update when behaviour changes**, not at the end of the project. Local skills live
   in `skills/<name>/SKILL.md` with their scripts in `skills/<name>/scripts/`.
2. New decisions are added to `decisions.md` with a `D-0xx` identifier.
3. New requirements are added to `requirements.md` with an `RF-xx` / `RF2-xx` identifier.
4. Commands used more than once go to `commands.md`.
5. Commits follow [Conventional Commits](https://www.conventionalcommits.org/):
   `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.

## Summary status

- **Phase 1 (technical MVP): implemented and verified** on 2026-11-18.
- **Phase 2 (polish): complete.** Done: full SM-2, popup/widget/toast, mode selector, no focus
  stealing, fullscreen detection, versioned migrations, normalization validated against 2,629 real
  answers (including romaji), per-lesson/difficult-word statistics, backup, restore, CSV history,
  import progress, file logging, custom icon, error retry, MSI/NSIS installers and orphan audio
  cleanup.
- **RF2-07 complete:** automatic/light/dark theme, interface in Spanish/English/Japanese,
  90-125% scale, keyboard navigation and visible focus.
- **RF2-09 complete:** new words by lesson/subsection order, reviews first (with optional mixing)
  and a daily limit for new words.
- **RF-04 / RF2-06 complete:** manual mapping of columns and fields per notetype when importing
  `.txt` and `.apkg`, with inspection and preview.
- **Phase 3 (extras): in progress.** R3-04a/b/c complete: XP by quality, levels with titles,
  flexible streak with lifelines, weekly goal, 16 achievements and per-lesson mastery.
- Real import of the `Genki 1.apkg` deck: **1,076 notes → 1,012 items**, **827 audio files**,
  **66 templates**.
- Windows only, Tauri 2 + React 19 + SQLite, no network.
- Pending: AnkiConnect, more decks and installer signing.
- CI and releases in `.github/workflows/`; the `v0.1.1` tag generates MSI and NSIS.
