# Requisitos

Leyenda: `[x]` implementado · `[~]` parcial · `[ ]` pendiente.

Los requisitos funcionales usan `RF-xx`; los no funcionales, `RNF-xx`. Los de fases futuras usan
`RF2-xx` / `RF3-xx`.

---

# Fase 1 — MVP técnico (implementado)

## RF-01 — Residencia en la bandeja

- [x] Icono en la bandeja con menú: Practicar ahora, Abrir kotoba, Pausar/Reanudar, Salir.
- [x] Click izquierdo abre la ventana principal.
- [x] Cerrar la ventana principal la oculta; no termina el proceso.
- [x] El estado de pausa se persiste y se refleja en el menú.
- [x] La ventana principal solo se muestra al arrancar si la biblioteca está vacía.

## RF-02 — Arranque con Windows

- [x] Plugin `tauri-plugin-autostart` integrado.
- [x] Toggle en Ajustes → Sistema.
- [~] Falta verificarlo en un arranque real de Windows (registro/entrada de inicio).

## RF-03 — Importación `.apkg`

- [x] Selector de archivo nativo.
- [x] Lectura de ZIP, `collection.anki21b`, zstd y SQLite schema 18.
- [x] Fallback a `collection.anki21` / `collection.anki2`.
- [x] Mapeo por nombre de campo con fallback posicional.
- [x] Extracción de audio con verificación SHA-1 y deduplicación por hash.
- [x] Resumen de importación: notas, ítems, audios, plantillas y avisos.
- [x] Idempotencia por `source_uid`.
- [x] Deduplicación por lectura + significado, conservando `item_sources`.

**Aceptación verificada:** `Genki 1.apkg` → 1.076 notas, 1.012 ítems, 827 audios, 66 plantillas,
0 avisos.

## RF-04 — Importación `.txt` (fallback)

- [x] Parseo de cabeceras `#separator` y `#tags column`.
- [x] Mapeo posicional de Genki (8 columnas) y modo reducido (3 columnas).
- [x] Sin extracción de audio (el `.txt` solo trae referencias).
- [~] No hay pantalla de mapeo manual de columnas.

## RF-05 — Ejercicio bidireccional

- [x] JP → ES: se muestra kanji (o kana si no hay kanji) y se escribe el significado.
- [x] ES → JP: se muestra el significado y se escribe en japonés (kanji o kana).
- [x] Entrada con IME nativa del sistema (`lang="ja"`).
- [x] Dirección aleatoria o alternada (`direction_mode`).
- [x] Enter envía sin interferir con la composición del IME.

## RF-06 — Normalización de respuestas (básica)

- [x] NFKC, minúsculas, sin diacríticos, sin espacios ni puntuación final.
- [x] ES → JP acepta kanji y variantes de lectura separadas por `／`.
- [x] Acepta formas con y sin paréntesis opcionales (`おやすみ（なさい）`).
- [x] Quita `*` y `〜` para comparar.
- [x] JP → ES acepta acepciones separadas por `;`.
- [x] JP → ES acepta la forma sin paréntesis ni `...`.
- [~] Quedan casos límite (partículas en el significado, variantes de IME, romaji).

## RF-07 — Feedback inmediato

- [x] Indicador correcto/incorrecto.
- [x] Respuesta esperada principal y variantes aceptadas.
- [x] Lectura en kana.
- [x] Significado.
- [x] Glosa de kanji (`significado`) cuando existe.
- [x] Audio bajo demanda.
- [x] Próximo intervalo (“Vuelve en …”).

## RF-08 — Audio

- [x] Audio automático en JP → ES si está activado.
- [x] Audio después de responder en ES → JP.
- [x] Reproducción desde bytes locales (sin servidor ni asset protocol).
- [x] Si falta el audio, la tarjeta sigue funcionando.

## RF-09 — Modo popup

- [x] Ventana frameless, transparente, always-on-top y sin taskbar.
- [x] Se crea bajo demanda y se reutiliza.
- [x] Posicionada en la esquina inferior derecha del monitor.
- [x] Arrastrable desde la cabecera.
- [x] Cerrar con Escape o con la X oculta la ventana sin registrar repaso.
- [~] No robar foco al aparecer: la ventana se crea con `focused(false)`, pero falta el flag Win32
  `WS_EX_NOACTIVATE` para garantizarlo.

## RF-10 — Scheduler

- [x] Intervalo aleatorio entre mínimo y máximo.
- [x] Primer reto ~75 s después de arrancar.
- [x] Comprobación cada 10 s.
- [x] No lanza otro reto si el popup sigue visible.
- [x] Se detiene si la app está pausada, en DND o se alcanzó el máximo diario.

## RF-11 — No molestar

- [x] Horario configurable con hora inicio/fin.
- [x] Soporta rangos que cruzan la medianoche.
- [x] Toggle de activación.

## RF-12 — Límite diario

- [x] `max_per_day` configurable (0 = sin límite).
- [x] Solo cuentan los repasos respondidos; cerrar el popup no cuenta.

## RF-13 — SRS SM-2

- [x] Estado por ítem en `srs_state`.
- [x] Calidad 1-5 derivada de acierto y latencia.
- [x] Pasos de aprendizaje (10 min, 1 día) y crecimiento por `ease`.
- [x] Fallo reinicia `reps`, vuelve a 10 min e incrementa `lapses`.
- [x] `ease` se actualiza con la fórmula SM-2 y suelo 1.3.
- [x] Tests de aprendizaje, crecimiento, fallo y suelo de `ease`.

