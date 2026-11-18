# Comandos

Todos los comandos se ejecutan desde la raíz del repositorio y están pensados para **PowerShell**
en Windows.

## Entorno

| Herramienta | Versión usada | Nota |
|---|---|---|
| Node.js | 26.13.1 | Instalado con Scoop; si `node` no está en `PATH`, añadir el directorio |
| npm | 11.20.0 | Se usa npm (hay `package-lock.json`) |
| Rust | 1.100.0 | Target `x86_64-pc-windows-msvc` |
| MSVC | Visual Studio 2022 | Necesario para enlazar Tauri |

Si `node` no está disponible en la sesión:

```powershell
$env:Path = "C:\Users\Javier\scoop\apps\nodejs-lts\current;$env:Path"
node --version
```

## Instalar dependencias

```powershell
npm install
```

## Desarrollo

```powershell
npm run tauri dev
```

Esto levanta Vite en `http://localhost:1420` y abre la app Tauri. La ventana principal se muestra
sola si la biblioteca está vacía; si ya hay ítems, la app vive en la bandeja.

## Build

```powershell
npm run tauri build
```

Build de producción (más lento, LTO activado). El instalador sale en `src-tauri/target/release/bundle/`.

## Comprobaciones

```powershell
npm run typecheck          # TypeScript
npm run build              # TypeScript + Vite
cd src-tauri; cargo check  # Rust sin enlazar
cd src-tauri; cargo test --lib
```

## Test de importación contra el mazo real

El test está marcado como `ignored` porque depende de un archivo local:

```powershell
cd src-tauri
cargo test --lib imports_real_genki_deck -- --ignored --nocapture
```

Comprueba: 1.076 notas, al menos 1.000 ítems, al menos 700 audios y 1.076 fuentes. Imprime el
`ImportSummary` completo.

### Test de normalización contra el mazo real

```powershell
cd src-tauri
cargo test --lib normalizes_real_deck_answers -- --ignored --nocapture
```

Valida **2.629 respuestas** generadas desde el mazo real (kanji, lecturas, romaji y significados) y
falla si alguna forma plausible es rechazada o si se acepta basura.

## Inspeccionar la base de datos local

```powershell
$db = "$env:APPDATA\com.kotoba.app\kotoba.sqlite3"
sqlite3 $db "select count(*) from items;"
sqlite3 $db "select count(*) from review_log;"
sqlite3 $db "select count(*) from srs_state;"
sqlite3 $db "select key, value from settings;"
sqlite3 $db "select id, reading, meaning, lesson, category from items limit 20;"
```

## Copia de seguridad

Desde **Ajustes → Datos → Exportar copia** se elige una carpeta y se crea
`kotoba-backup-<timestamp>/` con `kotoba.sqlite3` y `media/`. La base se copia tras un
`PRAGMA wal_checkpoint(TRUNCATE)` para que sea consistente.

**Restaurar copia** reemplaza la base y el audio actuales por los de la carpeta elegida.
**Exportar historial** guarda un CSV de `review_log` con fecha local, lectura, significado,
dirección, resultado, calidad y latencia.

## Borrar datos locales

Primero cerrar la app (menú de bandeja → Salir).

```powershell
# Borrado completo: palabras, audio, progreso y ajustes
Remove-Item "$env:APPDATA\com.kotoba.app" -Recurse -Force
```

Alternativas desde la app:

- **Ajustes → Reiniciar progreso**: borra `review_log` y `srs_state`, conserva las palabras.
- **Ajustes → Borrar biblioteca**: borra palabras, fuentes y progreso; obliga a reimportar.

## Logs

Ruta: `%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`.

Desde **Ajustes → Sistema → Abrir carpeta** se abre el directorio. El plugin escribe con nivel
`Info`; las importaciones, copias, restauraciones y el arranque del scheduler dejan traza.

## Diagnóstico de la UI con CDP

Útil para verificar pantallas sin depender de capturas de pantalla del escritorio.

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
npm run tauri dev
```

En otra terminal:

```powershell
Invoke-RestMethod "http://127.0.0.1:9222/json" | Select-Object title, url, webSocketDebuggerUrl
```

Desde CDP se puede evaluar JavaScript (`Runtime.evaluate`) y capturar la vista
(`Page.captureScreenshot`). La ventana `popup` y la ventana `main` aparecen como dos targets
distintos; se distinguen por `window.innerWidth` y `document.body.classList.contains("popup-window")`.

## Git

```powershell
git status
git add <archivos>
git commit -m "tipo(scope): resumen" -m "detalle opcional"
git log --oneline -10
```

Convención: **Conventional Commits**. Tipos usados: `feat`, `fix`, `docs`, `refactor`, `test`,
`chore`.

## Problemas típicos

| Síntoma | Causa / solución |
|---|---|
| `cargo` no encuentra `link.exe` | Abrir una Developer PowerShell de VS 2022 o instalar Build Tools con MSVC |
| `node` no se reconoce | Añadir el directorio de Scoop al `PATH` (ver arriba) |
| El `.apkg` falla al importar | Debe ser el formato nuevo de Anki (`collection.anki21b`); reexportar con Anki reciente |
| El popup no aparece | Comprobar que no está activo “Pausar”, el horario de no molestar o el máximo diario |
| La app no muestra ventana | Si ya hay biblioteca, se inicia en la bandeja: abrir desde el icono o el menú |
