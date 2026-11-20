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

## RF-04 - Importación `.txt` (fallback)

- [x] Parseo de cabeceras `#separator`, `#tags column` y `#columns`.
- [x] Mapeo posicional de Genki (8 columnas) y modo reducido (3 columnas).
- [x] Sin extracción de audio (el `.txt` solo trae referencias).
- [x] Pantalla de mapeo manual de columnas con vista previa de filas y asignación de lectura,
  kanji, significado, glosa, UID, etiquetas, lección y subapartado.

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
- [x] CI con build y tests en Windows.
- [x] Release automática con MSI y NSIS al empujar una etiqueta `v*`.
- [ ] Instalador firmado.

## RNF-05 — Mantenibilidad

- [x] Módulos Rust separados (`db`, `import`, `media`, `text`, `scheduler`, `shell`, `tray`).
- [x] Tests unitarios de normalización, parser de medios e importador.
- [ ] Migraciones versionadas (ahora se usa `CREATE TABLE IF NOT EXISTS`).
- [ ] Logging estructurado y manejo de errores visible al usuario.

---

# Fase 2 — Pulido

> Estado: completada. Los `[ ]` restantes son opcionales o de distribución (firma/CI).

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

- [x] Manejar partículas, anotaciones y placeholders del significado.
- [x] Aceptar romaji (Hepburn, kunrei, vocales largas y は→wa).
- [x] Normalizar variantes de IME (NFKC, medias anchuras, espacios ideográficos).
- [x] Tests con las 1.012 entradas reales: **2.629 respuestas validadas** (incluye romaji).

## RF2-05 — Estadísticas ampliadas

- [x] Progreso por lección (`L00`…`L12`).
- [x] Palabras difíciles / más falladas.
- [ ] Historial detallado y exportable.
- [ ] Rachas y objetivos configurables.

## RF2-06 - Importación más cómoda

- [x] Progreso de importación por eventos (`import:progress` cada 25 notas).
- [x] Inspección previa del `.apkg` por tipo de nota con nombres de campo y filas de ejemplo.
- [x] Mapeo manual de campos por tipo de nota: lectura, kanji, significado, glosa, audio, UID,
  etiquetas, lección y subapartado.
- [x] Mapeo manual de columnas en `.txt` (incluye `#columns` cuando existe).
- [x] Detección automática de campos por nombre con fallback posicional; el usuario puede ajustar.
- [x] Registro de la colación `unicase` de Anki para leer `fields.name` y `notetypes.name`.
- [~] Validación con mazos reales adicionales de varios tipos de nota (soportado por diseño).
- [x] Limpieza de audios huérfanos al borrar o reimportar.
- [ ] Importación de mazos distintos a Genki 1.

## RF2-07 - UX y accesibilidad

- [x] Icono e identidad visual propios (sello 言).
- [x] Navegación por teclado y foco visible: enlace “saltar al contenido”, `aria-current` en la
  navegación, `Enter` para comprobar y pasar al siguiente reto, `Esc` para cerrar el popup.
- [x] Textos de UI en español, inglés y japonés (interfaz, bandeja y notificaciones).
- [x] Ajuste de tamaño de la interfaz (90-125 %), con ventanas flotantes escaladas.
- [x] **Tema oscuro** con opción automática, clara y oscura; `auto` sigue el tema de Windows.
- [x] Respeto a `prefers-reduced-motion` y etiquetas accesibles en controles e iconos.

## RF2-08 - Robustez

- [x] Migraciones versionadas (`PRAGMA user_version` + `ensure_column`).
- [x] Copia de seguridad a carpeta (SQLite + `media/`).
- [x] Restaurar una copia desde la app (reemplaza base y audio).
- [x] Logging a archivo (`%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`).
- [x] Manejo de errores con reintento (práctica e importación).
- [x] Exportar historial de repasos (CSV).

## RF2-09 - Orden y límite de palabras nuevas

- [x] Una palabra es nueva si no tiene `srs_state`; no hay flag aparte.
- [x] Las nuevas siguen el orden del mazo: lección, subapartado y orden de importación
  (`ORDER BY lesson, category, id`).
