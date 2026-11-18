# Arquitectura

## Vista general

```
┌─────────────────────────────┐
│  WebView2 (React + TS)      │
│  main / popup               │
└──────────────┬──────────────┘
               │ invoke / events
┌──────────────▼──────────────┐
│  Tauri 2 (Rust)             │
│  commands · tray · scheduler│
│  import · text · media      │
└──────────────┬──────────────┘
               │ rusqlite
┌──────────────▼──────────────┐
│  SQLite + media/            │
│  %APPDATA%\com.kotoba.app   │
└─────────────────────────────┘
```

Sin red. El frontend solo habla con Rust mediante comandos; Rust es el único que toca SQLite y los
archivos.

## Módulos Rust (`src-tauri/src/`)

| Módulo | Responsabilidad |
|---|---|
| `main.rs` | Entrada binaria; llama a `kotoba_lib::run()` |
| `lib.rs` | Estado de la app, comandos Tauri, setup, ventanas, plugins |
| `models.rs` | Structs serde compartidos con el frontend |
| `db.rs` | Conexión SQLite, esquema, settings, ítems, repasos, SRS y estadísticas |
| `import.rs` | Importador `.apkg` / `.txt`, deduplicación y extracción de audio |
| `media.rs` | Parser del mapa de medios (protobuf de Anki + fallback JSON) |
| `text.rs` | Limpieza de HTML, normalización y validación de respuestas |
| `scheduler.rs` | Bucle del planificador pasivo y cálculo de DND |
| `shell.rs` | Mostrar/ocultar ventanas, posicionar popup, toggle de pausa |
| `tray.rs` | Icono, menú y eventos de la bandeja |

