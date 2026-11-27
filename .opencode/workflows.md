# Workflows

How to use the skills in `.agents/skills/` in kotoba: which flow applies to each request and what is
required to close. This document rules over the skills; they bring the how, here is the when.

## Agreed profile (2026-11-25)

| Decision | Value |
|---|---|
| Work enters through | loose ideas, bugs while using the app, `requirements.md` backlog, Dependabot |
| Pace | short, frequent sessions (30-60 min) |
| Integration | always branch + PR; one branch per issue |
| Issues | an issue for almost everything that changes behavior |
| Specs | only if the work does not fit in one session |
| Code review | full only on large PRs |
| Loose ideas | go to `inbox.md`; they become an issue when planning |
| Language | artifacts (code, commits, issues, PRs, specs, ADRs) in English; docs and conversation in Spanish |
| Autonomy | full technical; product, UX and scope are consulted |
| Done | local checks green, CI green, test for the change, `.opencode/` updated |
| Release | when closing a feature |

## The issue rule

- **Changes behavior → issue first.** Even a one-line bug.
- **Does not change behavior → direct PR, no issue.** Docs, version, formatting, CI.
- An issue is ready when it has: what, why, acceptance criteria and area.
- Our own issues are born with `ready-for-agent` (or `ready-for-human` if they require a human decision).
  They are not triaged: `/triage` is only for what comes from outside.

## Flow 0 — Capture an idea

1. Add an entry to [`inbox.md`](inbox.md) with date, area and one line of context.
2. Nothing else. No issue, no branch.
3. In the planning session: each idea becomes an issue or is discarded. It is marked `→ #N`
   in the inbox so it is not processed again.

## Flow 1 — Bug

1. Issue with reproduction steps, expected and actual behavior. Area in the title
   (`scheduler:`, `import:`, `ui:`, `updater:`).
2. Branch `fix/<issue>-slug`.
3. If the cause is not obvious, load the `diagnosing-bugs` skill **before** touching code: it requires
   a command that fails red for this bug. If it is obvious, `tdd` directly with the regression test.
4. Commit, PR, CI green, squash merge.
5. Update `state.md` if observable behavior changes.

## Flow 2 — Small change (one session)

A scoped fix or improvement that does change behavior but does not need a spec.

1. Issue (acceptance criteria in one or two lines).
2. Branch `feat/<issue>-slug` or `fix/<issue>-slug`.
3. Clean session: load `implement` with the issue number. It includes `tdd` and a review.
4. Local checks, commit, PR, CI green, merge.
5. Update `.opencode/state.md`.

## Flow 3 — Multi-session feature (main flow)

**Session 1 — think (a single window, no compaction):**

1. Load `grill-with-docs` on the idea. The interview sharpens the scope and leaves in the repo what
   is decided: `GLOSSARY.md` and `docs/adr/` are created only if terms or decisions that deserve
   it appear.
2. If a question can only be answered with code (state model, UI), stop and use `prototype`. It lives in
   a `prototype/<name>` branch and is referenced from the issue; it is not merged.
3. Load `to-spec`: publish the spec as an issue in English with the `ready-for-agent` label.
4. Load `to-tickets`: split the spec into vertical tickets, each one session long, with their
   dependencies as native GitHub blocking links. Review granularity and blocks before
   publishing.

**Sessions 2..N — build (one session per ticket):**

5. Pick a ticket with no open blocks. Branch `feat/<issue>-slug` from `main`.
6. Clean session: `implement <issue>`. Nothing from the previous session's context is needed.
7. Local checks, commit, PR. PR body: what, why, how it was tested, and the ticket it closes.
8. CI green, squash merge, delete branch.
9. Repeat with the next ticket at the frontier.

**Closing:**

10. `retro` on the last ticket's session or on the whole feature: what in the environment
    (AGENTS.md, checks, standards) would have prevented friction.
11. Release when the feature is complete and tested.

## Flow 4 — Trivial (docs, version, formatting, CI)

1. No issue. Branch `docs/...` or `chore/...`.
2. Change + CI green. No `/code-review`.

## Flow 5 — Dependabot

1. `dependabot-prs` skill: list, prepare the branch, resolve `Cargo.toml`/lockfile conflicts.
2. Green CI mandatory. Merge only then.
3. Bot PRs do not go through `/triage` or `/code-review`.

## Flow 6 — Release

Only when closing a feature (not on every PR).

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Update the README if the installer names change.
3. Commit `chore(release): vX.Y.Z` and push to `main`.
4. `git tag vX.Y.Z` + `git push origin vX.Y.Z`.
5. Wait for the release workflow and check that it attaches MSI, NSIS, their `.sig` and `latest.json`.

## Flow 7 — Technical debt

1. `improve-codebase-architecture` to surface deep-module opportunities.
2. The one you choose becomes an idea → Flow 3 (or Flow 2 if it is small).

## When the full flow is too much

| Situation | What to do |
|---|---|
| Fits in one session | Flow 2: issue + `implement` |
| Design question | `prototype`, never a spec |
| Huge and diffuse effort | `wayfinder`; then `to-spec` |
| You just want to understand the code | `codegraph_explore`, no flow |
| You need external data | `research` and return to the flow with the `.md` |

## Code review: when yes

Full review (`code-review`, two axes: Standards + Spec) when the PR:

- touches **more than one module** or exceeds ~200 lines of diff;
- changes the SQLite schema, migrations, IPC commands or the updater;
- touches `db.rs` (SM-2, SQL), `scheduler.rs` or release signing.

Otherwise green CI and re-reading the diff is enough. `implement` always does a quick review at the end;
on small PRs, that is the whole review.

## Definition of done

- [ ] `npm run typecheck` clean.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib` green.
- [ ] New or adjusted tests for the behavior that changes.
- [ ] CI green on the PR.
- [ ] `.opencode/state.md` updated; `decisions.md` if there is an architecture decision.
- [ ] UI changes tested by hand in `npm run tauri dev` (recommended, not blocking).

## Context hygiene

- `grill-with-docs` → `to-spec` → `to-tickets`: **a single window**, no `/compact` until the end.
- Each `implement` in a clean session. The ticket is self-contained.
- If the window approaches ~150k tokens before `to-tickets`, compact there, not mid-phase.

## Names

- Branches: `feat/<issue>-slug`, `fix/<issue>-slug`, `docs/slug`, `chore/slug`, `prototype/slug`.
- Commits: Conventional Commits in English (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
- Issues and PRs: title in English, one line; body with what/why/criteria.
