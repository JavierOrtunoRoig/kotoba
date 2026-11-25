# Estado del proyecto

Última verificación: **2026-11-25**.

**Repositorio público:** <https://github.com/JavierOrtunoRoig/kotoba> (código bajo MIT).

- README orientado a release pública, con capturas, privacidad, roadmap y resumen en inglés.
- Configuración de agentes versionada (`AGENTS.md`, `opencode.jsonc`, `.mcp.json`,
  `skills-lock.json`); el índice local `.codegraph/` queda ignorado.
- CI y releases en `.github/workflows/`: `ci.yml` valida TypeScript y Rust en cada push/PR, y
  `release.yml` publica MSI y NSIS al empujar una etiqueta `v*`. Release `v0.1.1` publicada.
- **Dependabot al día (2026-11-21):** los 7 PRs de la primera ronda están mergeados. `rand` 0.10
  obligó a migrar el scheduler a `rand::rng().random_range()` con el rasgo `RngExt`; `zip` 9,
  `zstd` 0.14, `sha1` 0.11, `windows-sys` 0.61, TypeScript 7 y las actions actualizadas sin
  cambios de código. Proceso automatizado como skill local: `skills/dependabot-prs/`.
- **Actualizaciones automáticas (2026-11-22):** `tauri-plugin-updater` + `tauri-plugin-process`, con
  `latest.json` en GitHub Releases y artefactos firmados con minisign. Aviso al arrancar y botón
  manual en Ajustes → Sistema. Clave privada fuera del repo y en secrets de GitHub.
- **Autostart solo en release (2026-11-22):** el binario debug usa el `devUrl` y abre consola. La UI
  deshabilita «Arrancar con Windows» cuando `Bootstrap.dev_build` es true para no registrar el exe de
  `target/debug` en el arranque de Windows.
- **Flujos de trabajo (2026-11-25):** skills de ingeniería de `mattpocock/skills` versionadas en
  `.agents/skills/`, con `.opencode/workflows.md` como playbook (issue → rama → PR, spec solo si es
  multi-sesión, review solo en PRs grandes) y `.opencode/inbox.md` para ideas sueltas.
- **Backlog en issues (2026-11-25):** el tracker se sembró desde `requirements.md`: #9 mazos de
  terceros, #10 audio en `.txt`, #11 historial en la app, #12 autostart real, #13 pantalla completa
  real y #14 firma de instaladores. AnkiConnect queda para una sesión de `grill-with-docs`.
- **Historial en la app (2026-11-25):** #11 añade el comando `review_history` paginado con filtros
  de dirección, resultado y fechas, y una tabla en Progreso con «cargar más» y estado vacío; la
  exportación CSV no cambia y no hay migración nueva.
- **Audio en `.txt` (2026-11-25):** #10 lee `[sound:…]` (también dentro de HTML) de la columna de
  audio del mapeo, copia desde una carpeta de medios elegida en el diálogo (por defecto, la hermana
  con el nombre del `.txt`) y reutiliza `media::store_bytes`: SHA-1, nombre por hash y deduplicación.
  Un archivo ausente suma `audio_missing` y un aviso, sin abortar la importación.

## Fase actual

**Fase 2 — Pulido: completada.**

Completado en la fase:

- **SM-2 completo** con calidad 1-5, `ease`, pasos de aprendizaje y tests.
- **Tres modos de aparición**: popup, widget persistente y toast nativo.
- **Selector de modo** en Ajustes y submenú “Modo” en la bandeja.
- **No robar foco**: show + devolución del foco a la ventana anterior.
- **Detección de pantalla completa/presentación** con `SHQueryUserNotificationState`.
- **Migraciones versionadas** con `PRAGMA user_version` (añadido `review_log.quality`).
- **Normalización validada** contra 2.629 respuestas reales del mazo, incluido romaji.
- **Entrada en romaji** (Hepburn, kunrei, vocales largas y は→wa).
- **Icono propio** generado con `tauri icon` (sello 言).
- **Distribución:** instaladores MSI y NSIS generados con `npm run tauri build`.
- **Reintento** en el estado vacío de práctica y en errores de importación.
- **Estadísticas ampliadas** con progreso por lección y palabras difíciles.
- **Gestión de datos**: limpieza de audios huérfanos y copia de seguridad a carpeta.

La Fase 1 sigue funcionando: importación, repaso bidireccional, bandeja, scheduler, popup,
progreso y ajustes.

**Fase 3 — Extras: en curso.** R3-04a (gamificación) completada:

