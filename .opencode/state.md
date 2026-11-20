# Estado del proyecto

Última verificación: **2026-11-18**.

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
| Tests unitarios Rust | 36 OK (+3 ignored: importación, normalización y progreso reales) |
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

## Datos de prueba en el equipo

La app tiene importado el mazo en `%APPDATA%\com.kotoba.app` y **26 repasos de prueba**, **57 XP**,
nivel 1 «Semilla», racha/mejor 1, **3 logros desbloqueados** (`first_step`, `reviews_25`,
`weekly_goal_1`), **1 salvavidas**, objetivo semanal 30, maestría por lección 90 % y notificaciones
de hito desactivadas. El modo se dejó en `popup`, el DND en 22:00–08:00 y la apariencia en
`auto`/español/100 %.

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
| Media | `.txt` sin audio | El fallback no puede extraer mp3 |
| Media | Pantalla completa real | Implementada, pero no probada contra un juego/presentación real |
| Baja | Accesibilidad | Foco y navegación por teclado mejorables |
| Baja | Gamificación | Logros/maestría y salvavidas implementados; métricas de retención fuera del MVP |
| Baja | CI / firma | El instalador funciona, pero no está firmado ni hay pipeline |

## Próximos pasos recomendados

1. **Fase 3:** AnkiConnect, más mazos y gramática.
2. **Distribución:** firmar el instalador y añadir CI.
3. **Pantalla completa real:** probar contra un juego/presentación.

## Cómo actualizar este documento

- Al cerrar una fase o un bloque de trabajo, actualizar métricas y deuda técnica.
- Añadir una fila a “Verificación funcional” solo si se probó de verdad.
- Mover requisitos completados de `requirements.md` y reflejarlos aquí.
