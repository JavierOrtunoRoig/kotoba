import { api } from "./api";

/** Reproduce el audio de un ítem sin bloquear la UI ni romper si falta. */
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
    // El audio es opcional: si falla, la tarjeta sigue funcionando.
  }
}