- Perfil `profile` con XP, nivel y mejor racha (migración v2).
- XP solo por aciertos ponderados por `quality`; niveles por umbrales.
- Racha flexible y objetivo semanal configurable (default 30).
- Notificación de objetivo semanal opt-in, sujeta a pausa/DND/pantalla completa.

## Métricas verificadas

| Métrica | Valor |
|---|---|
| Notas en `Genki 1.apkg` | 1.076 |
| Ítems únicos importados | 1.012 |
| Notas deduplicadas | 64 |
| Audios extraídos | 827 |
| Referencias de audio reutilizadas | 47 |
| Plantillas (`is_pattern`) | 66 |
| Avisos de importación | 0 |
| RAM en reposo | ~33 MB |
| Tests unitarios Rust | 94 OK (+4 ignored: importación, normalización, progreso y tipos de nota reales) |
| Respuestas reales validadas | 2.629 |
| Copia de seguridad | SQLite + 827 audios |
| Typecheck / build frontend | OK |

## Verificación funcional realizada

**Fase 1**

- Importación real desde la app: 1.076 → 1.012 ítems, 827 audios, 0 avisos.
- Reto ES → JP: `estudiante de tercer año` → `三年生` aceptado, con lectura `さんねんせい`, glosa
  `Tres - Año - Vida` y audio.
- Pantalla de progreso y ajustes.
- Popup lanzado por el scheduler.
- Cierre de la app y liberación de procesos.

**Fase 2**

- SM-2: tests de paso de aprendizaje, crecimiento, fallo y suelo de `ease`.
- Cambio de modo popup → widget → toast → popup desde Ajustes, aplicado en caliente.
- Widget persistente renderizado a 340×248 con reto activo y botón “Ocultar”.
- Popup renderizado a 392×280 con reto activo.
- `practice_now` respeta el modo seleccionado.
- Toast lanzado sin error; el plugin registra el click (`tap`) para abrir el popup.
- **Foco:** `GetForegroundWindow` antes y después de mostrar el popup devuelve el mismo handle;
  el popup no roba el foco.
- Migración de la base existente: `user_version = 1` y columna `quality` presente.
- **Normalización:** test contra el mazo real con **2.629 respuestas** (kanji, lecturas, romaji y
  significados) sin fallos.
- **Estadísticas:** progreso por lección y palabras difíciles renderizados con datos reales.
- **Copia de seguridad:** exportación a carpeta con `kotoba.sqlite3` y 827 audios.
- **Restauración:** borrado completo de la biblioteca y restauración desde la copia; vuelve a 1.012
  ítems y 827 audios.
- **Historial:** exportación CSV de `review_log` con fecha local, lectura, significado, dirección,
  resultado, calidad y latencia.
- **Progreso de importación:** evento `import:progress` cada 25 notas y test del callback.
- **Logs:** `tauri-plugin-log` escribe en `%LOCALAPPDATA%\com.kotoba.app\logs\kotoba.log`.
- **Icono:** sello 言 generado en todos los tamaños de escritorio con `tauri icon`.
- **Instalador:** `npm run tauri build` genera MSI (3,74 MiB) y NSIS (2,70 MiB).
- **Reintento:** botón en el estado vacío de práctica y en errores de importación.
- **Limpieza de medios:** test unitario que conserva el audio referenciado y borra el huérfano.

**Fase 3**

- **Gamificación R3-04a:** perfil, XP por calidad, nivel, racha/mejor racha y objetivo semanal
  renderizados en Progreso.
- 3 aciertos consecutivos → **9 XP**; el nivel sigue en 1 (umbral 100).
- **Notificación de objetivo semanal:** suprimida con DND activo; emitida con DND desactivado
  (verificado en `kotoba.log`: `objetivo semanal completado: 12 repasos`).
- **Gamificación R3-04b:** catálogo de 16 logros, panel de logros con progreso y fecha de
  desbloqueo, maestría por lección con umbral configurable y feedback `✦ Logro` al desbloquear.
- **Migración v3:** tabla `achievements` y columna `profile.weekly_goals_done`; verificada sobre
  una base v2 existente con test dedicado.
- **Logros medidos en la app:** `first_step` al primer repaso y `reviews_25` al llegar a 25;
  notificación emitida con `milestone_notifications` y anotada en el log.
- **Gamificación R3-04c:** salvavidas de racha (máx. 2) ganados al cumplir el objetivo semanal,
  reparación automática de un hueco (desactivable), títulos de nivel y curva de 0 a 5.800 XP.
