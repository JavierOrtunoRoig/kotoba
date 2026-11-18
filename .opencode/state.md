# Estado del proyecto

Última verificación: **2026-11-18**.

## Fase actual

**Fase 1 — MVP técnico: implementada y verificada.**

Funciona de punta a punta con el mazo real: importación, repaso bidireccional, bandeja, scheduler,
popup, progreso y ajustes.

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
| Tests unitarios Rust | 13 OK |
| Typecheck / build frontend | OK |

## Verificación funcional realizada

- Importación real desde la app (no solo test): 1.076 → 1.012 ítems, 827 audios, 0 avisos.
- Reto ES → JP: `estudiante de tercer año` → `三年生` aceptado, con lectura `さんねんせい`, glosa
  `Tres - Año - Vida` y audio.
- Pantalla de progreso con precisión, racha, dominadas, pendientes y gráfico de 14 días.
- Pantalla de ajustes con frecuencia, DND, contenido, audio y datos.
- Popup lanzado automáticamente por el scheduler con un reto.
- Cierre de la app y liberación de procesos.

## Datos de prueba en el equipo

La app ya tiene importado el mazo en `%APPDATA%\com.kotoba.app` y **1 repaso de prueba** registrado.

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
| Alta | No robar foco | El popup se crea con `focused(false)`, pero falta `WS_EX_NOACTIVATE` para garantizarlo |
| Alta | Detección de pantalla completa | No implementada; los retos pueden aparecer sobre juegos/presentaciones |
| Alta | SM-2 completo | El SRS actual es provisional y no usa `ease` |
| Media | Normalización | Casos límite con partículas, `...`, paréntesis y variantes de IME |
| Media | Importación bloqueante | El comando es síncrono; el usuario solo ve un spinner |
| Media | Migraciones | Solo `CREATE TABLE IF NOT EXISTS`; falta versionado de esquema |
| Media | Limpieza de medios | Borrar/reimportar no elimina audios huérfanos de `media/` |
| Media | `.txt` sin audio | El fallback no puede extraer mp3 |
| Baja | Icono | Sigue el icono por defecto de Tauri |
| Baja | Logging | No hay log a archivo ni pantalla de diagnóstico |
| Baja | Accesibilidad | Foco y navegación por teclado mejorables |
| Baja | CI / firma | No hay pipeline ni instalador firmado |

## Próximos pasos recomendados

1. **Fase 2.1 — No molestar real:** `WS_EX_NOACTIVATE` + detección de pantalla completa.
2. **Fase 2.2 — Modos:** widget flotante y toast nativo, con selector en Ajustes.
3. **Fase 2.3 — SRS:** SM-2 completo con calidad de respuesta y tests.
4. **Fase 2.4 — Normalización:** tests sobre las 1.012 entradas reales y casos de IME.
5. **Fase 2.5 — Robustez:** migraciones, logging y limpieza de medios.
6. **Fase 3:** AnkiConnect, más mazos, distribución e instalador.

## Cómo actualizar este documento

- Al cerrar una fase o un bloque de trabajo, actualizar métricas y deuda técnica.
- Añadir una fila a “Verificación funcional” solo si se probó de verdad.
- Mover requisitos completados de `requirements.md` y reflejarlos aquí.
