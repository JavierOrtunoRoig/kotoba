import { useCallback, useEffect, useRef, useState } from "react";
import {
  api,
  directionLabel,
  type Direction,
  type ReviewLogEntry,
  type Stats,
} from "./api";
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
            {t("{n} nuevas disponibles", { n: stats.new_available })}
          </span>
          {stats.new_per_day > 0 && (
            <span className="stat-s">
              {t("Nuevas hoy: {n} / {m}", {
                n: stats.new_today,
                m: stats.new_per_day,
              })}
            </span>
          )}
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

      <ReviewHistory language={language} />

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

const HISTORY_PAGE_SIZE = 25;

type HistoryFilters = {
  direction: "" | Direction;
  result: "" | "hit" | "miss";
  from: string;
  to: string;
};

const EMPTY_FILTERS: HistoryFilters = {
  direction: "",
  result: "",
  from: "",
  to: "",
};

function startOfDaySeconds(value: string): number | null {
  if (!value) return null;
  const [year, month, day] = value.split("-").map(Number);
  if (!year || !month || !day) return null;
  return Math.floor(new Date(year, month - 1, day).getTime() / 1000);
}

/** Medianoche del día siguiente, para que el rango incluya el día "hasta". */
function endOfDaySeconds(value: string): number | null {
  if (!value) return null;
  const [year, month, day] = value.split("-").map(Number);
  if (!year || !month || !day) return null;
  return Math.floor(new Date(year, month - 1, day + 1).getTime() / 1000);
}

function formatLatency(milliseconds: number): string {
  return `${(milliseconds / 1000).toFixed(1)} s`;
}

function ReviewHistory({ language }: { language: Language }) {
  const t = useT(language);
  const [filters, setFilters] = useState<HistoryFilters>(EMPTY_FILTERS);
  const [entries, setEntries] = useState<ReviewLogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const requestId = useRef(0);

  const load = useCallback(
    async (nextPage: number, replace: boolean) => {
      const request = ++requestId.current;
      if (replace) {
        setLoading(true);
      } else {
        setLoadingMore(true);
      }
      try {
        const result = await api.reviewHistory({
          page: nextPage,
          perPage: HISTORY_PAGE_SIZE,
          direction: filters.direction || null,
          result:
            filters.result === "" ? null : filters.result === "hit",
          fromTs: startOfDaySeconds(filters.from),
          toTs: endOfDaySeconds(filters.to),
        });
        if (request !== requestId.current) return;
        setEntries((current) =>
          replace ? result.entries : [...current, ...result.entries],
        );
        setTotal(result.total);
        setHasMore(result.has_more);
        setPage(result.page);
        setError(null);
      } catch (caught) {
        if (request !== requestId.current) return;
        setError(String(caught));
      } finally {
        if (request === requestId.current) {
          setLoading(false);
          setLoadingMore(false);
        }
      }
    },
    [filters],
  );

  useEffect(() => {
    void load(0, true);
  }, [load]);

  const filtersActive =
    filters.direction !== "" ||
    filters.result !== "" ||
    filters.from !== "" ||
    filters.to !== "";

  return (
    <article className="panel history-panel">
      <div className="panel-head">
        <div>
          <h2>{t("Historial")}</h2>
          <p className="muted">
            {t("Tus últimos repasos, del más reciente al más antiguo.")}
          </p>
        </div>
        {filtersActive && (
          <button
            type="button"
            className="ghost small"
            onClick={() => setFilters(EMPTY_FILTERS)}
          >
            {t("Limpiar filtros")}
          </button>
        )}
      </div>

      <div className="history-filters">
        <label>
          <span>{t("Dirección")}</span>
          <select
            value={filters.direction}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                direction: event.target.value as HistoryFilters["direction"],
              }))
            }
          >
            <option value="">{t("Todos")}</option>
            <option value="jp_to_es">{directionLabel("jp_to_es", language)}</option>
            <option value="es_to_jp">{directionLabel("es_to_jp", language)}</option>
          </select>
        </label>

        <label>
          <span>{t("Resultado")}</span>
          <select
            value={filters.result}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                result: event.target.value as HistoryFilters["result"],
              }))
            }
          >
            <option value="">{t("Todos")}</option>
            <option value="hit">{t("Acierto")}</option>
            <option value="miss">{t("Fallo")}</option>
          </select>
        </label>

        <label>
          <span>{t("Desde")}</span>
          <input
            type="date"
            value={filters.from}
            max={filters.to || undefined}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                from: event.target.value,
              }))
            }
          />
        </label>

        <label>
          <span>{t("Hasta")}</span>
          <input
            type="date"
            value={filters.to}
            min={filters.from || undefined}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                to: event.target.value,
              }))
            }
          />
        </label>
      </div>

      {error && <p className="inline-error">{error}</p>}

      {loading ? (
        <p className="muted small">{t("Cargando")}…</p>
      ) : entries.length === 0 ? (
        <p className="muted small history-empty">
          {filtersActive
            ? t("No hay repasos con estos filtros.")
            : t(
                "Aún no hay repasos registrados. Practica para empezar el historial.",
              )}
        </p>
      ) : (
        <>
          <div className="history-scroll">
            <table className="history-table">
              <thead>
                <tr>
                  <th scope="col">{t("Fecha")}</th>
                  <th scope="col">{t("Dirección")}</th>
                  <th scope="col">{t("Lectura")}</th>
                  <th scope="col">{t("Significado")}</th>
                  <th scope="col">{t("Resultado")}</th>
                  <th scope="col">{t("Calidad")}</th>
                  <th scope="col">{t("Latencia")}</th>
                </tr>
              </thead>
              <tbody>
                {entries.map((entry, index) => (
                  <tr key={`${entry.answered_at}-${index}`}>
                    <td className="history-date">
                      {entry.answered_at_local}
                    </td>
                    <td>{directionLabel(entry.direction, language)}</td>
                    <td lang="ja">{entry.reading}</td>
                    <td className="history-meaning">{entry.meaning}</td>
                    <td>
                      <span
                        className={`history-result ${
                          entry.result ? "hit" : "miss"
                        }`}
                      >
                        {entry.result ? t("Acierto") : t("Fallo")}
                      </span>
                    </td>
                    <td className="history-number">{entry.quality}</td>
                    <td className="history-number">
                      {formatLatency(entry.latency_ms)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="history-foot">
            <span className="muted small">
              {t("{n} de {m} repasos", {
                n: entries.length,
                m: total,
              })}
            </span>
            {hasMore && (
              <button
                type="button"
                className="ghost small"
                disabled={loadingMore}
                onClick={() => void load(page + 1, false)}
              >
                {t("Cargar más")}
              </button>
            )}
          </div>
        </>
      )}
    </article>
  );
}
