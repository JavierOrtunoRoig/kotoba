import { useCallback, useEffect, useState } from "react";
import { api, type Bootstrap } from "./api";
import ImportPanel from "./ImportPanel";
import PracticePanel from "./PracticePanel";
import StatsPanel from "./StatsPanel";
import SettingsPanel from "./SettingsPanel";
import {
  IconChart,
  IconPractice,
  IconSettings,
} from "./Icons";

type View = "practice" | "stats" | "settings";

const NAV: { id: View; label: string; Icon: typeof IconPractice }[] = [
  { id: "practice", label: "Practicar", Icon: IconPractice },
  { id: "stats", label: "Progreso", Icon: IconChart },
  { id: "settings", label: "Ajustes", Icon: IconSettings },
];

export default function MainApp() {
  const [boot, setBoot] = useState<Bootstrap | null>(null);
  const [view, setView] = useState<View>("practice");
  const [toast, setToast] = useState<string | null>(null);

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
    if (view === "stats") {
      void refresh();
    }
  }, [view, refresh]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4200);
    return () => window.clearTimeout(timer);
  }, [toast]);

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
        onImported={async () => {
          await refresh();
          setView("practice");
        }}
      />
    );
  }

  return (
    <div className="shell">
      <aside className="rail">
        <div className="brand">
          <span className="brand-kanji">言葉</span>
          <div className="brand-text">
            <span className="brand-latin">kotoba</span>
            <span className="brand-sub">repaso pasivo</span>
          </div>
        </div>

        <nav className="rail-nav" aria-label="Secciones">
          {NAV.map(({ id, label, Icon }) => (
            <button
              key={id}
              type="button"
              className={`nav-item ${view === id ? "active" : ""}`}
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
            {boot.settings.paused ? "En pausa" : "Activo"}
          </div>
          <div className="rail-meta">
            {boot.stats.due_now} pendientes · {boot.stats.total_items} palabras
          </div>
          <div className="rail-meta">
            {boot.settings.interval_min_minutes}–{boot.settings.interval_max_minutes} min
            entre retos
          </div>
        </div>
      </aside>

      <main className="content">
        {view === "practice" && (
          <PracticePanel settings={boot.settings} onRefresh={refresh} />
        )}
        {view === "stats" && (
          <StatsPanel onPractice={() => setView("practice")} />
        )}
        {view === "settings" && (
          <SettingsPanel
            boot={boot}
            onChanged={refresh}
            onToast={setToast}
          />
        )}
      </main>

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
