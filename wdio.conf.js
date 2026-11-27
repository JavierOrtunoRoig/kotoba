import { resolve } from 'node:path';
if (!process.env.KOTOBA_E2E_PROFILE || !['import', 'history'].includes(process.env.KOTOBA_E2E_PHASE)) {
  throw new Error('Run npm run test:e2e:windows to create an isolated test profile');
}
export const config = {
  runner: 'local',
  specs: ['./e2e/review.spec.js'],
  maxInstances: 1,
  outputDir: resolve(process.env.KOTOBA_E2E_PROFILE, 'logs'),
  capabilities: [{ browserName: 'tauri' }],
  services: [['@wdio/tauri-service', {
    appBinaryPath: resolve(process.env.KOTOBA_E2E_TARGET, 'debug/kotoba.exe'),
    driverProvider: 'embedded',
    embeddedPort: Number(process.env.KOTOBA_E2E_PORT),
    captureBackendLogs: true,
  }]],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { timeout: 120000 },
  logLevel: 'info',
};
