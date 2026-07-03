import { access, readFile } from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { chromium } from "playwright-core";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const distDir = path.join(rootDir, "dist");
const host = "127.0.0.1";
const port = parsePositiveInt(process.env.BENCH_BROWSER_PORT ?? "4174", "port");
const preset = process.argv[2] ?? "Trend Stack";
const samplesTarget = parsePositiveInt(process.argv[3] ?? "5", "samples");
const intervalMs = parsePositiveInt(process.env.BENCH_BROWSER_INTERVAL_MS ?? "1000", "interval");
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
  await page.goto(`http://${host}:${port}/`, { waitUntil: "networkidle" });
  await page.selectOption("#preset-select", { label: preset });
  await page.click("#preset-apply");
  await page.waitForFunction(() => {
    const text = document.querySelector("#perf")?.textContent ?? "";
    return text.includes("tick");
  });

  const samples = [];
  let previous = "";
  const deadline = Date.now() + samplesTarget * intervalMs * 4;
  while (samples.length < samplesTarget && Date.now() < deadline) {
    await page.waitForTimeout(intervalMs);
    const text = ((await page.textContent("#perf")) ?? "").trim();
    if (!text || text === previous) continue;
    previous = text;
    samples.push({
      raw: text,
      parsed: parsePerfText(text),
    });
  }

  const output = {
    url: `http://${host}:${port}/`,
    preset,
    samplesCollected: samples.length,
    samples,
    final: samples.at(-1)?.parsed ?? null,
  };
  console.log(JSON.stringify(output, null, 2));
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

function parsePerfText(text) {
  const match = text.match(
    /^tick ([\d.]+)ms\/([\d.]+)ms engine ([\d.]+)ms\/([\d.]+)ms candle ([\d.]+)ms\/([\d.]+)ms volume ([\d.]+)ms\/([\d.]+)ms ind ([\d.]+)ms\/([\d.]+)ms$/,
  );
  if (!match) return null;
  return {
    tick: pair(match[1], match[2]),
    engine: pair(match[3], match[4]),
    candle: pair(match[5], match[6]),
    volume: pair(match[7], match[8]),
    ind: pair(match[9], match[10]),
  };
}

function pair(latest, avg) {
  return {
    latestMs: Number.parseFloat(latest),
    avgMs: Number.parseFloat(avg),
  };
}

function parsePositiveInt(value, label) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${label} must be a positive integer`);
  }
  return parsed;
}
