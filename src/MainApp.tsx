import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import { api, type Bootstrap } from "./api";
import { applyAppearance } from "./appearance";
import { useT } from "./i18n";
import ImportPanel from "./ImportPanel";
import PracticePanel from "./PracticePanel";
import StatsPanel from "./StatsPanel";
import SettingsPanel from "./SettingsPanel";
import { findUpdate, installUpdate } from "./updates";
import {
  IconChart,
  IconPractice,
  IconSettings,
} from "./Icons";

type View = "practice" | "stats" | "settings";

/** Versión que el usuario decidió posponer, para no repetir el aviso en cada arranque. */
const SKIPPED_UPDATE = "kotoba.skipped_update";

export default function MainApp() {
  const [boot, setBoot] = useState<Bootstrap | null>(null);
  const [view, setView] = useState<View>("practice");
  const [toast, setToast] = useState<string | null>(null);

  const language = boot?.settings.language ?? "es";
  const t = useT(language);

  const refresh = useCallback(async () => {
    try {
      setBoot(await api.bootstrap());
    } catch (error) {
      setToast(String(error));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("settings:changed", () => {
      void refresh();
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [refresh]);

  useEffect(() => {
    if (view === "stats") {
      void refresh();
    }
  }, [view, refresh]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (boot) {
      applyAppearance(boot.settings.theme, boot.settings.font_scale);
    }
  }, [boot?.settings.theme, boot?.settings.font_scale]);

  // Aviso de actualización al arrancar. Solo en la app instalada: firmar y actualizar un
  // binario de desarrollo rompería el flujo de `tauri dev`.
  useEffect(() => {
    if (!boot || boot.dev_build) return;
    let alive = true;
    void (async () => {
      try {
        const update = await findUpdate();
        if (!alive || !update) return;
        if (window.localStorage.getItem(SKIPPED_UPDATE) === update.version) {
          await update.close();
          return;
        }

        const win = getCurrentWindow();
        await win.show();
        await win.setFocus();
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
          window.localStorage.setItem(SKIPPED_UPDATE, update.version);
          await update.close();
          return;
        }
        await installUpdate(update);
      } catch {
        // Sin red o sin `latest.json`: no molestar en el arranque.
      }
    })();
    return () => {
      alive = false;
    };
    // `t` cambia con el idioma; el aviso se lanza una vez por sesión.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [boot?.dev_build]);

  if (!boot) {
    return (
      <div className="boot">
        <span className="boot-seal">言</span>
        <span className="boot-word">kotoba</span>
      </div>
    );
  }

  if (boot.item_count === 0) {
    return (
      <ImportPanel
        language={boot.settings.language}
        onImported={async () => {
          await refresh();
          setView("practice");
        }}
      />
    );
  }

  const nav: { id: View; label: string; Icon: typeof IconPractice }[] = [
    { id: "practice", label: t("Practicar"), Icon: IconPractice },
    { id: "stats", label: t("Progreso"), Icon: IconChart },
    { id: "settings", label: t("Ajustes"), Icon: IconSettings },
  ];

  return (
    <div className="shell">
      <a className="skip-link" href="#main-content">
        {t("Saltar al contenido")}
      </a>

      <aside className="rail">
        <div className="brand">
          <span className="brand-kanji">言葉</span>
          <div className="brand-text">
            <span className="brand-latin">kotoba</span>
            <span className="brand-sub">{t("repaso pasivo")}</span>
          </div>
        </div>

        <nav className="rail-nav" aria-label={t("Secciones")}>
          {nav.map(({ id, label, Icon }) => (
            <button
              key={id}
              type="button"
              className={`nav-item ${view === id ? "active" : ""}`}
              aria-current={view === id ? "page" : undefined}
              onClick={() => setView(id)}
            >
              <Icon size={18} />
              {label}
            </button>
          ))}
        </nav>

        <div className="rail-foot">
          <div className="rail-status">
            <span className={`status-dot ${boot.settings.paused ? "paused" : ""}`} />
            {boot.settings.paused ? t("En pausa") : t("Activo")}
          </div>
          <div className="rail-meta">
            {t("{due} pendientes · {new} nuevas", {
              due: boot.stats.due_now,
              new: boot.stats.new_available,
            })}
          </div>
          <div className="rail-meta">
            {t("{min}–{max} min entre retos", {
              min: boot.settings.interval_min_minutes,
              max: boot.settings.interval_max_minutes,
            })}
          </div>
        </div>
      </aside>

      <main className="content" id="main-content" tabIndex={-1}>
        {view === "practice" && (
          <PracticePanel
            settings={boot.settings}
            nextDueSeconds={boot.stats.next_due_seconds}
            onRefresh={refresh}
          />
        )}
        {view === "stats" && (
          <StatsPanel language={language} onPractice={() => setView("practice")} />
        )}
        {view === "settings" && (
          <SettingsPanel
            boot={boot}
            onChanged={refresh}
            onToast={setToast}
          />
        )}
      </main>

      {toast && (
        <div className="toast" role="status" aria-live="polite">
          {toast}
        </div>
      )}
    </div>
  );
}
