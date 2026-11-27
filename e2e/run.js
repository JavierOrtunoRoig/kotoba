import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';

if (process.platform !== 'win32') throw new Error('Desktop E2E requires Windows');
const profile = mkdtempSync(join(tmpdir(), 'kotoba-e2e-'));
const target = resolve('src-tauri/target/e2e');
let child;
let interrupted = false;
function stop() {
  interrupted = true;
  if (child?.pid) spawnSync('taskkill', ['/pid', String(child.pid), '/t', '/f'], { stdio: 'ignore' });
}
process.on('SIGINT', stop);
process.on('SIGTERM', stop);
function run(args, env) {
  if (interrupted) throw new Error('Desktop E2E interrupted');
  return new Promise((resolveRun, reject) => {
    child = spawn('npm.cmd', args, { stdio: 'inherit', env, shell: true });
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      child = undefined;
      if (code === 0 && !interrupted) resolveRun();
      else reject(new Error(`npm failed (exit ${code}, signal ${signal ?? 'none'})`));
    });
  });
}
try {
  const port = await new Promise((resolvePort, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const selected = server.address().port;
      server.close((error) => error ? reject(error) : resolvePort(selected));
    });
  });
  const env = { ...process.env, CARGO_TARGET_DIR: target, KOTOBA_E2E_TARGET: target };
  await run(['run', 'tauri', '--', 'build', '--debug', '--no-bundle', '--features', 'tauri-plugin-wdio-webdriver,tauri-plugin-wdio', '--config', 'src-tauri/tauri.e2e.conf.json'], { ...env, VITE_KOTOBA_E2E: '1' });
  Object.assign(env, { APPDATA: profile, LOCALAPPDATA: profile, WEBVIEW2_USER_DATA_FOLDER: join(profile, 'webview'), KOTOBA_E2E_PROFILE: profile, KOTOBA_E2E_PORT: String(port) });
  for (const phase of ['import', 'history']) {
    console.log(`\nDesktop E2E phase: ${phase}`);
    await run(['exec', '--', 'wdio', 'run', 'wdio.conf.js'], { ...env, KOTOBA_E2E_PHASE: phase });
  }
} catch (error) {
  const logs = join(profile, 'logs');
  if (existsSync(logs)) {
    for (const file of readdirSync(logs, { withFileTypes: true })) {
      if (file.isFile()) console.error(`\n--- ${file.name} ---\n${readFileSync(join(logs, file.name), 'utf8').slice(-16384)}`);
    }
  }
  throw error;
} finally {
  if (child?.pid) stop();
  rmSync(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 500 });
}
