import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { api, type Bootstrap, type Settings } from "./api";
import {
  IconImport,
  IconRefresh,
  IconTrash,
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
    }
  }

  async function resetProgress() {
    if (
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
