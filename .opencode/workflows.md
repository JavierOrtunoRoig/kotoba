# Flujos de trabajo

Cómo usar las skills de `.agents/skills/` en kotoba: qué flujo toca según la petición y qué se
exige para cerrar. Este documento manda sobre las skills; ellas traen el cómo, aquí está el cuándo.

## Perfil acordado (2026-11-25)

| Decisión | Valor |
|---|---|
| El trabajo entra por | ideas sueltas, bugs al usar la app, backlog de `requirements.md`, Dependabot |
| Ritmo | sesiones cortas y frecuentes (30-60 min) |
| Integración | siempre rama + PR; una rama por issue |
| Issues | issue para casi todo lo que cambia comportamiento |
| Specs | solo si el trabajo no cabe en una sesión |
| Code review | completo solo en PRs grandes |
| Ideas sueltas | van a `inbox.md`; se convierten en issue al planificar |
| Idioma | artefactos (código, commits, issues, PRs, specs, ADRs) en inglés; docs y conversación en español |
| Autonomía | técnica total; producto, UX y alcance se consultan |
| Terminado | checks locales verdes, CI verde, test del cambio, `.opencode/` actualizado |
| Release | al cerrar un feature |

## Regla del issue

- **Cambia comportamiento → issue primero.** Incluso un bug de una línea.
- **No cambia comportamiento → PR directo, sin issue.** Docs, versión, formato, CI.
- Un issue está listo cuando tiene: qué, por qué, criterio de aceptación y área.
- Los issues propios nacen con `ready-for-agent` (o `ready-for-human` si requiere decisión humana).
  No se triajan: `/triage` es solo para lo que llega de fuera.

## Flujo 0 — Capturar una idea

1. Añadir una entrada a [`inbox.md`](inbox.md) con fecha, área y una línea de contexto.
2. Nada más. Sin issue, sin rama.
3. En la sesión de planificación: cada idea se convierte en issue o se descarta. Se marca `→ #N`
   en el inbox para no volver a procesarla.

## Flujo 1 — Bug

1. Issue con pasos de reproducción, comportamiento esperado y real. Área en el título
   (`scheduler:`, `import:`, `ui:`, `updater:`).
2. Rama `fix/<issue>-slug`.
3. Si la causa no es evidente, cargar la skill `diagnosing-bugs` **antes** de tocar código: exige
   un comando que falle en rojo por este bug. Si es evidente, `tdd` directo con el test de regresión.
4. Commit, PR, CI verde, merge con squash.
5. Actualizar `state.md` si cambia el comportamiento observable.

## Flujo 2 — Cambio pequeño (una sesión)

Un arreglo o mejora acotada que sí cambia comportamiento pero no necesita spec.

1. Issue (criterio de aceptación en una o dos líneas).
2. Rama `feat/<issue>-slug` o `fix/<issue>-slug`.
3. Sesión limpia: cargar `implement` con el número de issue. Lleva dentro `tdd` y una revisión.
4. Checks locales, commit, PR, CI verde, merge.
5. Actualizar `.opencode/state.md`.

## Flujo 3 — Feature de varias sesiones (flujo principal)

**Sesión 1 — pensar (una sola ventana, sin compactar):**

1. Cargar `grill-with-docs` sobre la idea. La entrevista afila el alcance y deja en el repo lo que
   se decida: `GLOSSARY.md` y `docs/adr/` se crean solo si aparecen términos o decisiones que lo
   merezcan.
2. Si una duda solo se responde con código (modelo de estado, UI), parar y usar `prototype`. Vive en
   una rama `prototype/<nombre>` y se referencia desde el issue; no se mergea.
3. Cargar `to-spec`: publica la spec como issue en inglés con label `ready-for-agent`.
4. Cargar `to-tickets`: parte la spec en tickets verticales, cada uno de una sesión, con sus
   dependencias como enlaces nativos de bloqueo de GitHub. Revisar granularidad y bloqueos antes de
   publicar.

