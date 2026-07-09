import { access, readFile } from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const distDir = path.join(rootDir, "dist");
const host = "127.0.0.1";
const port = parsePositiveInt(process.env.CHECK_BROWSER_PORT ?? "4175", "port");
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
try {
  browser = await chromium.launch({
    executablePath: chromePath,
    headless: true,
  });
  const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
  await page.goto(targetUrl(), { waitUntil: "domcontentloaded" });
  await page.selectOption("#preset-select", { label: "Formula Demo" });
  await page.click("#preset-apply");
  await page.waitForFunction(
    () => window.__rapidChartPresetBreakdown?.preset === "Formula Demo",
  );
  await page.waitForFunction(
    () => document.querySelector("#indicators")?.textContent?.includes("EMA Trend Strength") ?? false,
  );

  const beforePerf = await page.textContent("#perf");
  const ticked = await page.evaluate(() => window.__rapidChartBenchmarkTick?.() ?? false);
  if (!ticked) throw new Error("benchmarkTick() did not run");
  await page.waitForFunction(
    (previous) => {
      const text = document.querySelector("#perf")?.textContent ?? "";
      return text.length > 0 && text !== previous;
    },
    beforePerf ?? "",
  );

  const result = await page.evaluate(() => ({
    preset: window.__rapidChartPresetBreakdown?.preset ?? null,
    perf: document.querySelector("#perf")?.textContent ?? null,
    indicators: document.querySelector("#indicators")?.textContent ?? null,
  }));

  if (result.preset !== "Formula Demo") {
    throw new Error(`unexpected preset result: ${result.preset}`);
  }
  if (!result.indicators?.includes("EMA Trend Strength")) {
    throw new Error("formula indicator was not rendered");
  }
  if (!result.perf?.includes("engine")) {
    throw new Error("perf panel was not updated after live tick");
  }

  console.log(JSON.stringify(result, null, 2));
} finally {
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

function targetUrl() {
  const params = new URLSearchParams({ data: "fixture", fixture: "btcusdt-1m" });
  return `http://${host}:${port}/?${params.toString()}`;
}