- **Migración v4:** `profile.streak_savers` y tabla `streak_repairs`; probada sobre una base v2.
- **Salvavidas medido en la app:** objetivo 26 → repaso 26, `saver_earned = true`, 1 salvavidas,
  logros `weekly_goal_1` y aviso `objetivo semanal completado: 26 repasos`.
- **RF2-07 UX y accesibilidad:** tema oscuro `auto`/claro/oscuro siguiendo Windows, idioma
  español/inglés/japonés en interfaz, bandeja y notificaciones, y escala de interfaz 90-125 %.
- **Teclado:** enlace “saltar al contenido”, `aria-current`, `Enter` para comprobar/pasar y `Esc`
  para cerrar el popup; foco visible y `prefers-reduced-motion` respetado.
- **Verificado en la app:** tema claro/oscuro en vivo, textos en inglés y japonés, escala 110 % en
  Progreso y popup, y barra de racha con formato japonés correcto.
- **RF2-09 nuevas ordenadas:** las nuevas siguen `lesson → category → id`; verificado con el mazo
  real: `L00 / 01 Saludos` completo y salto a `L00 / 02 Numeros 0 100`.
- **Límite y prioridad:** `new_per_day = 10` y `new_mix_percent = 0` por defecto; contadores
  separados de repasos vencidos y nuevas; el scheduler no abre retos si no hay nada disponible.
- **RF-04 / RF2-06 mapeo manual:** inspección previa de `.txt` y `.apkg`, mapeo por columnas o por
  tipo de nota (lectura, kanji, significado, glosa, audio, UID, etiquetas, lección y subapartado),
  vista previa de filas y detección automática ajustable.
- **Verificado con el mazo real:** tipo `Genki Model ES` con campos `kana`, `kanjis`, `tipo`,
  `español`, `significado`, `sonido`, `id_orden` y `uid`; reimportación correcta manteniendo 1.012
  ítems. Se registró la colación `unicase` de Anki para leer los nombres de campo.

## Datos de prueba en el equipo

La app tiene importado el mazo en `%APPDATA%\com.kotoba.app` y **3 repasos de prueba**, **9 XP**,
nivel 1 «Semilla», racha/mejor 1, **1 logro** (`first_step`), **946 nuevas disponibles**,
**3 nuevas hoy de 10**, 0 repasos vencidos y 0 salvavidas. Objetivo semanal 30, maestría por
lección 90 % y notificaciones de hito desactivadas. El modo se dejó en `popup`, el DND en
22:00–08:00 y la apariencia en `auto`/español/100 %.

Para dejarlo limpio:

- **Ajustes → Reiniciar progreso**: conserva las palabras, borra historial e intervalos.
- **Ajustes → Borrar biblioteca**: borra también las palabras y obliga a reimportar.
- Borrado total manual: cerrar la app y eliminar `%APPDATA%\com.kotoba.app`.

## Entorno de desarrollo observado

| Elemento | Valor |
|---|---|
| Windows | 10/11, DPI 250 % |
| Node.js | 26.13.1 (Scoop) |
| npm | 11.20.0 |
| Rust | 1.100.0 |
| Target | `x86_64-pc-windows-msvc` |
| Visual Studio | 2022 |
| WebView2 | instalado |

Nota: en sesiones nuevas puede hacer falta añadir Node al `PATH`:

```powershell
$env:Path = "C:\Users\Javier\scoop\apps\nodejs-lts\current;$env:Path"
```

## Deuda técnica y riesgos

| Prioridad | Tema | Detalle |
|---|---|---|
| Media | Pantalla completa real | Implementada, pero no probada contra un juego/presentación real |
| Baja | Accesibilidad | Foco y navegación por teclado mejorables |
| Baja | Gamificación | Logros/maestría y salvavidas implementados; métricas de retención fuera del MVP |
| Baja | Firma de código | El pipeline de CI y las releases ya funcionan; falta firmar el MSI/NSIS |

## Próximos pasos recomendados

1. **Cerrar cabos de Fase 1-2:** #12 autostart real y #13 pantalla completa real (verificación
   manual, sin código).
2. **Fase 3:** #9 mazos de terceros; después, sesión de grill para
   AnkiConnect.
3. **Distribución:** #14 firma de los instaladores.

## Cómo actualizar este documento

- Al cerrar una fase o un bloque de trabajo, actualizar métricas y deuda técnica.
- Añadir una fila a “Verificación funcional” solo si se probó de verdad.
- Mover requisitos completados de `requirements.md` y reflejarlos aquí.
