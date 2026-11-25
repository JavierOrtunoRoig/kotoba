import { useCallback, useEffect, useRef, useState } from "react";
import {
  api,
  directionHint,
  directionLabel,
  formatInterval,
  japaneseDisplay,
  type AnswerResult,
  type Challenge,
  type Settings,
} from "./api";
import { useT } from "./i18n";
import { playAudio } from "./audio";
import { IconArrow, IconEye, IconRefresh, IconSound } from "./Icons";

type Props = {
  settings: Settings;
  nextDueSeconds: number | null;
  onRefresh: () => Promise<void> | void;
};

type Status = "loading" | "ready" | "answered" | "empty";

export default function PracticePanel({
  settings,
  nextDueSeconds,
  onRefresh,
}: Props) {
  const t = useT(settings.language);
  const [challenge, setChallenge] = useState<Challenge | null>(null);
  const [status, setStatus] = useState<Status>("loading");
  const [answer, setAnswer] = useState("");
  const [result, setResult] = useState<AnswerResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showReading, setShowReading] = useState(false);

  const shownAt = useRef(Date.now());
  const inputRef = useRef<HTMLInputElement>(null);
  const busy = useRef(false);

  const load = useCallback(
    async (excludeId?: number) => {
      setStatus("loading");
      setResult(null);
      setAnswer("");
      setError(null);
      setShowReading(false);

      try {
        const next = await api.nextChallenge(excludeId);
        if (!next) {
          setChallenge(null);
          setStatus("empty");
          return;
        }
        setChallenge(next);
        shownAt.current = Date.now();
        setStatus("ready");
        if (
          next.direction === "jp_to_es" &&
          settings.audio_autoplay &&
          next.item.has_audio
        ) {
          void playAudio(next.item.id);
        }
      } catch (caught) {
        setError(String(caught));
        setStatus("empty");
      }
    },
    [settings.audio_autoplay],
  );

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (status === "ready") {
      inputRef.current?.focus();
    }
  }, [status]);

  const submit = useCallback(
    async (override?: string) => {
      if (!challenge || busy.current) return;
      const value = override ?? answer;
      if (override === undefined && value.trim().length === 0) return;

      busy.current = true;
      try {
        const latency = Date.now() - shownAt.current;
        const res = await api.submitAnswer(
          challenge.item.id,
          challenge.direction,
          value,
          latency,
        );
        setResult(res);
        setStatus("answered");
        if (challenge.direction === "es_to_jp" && res.has_audio) {
          void playAudio(challenge.item.id);
        }
        await onRefresh();
      } catch (caught) {
        setError(String(caught));
      } finally {
        busy.current = false;
      }
    },
    [answer, challenge, onRefresh],
  );

  const next = useCallback(() => {
    void load(challenge?.item.id);
  }, [challenge?.item.id, load]);

  useEffect(() => {
    if (status !== "answered") return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Enter" || event.isComposing) return;
      const tag = (event.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "BUTTON") return;
      event.preventDefault();
      next();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [status, next]);

  if (status === "loading") {
    return (
      <section className="practice">
        <header className="practice-head">
          <span className="eyebrow">{t("Micro-reto")}</span>
        </header>
        <div className="flashcard skeleton">
          <div className="skeleton-line short" />
          <div className="skeleton-line big" />
          <div className="skeleton-line" />
        </div>
      </section>
    );
  }

  if (status === "empty" || !challenge) {
    const nextDue =
      nextDueSeconds === null
        ? null
        : formatInterval(Math.max(1, Math.ceil(nextDueSeconds / 60)), settings.language);
    return (
      <section className="practice">
        <header className="practice-head">
          <span className="eyebrow">{t("Micro-reto")}</span>
        </header>
        <div className="flashcard empty-card">
          <span className="empty-mark">空</span>
          <h2>
            {nextDue
              ? t("Nada que repasar todavía")
              : t("No hay palabras que practicar")}
          </h2>
          <p>
            {error ??
              (nextDue
                ? t("La próxima palabra vence en {interval}.", {
                    interval: nextDue,
                  })
                : t(
                    "Importa un mazo desde Ajustes o espera a que haya ítems disponibles.",
                  ))}
          </p>
          <div className="empty-actions">
            <button type="button" className="ghost" onClick={() => void load()}>
              <IconRefresh size={16} />
              {t("Reintentar")}
            </button>
          </div>
        </div>
      </section>
    );
  }

  const { item, direction } = challenge;
  const prompt =
    direction === "jp_to_es" ? japaneseDisplay(item) : item.meaning;

  return (
    <section className="practice">
      <header className="practice-head">
        <span className="eyebrow">{t("Micro-reto")}</span>
        <span className="direction">{directionLabel(direction, settings.language)}</span>
        <span className="due">
          {t("{n} pendientes", { n: challenge.due_count })}
          {challenge.new_count > 0 &&
            ` · ${t("{n} nuevas", { n: challenge.new_count })}`}
        </span>
      </header>

      <article className={`flashcard ${status === "answered" ? "answered" : ""}`}>
        <div className="card-top">
          <span>
            {item.lesson || "Genki 1"}
            {item.category ? ` · ${item.category}` : ""}
          </span>
          <span className="card-meta">
            {item.is_pattern && <em className="tag-pattern">{t("plantilla")}</em>}
            {item.irregular && <em className="tag-irregular">{t("irregular")}</em>}
          </span>
        </div>

        <div className="prompt">
          <p
            className={
              direction === "jp_to_es" ? "prompt-jp" : "prompt-es"
            }
            lang={direction === "jp_to_es" ? "ja" : "es"}
          >
            {prompt}
          </p>
          <p className="prompt-hint">
            {directionHint(direction, settings.language)}
          </p>
          {direction === "jp_to_es" && item.kanji && (
            <button
              type="button"
              className="reading-toggle"
              aria-expanded={showReading}
              onClick={() => setShowReading((value) => !value)}
            >
              <IconEye size={14} />
              {showReading ? t("Ocultar lectura") : t("Ver lectura")}
              {showReading && <span className="reading-value">{item.reading}</span>}
            </button>
          )}
        </div>

        {status !== "answered" ? (
          <form
            className="answer-form"
            onSubmit={(event) => {
              event.preventDefault();
              void submit();
            }}
          >
            <input
              ref={inputRef}
              className="answer-input"
              value={answer}
              onChange={(event) => setAnswer(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.nativeEvent.isComposing) {
                  event.preventDefault();
                  void submit();
                }
              }}
              aria-label={directionHint(direction, settings.language)}
              placeholder={
                direction === "jp_to_es"
                  ? t("Escribe el significado…")
                  : t("Kanji, kana o romaji…")
              }
              lang={direction === "es_to_jp" ? "ja" : "es"}
              autoComplete="off"
              autoCorrect="off"
              spellCheck={false}
            />
            <div className="actions">
              <button
                type="button"
                className="ghost"
                onClick={() => void submit("")}
                disabled={busy.current}
              >
                {t("No lo sé")}
              </button>
              <button
                type="submit"
                className="primary"
                disabled={answer.trim().length === 0}
                aria-keyshortcuts="Enter"
              >
                {t("Comprobar")}
                <span className="key-hint">↵</span>
              </button>
            </div>
          </form>
        ) : (
          result && (
            <div
              className={`feedback ${result.correct ? "ok" : "ko"}`}
              role="status"
              aria-live="polite"
            >
              <div className="feedback-head">
                <span className="feedback-mark">
                  {result.correct ? "正" : "×"}
                </span>
                <div>
                  <strong>{result.correct ? t("Correcto") : t("Casi")}</strong>
                  <span>
                    {result.correct
                      ? t("Bien hecho, sigue así.")
                      : t("La respuesta era:")}
                  </span>
                </div>
              </div>

              <div className="answer-block">
                <div className="answer-row">
                  <span className="label">{t("Japonés")}</span>
                  <span className="value jp" lang="ja">
                    {japaneseDisplay(item)}
                  </span>
                  <span className="reading">{item.reading}</span>
                </div>
                <div className="answer-row">
                  <span className="label">{t("Significado")}</span>
                  <span className="value">{result.meaning}</span>
                </div>
                {result.kanji_gloss && (
                  <div className="answer-row">
                    <span className="label">{t("Kanji")}</span>
                    <span className="value muted">{result.kanji_gloss}</span>
                  </div>
                )}
              </div>

              <div className="chips">
                {result.expected.slice(0, 5).map((value) => (
                  <span className="chip" key={value}>
                    {value}
                  </span>
                ))}
              </div>

              <div className="feedback-actions">
                <button
                  type="button"
                  className="ghost"
                  onClick={() => void playAudio(item.id)}
                  disabled={!result.has_audio}
                >
                  <IconSound size={16} />
                  {t("Oír")}
                </button>
                {result.xp_gained > 0 && (
                  <span className="xp-badge">+{result.xp_gained} XP</span>
                )}
                {result.leveled_up && (
                  <span className="level-up">
                    {t("¡Nivel {n}!", { n: result.level })}
                  </span>
                )}
                {result.unlocked.map((achievement) => (
                  <span className="achievement-chip" key={achievement.key}>
                    ✦ {t(achievement.title)}
                  </span>
                ))}
                {result.saver_earned && (
                  <span className="saver-chip">{t("+1 salvavidas")}</span>
                )}
                <span className="next-interval">
                  {t("Vuelve en {interval}", {
                    interval: formatInterval(
                      result.next_interval_minutes,
                      settings.language,
                    ),
                  })}
                </span>
                <button
                  type="button"
                  className="primary"
                  onClick={next}
                  aria-keyshortcuts="Enter"
                >
                  {t("Siguiente")}
                  <IconArrow size={16} />
                </button>
              </div>
            </div>
          )
        )}
      </article>

      {error && <p className="inline-error">{error}</p>}
    </section>
  );
}