## RF-14 — Estadísticas

- [x] Total de palabras, repasos totales y repasos de hoy.
- [x] Precisión de hoy.
- [x] Racha de días.
- [x] Palabras dominadas (intervalo ≥ 7 días).
- [x] Pendientes (respetando `include_patterns`).
- [x] Actividad de los últimos 14 días.

## RF-15 — Persistencia local

- [x] SQLite con WAL y claves foráneas.
- [x] Tablas `items`, `item_sources`, `review_log`, `srs_state`, `settings`.
- [x] Audio en `%APPDATA%\com.kotoba.app\media`.
- [x] Ajustes persistidos como JSON.

## RF-16 — Gestión de datos

- [x] Reimportar mazo sin perder el progreso de las palabras existentes.
- [x] Reiniciar progreso (borra historial e intervalos).
- [x] Borrar biblioteca (borra palabras, fuentes y progreso).

---

# Requisitos no funcionales

## RNF-01 — RAM

- [x] Objetivo < ~120 MB en reposo.
- [x] Medido en la verificación de Fase 1: **~33 MB** con la app en bandeja.

## RNF-02 — Privacidad

- [x] Sin red, sin telemetría, sin cuentas.
- [x] Todo el contenido vive en el equipo del usuario.

## RNF-03 — No interrumpir

- [~] No robar foco: parcial (ver RF-09).
- [ ] Detección de apps a pantalla completa (Fase 2).

## RNF-04 — Plataforma

- [x] Windows 10/11 con WebView2.
- [x] Build MSVC.
- [ ] Instalador firmado y CI (Fase 3).

## RNF-05 — Mantenibilidad

- [x] Módulos Rust separados (`db`, `import`, `media`, `text`, `scheduler`, `shell`, `tray`).
- [x] Tests unitarios de normalización, parser de medios e importador.
- [ ] Migraciones versionadas (ahora se usa `CREATE TABLE IF NOT EXISTS`).
- [ ] Logging estructurado y manejo de errores visible al usuario.

---

# Fase 2 — Pulido

## RF2-01 — Los 3 modos de aparición

- [x] Popup (Fase 1).
- [x] Widget flotante persistente, always-on-top, arrastrable y con retos en vivo.
- [x] Toast nativo de Windows (`tauri-plugin-notification`); al hacer click abre el popup.
- [x] Selector de modo en Ajustes y submenú “Modo” en la bandeja.
- [x] El modo se aplica al guardar y al arrancar (si es widget, aparece al iniciar).

## RF2-02 — No molestar de verdad

- [x] Detección de pantalla completa, presentaciones, juegos y modo silencio con
  `SHQueryUserNotificationState`.
- [x] El popup/widget se muestran y se devuelve el foco a la ventana anterior.
- [ ] Pausa automática durante llamadas o pantalla completa (opcional).

## RF2-03 — SRS SM-2 completo

- [x] Usar `ease` y calidad de respuesta (no solo acierto/fallo).
- [x] Intervalos con factor de facilidad.
- [x] `due_at` se guarda en UTC; las estadísticas usan hora local.
- [x] Tests de la progresión.

## RF2-04 — Normalización robusta

- [ ] Manejar partículas y anotaciones del significado de forma más fina.
- [ ] Aceptar romaji si el usuario lo escribe.
- [ ] Normalizar variantes de IME (medias anchuras, kana repetido, etc.).
- [ ] Tests con las 1.012 entradas reales.

## RF2-05 — Estadísticas ampliadas

- [ ] Progreso por lección (`L00`…`L12`).
- [ ] Palabras difíciles / más falladas.
- [ ] Historial detallado y exportable.
- [ ] Rachas y objetivos configurables.

## RF2-06 — Importación más cómoda

- [ ] Progreso de importación por eventos (no bloqueante).
- [ ] Pantalla de mapeo de campos para mazos no Genki.
- [ ] Limpieza de audios huérfanos al borrar/reimportar.
- [ ] Importación de mazos distintos a Genki 1.

## RF2-07 — UX y accesibilidad

- [ ] Icono e identidad visual propios (no el icono por defecto de Tauri).
- [ ] Navegación completa por teclado y foco visible.
- [ ] Textos de UI en japonés/inglés además de español.
- [ ] Ajustes de tamaño de fuente.

## RF2-08 — Robustez

- [x] Migraciones versionadas (`PRAGMA user_version` + `ensure_column`).
- [ ] Logging a archivo y pantalla de diagnóstico.
- [ ] Manejo de errores con reintento.
- [ ] Backup/exportación de la biblioteca y del progreso.

---

# Fase 3 — Extras

## RF3-01 — AnkiConnect

- [ ] Sincronización opt-in con Anki local.
- [ ] Detección de cambios en el mazo.
- [ ] No pisar el scheduling de Anki.

## RF3-02 — Más contenido

- [ ] Genki 2 y otros mazos.
- [ ] Gramática y partículas.
- [ ] Frases y listening.

## RF3-03 — Distribución

- [ ] Instalador MSI/NSIS firmado.
- [ ] Auto-update.
- [ ] CI con build y tests en Windows.

## RF3-04 — Gamificación

- [ ] Rachas visibles y logros.
- [ ] Objetivos semanales.
- [ ] Notificaciones de progreso.

---

# Fuera de alcance (por ahora)

- Cuentas, nube, sincronización multidispositivo.
- Redistribución del contenido del mazo.
- Edición de tarjetas dentro de kotoba.
- Versión móvil o web.
