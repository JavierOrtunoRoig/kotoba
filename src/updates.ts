import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

/** Consulta el `latest.json` publicado en GitHub Releases. */
export async function findUpdate(): Promise<Update | null> {
  return await check({ timeout: 15000 });
}

/** Descarga, instala y reinicia la app con la versión nueva. */
export async function installUpdate(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
