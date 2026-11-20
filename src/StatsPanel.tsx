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
  const lessons = stats.lessons ?? [];
  const difficult = stats.difficult ?? [];
  const max = Math.max(1, ...activity.map((day) => day.total));
  const achievements = stats.achievements ?? [];
  const unlockedCount = achievements.filter((a) => a.unlocked).length;

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
            {stats.streak_best > 0 && ` · mejor ${stats.streak_best}`}
            {stats.streak_savers > 0 &&
              ` · ${stats.streak_savers} salvavidas`}
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

      <div className="game-grid">
        <article className="panel level-panel">
          <div className="level-badge">{stats.level}</div>
          <div className="level-info">
            <div className="level-title">
              <span>
                Nivel {stats.level} · {stats.level_title}
              </span>
              <span className="level-xp">{stats.xp_total} XP</span>
            </div>
            <div className="level-bar">
              <div
                className="level-fill"
                style={{ width: `${stats.level_progress}%` }}
              />
            </div>
            <div className="muted small">
              {stats.next_level_xp
                ? `${Math.max(0, stats.next_level_xp - stats.xp_total)} XP para el siguiente nivel`
                : "Nivel máximo"}
            </div>
          </div>
        </article>

        <article className="panel goal-panel">
          <div className="panel-head">
            <div>
              <h2>Objetivo semanal</h2>
              <p className="muted">
                {stats.week_reviews} /{" "}
                {stats.weekly_goal === 0 ? "∞" : stats.weekly_goal} repasos
              </p>
            </div>
            {stats.weekly_goal > 0 &&
              stats.week_reviews >= stats.weekly_goal && (
                <span className="goal-done">Completado</span>
              )}
          </div>
          <div className="goal-bar">
            <div
              className="goal-fill"
              style={{
                width: `${
                  stats.weekly_goal > 0
                    ? Math.min(
                        100,
                        (stats.week_reviews / stats.weekly_goal) * 100,
                      )
                    : 0
                }%`,
              }}
            />
          </div>
          <span className="muted small">
            {stats.weekly_goal === 0
              ? "Sin objetivo: repasa a tu ritmo."
              : stats.week_reviews >= stats.weekly_goal
                ? "Objetivo cumplido. Sigue así."
                : "Cada repaso cuenta, también los fallos."}
          </span>
          <span className="muted small">
            {stats.streak_repairs > 0
              ? `${stats.streak_repairs} ${stats.streak_repairs === 1 ? "día reparado" : "días reparados"} con salvavidas.`
              : `Cada objetivo completado da un salvavidas (máx. ${stats.streak_savers_max}).`}
          </span>
        </article>
      </div>

      <article className="panel achievements-panel">
        <div className="panel-head">
          <div>
            <h2>Logros</h2>
            <p className="muted">
              {unlockedCount} de {achievements.length} desbloqueados. Los
              logros no caducan.
            </p>
          </div>
          <span className="achievements-count">
            {Math.round((unlockedCount / Math.max(1, achievements.length)) * 100)}
            %
          </span>
        </div>
        <div className="achievement-grid">
          {achievements.map((achievement) => {
            const percent =
              achievement.target > 0
                ? (achievement.progress / achievement.target) * 100
                : 0;
            const date = achievement.unlocked_at
              ? new Date(achievement.unlocked_at * 1000).toLocaleDateString(
                  "es-ES",
                  { day: "numeric", month: "short" },
                )
              : null;
            return (
              <div
                key={achievement.key}
                className={`achievement${achievement.unlocked ? " unlocked" : ""}`}
                title={achievement.description}
              >
                <span className="achievement-mark">
                  {achievement.unlocked ? "✓" : "•"}
                </span>
                <div className="achievement-body">
                  <div className="achievement-title">{achievement.title}</div>
                  <div className="achievement-desc">
                    {achievement.description}
                  </div>
                  {achievement.unlocked ? (
                    <span className="achievement-meta">
                      Desbloqueado{date ? ` · ${date}` : ""}
                    </span>
                  ) : (
                    <div className="achievement-progress">
                      <div className="achievement-bar">
                        <div
                          className="achievement-fill"
                          style={{ width: `${percent}%` }}
                        />
                      </div>
                      <span className="achievement-meta">
                        {achievement.progress}/{achievement.target}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </article>

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

      <div className="stats-columns">
        <article className="panel">
          <div className="panel-head">
            <div>
              <h2>Por lección</h2>
              <p className="muted">
                Progreso y precisión de cada capítulo. Una lección se domina con
                al menos el {stats.lesson_mastery_percent}% de sus palabras en 7
                días o más.
              </p>
            </div>
          </div>
          <div className="lesson-list">
            {lessons.map((lesson) => {
              const progress =
                lesson.total > 0 ? (lesson.mastered / lesson.total) * 100 : 0;
              return (
                <div className="lesson-row" key={lesson.lesson}>
                  <span className="lesson-name">{lesson.lesson}</span>
                  <div className="lesson-bar">
                    <div
                      className="lesson-fill"
                      style={{ width: `${progress}%` }}
                    />
                  </div>
                  <span className="lesson-meta">
                    {lesson.mastered}/{lesson.total}
                  </span>
                  <span
                    className={`lesson-badge${lesson.is_mastered ? " done" : ""}`}
                    title={`Maestría: ${Math.round(lesson.mastery_percent)}%`}
                  >
                    {lesson.is_mastered
                      ? "Dominada"
                      : `${Math.round(lesson.mastery_percent)}%`}
                  </span>
                  <span className="lesson-accuracy" title="Precisión">
                    {lesson.reviews > 0
                      ? `${Math.round(lesson.accuracy)}%`
                      : "—"}
                  </span>
                </div>
              );
            })}
          </div>
        </article>

        <article className="panel">
          <div className="panel-head">
            <div>
              <h2>Palabras difíciles</h2>
              <p className="muted">Las que más fallos acumulan.</p>
            </div>
          </div>
          {difficult.length === 0 ? (
            <p className="muted small">
              Aún no hay fallos registrados. Sigue repasando.
            </p>
          ) : (
            <ul className="difficult-list">
              {difficult.map((item) => (
                <li key={item.id}>
                  <span className="difficult-jp" lang="ja">
                    {item.japanese}
                  </span>
                  <span className="difficult-count">
                    {item.failures} {item.failures === 1 ? "fallo" : "fallos"}
                  </span>
                  <span className="difficult-meaning">{item.meaning}</span>
                </li>
              ))}
            </ul>
          )}
        </article>
      </div>

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