- [x] Al importar `.apkg` se leen las notas en orden estable (`ORDER BY notes.id`).
- [x] Repasos vencidos antes que nuevas por defecto (`new_mix_percent = 0`).
- [x] Mezcla configurable 0-100 %: porcentaje de nuevas cuando también hay repasos pendientes.
- [x] Límite diario de nuevas (`new_per_day`, default 10; 0 = ninguna) contado por la primera
  respuesta del día.
- [x] Contadores separados: repasos vencidos, nuevas disponibles y nuevas introducidas hoy.
- [x] El scheduler no abre retos si no hay repasos vencidos ni nuevas dentro del límite.
- [x] Tests de orden por lección/subapartado, prioridad de repasos, mezcla y límite diario.

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

- [x] CI con build y tests en Windows.
- [x] Release automática con MSI y NSIS al empujar una etiqueta `v*`.
- [ ] Instalador MSI/NSIS firmado.
- [ ] Auto-update.

## RF3-04 — Gamificación

**Carácter:** híbrido. Silenciosa por defecto; las notificaciones de hito son opt-in y respetan
pausa, DND y pantalla completa. Racha flexible. Todo local, sin cuentas ni social.

**Guardarraíles:** el XP solo premia calidad (no volumen); la racha no genera culpa; los premios no
pueden incentivar respuestas rápidas/malas que rompan el SM-2.

### R3-04a — MVG (implementado)

- [x] Perfil `profile` con `xp_total`, `level` y `streak_best` (migración v2).
- [x] XP solo por aciertos: quality 3→+1, 4→+2, 5→+3; fallo → 0.
- [x] Niveles por umbrales de XP (12 niveles).
- [x] Racha flexible + mejor racha histórica.
- [x] Objetivo semanal configurable (default 30; 0 = sin objetivo).
- [x] Pantalla Progreso: nivel, XP, barra de nivel, racha/mejor racha y objetivo semanal.
- [x] Feedback: `+X XP` y aviso de subida de nivel en práctica y popup.
- [x] Notificación nativa al completar el objetivo semanal, opt-in, respeta pausa/DND/pantalla
  completa y nunca avisa de “racha en peligro”.
- [x] Tests de XP, niveles, racha, objetivo y migración.

### R3-04b — Logros y maestría (implementado)

- [x] Catálogo de 16 logros: repasos, palabras dominadas, racha, lecciones maestras, nivel 5 y
  objetivo semanal cumplido.
- [x] Progreso de cada logro visible y desbloqueo permanente.
- [x] Maestría por lección con umbral configurable (default 90 %): porcentaje de palabras con
  intervalo ≥ 7 días sobre el total de la lección.
- [x] Pantalla de logros en Progreso y feedback inmediato al desbloquear.
- [x] Notificaciones opt-in al desbloquear, sujetas a pausa/DND/pantalla completa.
- [x] Migración v3 (`achievements`, `profile.weekly_goals_done`).
- [x] Tests de desbloqueo único y umbral de maestría.

### R3-04c — Refinamiento (implementado)

- [x] Salvavidas de racha: cada objetivo semanal completado da uno (máximo 2).
- [x] Reparación automática de un día perdido, opcional en Ajustes; nunca repara más de un
  hueco por repaso ni gasta salvavidas sin actividad reciente.
- [x] Salvavidas visibles en Progreso y feedback `+1 salvavidas` al ganarlos.
- [x] Curva de niveles más suave (80 → 5.800 XP) y títulos por tramo (Semilla, Brote, Raíz,
  Rama, Árbol, Bosque).
- [x] Migración v4 (`profile.streak_savers`, tabla `streak_repairs`) y tests de reparación,
  tope de salvavidas y migración.
- [ ] Métricas de retención a medio plazo: fuera del MVP local (sin telemetría ni nube); la
  pantalla de Progreso ya muestra actividad de 14 días y progreso por lección.

---

# Fuera de alcance (por ahora)

- Cuentas, nube, sincronización multidispositivo.
- Redistribución del contenido del mazo.
- Edición de tarjetas dentro de kotoba.
- Versión móvil o web.
