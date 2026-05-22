/**
 * Isomorphic parity gate: same config → browser SVG === server SVG (modes 1–3).
 *
 * Mode 0 is excluded because per-piece path ordering may differ between the
 * browser (JS insertion order) and server (Wasm deterministic iteration) without
 * affecting cut quality.
 */
import { test, expect, Page } from '@playwright/test';

const API_URL = process.env.API_URL ?? 'http://localhost:8080';
const APP_URL = process.env.APP_URL ?? 'http://localhost:3000';

const SEED42_CONFIG = {
  seed: 42,
  ncols: 10,
  nrows: 10,
  minPiece: 4,
  maxPiece: 50,
  tileRadius: 6,
  frame: 6,
  frameCorner: 4,
  arcShape: 0,
  borderSvg: null,
};

// Map from export mode to button label in ExportButtons.tsx
const MODE_LABELS: Record<number, string> = {
  1: 'Non-overlapping',
  2: 'Single path (Trotec)',
  3: 'Colored SVG',
};

async function fetchText(url: string): Promise<string> {
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`HTTP ${resp.status} from ${url}`);
  return resp.text();
}

async function postJson(url: string, body: unknown): Promise<unknown> {
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} from ${url}`);
  return resp.json();
}

async function generateInBrowser(page: Page, mode: number): Promise<string> {
  await page.goto(`${APP_URL}/configurator`);

  // Wait for Wasm to load
  await page.waitForSelector('button:has-text("Generate Jigsaw"):not([disabled])', { timeout: 15000 });

  // Set seed=42 via number input (Seed field is first)
  const seedInput = page.locator('input[type="number"]').first();
  await seedInput.fill('42');
  await seedInput.press('Tab');

  await page.click('button:has-text("Generate Jigsaw")');
  await page.waitForSelector('text=/\\d+ pieces/', { timeout: 15000 });

  // Trigger the export download
  const [download] = await Promise.all([
    page.waitForEvent('download'),
    page.click(`button:has-text("${MODE_LABELS[mode]}")`),
  ]);

  const path = await download.path();
  const fs = require('fs');
  return fs.readFileSync(path, 'utf-8');
}

test.describe('Isomorphic parity (browser Wasm === server Wasm)', () => {
  let configId: string;

  test.beforeAll(async () => {
    const resp = await postJson(`${API_URL}/api/puzzle/generate`, SEED42_CONFIG) as { id: string };
    configId = resp.id;
    expect(configId).toBeTruthy();
  });

  for (const mode of [1, 2, 3]) {
    test(`mode ${mode}: "${MODE_LABELS[mode]}" SVG matches server output`, async ({ page }) => {
      const browserSvg = await generateInBrowser(page, mode);
      const serverSvg = await fetchText(`${API_URL}/api/puzzle/${configId}/export?mode=${mode}`);

      if (browserSvg !== serverSvg) {
        // Write both to disk for CI artifact diffing
        const fs = require('fs');
        fs.writeFileSync(`/tmp/browser-mode${mode}.svg`, browserSvg);
        fs.writeFileSync(`/tmp/server-mode${mode}.svg`, serverSvg);
      }
      expect(browserSvg).toBe(serverSvg);
    });
  }
});
