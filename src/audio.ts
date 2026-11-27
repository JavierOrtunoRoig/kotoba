import { api } from "./api";

/** Plays an item's audio without blocking the UI or breaking if it is missing. */
export async function playAudio(itemId: number): Promise<void> {
  try {
    const bytes = await api.getAudio(itemId);
    if (!bytes || bytes.length === 0) return;

    const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);

    const cleanup = () => URL.revokeObjectURL(url);
    audio.addEventListener("ended", cleanup, { once: true });
    audio.addEventListener("error", cleanup, { once: true });

    await audio.play();
  } catch {
    // Audio is optional: if it fails, the card keeps working.
  }
}
