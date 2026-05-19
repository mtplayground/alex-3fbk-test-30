import { defineConfig, devices } from '@playwright/test';

const liveMode = process.env.ZEROCLAW_E2E_LIVE === '1';
const webPort = Number(process.env.E2E_WEB_PORT ?? 8080);
const baseURL = process.env.E2E_WEB_BASE_URL ?? `http://127.0.0.1:${webPort}`;

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['html', { open: 'never' }], ['list']] : 'list',
  timeout: 90_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL,
    trace: 'retain-on-failure',
  },
  webServer: liveMode
    ? {
        command: '../scripts/e2e.sh',
        url: baseURL,
        reuseExistingServer: !process.env.CI,
        timeout: 180_000,
      }
    : {
        command: `npm run dev -- --host 127.0.0.1 --port ${webPort}`,
        url: baseURL,
        reuseExistingServer: !process.env.CI,
        timeout: 90_000,
      },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