## Estado compartido

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub media_dir: PathBuf,
}
```

Se registra con `app.manage(AppState { ... })` en `setup`. Los comandos lo reciben como
`State<AppState>`.

## Esquema SQLite

### `items`

| Columna | Tipo | Nota |
|---|---|---|
| `id` | INTEGER PK | |
| `source_uid` | TEXT UNIQUE | `uid` de Anki, clave de idempotencia |
| `kanji` | TEXT NULL | forma en kanji |
| `reading` | TEXT | kana |
| `meaning` | TEXT | significado principal en español (limpio) |
| `meaning_plain` | TEXT | significado normalizado para claves |
| `kanji_gloss` | TEXT NULL | glosa kanji a kanji (`significado`) |
| `audio_file` | TEXT NULL | nombre `<sha1>.mp3` dentro de `media/` |
| `lesson` | TEXT | `L00`…`L12` |
| `category` | TEXT | `01 Vocabulario`, `03 Partes Del Cuerpo`, … |
| `tags` | TEXT | tags de Anki separados por espacio |
| `is_pattern` | INTEGER | `〜`, `+`, `...` |
| `irregular` | INTEGER | `*` en la lectura |
| `dedup_key` | TEXT UNIQUE | `normalize_key(reading) + "|" + normalize_key(meaning)` |
| `created_at` | INTEGER | Unix UTC |

### `item_sources`

`item_id`, `source_uid`, `lesson`, `category`. Permite que un ítem deduplicado recuerde todas las
notas de Anki que lo originaron.

### `review_log`

`id`, `item_id`, `shown_at`, `answered_at`, `result` (1/0), `latency_ms`, `direction`
(`jp_to_es` / `es_to_jp`).

### `srs_state`

`item_id` PK, `interval_minutes`, `ease` (2.5 por ahora), `due_at`, `reps`, `lapses`.

### `settings`

`key` / `value`. Toda la configuración se guarda como JSON bajo la clave `settings`.

### Índices

`idx_srs_due`, `idx_review_answered`, `idx_items_pattern`.

## Flujo de importación `.apkg`

1. Abrir el ZIP.
2. Buscar `collection.anki21b` → descomprimir zstd → escribir SQLite temporal → abrir con
   `rusqlite`. Fallback a `collection.anki21` / `collection.anki2`.
3. Leer `fields` (por `mid`), `cards` + `decks` (para lección/categoría) y `notes` (`flds`, `tags`).
4. Leer `media` → zstd → protobuf (`MediaEntry { name, size, sha1 }`) o JSON antiguo.
5. Para cada nota:
   - separar `flds` por `\x1f`;
   - mapear campos por nombre (fallback posicional);
   - limpiar HTML/entidades y normalizar;
   - calcular `is_pattern`, `irregular` y `dedup_key`;
   - extraer el audio referenciado (`[sound:...]`), verificar SHA-1 y escribirlo en
     `media/<sha1>.<ext>`;
   - insertar o reutilizar el ítem y registrar `item_sources`.
6. Commit y devolver `ImportSummary`.

El audio se extrae una sola vez por nombre; las referencias repetidas cuentan como `audio_reused`.

## Flujo de repaso

1. El scheduler (hilo propio) despierta cada 10 s.
2. Si toca, llama a `shell::show_popup`.
3. El popup recibe el evento `challenge:show` y pide `next_challenge`.
4. `db::next_item` elige un ítem vencido (o cualquiera si no hay vencidos) excluyendo el actual y
   filtrando plantillas.
5. El usuario responde → `submit_answer` valida, registra en `review_log`, actualiza `srs_state` y
   devuelve `AnswerResult`.
6. El frontend muestra el feedback, el audio y el próximo intervalo.
7. “Siguiente” pide otro reto; “Ahora no” oculta el popup sin registrar nada.

## Comandos IPC

| Comando | Entrada | Salida |
|---|---|---|
| `bootstrap` | — | `Bootstrap { item_count, settings, stats }` |
| `get_settings` | — | `Settings` |
| `update_settings` | `settings` | `Settings` saneado |
| `get_stats` | — | `Stats` con actividad de 14 días |
| `import_deck` | `path` | `ImportSummary` |
| `next_challenge` | `excludeId?` | `Challenge \| null` |
| `submit_answer` | `itemId`, `direction`, `answer`, `latencyMs` | `AnswerResult` |
| `get_audio` | `itemId` | `number[] \| null` (bytes del mp3) |
| `reset_progress` | — | `Stats` |
| `clear_library` | — | `void` |
| `practice_now` | — | `void` (abre el popup) |
| `close_popup` | — | `void` |
| `show_main_window` | — | `void` |

Tauri convierte los argumentos `snake_case` a `camelCase` en JS (`itemId`, `latencyMs`,
`excludeId`).

## Frontend (`src/`)

| Archivo | Responsabilidad |
|---|---|
| `App.tsx` | Detecta si la ventana es `popup` por `getCurrentWindow().label` |
| `MainApp.tsx` | Bootstrap, rail de navegación, vistas y estado global |
| `PracticePanel.tsx` | Tarjeta de repaso, input, feedback, audio y atajos |
| `PopupApp.tsx` | Versión compacta para la ventana popup; escucha `challenge:show` |
| `ImportPanel.tsx` | Selector `.apkg`, resumen y avisos |
| `StatsPanel.tsx` | Métricas y gráfico de actividad |
| `SettingsPanel.tsx` | Frecuencia, DND, contenido, audio, autostart y datos |
| `api.ts` | Tipos y wrappers de `invoke` |
| `audio.ts` | Reproducción de bytes locales con Blob URL |
| `Icons.tsx` | Iconos SVG inline |
| `styles.css` | Sistema visual “washi” y estilos de todas las vistas |

El popup y la ventana principal comparten el mismo bundle; se distinguen por el label de la ventana.

## Scheduler

- Hilo `std::thread` que duerme 10 s entre comprobaciones.
- `next_run` es un `Instant`; el primer reto se programa a los ~75 s.
- Antes de mostrar: comprobar que el popup no esté visible, que haya ítems, que no esté pausado,
  que no sea DND y que no se haya alcanzado `max_per_day`.
- Tras mostrar: `next_run = now + rand(min..=max) minutos`.

## SRS provisional

```
INTERVALS_MINUTES = [10, 60, 360, 1440, 4320, 10080, 20160, 43200]
```

- Acierto: `reps += 1`, intervalo = `INTERVALS[min(reps, 7)]`.
- Fallo: `reps = 0`, intervalo = 10 min, `lapses += 1`.
- `ease` se guarda a 2.5 y aún no se usa.

## Settings por defecto

| Clave | Valor |
|---|---|
| `interval_min_minutes` | 10 |
| `interval_max_minutes` | 25 |
| `max_per_day` | 40 (0 = sin límite) |
| `paused` | `false` |
| `dnd_enabled` | `true` |
| `dnd_start` / `dnd_end` | `22:00` / `08:00` |
| `audio_autoplay` | `true` |
| `direction_mode` | `random` (`random` \| `alternate`) |
| `include_patterns` | `false` |
| `launch_at_startup` | `false` |

## Datos locales

```
%APPDATA%\com.kotoba.app\
  kotoba.sqlite3
  kotoba.sqlite3-wal
  kotoba.sqlite3-shm
  media\
    <sha1>.mp3
```

## Ventanas y capacidades

- `main`: 1120×780 lógica, mínimo 820×600, oculta hasta decidir si hay biblioteca.
- `popup`: 392×280, frameless, transparente, always-on-top, skip-taskbar, sin foco inicial.
- Capabilities: `core:default`, show/hide/close/set-focus/start-dragging, `dialog:default`,
  `autostart:default`.
- `csp: null` por comodidad de desarrollo; no hay contenido remoto.
- El audio no usa `assetProtocol`: se sirve como bytes por IPC y se reproduce con Blob URL.
