# kotoba

> Entrenador de vocabulario japonés **pasivo** para escritorio — micro-retos que aparecen mientras trabajas.

App de escritorio para Windows que vive en la bandeja del sistema y, de vez en cuando, lanza un
micro-ejercicio de vocabulario (5-10 s). No se trata de sentarte a estudiar: la app te trae el repaso
a ti. Por detrás usa **repetición espaciada invisible** para que el contenido se retenga.

El vocabulario se importa desde el mazo de Anki **"Genki 1"** (formato `.apkg` o `.txt`).

## Estado

**Fase 1 — MVP técnico implementado.** Ver [`docs/plan.md`](docs/plan.md).

Lo que ya funciona:

- Scaffold **Tauri 2 + React + TypeScript + Vite**.
- **SQLite** local con `items`, `item_sources`, `review_log`, `srs_state` y `settings`.
- **Importador `.apkg`** (formato nuevo de Anki: SQLite schema 18 + zstd + mapa protobuf de medios),
  con extracción y verificación SHA-1 del audio.
- **Ejercicio bidireccional** ES ↔ JP con IME japonés, feedback y audio.
- **Bandeja del sistema** con menú (practicar, pausar, abrir, salir).
- **Scheduler** por intervalo aleatorio, máximo diario y horario de no molestar.
- **Ventana popup** flotante para los micro-retos.
- Pantallas de importación, progreso y ajustes.

Pendiente para la Fase 2: los 3 modos de aparición, detección de pantalla completa, normalización
más robusta, SRS SM-2 completo y estadísticas ampliadas.

## Stack

- [Tauri 2](https://tauri.app/) (Rust) sobre WebView2.
- React 19 + TypeScript + Vite.
- SQLite embebido con `rusqlite`.
- `zip` + `zstd` para leer los `.apkg`.

## Desarrollo

Requisitos: Node.js 20+, Rust stable y los Build Tools de Visual Studio (MSVC).

```powershell
npm install
npm run tauri dev
```

Build de producción:

```powershell
npm run tauri build
```

Comprobaciones:

```powershell
npm run typecheck
cd src-tauri; cargo test
```

El test de integración del importador usa el mazo real y está marcado como `ignored`:

```powershell
cd src-tauri; cargo test --lib imports_real_genki_deck -- --ignored --nocapture
```

## Importar el mazo

1. En Anki: *Archivo → Exportar → Paquete de mazo de Anki (`.apkg`)* del mazo "Genki 1".
2. En kotoba: **Ajustes → Reimportar mazo** (o la pantalla de bienvenida del primer arranque).
3. kotoba mapea por nombre los campos `kana`, `kanjis`, `español`, `significado`, `sonido` y `uid`,
   extrae el audio y deduplica las palabras repetidas entre capítulos.

Los datos se guardan en `%APPDATA%\com.kotoba.app` (SQLite + `media/`). Nada sale del equipo.

## Estructura

```
src/                 Frontend React (pantallas y componentes)
src-tauri/src/
  db.rs              Esquema SQLite, settings, SRS ligero y estadísticas
  import.rs          Importador .apkg / .txt
  media.rs           Parser del mapa de medios (protobuf + zstd)
  text.rs            Limpieza de HTML, normalización y validación de respuestas
  scheduler.rs       Planificador pasivo
  shell.rs           Ventanas y toggle de pausa
  tray.rs            Icono y menú de la bandeja
```

## Licencia

Uso personal. El contenido del mazo de Anki no se redistribuye con la app.
