import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  api,
  directionHint,
  directionLabel,
  japaneseDisplay,
  type AnswerResult,
  type Challenge,
  type Settings,
} from "./api";
import { playAudio } from "./audio";
import { IconArrow, IconClose, IconSound } from "./Icons";

type Status = "loading" | "ready" | "answered" | "empty";

type Props = {
  variant: "popup" | "widget";
};

export default function PopupApp({ variant }: Props) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [challenge, setChallenge] = useState<Challenge | null>(null);
  const [status, setStatus] = useState<Status>("loading");
  const [answer, setAnswer] = useState("");
  const [result, setResult] = useState<AnswerResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const shownAt = useRef(Date.now());
  const inputRef = useRef<HTMLInputElement>(null);
  const busy = useRef(false);

  const load = useCallback(
    async (excludeId?: number) => {
      setStatus("loading");
      setResult(null);
      setAnswer("");
      setError(null);
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
          (settings?.audio_autoplay ?? true) &&
          next.item.has_audio
        ) {
          void playAudio(next.item.id);
        }
      } catch (caught) {
        setError(String(caught));
        setStatus("empty");
      }
    },
    [settings?.audio_autoplay],
  );

  useEffect(() => {
    void load();
    // Solo al montar: el resto de cargas vienen por evento.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    api
      .getSettings()
      .then(setSettings)
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    const unlistenShow = listen("challenge:show", () => {
      void load();
    });
    const unlistenSettings = listen<Settings>("settings:changed", (event) => {
      setSettings(event.payload);
    });
    return () => {
      void unlistenShow.then((fn) => fn());
      void unlistenSettings.then((fn) => fn());
    };
  }, [load]);

  useEffect(() => {
    if (status === "ready") {
      inputRef.current?.focus();
    }
  }, [status]);

  const dismiss = useCallback(() => {
    if (variant === "widget") {
      void api.hideWidget();
    } else {
      void api.closePopup();
    }
  }, [variant]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        dismiss();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [dismiss]);

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
      } catch (caught) {
        setError(String(caught));
      } finally {
        busy.current = false;
      }
    },
    [answer, challenge],
  );

  const next = useCallback(() => {
    void load(challenge?.item.id);
  }, [challenge?.item.id, load]);

  const openApp = async () => {
    await api.showMainWindow();
    dismiss();
  };

  const direction = challenge?.direction;

  return (
    <div className="popup-shell">
      <article className={`popup-card ${variant}`}>
        <header className="popup-top" data-tauri-drag-region>
          <span className="drag" data-tauri-drag-region>
            <span className="popup-seal" data-tauri-drag-region>
              言
            </span>
            kotoba
          </span>
          <span className="popup-direction">
            {direction ? directionLabel(direction) : "micro-reto"}
          </span>
          <button
            type="button"
            className="popup-close"
            onClick={dismiss}
            aria-label={variant === "widget" ? "Ocultar widget" : "Cerrar"}
          >
            <IconClose size={14} />
          </button>
        </header>

        {status === "loading" && (
          <div className="popup-body">
            <div className="skeleton-line short" />
            <div className="skeleton-line big" />
            <div className="skeleton-line" />
          </div>
        )}

        {status === "empty" && (
          <div className="popup-body popup-empty">
            <span className="empty-mark small">空</span>
            <p>{error ?? "No hay palabras disponibles."}</p>
            <button type="button" className="ghost small" onClick={openApp}>
              Abrir kotoba
            </button>
          </div>
        )}

        {challenge && status !== "empty" && status !== "loading" && (
          <div className="popup-body">
            <p
              className={
                challenge.direction === "jp_to_es"
                  ? "popup-prompt jp"
                  : "popup-prompt es"
              }
              lang={challenge.direction === "jp_to_es" ? "ja" : "es"}
            >
              {challenge.direction === "jp_to_es"
                ? japaneseDisplay(challenge.item)
                : challenge.item.meaning}
            </p>

            {status !== "answered" ? (
              <form
                className="popup-form"
                onSubmit={(event) => {
                  event.preventDefault();
                  void submit();
                }}
              >
                <input
                  ref={inputRef}
                  className="popup-input"
                  value={answer}
                  onChange={(event) => setAnswer(event.currentTarget.value)}
                  onKeyDown={(event) => {
                    if (
                      event.key === "Enter" &&
                      !event.nativeEvent.isComposing
                    ) {
                      event.preventDefault();
                      void submit();
                    }
                  }}
                  placeholder={directionHint(challenge.direction)}
                  lang={challenge.direction === "es_to_jp" ? "ja" : "es"}
                  autoComplete="off"
                  spellCheck={false}
                />
                <div className="popup-actions">
                  <button
                    type="button"
                    className="ghost small"
                    onClick={dismiss}
                  >
                    {variant === "widget" ? "Ocultar" : "Ahora no"}
                  </button>
                  <button
                    type="submit"
                    className="primary small"
                    disabled={answer.trim().length === 0}
                  >
                    Comprobar
                    <span className="key-hint">↵</span>
                  </button>
                </div>
              </form>
            ) : (
              result && (
                <div
                  className={`popup-feedback ${result.correct ? "ok" : "ko"}`}
                >
                  <div className="pf-head">
                    <span className="feedback-mark">
                      {result.correct ? "正" : "×"}
                    </span>
                    <strong>{result.correct ? "Correcto" : "Casi"}</strong>
                  </div>
                  <p className="pf-jp" lang="ja">
                    {japaneseDisplay(challenge.item)}
                    <span className="pf-reading">
                      {challenge.item.reading}
                    </span>
                  </p>
                  <p className="pf-meaning">{result.meaning}</p>
                  {result.xp_gained > 0 && (
                    <p className="pf-xp">
                      +{result.xp_gained} XP
                      {result.leveled_up && ` · ¡Nivel ${result.level}!`}
                    </p>
                  )}
                  {result.unlocked.length > 0 && (
                    <p className="pf-achievement">
                      ✦ {result.unlocked.map((a) => a.title).join(" · ")}
                    </p>
                  )}
                  {result.saver_earned && (
                    <p className="pf-achievement">+1 salvavidas de racha</p>
                  )}
                  <div className="popup-actions">
                    <button
                      type="button"
                      className="ghost small"
                      onClick={() => void playAudio(challenge.item.id)}
                      disabled={!result.has_audio}
                      aria-label="Reproducir audio"
                    >
                      <IconSound size={14} />
                    </button>
                    <button
                      type="button"
                      className="ghost small"
                      onClick={openApp}
                    >
                      Abrir app
                    </button>
                    <button
                      type="button"
                      className="primary small"
                      onClick={next}
                    >
                      Siguiente
                      <IconArrow size={14} />
                    </button>
                  </div>
                </div>
              )
            )}
          </div>
        )}
      </article>
    </div>
  );
}
