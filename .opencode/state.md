# Estado del proyecto

Última verificación: **2026-11-18**.

## Fase actual

**Fase 2 — Pulido: en curso.**

Completado en esta iteración:

- **SM-2 completo** con calidad 1-5, `ease`, pasos de aprendizaje y tests.
- **Tres modos de aparición**: popup, widget persistente y toast nativo.
- **Selector de modo** en Ajustes y submenú “Modo” en la bandeja.
- **No robar foco**: show + devolución del foco a la ventana anterior.
- **Detección de pantalla completa/presentación** con `SHQueryUserNotificationState`.
- **Migraciones versionadas** con `PRAGMA user_version` (añadido `review_log.quality`).
- **Normalización validada** contra 2.629 respuestas reales del mazo, incluido romaji.
- **Entrada en romaji** (Hepburn, kunrei, vocales largas y は→wa).
- **Estadísticas ampliadas** con progreso por lección y palabras difíciles.
- **Gestión de datos**: limpieza de audios huérfanos y copia de seguridad a carpeta.

La Fase 1 sigue funcionando: importación, repaso bidireccional, bandeja, scheduler, popup,
progreso y ajustes.

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
| Tests unitarios Rust | 24 OK (+3 ignored: importación, normalización y progreso reales) |
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
- **Limpieza de medios:** test unitario que conserva el audio referenciado y borra el huérfano.

## Datos de prueba en el equipo

La app tiene importado el mazo en `%APPDATA%\com.kotoba.app` y **2 repasos de prueba** (anteriores
a SM-2, con `quality = 0`). El modo se dejó en `popup`.

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
| Baja | Icono | Sigue el icono por defecto de Tauri |
| Baja | Errores con reintento | Los errores se muestran, pero no hay reintento automático |
| Baja | Accesibilidad | Foco y navegación por teclado mejorables |
| Baja | CI / firma | No hay pipeline ni instalador firmado |

## Próximos pasos recomendados

1. **Fase 2.6 — Robustez:** manejo de errores con reintento.
2. **Fase 2.7 — Distribución:** icono propio, instalador y CI.
3. **Fase 2.8 — Pantalla completa real:** probar contra un juego/presentación.
4. **Fase 3:** AnkiConnect, más mazos y gramática.

## Cómo actualizar este documento

- Al cerrar una fase o un bloque de trabajo, actualizar métricas y deuda técnica.
- Añadir una fila a “Verificación funcional” solo si se probó de verdad.
- Mover requisitos completados de `requirements.md` y reflejarlos aquí.
