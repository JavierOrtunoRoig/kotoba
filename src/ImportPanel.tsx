import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import {
  api,
  type ImportInspection,
  type ImportMapping,
  type ImportProgress,
  type ImportSummary,
} from "./api";
import { useT, type Language } from "./i18n";
import MappingDialog from "./MappingDialog";
import { IconArrow, IconCheck, IconImport, IconRefresh } from "./Icons";

type Props = {
  language: Language;
  onImported: () => Promise<void> | void;
};

export default function ImportPanel({ language, onImported }: Props) {
  const t = useT(language);
  const [busy, setBusy] = useState(false);
  const [summary, setSummary] = useState<ImportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [inspection, setInspection] = useState<ImportInspection | null>(null);
  const [pendingPath, setPendingPath] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<ImportProgress>("import:progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  async function pick() {
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: t("Mazo de Anki"), extensions: ["apkg", "txt"] },
          { name: t("Todos"), extensions: ["*"] },
        ],
      });
      if (!selected || Array.isArray(selected)) return;

      setBusy(true);
      setProgress(null);
      const info = await api.inspectImport(selected);
      setInspection(info);
      setPendingPath(selected);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function runImport(mapping: ImportMapping) {
    if (!pendingPath) return;
    setBusy(true);
    setProgress(null);
    setError(null);
    try {
      const result = await api.importDeck(pendingPath, mapping);
      setSummary(result);
      setInspection(null);
      setPendingPath(null);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  return (
    <div className="import-screen">
      <section className="import-card">
        <div className="import-head">
          <span className="import-seal">言</span>
          <div>
            <span className="eyebrow">{t("Bienvenido")}</span>
            <h1>{t("Trae tu mazo")}</h1>
          </div>
        </div>

        {!summary ? (
          <>
            <p className="sub">
              {t(
                "kotoba repasa por ti el vocabulario de tu mazo {deck}. Importa el archivo {format} exportado de Anki y la app extraerá las palabras y el audio.",
                { deck: "Genki 1", format: ".apkg" },
              )}
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
                  <strong>{t("Leyendo el mazo…")}</strong>
                  <span className="small">
                    {t("Extrayendo notas y audio. Puede tardar unos segundos.")}
                  </span>
                  <div className="progress-track">
                    <div
                      className="progress-fill"
                      style={{
                        width: `${
                          progress && progress.total > 0
                            ? Math.min(
                                100,
                                (progress.current / progress.total) * 100,
                              )
                            : 0
                        }%`,
                      }}
                    />
                  </div>
                  <span className="small">
                    {progress
                      ? `${progress.current} / ${progress.total}`
                      : t("Preparando…")}
                  </span>
                </>
              ) : (
                <>
                  <IconImport size={30} />
                  <strong>{t("Elegir archivo .apkg")}</strong>
                  <span className="small">
                    {t("También acepta exports .txt de Anki (sin audio)")}
                  </span>
                </>
              )}
            </button>

            {error && (
              <div className="import-error">
                <p className="inline-error">{error}</p>
                <button
                  type="button"
                  className="ghost"
                  onClick={() => void pick()}
                >
                  <IconRefresh size={16} />
                  {t("Reintentar")}
                </button>
              </div>
            )}

            <ul className="import-notes">
              <li>
                <IconCheck size={14} />{" "}
                {t("Todo se guarda en tu equipo. Nada sale de aquí.")}
              </li>
              <li>
                <IconCheck size={14} />{" "}
                {t("El progreso de Anki no se toca: kotoba usa su propio repaso.")}
              </li>
              <li>
                <IconCheck size={14} />{" "}
                {t("Puedes reimportar o borrar los datos desde Ajustes.")}
              </li>
            </ul>
          </>
        ) : (
          <>
            <p className="sub">
              {t(
                "Mazo importado: {name}. Esto es lo que ha quedado listo para repasar.",
                { name: summary.deck_name },
              )}
            </p>

            <div className="summary-grid">
              <div className="summary-item">
                <span className="summary-value">{summary.items_inserted}</span>
                <span className="summary-label">{t("palabras")}</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">
                  {summary.audio_extracted + summary.audio_reused}
                </span>
                <span className="summary-label">{t("audios")}</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">{summary.patterns}</span>
                <span className="summary-label">{t("plantillas")}</span>
              </div>
              <div className="summary-item">
                <span className="summary-value">{summary.notes}</span>
                <span className="summary-label">{t("notas leídas")}</span>
              </div>
            </div>

            {summary.items_merged > 0 && (
              <p className="small muted">
                {t(
                  "{n} notas duplicadas se han unido a su palabra principal.",
                  { n: summary.items_merged },
                )}
              </p>
            )}

            {summary.warnings.length > 0 && (
              <details className="warnings">
                <summary>
                  {t("{n} avisos durante la importación", {
                    n: summary.warnings.length,
                  })}
                </summary>
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
                {t("Importar otro")}
              </button>
              <button
                type="button"
                className="primary"
                onClick={() => void onImported()}
              >
                {t("Empezar a repasar")}
                <IconArrow size={16} />
              </button>
            </div>
          </>
        )}
      </section>

      {inspection && (
        <MappingDialog
          inspection={inspection}
          language={language}
          busy={busy}
          error={error}
          onCancel={() => {
            setInspection(null);
            setPendingPath(null);
          }}
          onImport={runImport}
        />
      )}
    </div>
  );
}
