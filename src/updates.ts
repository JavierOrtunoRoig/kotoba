import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

/** Checks the `latest.json` published on GitHub Releases. */
export async function findUpdate(): Promise<Update | null> {
  return await check({ timeout: 15000 });
}

/** Downloads, installs and restarts the app with the new version. */
export async function installUpdate(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
