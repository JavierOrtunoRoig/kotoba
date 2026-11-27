import { useEffect, useState } from "react";
import { ask, open, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import {
  api,
  type Bootstrap,
  type ImportInspection,
  type ImportMapping,
  type ImportProgress,
  type Settings,
} from "./api";
import { applyAppearance } from "./appearance";
import { LANGUAGES, useT } from "./i18n";
import MappingDialog from "./MappingDialog";
import { findUpdate, installUpdate } from "./updates";
import {
  IconArchive,
  IconDownload,
  IconFolder,
  IconImport,
  IconRefresh,
  IconTrash,
  IconUpload,
} from "./Icons";

type Props = {
  boot: Bootstrap;
  onChanged: () => Promise<void> | void;
  onToast: (message: string) => void;
};

function Toggle({
  checked,
  onChange,
  label,
  disabled,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-disabled={disabled || undefined}
      aria-label={label}
      className={`switch ${checked ? "on" : ""}`}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    />
  );
}

export default function SettingsPanel({ boot, onChanged, onToast }: Props) {
  const [settings, setSettings] = useState<Settings>(boot.settings);
  const [saving, setSaving] = useState(false);
  const [busy, setBusy] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [inspection, setInspection] = useState<ImportInspection | null>(null);
  const [pendingPath, setPendingPath] = useState<string | null>(null);
  const t = useT(settings.language);

  useEffect(() => {
    const unlisten = listen<ImportProgress>("import:progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    // In development the binary is a console app and loads the devUrl: starting with Windows leaves
    // an open window and an error page. Only allowed from the installed app.
    if (boot.dev_build) {
      setAutostart(false);
      setSettings((current) => ({ ...current, launch_at_startup: false }));
      return;
    }
    isAutostartEnabled()
      .then((enabled) => {
        setAutostart(enabled);
        setSettings((current) => ({ ...current, launch_at_startup: enabled }));
      })
      .catch(() => undefined);
  }, [boot.dev_build]);

  // Autostart is applied instantly by the plugin; it does not count as an unsaved change.
  const dirty =
    JSON.stringify({ ...settings, launch_at_startup: false }) !==
    JSON.stringify({ ...boot.settings, launch_at_startup: false });

  useEffect(() => {
    applyAppearance(settings.theme, settings.font_scale);
  }, [settings.theme, settings.font_scale]);

  function update<K extends keyof Settings>(key: K, value: Settings[K]) {
    setSettings((current) => ({ ...current, [key]: value }));
  }

  function changeMinInterval(value: number) {
    setSettings((current) => ({
      ...current,
      interval_min_minutes: value,
      interval_max_minutes: Math.max(current.interval_max_minutes, value),
    }));
  }

  function changeMaxInterval(value: number) {
    setSettings((current) => ({
      ...current,
      interval_max_minutes: Math.max(value, current.interval_min_minutes),
    }));
  }

  async function save() {
    setSaving(true);
    try {
      const saved = await api.updateSettings(settings);
      setSettings(saved);
      onToast(t("Ajustes guardados"));
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setSaving(false);
    }
  }

  async function openLogs() {
    try {
      await api.openLogs();
    } catch (caught) {
      onToast(String(caught));
    }
  }

  async function toggleAutostart(next: boolean) {
    if (boot.dev_build) {
      onToast(t("No disponible en la compilación de desarrollo."));
      return;
    }
    try {
      if (next) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      setAutostart(next);
      update("launch_at_startup", next);
    } catch (caught) {
      onToast(String(caught));
    }
  }

  async function checkUpdates() {
    if (boot.dev_build) {
      onToast(t("No disponible en la compilación de desarrollo."));
      return;
    }
    setUpdating(true);
    try {
      const update = await findUpdate();
      if (!update) {
        onToast(t("Ya tienes la última versión."));
        return;
      }
      const accepted = await ask(
        t("kotoba {version} está disponible. ¿Actualizar ahora?", {
          version: update.version,
        }),
        {
          title: t("Actualización disponible"),
          kind: "info",
          okLabel: t("Actualizar ahora"),
          cancelLabel: t("Ahora no"),
        },
      );
      if (!accepted) {
        await update.close();
        return;
      }
      onToast(t("Descargando la actualización…"));
      await installUpdate(update);
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setUpdating(false);
    }
  }

  async function reimport() {
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
    try {
      const info = await api.inspectImport(selected);
      setInspection(info);
      setPendingPath(selected);
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function runMappedImport(mapping: ImportMapping) {
    if (!pendingPath) return;
    setBusy(true);
    setProgress(null);
    try {
      const summary = await api.importDeck(pendingPath, mapping);
      onToast(
        t("Importadas {words} palabras y {audios} audios", {
          words: summary.items_inserted,
          audios: summary.audio_extracted + summary.audio_reused,
        }),
      );
      setInspection(null);
      setPendingPath(null);
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  async function exportBackup() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;

    setBusy(true);
    try {
      const folder = await api.exportBackup(selected);
      onToast(t("Copia guardada en {folder}", { folder }));
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function restoreBackup() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    if (
      !window.confirm(
        t("Se reemplazarán las palabras, el audio y el progreso actuales. ¿Continuar?"),
      )
    ) {
      return;
    }

    setBusy(true);
    try {
      const restored = await api.restoreBackup(selected);
      setSettings(restored);
      onToast(t("Copia restaurada"));
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function exportHistory() {
    const selected = await saveDialog({
      defaultPath: "kotoba-history.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!selected) return;

    setBusy(true);
    try {
      const path = await api.exportHistory(selected);
      onToast(t("Historial guardado en {path}", { path }));
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function resetProgress() {
    if (
      !window.confirm(
        t(
          "¿Reiniciar el repaso? Se borra el historial y los intervalos, pero se conservan las palabras.",
        ),
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await api.resetProgress();
      onToast(t("Progreso reiniciado"));
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function clearLibrary() {
    if (
      !window.confirm(
        t(
          "¿Borrar todas las palabras importadas? Tendrás que volver a importar el mazo.",
        ),
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await api.clearLibrary();
      onToast(t("Biblioteca borrada"));
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  const themeDescription =
    settings.theme === "auto"
      ? t("Sigue el tema de Windows.")
      : settings.theme === "light"
        ? t("Tema claro, papel y tinta.")
        : t("Tema oscuro, lectura nocturna.");

  return (
    <section className="settings">
      <header className="view-head">
        <div>
          <span className="eyebrow">{t("Ajustes")}</span>
          <h1>{t("Tu ritmo")}</h1>
          <p className="sub">
            {t(
              "Un equilibrio entre no olvidar y no molestar. Los cambios se aplican al guardar.",
            )}
          </p>
        </div>
      </header>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Apariencia")}</h2>
            <p className="muted">{t("Tema, idioma y tamaño de la interfaz.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Tema")}</div>
            <div className="field-desc">{themeDescription}</div>
          </div>
          <div className="segmented" role="group" aria-label={t("Tema")}>
            {(
              [
                ["auto", t("Auto")],
                ["light", t("Claro")],
                ["dark", t("Oscuro")],
              ] as const
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={settings.theme === value ? "active" : ""}
                aria-pressed={settings.theme === value}
                onClick={() => update("theme", value)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Idioma")}</div>
            <div className="field-desc">
              {t("Cambia los textos de la interfaz y de la bandeja.")}
            </div>
          </div>
          <div className="segmented" role="group" aria-label={t("Idioma")}>
            {LANGUAGES.map((language) => (
              <button
                key={language.id}
                type="button"
                lang={language.id}
                className={settings.language === language.id ? "active" : ""}
                aria-pressed={settings.language === language.id}
                onClick={() => update("language", language.id)}
              >
                {language.label}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Tamaño de la interfaz")}</div>
            <div className="field-desc">
              {t("Escala el texto y los controles de la interfaz.")}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={90}
              max={125}
              step={5}
              value={settings.font_scale}
              aria-label={t("Tamaño de la interfaz")}
              onChange={(event) =>
                update("font_scale", Number(event.currentTarget.value))
              }
            />
            <output>{settings.font_scale}%</output>
          </div>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Frecuencia")}</h2>
            <p className="muted">{t("Cuándo y cuánto aparece el repaso.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Intervalo entre retos")}</div>
            <div className="field-desc">
              {t("Cada cuánto puede aparecer un micro-reto.")}
            </div>
          </div>
          <div className="range-pair">
            <label className="range-control">
              <span>{t("mín")}</span>
              <input
                type="range"
                min={1}
                max={120}
                value={settings.interval_min_minutes}
                onChange={(event) =>
                  changeMinInterval(Number(event.currentTarget.value))
                }
              />
              <output>
                {t("{n} min", { n: settings.interval_min_minutes })}
              </output>
            </label>
            <label className="range-control">
              <span>{t("máx")}</span>
              <input
                type="range"
                min={1}
                max={180}
                value={settings.interval_max_minutes}
                onChange={(event) =>
                  changeMaxInterval(Number(event.currentTarget.value))
                }
              />
              <output>
                {t("{n} min", { n: settings.interval_max_minutes })}
              </output>
            </label>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Máximo por día")}</div>
            <div className="field-desc">
              {settings.max_per_day === 0
                ? t("Sin límite diario.")
                : t("Hasta {n} repasos al día.", { n: settings.max_per_day })}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={200}
              step={5}
              value={settings.max_per_day}
              aria-label={t("Máximo por día")}
              onChange={(event) =>
                update("max_per_day", Number(event.currentTarget.value))
              }
            />
            <output>
              {settings.max_per_day === 0 ? "∞" : settings.max_per_day}
            </output>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Pausar repasos")}</div>
            <div className="field-desc">
              {t("La app sigue en la bandeja, pero no interrumpe.")}
            </div>
          </div>
          <Toggle
            checked={settings.paused}
            onChange={(value) => update("paused", value)}
            label={t("Pausar repasos")}
          />
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("No molestar")}</h2>
            <p className="muted">{t("Horas en las que kotoba se queda callado.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Activar horario de silencio")}</div>
            <div className="field-desc">
              {t("Útil para la noche o para reuniones.")}
            </div>
          </div>
          <Toggle
            checked={settings.dnd_enabled}
            onChange={(value) => update("dnd_enabled", value)}
            label={t("Activar horario de silencio")}
          />
        </div>

        <div className={`field ${!settings.dnd_enabled ? "disabled" : ""}`}>
          <div>
            <div className="field-label">{t("Desde / hasta")}</div>
            <div className="field-desc">
              {t("Si la hora final es menor, el rango cruza la medianoche.")}
            </div>
          </div>
          <div className="time-pair">
            <input
              type="time"
              value={settings.dnd_start}
              disabled={!settings.dnd_enabled}
              aria-label={t("Desde")}
              onChange={(event) => update("dnd_start", event.currentTarget.value)}
            />
            <span>→</span>
            <input
              type="time"
              value={settings.dnd_end}
              disabled={!settings.dnd_enabled}
              aria-label={t("Hasta")}
              onChange={(event) => update("dnd_end", event.currentTarget.value)}
            />
          </div>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Progreso")}</h2>
            <p className="muted">{t("Nivel, racha, logros y objetivos.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Objetivo semanal")}</div>
            <div className="field-desc">
              {t("Repasos por semana. 0 = sin objetivo.")}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={100}
              step={5}
              value={settings.weekly_goal}
              aria-label={t("Objetivo semanal")}
              onChange={(event) =>
                update("weekly_goal", Number(event.currentTarget.value))
              }
            />
            <output>
              {settings.weekly_goal === 0 ? "∞" : settings.weekly_goal}
            </output>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Maestría por lección")}</div>
            <div className="field-desc">
              {t(
                "Porcentaje de palabras con intervalo de 7 días o más para dar una lección por dominada.",
              )}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={50}
              max={100}
              step={5}
              value={settings.lesson_mastery_percent}
              aria-label={t("Maestría por lección")}
              onChange={(event) =>
                update(
                  "lesson_mastery_percent",
                  Number(event.currentTarget.value),
                )
              }
            />
            <output>{settings.lesson_mastery_percent}%</output>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Reparar racha")}</div>
            <div className="field-desc">
              {t(
                "Si fallas un día, gastar un salvavidas para mantener la racha. Los salvavidas se ganan al cumplir el objetivo semanal.",
              )}
            </div>
          </div>
          <Toggle
            checked={settings.streak_repair}
            onChange={(value) => update("streak_repair", value)}
            label={t("Reparar racha")}
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Notificaciones de hitos")}</div>
            <div className="field-desc">
              {t(
                "Avisar al completar el objetivo semanal o desbloquear un logro. Respeta el modo no molestar y la pantalla completa.",
              )}
            </div>
          </div>
          <Toggle
            checked={settings.milestone_notifications}
            onChange={(value) => update("milestone_notifications", value)}
            label={t("Notificaciones de hitos")}
          />
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Aparición")}</h2>
            <p className="muted">{t("Cómo y dónde aparece el micro-reto.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Modo")}</div>
            <div className="field-desc">
              {t(
                "Popup: tarjeta flotante. Widget: ventana siempre visible. Toast: notificación nativa de Windows.",
              )}
            </div>
          </div>
          <div
            className="segmented"
            role="group"
            aria-label={t("Modo de aparición")}
          >
            {(["popup", "widget", "toast"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                className={settings.mode === mode ? "active" : ""}
                aria-pressed={settings.mode === mode}
                onClick={() => update("mode", mode)}
              >
                {mode === "popup" ? "Popup" : mode === "widget" ? "Widget" : "Toast"}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Silenciar en pantalla completa")}</div>
            <div className="field-desc">
              {t(
                "No interrumpir juegos, vídeos, presentaciones ni el modo silencio de Windows.",
              )}
            </div>
          </div>
          <Toggle
            checked={settings.suppress_fullscreen}
            onChange={(value) => update("suppress_fullscreen", value)}
            label={t("Silenciar en pantalla completa")}
          />
        </div>

        {settings.mode === "toast" && (
          <p className="small muted">
            {t(
              "El toast es una notificación nativa: toca la notificación o el icono de la bandeja para abrir el reto.",
            )}
          </p>
        )}
        {settings.mode === "widget" && (
          <p className="small muted">
            {t(
              "El widget aparece en la esquina inferior derecha. Puedes arrastrarlo y ocultarlo cuando quieras.",
            )}
          </p>
        )}
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Contenido")}</h2>
            <p className="muted">{t("Qué se pregunta y cómo.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Dirección del ejercicio")}</div>
            <div className="field-desc">
              {t(
                "JP → ES para reconocer; ES → JP para producir. Alternar mezcla ambas.",
              )}
            </div>
          </div>
          <div className="segmented" role="group" aria-label={t("Dirección")}>
            {(["random", "alternate"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                className={settings.direction_mode === mode ? "active" : ""}
                aria-pressed={settings.direction_mode === mode}
                onClick={() => update("direction_mode", mode)}
              >
                {mode === "random" ? t("Aleatoria") : t("Alternar")}
              </button>
            ))}
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Incluir plantillas y gramática")}</div>
            <div className="field-desc">
              {t(
                "Notas con 〜, «+ negative» o «...». Recomendado dejarlas fuera al principio.",
              )}
            </div>
          </div>
          <Toggle
            checked={settings.include_patterns}
            onChange={(value) => update("include_patterns", value)}
            label={t("Incluir plantillas")}
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Audio automático en JP → ES")}</div>
            <div className="field-desc">
              {t("En ES → JP el audio suena solo después de responder.")}
            </div>
          </div>
          <Toggle
            checked={settings.audio_autoplay}
            onChange={(value) => update("audio_autoplay", value)}
            label={t("Audio automático")}
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Nuevas por día")}</div>
            <div className="field-desc">
              {t(
                "Palabras nuevas que se introducen cada día, en orden de lección y subapartado. 0 = ninguna.",
              )}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={50}
              step={1}
              value={settings.new_per_day}
              aria-label={t("Nuevas por día")}
              onChange={(event) =>
                update("new_per_day", Number(event.currentTarget.value))
              }
            />
            <output>{settings.new_per_day}</output>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Mezcla de nuevas")}</div>
            <div className="field-desc">
              {t(
                "0 = primero los repasos vencidos. Con repasos pendientes, este porcentaje decide cuántas nuevas se muestran.",
              )}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={100}
              step={10}
              value={settings.new_mix_percent}
              aria-label={t("Mezcla de nuevas")}
              onChange={(event) =>
                update(
                  "new_mix_percent",
                  Number(event.currentTarget.value),
                )
              }
            />
            <output>{settings.new_mix_percent}%</output>
          </div>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Sistema")}</h2>
            <p className="muted">{t("Integración con Windows.")}</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Arrancar con Windows")}</div>
            <div className="field-desc">
              {boot.dev_build
                ? t("No disponible en la compilación de desarrollo.")
                : t("kotoba se inicia en la bandeja sin abrir ninguna ventana.")}
            </div>
          </div>
          <Toggle
            checked={autostart}
            onChange={(value) => void toggleAutostart(value)}
            label={t("Arrancar con Windows")}
            disabled={boot.dev_build}
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Actualizaciones")}</div>
            <div className="field-desc">
              {boot.dev_build
                ? t("No disponible en la compilación de desarrollo.")
                : t("Comprueba si hay una versión nueva de kotoba.")}
            </div>
          </div>
          <button
            type="button"
            className="ghost"
            disabled={updating || boot.dev_build}
            onClick={() => void checkUpdates()}
          >
            <IconRefresh size={16} />
            {updating ? t("Comprobando…") : t("Buscar actualizaciones")}
          </button>
        </div>

        <div className="field">
          <div>
            <div className="field-label">{t("Registro de la app")}</div>
            <div className="field-desc">
              {t("Abre la carpeta con los logs para diagnosticar problemas.")}
            </div>
          </div>
          <button
            type="button"
            className="ghost"
            onClick={() => void openLogs()}
          >
            <IconFolder size={16} />
            {t("Abrir carpeta")}
          </button>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>{t("Datos")}</h2>
            <p className="muted">
              {t(
                "Todo es local. Puedes reimportar el mazo sin perder el progreso de las palabras que ya existen.",
              )}
            </p>
          </div>
        </div>

        <div className="data-actions">
          <button
            type="button"
            className="ghost"
            onClick={() => void reimport()}
            disabled={busy}
          >
            <IconImport size={16} />
            {t("Reimportar mazo")}
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void exportBackup()}
            disabled={busy}
          >
            <IconArchive size={16} />
            {t("Exportar copia")}
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void restoreBackup()}
            disabled={busy}
          >
            <IconUpload size={16} />
            {t("Restaurar copia")}
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void exportHistory()}
            disabled={busy}
          >
            <IconDownload size={16} />
            {t("Exportar historial")}
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void resetProgress()}
            disabled={busy}
          >
            <IconRefresh size={16} />
            {t("Reiniciar progreso")}
          </button>
          <button
            type="button"
            className="ghost danger"
            onClick={() => void clearLibrary()}
            disabled={busy}
          >
            <IconTrash size={16} />
            {t("Borrar biblioteca")}
          </button>
        </div>

        {busy && progress && (
          <div className="inline-progress">
            <div className="progress-track">
              <div
                className="progress-fill"
                style={{
                  width: `${
                    progress.total > 0
                      ? Math.min(100, (progress.current / progress.total) * 100)
                      : 0
                  }%`,
                }}
              />
            </div>
            <span className="small muted">
              {progress.current} / {progress.total}
            </span>
          </div>
        )}
      </article>

      <div className="settings-actions">
        <span className="muted">
          {dirty ? t("Hay cambios sin guardar.") : t("Todo guardado.")}
        </span>
        <div className="settings-buttons">
          <button
            type="button"
            className="ghost"
            onClick={() => setSettings(boot.settings)}
            disabled={!dirty || saving}
          >
            {t("Descartar")}
          </button>
          <button
            type="button"
            className="primary"
            onClick={() => void save()}
            disabled={!dirty || saving}
          >
            {saving ? t("Guardando…") : t("Guardar cambios")}
          </button>
        </div>
      </div>

      {inspection && (
        <MappingDialog
          inspection={inspection}
          language={settings.language}
          busy={busy}
          onCancel={() => {
            setInspection(null);
            setPendingPath(null);
          }}
          onImport={runMappedImport}
        />
      )}
    </section>
  );
}
