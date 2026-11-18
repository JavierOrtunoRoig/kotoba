# Plan: kotoba — repaso pasivo de japonés (Genki 1)

## Resumen

Aplicación de escritorio para **Windows** que vive en la bandeja del sistema y lanza **micro-retos de
vocabulario** (5-10 s) en momentos aleatorios. El vocabulario sale del **mazo Anki "Genki 1"** (ya en
español). Estudio **pasivo**: no abres una app a estudiar, la app te lo trae a ti. Repetición espaciada
"invisible" por detrás para que realmente retenga.

## Decisiones cerradas

- **Stack**: Tauri 2 (Rust) + WebView2
- **OS**: solo Windows
- **MVP**: solo vocabulario (bidireccional ES↔JP)
- **Apariciones**: 3 modos configurables (widget / popup / toast)
- **Contenido**: export del mazo Anki "Genki 1" (definiciones ya en español)
- **Extracción**: importar desde archivo `.txt`/`.apkg` exportado de Anki

## Stack

- **Tauri 2** (Rust) + **WebView2** de Windows.
- Frontend: **React + TypeScript + Vite**.
- Backend Rust: bandeja, scheduler, gestión de ventanas.
- Storage: **SQLite** (`tauri-plugin-sql` o `rusqlite`).
- Notificaciones nativas: `tauri-plugin-notification`.

## Requisitos funcionales

1. **Residencia**: icono en bandeja con menú (Pausar/Reanudar · Modo · Estadísticas · Config · Salir) y
   arranque con Windows.
2. **Importación de contenido**: importar el vocabulario desde un **export `.txt`** (tab-separado) o
   **`.apkg`** del mazo "Genki 1". Mapeo de campos configurable en el primer import.
3. **Ejercicio bidireccional**: mostrar palabra en un idioma → escribirla en el otro (con IME japonés).
4. **Normalización de respuestas**: aceptar kana y kanji, tolerar variantes de lectura.
5. **Feedback inmediato** + lectura en kana de la palabra tras responder.
6. **3 modos de aparición** (configurables): widget flotante / popup / toast de sistema.
7. **No molestar**: horario, detección de app a pantalla completa, máximo por día.
8. **SRS pasiva** (SM-2 ligero) por ítem, invisible para el usuario.
9. **Estadísticas simples**: % aciertos, racha, palabras dominadas.

## Requisitos no funcionales

- RAM < ~120 MB en reposo (es una app siempre encendida).
- No interrumpir apps a pantalla completa ni robar el foco del teclado al aparecer.
- Persistencia local completa en SQLite (sin nube).

## Modelo de datos

```
items       → id, word (kanji), reading (kana), meaning (ES), lesson, tags
review_log  → id, item_id, shown_at, result, latency_ms, direction
srs_state   → item_id, interval, ease, due_at
settings    → mode, interval_range, do_not_disturb, max_per_day
```

## Flujo de importación desde Anki

1. En Anki: *Archivo → Exportar → Notas en texto plano (.txt)* (o `.apkg`) del mazo "Genki 1".
2. La app lee el archivo y muestra una **previsualización del mapeo de campos** (`word`/`reading`/
   `meaning` detectados por cabecera) para confirmar.
3. Guarda una copia en su propia SQLite con SRS independiente (Anki = estudio activo; esta app = repaso
   pasivo; no se pisan).

## UX de los 3 modos

- **Widget flotante**: ventana frameless, always-on-top, transparente, siempre visible proponiendo
  retos.
- **Popup**: ventana pequeña junto a la bandeja; responde o descarta, se va sola.
- **Toast**: notificación nativa de Windows que abres solo si quieres.

## Roadmap

1. **Fase 0** — confirmar campos exactos del export del mazo.
2. **Fase 1 — MVP técnico**: scaffold Tauri 2, bandeja, scheduler por intervalo, import `.txt`,
   ejercicio bidireccional, SQLite, un modo de ventana.
3. **Fase 2 — Pulido**: los 3 modos, no-molestar, normalización robusta, SRS, estadísticas.
4. **Fase 3 — Extras**: sincronización AnkiConnect, rachas, tipos de gramática/partículas, más mazos.

## Nota de licencia

El mazo de Anki es para **uso personal**; la app consume tu propio export y no redistribuye contenido
del libro.
