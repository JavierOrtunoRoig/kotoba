---
name: dependabot-prs
description: Manages kotoba's Dependabot PRs — list bumps, bring the branch, resolve Cargo conflicts, run tests and merge only with green CI. Use when the user asks to fix, review or approve Dependabot PRs or dependency bumps.
---

# Dependabot PRs

AI automation in the loop. The scripts do the deterministic part (GitHub API, git, manifest
conflicts, tests); the AI decides the code changes and validates before merging.

## When to use

- "Fix the Dependabot PRs", "merge the bumps", "review outdated dependencies".
- Do not use for human PRs or for bumps that change APIs: those require judgment, but the
  verification flow still applies.

## Flow

All scripts run from any directory in the repo with
`powershell -NoProfile -File .opencode/skills/dependabot-prs/scripts/<script>.ps1 ...`.
They get the token with `git credential fill`; `gh auth login` is not needed.

1. **List.** `list-prs.ps1 [-All]` → JSON with number, title, branch, sha, `ci` and `mergeable`.
   By default it only returns bot PRs with the title `bump X from Y to Z`. `-All` includes the rest.
   - `ci: success` → candidate for direct merge.
   - `ci: pending` → wait with `checks.ps1 -Pr N -Wait`.
   - `ci: failure` → continue to step 2.

2. **Prepare branch.** `prepare.ps1 -Pr N`:
   - `fetch` of `main` and the PR branch, `checkout -B <branch> origin/<branch>` (requires a clean tree).
   - merge of `origin/main`.
   - `Cargo.lock`: takes the version from `main` and regenerates it with `cargo check` (since the PR
     manifest asks for another version).
   - `Cargo.toml`: if there is a conflict, keeps the PR version for the keys the PR changed and the
     `main` one for the others (works with `x = "v"` and with `x = { version = "v", ... }`).
   - Closes the merge with a commit. Prints JSON with `conflictsFixed`, `lockRegenerated` and the
     result of `cargo check`. **Does not push.**

3. **Fix code.** If `cargoCheck.ok` is `false`, read `cargoCheck.tail`: they are API errors
   (e.g. rand 0.10: `thread_rng` → `rng`, `gen_range` → `random_range`, `RngExt` trait). Edit,
   commit and re-run `cargo check` until it passes. If `prepare.ps1` threw an error for conflicts
   unrelated to Cargo, resolve them by hand before continuing.

4. **Verify.** `verify.ps1 -Pr N` runs `cargo check`, `cargo test --lib` and
   `npm run typecheck`. It saves `$env:TEMP\kotoba-dependabot\N.verify.json` with `passed`, `headSha`
   and the output of each step. Read the JSON: if any `ok` is `false`, fix and repeat. Adds the
   Scoop Node directory to PATH if `node` is not visible.

5. **Merge.** `merge.ps1 -Pr N [-Approve]`:
   - requires green verification for the current local HEAD, a clean tree and a mergeable PR;
   - pushes the branch;
   - waits for the CI of the new sha to be green (timeout 15 min, poll every 20 s);
   - optionally sends an `APPROVE` review, and does a squash merge with the API.
   - `checks.ps1 -Pr N [-Wait] [-TimeoutSeconds S]` lets you inspect the CI separately.

## Rules

- Never push to `main`, nor force-push to Dependabot branches.
- Never merge with red CI, a failed local verification or open conflicts.
- If the PR branch comes from a fork, `prepare.ps1` refuses: handle it manually.
- A major version change can break the API. Trust the compiler, do not assume the bump
  is manifest-only.
- If `verify.ps1` passes but the CI fails, do not retry the merge: read the check and go back to step 3.
- After merging several bumps in a row, the other branches become outdated; run
  `prepare.ps1` again if "Pull Request has merge conflicts" appears.

## Notes

- Versions to sync for a release: `package.json`, `src-tauri/Cargo.toml`,
  `src-tauri/tauri.conf.json` (the latter rules in the `release.yml` workflow).
- The current repo status and build commands are in `.opencode/state.md` and
  `.opencode/commands.md`.
