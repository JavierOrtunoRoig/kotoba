import { useCallback, useEffect, useState } from "react";
import { api, type Stats } from "./api";
import { IconArrow, IconFlame, IconSeed, IconTarget } from "./Icons";

type Props = {
  onPractice: () => void;
};

export default function StatsPanel({ onPractice }: Props) {
  const [stats, setStats] = useState<Stats | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStats(await api.getStats());
      setError(null);
    } catch (caught) {
      setError(String(caught));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!stats) {
    return (
      <section className="stats">
        <header className="view-head">
          <div>
            <span className="eyebrow">Progreso</span>
            <h1>Tu progreso</h1>
          </div>
        </header>
        <div className="skeleton-card">
          <div className="skeleton-line short" />
          <div className="skeleton-line big" />
        </div>
      </section>
    );
  }

  const activity = stats.activity ?? [];
  const max = Math.max(1, ...activity.map((day) => day.total));

  return (
    <section className="stats">
      <header className="view-head">
        <div>
          <span className="eyebrow">Progreso</span>
          <h1>Tu progreso</h1>
          <p className="sub">
            Sin colas ni deberes: solo lo que has ido reteniendo sin darte cuenta.
          </p>
        </div>
        <button type="button" className="ghost" onClick={() => void refresh()}>
          Actualizar
        </button>
      </header>

      {error && <p className="inline-error">{error}</p>}

      <div className="stat-grid">
        <article className="stat">
          <span className="stat-k">
            <IconTarget size={14} /> Precisión hoy
          </span>
          <span className="stat-v">
            {Math.round(stats.accuracy_today)}
            <small>%</small>
          </span>
          <span className="stat-s">
            {stats.correct_today} de {stats.reviewed_today}{" "}
            {stats.reviewed_today === 1 ? "repaso" : "repasos"}
          </span>
        </article>

        <article className="stat">
          <span className="stat-k">
            <IconFlame size={14} /> Racha
          </span>
          <span className="stat-v">
            {stats.streak_days}
            <small>d</small>
          </span>
          <span className="stat-s">
            {stats.streak_days === 1 ? "día seguido" : "días seguidos"}
          </span>
        </article>

        <article className="stat">
          <span className="stat-k">
            <IconSeed size={14} /> Dominadas
          </span>
          <span className="stat-v">{stats.mastered}</span>
          <span className="stat-s">con intervalo de 7 días o más</span>
        </article>

        <article className="stat">
          <span className="stat-k">Pendientes</span>
          <span className="stat-v">{stats.due_now}</span>
          <span className="stat-s">{stats.total_items} palabras en total</span>
        </article>
      </div>

      <article className="panel activity-panel">
        <div className="panel-head">
          <div>
            <h2>Últimos 14 días</h2>
            <p className="muted">
              Barra clara: repasos · barra oscura: aciertos
            </p>
          </div>
          <span className="activity-total">
            {stats.total_reviews}{" "}
            {stats.total_reviews === 1 ? "repaso" : "repasos"} en total
          </span>
        </div>

        <div className="activity">
          {activity.map((day, index) => {
            const height = (day.total / max) * 100;
            const correctHeight =
              day.total > 0 ? (day.correct / day.total) * 100 : 0;
            const date = new Date(`${day.date}T00:00:00`);
            const label =
              index === activity.length - 1
                ? "hoy"
                : date
                    .toLocaleDateString("es-ES", { weekday: "short" })
                    .slice(0, 2);
            return (
              <div
                className="activity-bar"
                key={day.date}
                title={`${day.date}: ${day.total} repasos, ${day.correct} aciertos`}
              >
                <div className="bar-track">
                  <div className="bar" style={{ height: `${height}%` }}>
                    <div
                      className="bar-correct"
                      style={{ height: `${correctHeight}%` }}
                    />
                  </div>
                </div>
                <span className="day">{label}</span>
                <span className="count">{day.total || ""}</span>
              </div>
            );
          })}
        </div>
      </article>

      <div className="stats-actions">
        <p className="muted">
          Un micro-reto de 10 segundos vale más que una sesión que nunca empiezas.
        </p>
        <button type="button" className="primary" onClick={onPractice}>
          Practicar ahora
          <IconArrow size={16} />
        </button>
      </div>
    </section>
  );
}
