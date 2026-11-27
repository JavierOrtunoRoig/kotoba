# Commands

All commands run from the repository root and are intended for **PowerShell** on Windows.

## Environment

| Tool | Version used | Note |
|---|---|---|
| Node.js | 26.13.1 | Installed with Scoop; if `node` is not on `PATH`, add the directory |
| npm | 11.20.0 | npm is used (there is a `package-lock.json`) |
| Rust | 1.100.0 | Target `x86_64-pc-windows-msvc` |
| MSVC | Visual Studio 2022 | Required to link Tauri |

If `node` is not available in the session:

```powershell
$env:Path = "C:\Users\Javier\scoop\apps\nodejs-lts\current;$env:Path"
node --version
```

## Install dependencies

```powershell
npm install
```

## Development

```powershell
npm run tauri dev
```

This starts Vite at `http://localhost:1420` and opens the Tauri app. The main window shows on its
own if the library is empty; if there are already items, the app lives in the tray.

The development binary is a console build and loads the `devUrl`: **do not enable “Launch at
startup”** in that build. If it runs without Vite (for example, from autostart), it opens a console
with the logs and an `ERR_CONNECTION_REFUSED` page. The option is disabled in debug builds; for
autostart, install the MSI/NSIS from a release and enable it from that app.

## Build

```powershell
npm run tauri build
```

Production build (slower, LTO enabled). The installer is output to `src-tauri/target/release/bundle/`.

With `bundle.createUpdaterArtifacts` enabled, the release build requires the signing key:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$env:USERPROFILE\.tauri\kotoba-updater\kotoba.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<key password>"
npm run tauri build
```

## Automatic updates

- Plugin: `tauri-plugin-updater` + `tauri-plugin-process`.
- Endpoint: `https://github.com/JavierOrtunoRoig/kotoba/releases/latest/download/latest.json`.
- `latest.json` points to the NSIS (`updaterJsonPreferNsis: true`): per-user install with no admin.
- Public key: `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`.
- Private key: `%USERPROFILE%\.tauri\kotoba-updater\kotoba.key` (never in the repo; `*.key` ignored).
- GitHub secrets: `TAURI_SIGNING_PRIVATE_KEY` (content) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- Generate a new pair (breaks updates for existing installs):

  ```powershell
  npx tauri signer generate -w "$env:USERPROFILE\.tauri\kotoba-updater\kotoba.key"
  ```

- Check manually: Settings → System → “Check for updates”.

## Publishing a release

1. Bump the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Create and push the tag:

   ```powershell
   git tag v0.1.1
   git push origin v0.1.1
   ```

3. GitHub Actions builds and publishes to **Releases** the MSI, the NSIS, their `.sig` files and
   `latest.json` for the updater. The `release.yml` workflow fails if the tag does not match the
   version in `tauri.conf.json` or if the signing secrets are missing. It can also be triggered
   manually from the Actions tab by specifying the tag.

CI (`ci.yml`) runs on every push to `main` and on every pull request.

## Checks

```powershell
npm run typecheck          # TypeScript
npm run build              # TypeScript + Vite
cd src-tauri; cargo check  # Rust without linking
cd src-tauri; cargo test --lib
```

## Import test against the real deck

The test is marked `ignored` because it depends on a local file:

```powershell
cd src-tauri
cargo test --lib imports_real_genki_deck -- --ignored --nocapture
```

It checks: 1,076 notes, at least 1,000 items, at least 700 audio files and 1,076 sources. It prints
the full `ImportSummary`.

### Normalization test against the real deck

```powershell
cd src-tauri
cargo test --lib normalizes_real_deck_answers -- --ignored --nocapture
```

It validates **2,629 answers** generated from the real deck (kanji, readings, romaji and meanings)
and fails if any plausible form is rejected or if garbage is accepted.

## Inspecting the local database

```powershell
$db = "$env:APPDATA\com.kotoba.app\kotoba.sqlite3"
sqlite3 $db "select count(*) from items;"
sqlite3 $db "select count(*) from review_log;"
sqlite3 $db "select count(*) from srs_state;"
sqlite3 $db "select key, value from settings;"
sqlite3 $db "select id, reading, meaning, lesson, category from items limit 20;"
```

## Backup

From **Settings → Data → Export backup** you pick a folder and `kotoba-backup-<timestamp>/` is
created with `kotoba.sqlite3` and `media/`. The database is copied after a
`PRAGMA wal_checkpoint(TRUNCATE)` so it is consistent.

**Restore backup** replaces the current database and audio with those from the chosen folder.
**Export history** saves a CSV of `review_log` with local date, reading, meaning, direction, result,
quality and latency.

## Deleting local data

First close the app (tray menu → Quit).

```powershell
# Full deletion: words, audio, progress and settings
Remove-Item "$env:APPDATA\com.kotoba.app" -Recurse -Force
```

Alternatives from the app:

- **Settings → Reset progress**: deletes `review_log` and `srs_state`, keeps the words.
- **Settings → Delete library**: deletes words, sources and progress; forces a reimport.

## Logs

Path: `%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`.

**Settings → System → Open folder** opens the directory. The plugin writes at `Info` level; imports,
backups, restores and scheduler startup leave a trace.

## Diagnosing the UI with CDP

Useful for verifying screens without relying on desktop screenshots.

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
npm run tauri dev
```

In another terminal:

```powershell
Invoke-RestMethod "http://127.0.0.1:9222/json" | Select-Object title, url, webSocketDebuggerUrl
```

From CDP you can evaluate JavaScript (`Runtime.evaluate`) and capture the view
(`Page.captureScreenshot`). The `popup` window and the `main` window appear as two separate targets;
they are told apart by `window.innerWidth` and `document.body.classList.contains("popup-window")`.

## Git

```powershell
git status
git add <files>
git commit -m "type(scope): summary" -m "optional detail"
git log --oneline -10
```

Convention: **Conventional Commits**. Types used: `feat`, `fix`, `docs`, `refactor`, `test`,
`chore`.

## Common problems

| Symptom | Cause / fix |
|---|---|
| `cargo` cannot find `link.exe` | Open a VS 2022 Developer PowerShell or install Build Tools with MSVC |
| `node` is not recognized | Add the Scoop directory to `PATH` (see above) |
| The `.apkg` fails to import | It must be the new Anki format (`collection.anki21b`); re-export with a recent Anki |
| The popup does not appear | Check that “Pause”, the do-not-disturb window or the daily maximum is not active |
| The app shows no window | If there is already a library, it starts in the tray: open it from the icon or the menu |
