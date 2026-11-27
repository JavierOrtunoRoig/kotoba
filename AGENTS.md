<!-- CODEGRAPH_START -->
## CodeGraph

In repositories indexed by CodeGraph (a `.codegraph/` directory exists at the repo root), reach for it BEFORE grep/find or reading files when you need to understand or locate code:

- **MCP tool** (when available): `codegraph_explore` answers most code questions in one call — the relevant symbols' verbatim source plus the call paths between them, including dynamic-dispatch hops grep can't follow. Name a file or symbol in the query to read its current line-numbered source. If it's listed but deferred, load it by name via tool search.
- **Shell** (always works): `codegraph explore "<symbol names or question>"` prints the same output.

If there is no `.codegraph/` directory, skip CodeGraph entirely — indexing is the user's decision.
<!-- CODEGRAPH_END -->

## Agent skills

### Issue tracker

Issues and specs live in the GitHub Issues of `JavierOrtunoRoig/kotoba`, via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the root (created when needed). Current knowledge lives in `.opencode/`. See `docs/agents/domain.md`.

## How to work here

Full playbook in `.opencode/workflows.md`. These rules take precedence over the skills' judgement:

- **Behaviour change → issue first** (`gh issue create`, body in English). Docs, version or CI →
  direct PR without an issue. Our own issues are not triaged.
- **Never commit directly to `main`**: branch + PR, one branch per issue.
- **Technical autonomy**: decide and implement without asking. Consult only on product, UX and scope.
- **English artifacts** (code, commits, issues, PRs, specs). Repository docs are English; conversation stays Spanish.
- **Done** = local checks green + CI green + test for the change + `.opencode/state.md` updated.
- **Release** only when closing a feature, not on every PR.
- **Loose ideas** → `.opencode/inbox.md`.

### Routing

| If they ask for… | Do |
|---|---|
| "note this idea" | add it to `.opencode/inbox.md` |
| "fix X" / a bug | issue + `fix/N-slug` branch; `diagnosing-bugs` if the cause isn't obvious |
| "implement #N" | clean session, `feat/N-slug` branch, `implement` skill |
| "I want X" (feature) | `grill-with-docs`; if it doesn't fit in one session, `to-spec` → `to-tickets` |
| "review this" | `code-review` |
| "publish release" | flow 6 of `workflows.md` |
| "update dependencies" | `dependabot-prs` skill |
| "this is badly designed" | `improve-codebase-architecture` |
| "research X" | `research` |
