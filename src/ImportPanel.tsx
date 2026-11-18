import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api, type ImportSummary } from "./api";
import { IconArrow, IconCheck, IconImport } from "./Icons";

type Props = {
  onImported: () => Promise<void> | void;
};

export default function ImportPanel({ onImported }: Props) {
  const [busy, setBusy] = useState(false);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function pick() {
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Mazo de Anki", extensions: ["apkg", "txt"] },
          { name: "Todos", extensions: ["*"] },
        ],
      });
      if (!selected || Array.isArray(selected)) return;

      setBusy(true);
      const result = await api.importDeck(selected);
      setSummary(result);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="import-screen">
      <section className="import-card">
        <div className="import-head">
          <span className="import-seal">言</span>
          <div>
            <span className="eyebrow">Bienvenido</span>
            <h1>Trae tu mazo</h1>
          </div>
        </div>

        {!summary ? (
          <>
            <p className="sub">
              kotoba repasa por ti el vocabulario de tu mazo <strong>Genki 1</strong>.
              Importa el archivo <code>.apkg</code> exportado de Anki y la app
              extraerá las palabras y el audio.
            </p>

            <button
              type="button"
              className="dropzone"
              onClick={() => void pick()}
              disabled={busy}
            >
              {busy ? (
                <>
                  <span className="spinner" />
                  <strong>Leyendo el mazo…</strong>
                  <span className="small">
                    Extrayendo notas y audio. Puede tardar unos segundos.
                  </span>
                </>
              ) : (
                <>
                  <IconImport size={30} />
                  <strong>Elegir archivo .apkg</strong>
                  <span className="small">
                    También acepta exports .txt de Anki (sin audio)
                  </span>
                </>
              )}
            </button>

            {error && <p className="inline-error">{error}</p>}

            <ul className="import-notes">
              <li>
                <IconCheck size={14} /> Todo se guarda en tu equipo. Nada sale de aquí.
              </li>
              <li>
                <IconCheck size={14} /> El progreso de Anki no se toca: kotoba usa su
                propio repaso.
              </li>
              <li>
                <IconCheck size={14} /> Puedes reimportar o borrar los datos desde
                Ajustes.
              </li>
            </ul>
          </>
        ) : (
          <>
            <p className="sub">
              Mazo importado: <strong>{summary.deck_name}</strong>. Esto es lo que
              ha quedado listo para repasar.
            </p>

            <div className="summary-grid">
              <div className="summary-item">
                <span className="summary-value">{summary.items_inserted}</span>
                <span className="summary-label">palabras</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">
                  {summary.audio_extracted + summary.audio_reused}
                </span>
                <span className="summary-label">audios</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">{summary.patterns}</span>
                <span className="summary-label">plantillas</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">{summary.notes}</span>
                <span className="summary-label">notas leídas</span>
              </div>
            </div>

            {summary.items_merged > 0 && (
              <p className="small muted">
                {summary.items_merged} notas duplicadas se han unido a su palabra
                principal.
              </p>
            )}

            {summary.warnings.length > 0 && (
              <details className="warnings">
                <summary>{summary.warnings.length} avisos durante la importación</summary>
                <ul>
                  {summary.warnings.map((warning) => (
                    <li key={warning}>{warning}</li>
                  ))}
                </ul>
              </details>
            )}

            {error && <p className="inline-error">{error}</p>}

            <div className="import-actions">
              <button
                type="button"
                className="ghost"
                onClick={() => {
                  setSummary(null);
                  setError(null);
                }}
              >
                Importar otro
              </button>
              <button
                type="button"
                className="primary"
                onClick={() => void onImported()}
              >
                Empezar a repasar
                <IconArrow size={16} />
              </button>
            </div>
          </>
        )}
      </section>
    </div>
  );
}
