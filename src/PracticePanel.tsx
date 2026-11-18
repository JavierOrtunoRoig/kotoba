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
import { playAudio } from "./audio";
import { IconArrow, IconEye, IconSound } from "./Icons";

type Props = {
  settings: Settings;
  onRefresh: () => Promise<void> | void;
};

type Status = "loading" | "ready" | "answered" | "empty";

export default function PracticePanel({ settings, onRefresh }: Props) {
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

  if (status === "loading") {
    return (
      <section className="practice">
        <header className="practice-head">
          <span className="eyebrow">Micro-reto</span>
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
    return (
      <section className="practice">
        <header className="practice-head">
          <span className="eyebrow">Micro-reto</span>
        </header>
        <div className="flashcard empty-card">
          <span className="empty-mark">空</span>
          <h2>No hay palabras que practicar</h2>
          <p>
            {error ??
              "Importa un mazo desde Ajustes o espera a que haya ítems disponibles."}
          </p>
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
        <span className="eyebrow">Micro-reto</span>
        <span className="direction">{directionLabel(direction)}</span>
        <span className="due">{challenge.due_count} pendientes</span>
      </header>

      <article className={`flashcard ${status === "answered" ? "answered" : ""}`}>
        <div className="card-top">
          <span>
            {item.lesson || "Genki 1"}
            {item.category ? ` · ${item.category}` : ""}
          </span>
          <span className="card-meta">
            {item.is_pattern && <em className="tag-pattern">plantilla</em>}
            {item.irregular && <em className="tag-irregular">irregular</em>}
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
          <p className="prompt-hint">{directionHint(direction)}</p>
          {direction === "jp_to_es" && item.kanji && (
            <button
              type="button"
              className="reading-toggle"
              onClick={() => setShowReading((value) => !value)}
            >
              <IconEye size={14} />
              {showReading ? "Ocultar lectura" : "Ver lectura"}
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
              placeholder={
                direction === "jp_to_es"
                  ? "Escribe el significado…"
                  : "Kanji, kana o romaji…"
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
                No lo sé
              </button>
              <button
                type="submit"
                className="primary"
                disabled={answer.trim().length === 0}
              >
                Comprobar
                <span className="key-hint">↵</span>
              </button>
            </div>
          </form>
        ) : (
          result && (
            <div className={`feedback ${result.correct ? "ok" : "ko"}`}>
              <div className="feedback-head">
                <span className="feedback-mark">
                  {result.correct ? "正" : "×"}
                </span>
                <div>
                  <strong>{result.correct ? "Correcto" : "Casi"}</strong>
                  <span>
                    {result.correct
                      ? "Bien hecho, sigue así."
                      : "La respuesta era:"}
                  </span>
                </div>
              </div>

              <div className="answer-block">
                <div className="answer-row">
                  <span className="label">Japonés</span>
                  <span className="value jp" lang="ja">
                    {japaneseDisplay(item)}
                  </span>
                  <span className="reading">{item.reading}</span>
                </div>
                <div className="answer-row">
                  <span className="label">Significado</span>
                  <span className="value">{result.meaning}</span>
                </div>
                {result.kanji_gloss && (
                  <div className="answer-row">
                    <span className="label">Kanji</span>
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
                  Oír
                </button>
                <span className="next-interval">
                  Vuelve en {formatInterval(result.next_interval_minutes)}
                </span>
                <button type="button" className="primary" onClick={next}>
                  Siguiente
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
