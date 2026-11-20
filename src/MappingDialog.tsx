import { useState } from "react";
import { createPortal } from "react-dom";
import type {
  FieldMapping,
  ImportInspection,
  ImportMapping,
  NoteTypeInfo,
} from "./api";
import { useT, type Language } from "./i18n";
import { IconClose } from "./Icons";

type Props = {
  inspection: ImportInspection;
  language: Language;
  busy?: boolean;
  error?: string | null;
  onCancel: () => void;
  onImport: (mapping: ImportMapping) => void | Promise<void>;
};

type Target = {
  key: keyof FieldMapping;
  label: string;
  required?: boolean;
  apkgOnly?: boolean;
};

const TARGETS: Target[] = [
  { key: "reading", label: "Lectura", required: true },
  { key: "kanji", label: "Kanji" },
  { key: "meaning", label: "Significado", required: true },
  { key: "gloss", label: "Glosa de kanji" },
  { key: "audio", label: "Audio", apkgOnly: true },
  { key: "uid", label: "UID" },
  { key: "tags", label: "Etiquetas" },
  { key: "lesson", label: "Lección" },
  { key: "category", label: "Subapartado" },
];

function emptyMapping(): FieldMapping {
  return {
    reading: null,
    kanji: null,
    meaning: null,
    gloss: null,
    audio: null,
    uid: null,
    tags: null,
    lesson: null,
    category: null,
  };
}

function withDefaults(mapping?: FieldMapping | null): FieldMapping {
  return { ...emptyMapping(), ...(mapping ?? {}) };
}

function isValid(mapping: FieldMapping): boolean {
  return mapping.reading !== null && mapping.meaning !== null;
}

function MappingSection({
  title,
  subtitle,
  columns,
  sample,
  mapping,
  onChange,
  includeAudio,
  t,
}: {
  title: string;
  subtitle: string;
  columns: string[];
  sample: string[][];
  mapping: FieldMapping;
  onChange: (mapping: FieldMapping) => void;
  includeAudio: boolean;
  t: ReturnType<typeof useT>;
}) {
  const targets = TARGETS.filter((target) => includeAudio || !target.apkgOnly);

  return (
    <section className="mapping-section">
      <div className="mapping-head">
        <div>
          <h3>{title}</h3>
          <p className="muted small">{subtitle}</p>
        </div>
      </div>

      {columns.length > 0 && (
        <div className="mapping-preview">
          <span className="mapping-caption">{t("Filas de ejemplo")}</span>
          <div className="mapping-table-wrap">
            <table>
              <thead>
                <tr>
                  {columns.map((column, index) => (
                    <th key={`${column}-${index}`}>
                      {index + 1} · {column}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {sample.map((row, rowIndex) => (
                  <tr key={rowIndex}>
                    {columns.map((_, index) => (
                      <td key={index}>{row[index] ?? ""}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="mapping-grid">
        {targets.map((target) => (
          <label className="mapping-field" key={target.key}>
            <span>
              {t(target.label)}
              {target.required && <strong className="required"> *</strong>}
            </span>
            <select
              value={mapping[target.key] ?? ""}
              aria-label={t(target.label)}
              onChange={(event) => {
                const value = event.currentTarget.value;
                onChange({
                  ...mapping,
                  [target.key]: value === "" ? null : Number(value),
                });
              }}
            >
              <option value="">{t("Sin asignar")}</option>
              {columns.map((column, index) => (
                <option key={`${target.key}-${index}`} value={index}>
                  {index + 1} · {column}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>
    </section>
  );
}

export default function MappingDialog({
  inspection,
  language,
  busy = false,
  error,
  onCancel,
  onImport,
}: Props) {
  const t = useT(language);
  const [txtMapping, setTxtMapping] = useState<FieldMapping>(() =>
    withDefaults(inspection.suggested),
  );
  const [noteMappings, setNoteMappings] = useState<Record<number, FieldMapping>>(
    () =>
      Object.fromEntries(
        inspection.note_types.map((noteType) => [
          noteType.mid,
          withDefaults(noteType.suggested),
        ]),
      ),
  );

  const isTxt = inspection.kind === "txt";
  const valid = isTxt
    ? isValid(txtMapping)
    : inspection.note_types.every((noteType) =>
        isValid(noteMappings[noteType.mid] ?? emptyMapping()),
      );

  const mapping: ImportMapping = isTxt
    ? { txt: txtMapping, note_types: [] }
    : {
        txt: null,
        note_types: inspection.note_types.map(
          (noteType: NoteTypeInfo) => ({
            mid: noteType.mid,
            mapping: noteMappings[noteType.mid] ?? emptyMapping(),
          }),
        ),
      };

  return createPortal(
    <div
      className="mapping-overlay"
      role="dialog"
      aria-modal="true"
      aria-label={t("Mapear campos")}
    >
      <div className="mapping-card">
        <header className="mapping-top">
          <div>
            <span className="eyebrow">{inspection.deck_name}</span>
            <h2>{t("Mapear campos")}</h2>
            <p className="muted">
              {t(
                "Asigna cada dato de kotoba a una columna del mazo. Lectura y significado son obligatorios.",
              )}
            </p>
          </div>
          <button
            type="button"
            className="popup-close"
            onClick={onCancel}
            aria-label={t("Cancelar")}
          >
            <IconClose size={14} />
          </button>
        </header>

        <div className="mapping-scroll">
          {isTxt ? (
            <MappingSection
              title={inspection.deck_name}
              subtitle={`${inspection.total_notes} ${t("notas")} · ${inspection.columns.length} ${t("columnas")}`}
              columns={inspection.columns}
              sample={inspection.sample}
              mapping={txtMapping}
              onChange={setTxtMapping}
              includeAudio={false}
              t={t}
            />
          ) : (
            inspection.note_types.map((noteType) => (
              <MappingSection
                key={noteType.mid}
                title={noteType.name || `${t("Tipo de nota")} ${noteType.mid}`}
                subtitle={`${noteType.count} ${t("notas")} · ${noteType.fields.length} ${t("columnas")}`}
                columns={noteType.fields}
                sample={noteType.sample}
                mapping={noteMappings[noteType.mid] ?? emptyMapping()}
                onChange={(next) =>
                  setNoteMappings((current) => ({
                    ...current,
                    [noteType.mid]: next,
                  }))
                }
                includeAudio
                t={t}
              />
            ))
          )}
        </div>

        <footer className="mapping-actions">
          <span className="muted small">
            {valid
              ? t("Listo para importar.")
              : t("Selecciona lectura y significado para continuar.")}
          </span>
          <div className="mapping-buttons">
            <button type="button" className="ghost" onClick={onCancel}>
              {t("Cancelar")}
            </button>
            <button
              type="button"
              className="primary"
              disabled={!valid || busy}
              onClick={() => void onImport(mapping)}
            >
              {busy ? t("Importando…") : t("Importar con este mapeo")}
            </button>
          </div>
        </footer>
        {error && (
          <p className="inline-error mapping-error" role="alert">
            {error}
          </p>
        )}
      </div>
    </div>,
    document.body,
  );
}
