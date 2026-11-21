---
name: dependabot-prs
description: Gestiona los PR de Dependabot de kotoba — listar bumps, traer la rama, resolver conflictos de Cargo, ejecutar tests y mergear solo con CI verde. Usar cuando el usuario pida arreglar, revisar o aprobar PRs de Dependabot o bumps de dependencias.
---

# PRs de Dependabot

Automatización con IA en el bucle. Los scripts hacen lo determinista (API de GitHub, git, conflictos
de manifiesto, tests); la IA decide los cambios de código y valida antes de mergear.

## Cuándo usar

- "Arregla los PRs de Dependabot", "mergea los bumps", "revisa dependencias desactualizadas".
- No usar para PRs humanos ni para bumps que cambien APIs: esos requieren criterio, pero el flujo
  de verificación sigue sirviendo.

## Flujo

Todos los scripts se ejecutan desde cualquier directorio del repo con
`powershell -NoProfile -File .opencode/skills/dependabot-prs/scripts/<script>.ps1 ...`.
Sacan el token con `git credential fill`; no hace falta `gh auth login`.

1. **Listar.** `list-prs.ps1 [-All]` → JSON con número, título, rama, sha, `ci` y `mergeable`.
   Por defecto solo devuelve PRs de bots con título `bump X from Y to Z`. `-All` incluye el resto.
   - `ci: success` → candidato a merge directo.
   - `ci: pending` → esperar con `checks.ps1 -Pr N -Wait`.
   - `ci: failure` → seguir al paso 2.

2. **Preparar rama.** `prepare.ps1 -Pr N`:
   - `fetch` de `main` y de la rama del PR, `checkout -B <rama> origin/<rama>` (exige árbol limpio).
   - merge de `origin/main`.
   - `Cargo.lock`: toma la versión de `main` y la regenera con `cargo check` (ya que el manifiesto
     del PR pide otra versión).
   - `Cargo.toml`: si hay conflicto, conserva la versión del PR en las claves que el PR cambió y la
     de `main` en las demás (funciona con `x = "v"` y con `x = { version = "v", ... }`).
   - Cierra el merge con un commit. Imprime JSON con `conflictsFixed`, `lockRegenerated` y el
     resultado de `cargo check`. **No hace push.**

3. **Arreglar código.** Si `cargoCheck.ok` es `false`, leer `cargoCheck.tail`: son errores de API
   (ej. rand 0.10: `thread_rng` → `rng`, `gen_range` → `random_range`, rasgo `RngExt`). Editar,
   commitear y volver a `cargo check` hasta que pase. Si `prepare.ps1` lanzó error por conflictos
   ajenos a Cargo, resolverlos a mano antes de seguir.

4. **Verificar.** `verify.ps1 -Pr N` ejecuta `cargo check`, `cargo test --lib` y
   `npm run typecheck`. Guarda `$env:TEMP\kotoba-dependabot\N.verify.json` con `passed`, `headSha`
   y la salida de cada paso. Leer el JSON: si algún `ok` es `false`, arreglar y repetir. Añade el
   directorio de Node de Scoop al PATH si `node` no está visible.

5. **Mergear.** `merge.ps1 -Pr N [-Approve]`:
   - exige verificación verde para el HEAD local actual, árbol limpio y PR mergeable;
   - hace push de la rama;
   - espera a que la CI del nuevo sha esté en verde (timeout 15 min, sondeo cada 20 s);
   - opcionalmente envía review `APPROVE`, y hace squash merge con la API.
   - `checks.ps1 -Pr N [-Wait] [-TimeoutSeconds S]` permite inspeccionar la CI por separado.

## Reglas

- Nunca hacer push a `main`, ni force-push a ramas de Dependabot.
- Nunca mergear con CI roja, con verificación local fallida o con conflictos abiertos.
- Si la rama del PR viene de un fork, `prepare.ps1` se niega: tratar manualmente.
- Un cambio de versión mayor puede romper la API. Confiar en el compilador, no asumir que el bump
  es solo de manifiesto.
- Si `verify.ps1` pasa pero la CI falla, no reintentar el merge: leer el check y volver al paso 3.
- Tras mergear varios bumps seguidos, las demás ramas se quedan desactualizadas; ejecutar
  `prepare.ps1` de nuevo si aparece "Pull Request has merge conflicts".

## Notas

- Versiones a sincronizar para una release: `package.json`, `src-tauri/Cargo.toml`,
  `src-tauri/tauri.conf.json` (esta última manda en el workflow `release.yml`).
- El estado actual del repo y los comandos de build están en `.opencode/state.md` y
  `.opencode/commands.md`.