**Sesiones 2..N — construir (una sesión por ticket):**

5. Elegir un ticket sin bloqueos abiertos. Rama `feat/<issue>-slug` desde `main`.
6. Sesión limpia: `implement <issue>`. Nada del contexto de la sesión anterior hace falta.
7. Checks locales, commit, PR. Cuerpo del PR: qué, por qué, cómo se probó, y el ticket que cierra.
8. CI verde, merge con squash, borrar rama.
9. Repetir con el siguiente ticket de la frontera.

**Cierre:**

10. `retro` sobre la sesión del último ticket o sobre el feature completo: qué del entorno
    (AGENTS.md, checks, estándares) habría evitado fricción.
11. Release cuando el feature está completo y probado.

## Flujo 4 — Trivial (docs, versión, formato, CI)

1. Sin issue. Rama `docs/...` o `chore/...`.
2. Cambio + CI verde. Sin `/code-review`.

## Flujo 5 — Dependabot

1. Skill `dependabot-prs`: listar, preparar la rama, resolver conflictos de `Cargo.toml`/lockfiles.
2. CI verde obligatorio. Merge solo entonces.
3. Los PRs del bot no pasan por `/triage` ni por `/code-review`.

## Flujo 6 — Release

Solo cuando se cierra un feature (no en cada PR).

1. Subir versión en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
2. Actualizar el README si cambian los nombres de los instaladores.
3. Commit `chore(release): vX.Y.Z` y push a `main`.
4. `git tag vX.Y.Z` + `git push origin vX.Y.Z`.
5. Esperar el workflow de release y comprobar que adjunta MSI, NSIS, sus `.sig` y `latest.json`.

## Flujo 7 — Deuda técnica

1. `improve-codebase-architecture` para sacar oportunidades de módulos profundos.
2. La que elijas se convierte en idea → Flujo 3 (o Flujo 2 si es pequeña).

## Cuándo el flujo completo es demasiado

| Situación | Qué hacer |
|---|---|
| Cabe en una sesión | Flujo 2: issue + `implement` |
| Duda de diseño | `prototype`, nunca spec |
| Esfuerzo enorme y difuso | `wayfinder`; después `to-spec` |
| Solo quieres entender el código | `codegraph_explore`, sin flujo |
| Necesitas datos externos | `research` y volver al flujo con el `.md` |

## Code review: cuándo sí

Revisión completa (`code-review`, dos ejes: Standards + Spec) cuando el PR:

- toca **más de un módulo** o supera ~200 líneas de diff;
- cambia esquema SQLite, migraciones, comandos IPC o el updater;
- toca `db.rs` (SM-2, SQL), `scheduler.rs` o la firma de releases.

En el resto basta CI verde y releer el diff. `implement` siempre hace una revisión rápida al final;
en PRs pequeños, esa es toda la revisión.

## Definition of done

- [ ] `npm run typecheck` limpio.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib` verde.
- [ ] Tests nuevos o ajustados para el comportamiento que cambia.
- [ ] CI verde en el PR.
- [ ] `.opencode/state.md` actualizado; `decisions.md` si hay decisión de arquitectura.
- [ ] Cambios de UI probados a mano en `npm run tauri dev` (recomendado, no bloqueante).

## Higiene de contexto

- `grill-with-docs` → `to-spec` → `to-tickets`: **una sola ventana**, sin `/compact` hasta el final.
- Cada `implement` en sesión limpia. El ticket es autocontenido.
- Si la ventana se acerca a ~150k tokens antes de `to-tickets`, compactar ahí, no a mitad de fase.

## Nombres

- Ramas: `feat/<issue>-slug`, `fix/<issue>-slug`, `docs/slug`, `chore/slug`, `prototype/slug`.
- Commits: Conventional Commits en inglés (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
- Issues y PRs: título en inglés, una línea; cuerpo con qué/por qué/criterio.
