import { access, readFile } from "node:fs/promises";
import http from "node:http";
import { performance } from "node:perf_hooks";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const distDir = path.join(rootDir, "dist");
const host = "127.0.0.1";
const port = parsePositiveInt(process.env.BENCH_BROWSER_PORT ?? "4174", "port");
const samplesTarget = parsePositiveInt(process.argv[2] ?? "5", "samples");
const dataMode = process.env.BENCH_DATA_MODE ?? "fixture";
const fixtureName = process.env.BENCH_FIXTURE ?? "btcusdt-1m";
const chromePath =
  process.env.CHROME_EXECUTABLE ??
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

await access(path.join(distDir, "index.html"));

const server = http.createServer(async (req, res) => {
  try {
    const requestPath = sanitizePath(req.url ?? "/");
    const filePath = path.join(distDir, requestPath);
    const body = await readFile(filePath);
    res.writeHead(200, { "content-type": contentType(filePath) });
    res.end(body);
  } catch {
    res.writeHead(404);
    res.end("not found");
  }
});

await new Promise((resolve, reject) => {
  server.once("error", reject);
  server.listen(port, host, resolve);
});

let browser;
let context;
try {
  browser = await chromium.launch({
    executablePath: chromePath,
    headless: true,
  });
  context = await browser.newContext({ viewport: { width: 1440, height: 960 } });

  if (dataMode === "cached") {
    const warmupPage = await context.newPage();
    await warmupPage.goto(targetUrl(), { waitUntil: "domcontentloaded" });
    await warmupPage.waitForFunction(() => {
      const text = document.querySelector("#status")?.textContent ?? "";
      return /\s(live|cached|fixture:.+)$/.test(text);
    });
    await warmupPage.close();
  }

  const samples = [];
  for (let i = 0; i < samplesTarget; i += 1) {
    const page = await context.newPage();
    const start = performance.now();
    await page.goto(targetUrl(), { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => {
      const text = document.querySelector("#status")?.textContent ?? "";
      return /\s(live|cached|fixture:.+)$/.test(text);
    });
    const totalMs = performance.now() - start;
    const nav = await page.evaluate(() => {
      const entry = performance.getEntriesByType("navigation")[0];
      if (!(entry instanceof PerformanceNavigationTiming)) return null;
      return {
        domContentLoadedMs: entry.domContentLoadedEventEnd,
        loadEventMs: entry.loadEventEnd,
      };
    });
    const breakdown = await page.evaluate(() => window.__rapidChartLoadBreakdown ?? null);
    const status = ((await page.textContent("#status")) ?? "").trim();
    samples.push({
      totalMs,
      domContentLoadedMs: nav?.domContentLoadedMs ?? null,
      loadEventMs: nav?.loadEventMs ?? null,
      breakdown,
      status,
    });
    await page.close();
  }

  console.log(JSON.stringify({
    url: targetUrl(),
    dataMode,
    fixtureName: dataMode === "fixture" ? fixtureName : null,
    samplesCollected: samples.length,
    samples,
    medianTotalMs: median(samples.map((sample) => sample.totalMs)),
    meanTotalMs: mean(samples.map((sample) => sample.totalMs)),
  }, null, 2));
} finally {
  await context?.close();
  await browser?.close();
  await new Promise((resolve, reject) => server.close((error) => (error ? reject(error) : resolve())));
}

function sanitizePath(urlPath) {
  const pathname = new URL(urlPath, `http://${host}:${port}`).pathname;
  if (pathname === "/") return "index.html";
  return pathname.replace(/^\/+/, "");
}

function contentType(filePath) {
  const ext = path.extname(filePath);
  switch (ext) {
    case ".html":
      return "text/html; charset=utf-8";
    case ".js":
      return "text/javascript; charset=utf-8";
    case ".css":
      return "text/css; charset=utf-8";
    case ".wasm":
      return "application/wasm";
    case ".json":
      return "application/json; charset=utf-8";
    default:
      return "application/octet-stream";
  }
}

function parsePositiveInt(value, label) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${label} must be a positive integer`);
  }
  return parsed;
}

function mean(values) {
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

function targetUrl() {
  const params = new URLSearchParams({ data: dataMode });
  if (dataMode === "fixture") params.set("fixture", fixtureName);
  return `http://${host}:${port}/?${params.toString()}`;
}
