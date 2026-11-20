# Base de conocimiento de kotoba

Esta carpeta guarda el contexto importante del proyecto para que no se pierda entre sesiones,
personas o agentes. **No es configuración de OpenCode**: es documentación versionada del proyecto.

La visión de producto sigue estando en [`../docs/plan.md`](../docs/plan.md).

## Índice

| Documento | Para qué sirve |
|---|---|
| [`commands.md`](commands.md) | Comandos de desarrollo, build, tests, inspección de datos y diagnóstico |
| [`decisions.md`](decisions.md) | Decisiones de arquitectura y producto, con su motivo y consecuencias |
| [`requirements.md`](requirements.md) | Requisitos implementados, criterios de aceptación y backlog futuro |
| [`architecture.md`](architecture.md) | Módulos, esquema SQLite, flujo de importación, comandos IPC y UI |
| [`state.md`](state.md) | Estado actual, métricas verificadas, datos de prueba y deuda técnica |

## Reglas de mantenimiento

1. **Actualizar al cambiar el comportamiento**, no al final del proyecto.
2. Las decisiones nuevas se añaden a `decisions.md` con un identificador `D-0xx`.
3. Los requisitos nuevos se añaden a `requirements.md` con un identificador `RF-xx` / `RF2-xx`.
4. Los comandos que se usen más de una vez van a `commands.md`.
5. Los commits siguen [Conventional Commits](https://www.conventionalcommits.org/):
   `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.

## Estado resumido

- **Fase 1 (MVP técnico): implementada y verificada** el 2026-11-18.
- **Fase 2 (pulido): completada.** Hecho: SM-2 completo, popup/widget/toast, selector de modo,
  no robar foco, detección de pantalla completa, migraciones versionadas, normalización validada
  contra 2.629 respuestas reales (incluido romaji), estadísticas por lección/palabras difíciles,
  copia de seguridad, restauración, historial CSV, progreso de importación, logs a archivo, icono
  propio, reintento de errores, instaladores MSI/NSIS y limpieza de audios huérfanos.
- **RF2-07 completada:** tema oscuro automático/claro/oscuro, interfaz en español/inglés/japonés,
  escala 90-125 %, navegación por teclado y foco visible.
- **Fase 3 (extras): en curso.** R3-04a/b/c completadas: XP por calidad, niveles con títulos, racha
  flexible con salvavidas, objetivo semanal, 16 logros y maestría por lección.
- Importación real del mazo `Genki 1.apkg`: **1.076 notas → 1.012 ítems**, **827 audios**,
  **66 plantillas**.
- Windows only, Tauri 2 + React 19 + SQLite, sin red.
- Pendiente: AnkiConnect, más mazos, firma/CI.
