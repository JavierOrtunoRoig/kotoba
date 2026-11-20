import { useEffect, useState } from "react";
import { open, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { api, type Bootstrap, type ImportProgress, type Settings } from "./api";
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
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`switch ${checked ? "on" : ""}`}
      onClick={() => onChange(!checked)}
    />
  );
}

export default function SettingsPanel({ boot, onChanged, onToast }: Props) {
  const [settings, setSettings] = useState<Settings>(boot.settings);
  const [saving, setSaving] = useState(false);
  const [busy, setBusy] = useState(false);
  const [autostart, setAutostart] = useState(false);
  const [progress, setProgress] = useState<ImportProgress | null>(null);

  useEffect(() => {
    const unlisten = listen<ImportProgress>("import:progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    isAutostartEnabled()
      .then((enabled) => {
        setAutostart(enabled);
        setSettings((current) => ({ ...current, launch_at_startup: enabled }));
      })
      .catch(() => undefined);
  }, []);

  const dirty = JSON.stringify(settings) !== JSON.stringify(boot.settings);

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
      onToast("Ajustes guardados");
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

  async function reimport() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Mazo de Anki", extensions: ["apkg", "txt"] },
        { name: "Todos", extensions: ["*"] },
      ],
    });
    if (!selected || Array.isArray(selected)) return;

    setBusy(true);
    setProgress(null);
    try {
      const summary = await api.importDeck(selected);
      onToast(
        `Importadas ${summary.items_inserted} palabras y ${
          summary.audio_extracted + summary.audio_reused
        } audios`,
      );
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
      onToast(`Copia guardada en ${folder}`);
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
        "Se reemplazarán las palabras, el audio y el progreso actuales. ¿Continuar?",
      )
    ) {
      return;
    }

    setBusy(true);
    try {
      const restored = await api.restoreBackup(selected);
      setSettings(restored);
      onToast("Copia restaurada");
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
      onToast(`Historial guardado en ${path}`);
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  async function resetProgress() {    if (
      !window.confirm(
        "¿Reiniciar el repaso? Se borra el historial y los intervalos, pero se conservan las palabras.",
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await api.resetProgress();
      onToast("Progreso reiniciado");
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
        "¿Borrar todas las palabras importadas? Tendrás que volver a importar el mazo.",
      )
    ) {
      return;
    }
    setBusy(true);
    try {
      await api.clearLibrary();
      onToast("Biblioteca borrada");
      await onChanged();
    } catch (caught) {
      onToast(String(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="settings">
      <header className="view-head">
        <div>
          <span className="eyebrow">Ajustes</span>
          <h1>Tu ritmo</h1>
          <p className="sub">
            Un equilibrio entre no olvidar y no molestar. Los cambios se aplican
            al guardar.
          </p>
        </div>
      </header>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Frecuencia</h2>
            <p className="muted">Cuándo y cuánto aparece el repaso.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Intervalo entre retos</div>
            <div className="field-desc">
              Cada cuánto puede aparecer un micro-reto.
            </div>
          </div>
          <div className="range-pair">
            <label className="range-control">
              <span>mín</span>
              <input
                type="range"
                min={1}
                max={120}
                value={settings.interval_min_minutes}
                onChange={(event) =>
                  changeMinInterval(Number(event.currentTarget.value))
                }
              />
              <output>{settings.interval_min_minutes} min</output>
            </label>
            <label className="range-control">
              <span>máx</span>
              <input
                type="range"
                min={1}
                max={180}
                value={settings.interval_max_minutes}
                onChange={(event) =>
                  changeMaxInterval(Number(event.currentTarget.value))
                }
              />
              <output>{settings.interval_max_minutes} min</output>
            </label>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Máximo por día</div>
            <div className="field-desc">
              {settings.max_per_day === 0
                ? "Sin límite diario."
                : `Hasta ${settings.max_per_day} repasos al día.`}
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={200}
              step={5}
              value={settings.max_per_day}
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
            <div className="field-label">Pausar repasos</div>
            <div className="field-desc">
              La app sigue en la bandeja, pero no interrumpe.
            </div>
          </div>
          <Toggle
            checked={settings.paused}
            onChange={(value) => update("paused", value)}
            label="Pausar repasos"
          />
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>No molestar</h2>
            <p className="muted">Horas en las que kotoba se queda callado.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Activar horario de silencio</div>
            <div className="field-desc">
              Útil para la noche o para reuniones.
            </div>
          </div>
          <Toggle
            checked={settings.dnd_enabled}
            onChange={(value) => update("dnd_enabled", value)}
            label="Activar horario de silencio"
          />
        </div>

        <div className={`field ${!settings.dnd_enabled ? "disabled" : ""}`}>
          <div>
            <div className="field-label">Desde / hasta</div>
            <div className="field-desc">
              Si la hora final es menor, el rango cruza la medianoche.
            </div>
          </div>
          <div className="time-pair">
            <input
              type="time"
              value={settings.dnd_start}
              disabled={!settings.dnd_enabled}
              onChange={(event) => update("dnd_start", event.currentTarget.value)}
            />
            <span>→</span>
            <input
              type="time"
              value={settings.dnd_end}
              disabled={!settings.dnd_enabled}
              onChange={(event) => update("dnd_end", event.currentTarget.value)}
            />
          </div>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Progreso</h2>
            <p className="muted">Nivel, racha, logros y objetivos.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Objetivo semanal</div>
            <div className="field-desc">
              Repasos por semana. 0 = sin objetivo.
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={0}
              max={100}
              step={5}
              value={settings.weekly_goal}
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
            <div className="field-label">Maestría por lección</div>
            <div className="field-desc">
              Porcentaje de palabras con intervalo de 7 días o más para dar una
              lección por dominada.
            </div>
          </div>
          <div className="range-control single">
            <input
              type="range"
              min={50}
              max={100}
              step={5}
              value={settings.lesson_mastery_percent}
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
            <div className="field-label">Reparar racha</div>
            <div className="field-desc">
              Si fallas un día, gastar un salvavidas para mantener la racha. Los
              salvavidas se ganan al cumplir el objetivo semanal.
            </div>
          </div>
          <Toggle
            checked={settings.streak_repair}
            onChange={(value) => update("streak_repair", value)}
            label="Reparar racha"
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">Notificaciones de hitos</div>
            <div className="field-desc">
              Avisar al completar el objetivo semanal o desbloquear un logro.
              Respeta el modo no molestar y la pantalla completa.
            </div>
          </div>
          <Toggle
            checked={settings.milestone_notifications}
            onChange={(value) => update("milestone_notifications", value)}
            label="Notificaciones de hitos"
          />
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Aparición</h2>
            <p className="muted">Cómo y dónde aparece el micro-reto.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Modo</div>
            <div className="field-desc">
              Popup: tarjeta flotante. Widget: ventana siempre visible. Toast:
              notificación nativa de Windows.
            </div>
          </div>
          <div className="segmented" role="group" aria-label="Modo de aparición">
            <button
              type="button"
              className={settings.mode === "popup" ? "active" : ""}
              onClick={() => update("mode", "popup")}
            >
              Popup
            </button>
            <button
              type="button"
              className={settings.mode === "widget" ? "active" : ""}
              onClick={() => update("mode", "widget")}
            >
              Widget
            </button>
            <button
              type="button"
              className={settings.mode === "toast" ? "active" : ""}
              onClick={() => update("mode", "toast")}
            >
              Toast
            </button>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Silenciar en pantalla completa</div>
            <div className="field-desc">
              No interrumpir juegos, vídeos, presentaciones ni el modo silencio
              de Windows.
            </div>
          </div>
          <Toggle
            checked={settings.suppress_fullscreen}
            onChange={(value) => update("suppress_fullscreen", value)}
            label="Silenciar en pantalla completa"
          />
        </div>

        {settings.mode === "toast" && (
          <p className="small muted">
            El toast es una notificación nativa: toca la notificación o el icono
            de la bandeja para abrir el reto.
          </p>
        )}
        {settings.mode === "widget" && (
          <p className="small muted">
            El widget aparece en la esquina inferior derecha. Puedes arrastrarlo
            y ocultarlo cuando quieras.
          </p>
        )}
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Contenido</h2>
            <p className="muted">Qué se pregunta y cómo.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Dirección del ejercicio</div>
            <div className="field-desc">
              JP → ES para reconocer; ES → JP para producir. Alternar mezcla
              ambas.
            </div>
          </div>
          <div className="segmented" role="group" aria-label="Dirección">
            <button
              type="button"
              className={settings.direction_mode === "random" ? "active" : ""}
              onClick={() => update("direction_mode", "random")}
            >
              Aleatoria
            </button>
            <button
              type="button"
              className={settings.direction_mode === "alternate" ? "active" : ""}
              onClick={() => update("direction_mode", "alternate")}
            >
              Alternar
            </button>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Incluir plantillas y gramática</div>
            <div className="field-desc">
              Notas con 〜, «+ negative» o «...». Recomendado dejarlas fuera al
              principio.
            </div>
          </div>
          <Toggle
            checked={settings.include_patterns}
            onChange={(value) => update("include_patterns", value)}
            label="Incluir plantillas"
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">Audio automático en JP → ES</div>
            <div className="field-desc">
              En ES → JP el audio suena solo después de responder.
            </div>
          </div>
          <Toggle
            checked={settings.audio_autoplay}
            onChange={(value) => update("audio_autoplay", value)}
            label="Audio automático"
          />
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Sistema</h2>
            <p className="muted">Integración con Windows.</p>
          </div>
        </div>

        <div className="field">
          <div>
            <div className="field-label">Arrancar con Windows</div>
            <div className="field-desc">
              kotoba se inicia en la bandeja sin abrir ninguna ventana.
            </div>
          </div>
          <Toggle
            checked={autostart}
            onChange={(value) => void toggleAutostart(value)}
            label="Arrancar con Windows"
          />
        </div>

        <div className="field">
          <div>
            <div className="field-label">Registro de la app</div>
            <div className="field-desc">
              Abre la carpeta con los logs para diagnosticar problemas.
            </div>
          </div>
          <button
            type="button"
            className="ghost"
            onClick={() => void openLogs()}
          >
            <IconFolder size={16} />
            Abrir carpeta
          </button>
        </div>
      </article>

      <article className="panel">
        <div className="panel-head">
          <div>
            <h2>Datos</h2>
            <p className="muted">
              Todo es local. Puedes reimportar el mazo sin perder el progreso de
              las palabras que ya existen.
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
            Reimportar mazo
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void exportBackup()}
            disabled={busy}
          >
            <IconArchive size={16} />
            Exportar copia
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void restoreBackup()}
            disabled={busy}
          >
            <IconUpload size={16} />
            Restaurar copia
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void exportHistory()}
            disabled={busy}
          >
            <IconDownload size={16} />
            Exportar historial
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => void resetProgress()}
            disabled={busy}
          >
            <IconRefresh size={16} />
            Reiniciar progreso
          </button>
          <button
            type="button"
            className="ghost danger"
            onClick={() => void clearLibrary()}
            disabled={busy}
          >
            <IconTrash size={16} />
            Borrar biblioteca
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
          {dirty ? "Hay cambios sin guardar." : "Todo guardado."}
        </span>
        <div className="settings-buttons">
          <button
            type="button"
            className="ghost"
            onClick={() => setSettings(boot.settings)}
            disabled={!dirty || saving}
          >
            Descartar
          </button>
          <button
            type="button"
            className="primary"
            onClick={() => void save()}
            disabled={!dirty || saving}
          >
            {saving ? "Guardando…" : "Guardar cambios"}
          </button>
        </div>
      </div>
    </section>
  );
}
