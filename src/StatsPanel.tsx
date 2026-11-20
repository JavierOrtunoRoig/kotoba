import { useCallback, useEffect, useState } from "react";
import { api, type Stats } from "./api";
import { useT, type Language } from "./i18n";
import { IconArrow, IconFlame, IconSeed, IconTarget } from "./Icons";

type Props = {
  language: Language;
  onPractice: () => void;
};

const LOCALES: Record<Language, string> = {
  es: "es-ES",
  en: "en-GB",
  ja: "ja-JP",
};

export default function StatsPanel({ language, onPractice }: Props) {
  const t = useT(language);
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
            <span className="eyebrow">{t("Progreso")}</span>
            <h1>{t("Tu progreso")}</h1>
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
  const separator = language === "ja" ? "・" : " · ";
  const streakParts = [
    stats.streak_days === 1 ? t("día seguido") : t("días seguidos"),
  ];
  if (stats.streak_best > 0) {
    streakParts.push(t("mejor {n}", { n: stats.streak_best }));
  }
  if (stats.streak_savers > 0) {
    streakParts.push(
      stats.streak_savers === 1
        ? t("1 salvavidas")
        : t("{n} salvavidas", { n: stats.streak_savers }),
    );
  }

  return (
    <section className="stats">
      <header className="view-head">
        <div>
          <span className="eyebrow">{t("Progreso")}</span>
          <h1>{t("Tu progreso")}</h1>
          <p className="sub">
            {t(
              "Sin colas ni deberes: solo lo que has ido reteniendo sin darte cuenta.",
            )}
          </p>
        </div>
        <button type="button" className="ghost" onClick={() => void refresh()}>
          {t("Actualizar")}
        </button>
      </header>

      {error && <p className="inline-error">{error}</p>}

      <div className="stat-grid">
        <article className="stat">
          <span className="stat-k">
            <IconTarget size={14} /> {t("Precisión hoy")}
          </span>
          <span className="stat-v">
            {Math.round(stats.accuracy_today)}
            <small>%</small>
          </span>
          <span className="stat-s">
            {t("{correct} de {total} repasos", {
              correct: stats.correct_today,
              total: stats.reviewed_today,
            })}
          </span>
        </article>

        <article className="stat">
          <span className="stat-k">
            <IconFlame size={14} /> {t("Racha")}
          </span>
          <span className="stat-v">
            {stats.streak_days}
            <small>d</small>
          </span>
          <span className="stat-s">
            {streakParts.join(separator)}
          </span>
        </article>

        <article className="stat">
          <span className="stat-k">
            <IconSeed size={14} /> {t("Dominadas")}
          </span>
          <span className="stat-v">{stats.mastered}</span>
          <span className="stat-s">{t("con intervalo de 7 días o más")}</span>
        </article>

        <article className="stat">
          <span className="stat-k">{t("Pendientes")}</span>
          <span className="stat-v">{stats.due_now}</span>
          <span className="stat-s">
            {t("{n} palabras en total", { n: stats.total_items })}
          </span>
        </article>
      </div>

      <div className="game-grid">
        <article className="panel level-panel">
          <div className="level-badge">{stats.level}</div>
          <div className="level-info">
            <div className="level-title">
              <span>
                {t("Nivel {n} · {title}", {
                  n: stats.level,
                  title: t(stats.level_title),
                })}
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
                ? t("{n} XP para el siguiente nivel", {
                    n: Math.max(0, stats.next_level_xp - stats.xp_total),
                  })
                : t("Nivel máximo")}
            </div>
          </div>
        </article>

        <article className="panel goal-panel">
          <div className="panel-head">
            <div>
              <h2>{t("Objetivo semanal")}</h2>
              <p className="muted">
                {t("{n} / {m} repasos", {
                  n: stats.week_reviews,
                  m: stats.weekly_goal === 0 ? "∞" : stats.weekly_goal,
                })}
              </p>
            </div>
            {stats.weekly_goal > 0 &&
              stats.week_reviews >= stats.weekly_goal && (
                <span className="goal-done">{t("Completado")}</span>
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
              ? t("Sin objetivo: repasa a tu ritmo.")
              : stats.week_reviews >= stats.weekly_goal
                ? t("Objetivo cumplido. Sigue así.")
                : t("Cada repaso cuenta, también los fallos.")}
          </span>
          <span className="muted small">
            {stats.streak_repairs > 0
              ? stats.streak_repairs === 1
                ? t("{n} día reparado con salvavidas.", {
                    n: stats.streak_repairs,
                  })
                : t("{n} días reparados con salvavidas.", {
                    n: stats.streak_repairs,
                  })
              : t("Cada objetivo completado da un salvavidas (máx. {n}).", {
                  n: stats.streak_savers_max,
                })}
          </span>
        </article>
      </div>

      <article className="panel achievements-panel">
        <div className="panel-head">
          <div>
            <h2>{t("Logros")}</h2>
            <p className="muted">
              {t("{n} de {m} desbloqueados. Los logros no caducan.", {
                n: unlockedCount,
                m: achievements.length,
              })}
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
            const title = t(achievement.title);
            const description = t(achievement.description);
            const date = achievement.unlocked_at
              ? new Date(achievement.unlocked_at * 1000).toLocaleDateString(
                  LOCALES[language],
                  { day: "numeric", month: "short" },
                )
              : null;
            return (
              <div
                key={achievement.key}
                className={`achievement${achievement.unlocked ? " unlocked" : ""}`}
                title={description}
              >
                <span className="achievement-mark">
                  {achievement.unlocked ? "✓" : "•"}
                </span>
                <div className="achievement-body">
                  <div className="achievement-title">{title}</div>
                  <div className="achievement-desc">{description}</div>
                  {achievement.unlocked ? (
                    <span className="achievement-meta">
                      {t("Desbloqueado")}
                      {date ? ` · ${date}` : ""}
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
            <h2>{t("Últimos 14 días")}</h2>
            <p className="muted">
              {t("Barra clara: repasos · barra oscura: aciertos")}
            </p>
          </div>
          <span className="activity-total">
            {t("{n} repasos en total", { n: stats.total_reviews })}
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
                ? t("hoy")
                : date
                    .toLocaleDateString(LOCALES[language], {
                      weekday: "narrow",
                    })
                    .slice(0, 2);
            return (
              <div
                className="activity-bar"
                key={day.date}
                title={t("{date}: {total} repasos, {correct} aciertos", {
                  date: day.date,
                  total: day.total,
                  correct: day.correct,
                })}
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
              <h2>{t("Por lección")}</h2>
              <p className="muted">
                {t(
                  "Progreso y precisión de cada capítulo. Una lección se domina con al menos el {n}% de sus palabras en 7 días o más.",
                  { n: stats.lesson_mastery_percent },
                )}
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
                    title={t("Maestría: {n}%", {
                      n: Math.round(lesson.mastery_percent),
                    })}
                  >
                    {lesson.is_mastered
                      ? t("Dominada")
                      : `${Math.round(lesson.mastery_percent)}%`}
                  </span>
                  <span className="lesson-accuracy" title={t("Precisión")}>
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
              <h2>{t("Palabras difíciles")}</h2>
              <p className="muted">{t("Las que más fallos acumulan.")}</p>
            </div>
          </div>
          {difficult.length === 0 ? (
            <p className="muted small">
              {t("Aún no hay fallos registrados. Sigue repasando.")}
            </p>
          ) : (
            <ul className="difficult-list">
              {difficult.map((item) => (
                <li key={item.id}>
                  <span className="difficult-jp" lang="ja">
                    {item.japanese}
                  </span>
                  <span className="difficult-count">
                    {item.failures === 1
                      ? t("{n} fallo", { n: item.failures })
                      : t("{n} fallos", { n: item.failures })}
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
          {t(
            "Un micro-reto de 10 segundos vale más que una sesión que nunca empiezas.",
          )}
        </p>
        <button type="button" className="primary" onClick={onPractice}>
          {t("Practicar ahora")}
          <IconArrow size={16} />
        </button>
      </div>
    </section>
  );
}
