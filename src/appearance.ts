import type { Settings } from "./api";

let media: MediaQueryList | null = null;
let mediaHandler: (() => void) | null = null;

/**
 * Aplica tema y escala tipográfica. En `auto` sigue el tema de Windows en vivo.
 */
export function applyAppearance(
  theme: Settings["theme"],
  fontScale: number,
): void {
  const root = document.documentElement;
  root.style.setProperty("--ui-scale", String(fontScale / 100));

  const resolve = () => {
    const dark =
      theme === "dark" ||
      (theme === "auto" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    root.dataset.theme = dark ? "dark" : "light";
    root.style.colorScheme = dark ? "dark" : "light";
  };

  resolve();

  if (media && mediaHandler) {
    media.removeEventListener("change", mediaHandler);
    media = null;
    mediaHandler = null;
  }
  if (theme === "auto") {
    media = window.matchMedia("(prefers-color-scheme: dark)");
    mediaHandler = resolve;
    media.addEventListener("change", mediaHandler);
  }
}
