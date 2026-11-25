<!-- CODEGRAPH_START -->
## CodeGraph

In repositories indexed by CodeGraph (a `.codegraph/` directory exists at the repo root), reach for it BEFORE grep/find or reading files when you need to understand or locate code:

- **MCP tool** (when available): `codegraph_explore` answers most code questions in one call — the relevant symbols' verbatim source plus the call paths between them, including dynamic-dispatch hops grep can't follow. Name a file or symbol in the query to read its current line-numbered source. If it's listed but deferred, load it by name via tool search.
- **Shell** (always works): `codegraph explore "<symbol names or question>"` prints the same output.

If there is no `.codegraph/` directory, skip CodeGraph entirely — indexing is the user's decision.
<!-- CODEGRAPH_END -->

## Agent skills

### Issue tracker

Issues y specs viven en GitHub Issues de `JavierOrtunoRoig/kotoba`, vía `gh`. Ver `docs/agents/issue-tracker.md`.

### Triage labels

Vocabulario canónico: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. Ver `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` y `docs/adr/` en la raíz (se crean cuando haga falta). El conocimiento actual está en `.opencode/`. Ver `docs/agents/domain.md`.

## Cómo trabajar aquí

Playbook completo en `.opencode/workflows.md`. Estas reglas mandan sobre el criterio de las skills:

- **Cambia comportamiento → issue primero** (`gh issue create`, cuerpo en inglés). Docs, versión o CI →
  PR directo sin issue. Los issues propios no se triajan.
- **Nunca commit directo a `main`**: rama + PR, una rama por issue.
- **Autonomía técnica**: decide e implementa sin preguntar. Consulta solo producto, UX y alcance.
- **Artefactos en inglés** (código, commits, issues, PRs, specs). Docs del repo y conversación en español.
- **Terminado** = checks locales verdes + CI verde + test del cambio + `.opencode/state.md` actualizado.
- **Release** solo al cerrar un feature, no en cada PR.
- **Ideas sueltas** → `.opencode/inbox.md`.

### Enrutado

| Si piden… | Hacer |
|---|---|
| «apunta esta idea» | añadirla a `.opencode/inbox.md` |
| «arregla X» / un bug | issue + rama `fix/N-slug`; `diagnosing-bugs` si la causa no es obvia |
| «implementa #N» | sesión limpia, rama `feat/N-slug`, skill `implement` |
| «quiero X» (feature) | `grill-with-docs`; si no cabe en una sesión, `to-spec` → `to-tickets` |
| «revisa esto» | `code-review` |
| «publica release» | flujo 6 de `workflows.md` |
| «actualiza dependencias» | skill `dependabot-prs` |
| «esto está mal diseñado» | `improve-codebase-architecture` |
| «investiga X» | `research` |

