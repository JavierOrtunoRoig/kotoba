---
name: release-kotoba
description: Kotoba-specific release workflow. Bumps package.json, src-tauri/Cargo.toml and src-tauri/tauri.conf.json together, updates README version references, writes an English-only CHANGELOG.md, commits, tags and publishes the GitHub release. Use when releasing kotoba.
---

# Kotoba Release

Use this instead of the generic `release-skills` skill for kotoba. It enforces the
three version files and an English-only changelog.

## When to use

- "release kotoba", "new version", "bump version", "publish release".

## Files that must stay in sync

- `package.json` → `$.version`
- `src-tauri/Cargo.toml` → `package.version`
- `src-tauri/tauri.conf.json` → `$.version`
- `README.md` → installer names and paths containing the version

## Workflow

### 1. Detect last tag and changes

```powershell
$lastTag = git tag --sort=-v:refname | Select-Object -First 1
git log "$lastTag..HEAD" --oneline
```

### 2. Determine version bump

- `--major` / `--minor` / `--patch` if provided.
- Breaking change → major.
- `feat:` present → minor.
- Otherwise → patch.

Current version is read from `src-tauri/tauri.conf.json`.

### 3. Update all three version files

Set the same `X.Y.Z` in:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

### 4. Update README.md

Replace every occurrence of the old version string (e.g. `0.1.1`) in installer
names and bundle paths with the new one.

### 5. Generate English changelog

Only maintain `CHANGELOG.md` (English). Prepend a section:

```markdown
## X.Y.Z - YYYY-MM-DD

### Features
- ...

### Fixes
- ...
```

Omit empty sections. Do not create or update any other `CHANGELOG*.md`.

### 6. Confirm with the user

Show the proposed version, commit list, changelog preview, and whether to push
and publish a GitHub Release.

### 7. Commit, tag and push

```powershell
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md CHANGELOG.md
git commit -m "chore(release): vX.Y.Z"
git tag -a vX.Y.Z -F <release-notes-file>
git push origin main
git push origin vX.Y.Z
```

The annotated tag message must be the new changelog section.

### 8. Verify

Wait for the `release.yml` workflow and confirm it attaches MSI, NSIS, their
`.sig` files and `latest.json`.

## Notes

- Do not add Co-Authored-By to the release commit.
- `tauri.conf.json` is the version validated by `release.yml`; never leave it
  behind.
- Keep the changelog English-only for kotoba.
