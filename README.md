# kotoba

> Entrenador de vocabulario japonés **pasivo** para escritorio — micro-retos que aparecen mientras trabajas.

App de escritorio para Windows que vive en la bandeja del sistema y, de vez en cuando, lanza un
micro-ejercicio de vocabulario (5-10 s). No se trata de sentarte a estudiar: la app te trae el repaso
a ti. Por detrás usa **repetición espaciada invisible** para que el contenido se retenga.

El vocabulario se importa desde un export del mazo de Anki **"Genki 1"**.

## Estado

Fase de planificación / arranque del proyecto. Ver [`docs/plan.md`](docs/plan.md).

## Stack previsto

- [Tauri 2](https://tauri.app/) (Rust) sobre WebView2
- React + TypeScript + Vite (frontend)
- SQLite (persistencia local)

## Concepto

- **Pasivo ≠ sin esfuerzo**: el micro-reto es *active recall*; lo pasivo es no tener que abrir una app
  ni decidir cuándo estudiar.
- **SRS invisible**: cada palabra tiene un intervalo que crece al acertar. Sin colas ni pendientes.
- **3 modos de aparición** configurables: widget flotante, popup, toast de sistema.

## Repositorio

Primer commit: scaffolding del proyecto (README + plan + git).

## Licencia

Uso personal. El contenido del mazo de Anki no se redistribuye con la app.
