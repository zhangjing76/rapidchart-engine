/**
 * Indicator benchmark: measures compute + incremental update cost for every indicator category.
 *
 * Usage:
 *   npm run bench:indicators
 *   node scripts/bench-indicators.mjs [size] [runs]
 *
 * Defaults: size=5000, runs=5
 */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

import initWasm, { ChartEngine } from "../pkg/rapidchart_engine.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const wasmPath = path.join(rootDir, "pkg", "rapidchart_engine_bg.wasm");
const outDir = path.join(rootDir, "benchmarks", "results");

const size = parsePositiveInt(process.argv[2] ?? "5000", "size");
const runs = parsePositiveInt(process.argv[3] ?? "5", "runs");

const wasmOutput = await initWasm({
  module_or_path: await readFile(wasmPath),
});

await mkdir(outDir, { recursive: true });

// ─── Indicator groups by category ─────────────────────────────────────
const indicatorGroups = {
  "Moving Average": [
    { kind: "SMA", period: 20 },
    { kind: "EMA", period: 20 },
    { kind: "WMA", period: 20 },
    { kind: "HMA", period: 20 },
    { kind: "DEMA", period: 20 },
    { kind: "TEMA", period: 20 },
    { kind: "TRIMA", period: 20 },
    { kind: "VWMA", period: 20 },
  ],
  "Averages/Bands": [
    { kind: "ENVELOPE", period: 20, multiplier: 2 },
    { kind: "KELTNER", period: 20, multiplier: 2 },
    { kind: "DONCHIAN", period: 20 },
    { kind: "ALLIGATOR" },
    { kind: "ATR_BANDS", period: 14, multiplier: 2 },
    { kind: "HIGH_LOW_BANDS", period: 14 },
    { kind: "GMMA" },
    { kind: "RAINBOW_MA", period: 2 },
  ],
  "Momentum/Oscillator": [
    { kind: "RSI", period: 14 },
    { kind: "MACD", fast: 12, slow: 26, signal: 9 },
    { kind: "STOCHASTIC", period: 14, smooth: 3 },
    { kind: "CCI", period: 20 },
    { kind: "AWESOME_OSCILLATOR" },
    { kind: "CHANDE_MOMENTUM", period: 14 },
    { kind: "COPPOCK_CURVE" },
    { kind: "EHLER_FISHER", period: 10 },
    { kind: "ELDER_RAY", period: 13 },
    { kind: "SCHAFF_TREND_CYCLE", fast: 23, slow: 50, stoch_period: 10 },
    { kind: "STOCHASTIC_MOMENTUM", period: 13, smooth: 3 },
    { kind: "RELATIVE_VIGOR", period: 10 },
    { kind: "PRICE_MOMENTUM_OSCILLATOR", period: 35, smooth: 20 },
    { kind: "VOLUME_OSCILLATOR", fast: 5, slow: 10 },
  ],
  "Money Flow": [
    { kind: "MFI", period: 14 },
    { kind: "CMF", period: 20 },
    { kind: "FORCE_INDEX", period: 13 },
    { kind: "CHAIKIN_OSCILLATOR", fast: 3, slow: 10 },
    { kind: "KLINGER_VOLUME" },
    { kind: "TWIGGS_MONEY_FLOW", period: 21 },
    { kind: "PRICE_VOLUME_TREND" },
    { kind: "TRADE_VOLUME_INDEX" },
  ],
  "Trend Analysis": [
    { kind: "ADX", period: 14 },
    { kind: "AROON", period: 14 },
    { kind: "SUPERTREND", period: 10, multiplier: 3 },
    { kind: "PARABOLIC_SAR", psar_step: 0.02, psar_max_step: 0.2 },
    { kind: "CHOPPINESS_INDEX", period: 14 },
    { kind: "ELDER_IMPULSE", period: 13 },
    { kind: "GONOGO_TREND", period: 14 },
    { kind: "VORTEX_INDICATOR", period: 14 },
    { kind: "ZIGZAG", multiplier: 5 },
    { kind: "PSYCHOLOGICAL_LINE", period: 12 },
    { kind: "SHINOHARA_INTENSITY", period: 26 },
  ],
  "Volatility": [
    { kind: "ATR", period: 14 },
    { kind: "BB", period: 20, multiplier: 2 },
    { kind: "HISTORICAL_VOLATILITY", period: 20 },
    { kind: "BOLLINGER_BANDWIDTH", period: 20, multiplier: 2 },
    { kind: "DONCHIAN_WIDTH", period: 20 },
    { kind: "MASS_INDEX", period: 25 },
    { kind: "RELATIVE_VOLATILITY", period: 14 },
    { kind: "TRUE_RANGE" },
    { kind: "GOPALAKRISHNAN_RANGE", period: 14 },
    { kind: "ULCER_INDEX", period: 14 },
  ],
  "Statistical": [
    { kind: "STDDEV", period: 20 },
    { kind: "LINEAR_REGRESSION", period: 20 },
    { kind: "LINEAR_REG_FORECAST", period: 14 },
    { kind: "LINEAR_REG_INTERCEPT", period: 14 },
    { kind: "LINEAR_REG_SLOPE", period: 14 },
    { kind: "LINEAR_REG_R2", period: 14 },
    { kind: "CORRELATION_COEFFICIENT", period: 14 },
    { kind: "RANDOM_WALK_INDEX", period: 14 },
    { kind: "MEDIAN_PRICE" },
    { kind: "TYPICAL_PRICE" },
    { kind: "WEIGHTED_CLOSE" },
  ],
  "Compare": [
    { kind: "BETA", period: 20 },
    { kind: "PERFORMANCE_INDEX" },
    { kind: "PRICE_RELATIVE", period: 14 },
  ],
  "Support/Resistance": [
    { kind: "PIVOT_POINTS" },
    { kind: "DARVAS_BOX" },
    { kind: "VOLUME_PROFILE", period: 24 },
    { kind: "PRIME_NUMBER_BANDS" },
  ],
  "Volume": [
    { kind: "OBV" },
    { kind: "VWAP" },
    { kind: "ANCHORED_VWAP", anchor: 0 },
    { kind: "VOLUME_CHART" },
    { kind: "VOLUME_ROC", period: 14 },
    { kind: "VOLUME_UNDERLAY" },
    { kind: "PROJECTED_AGGREGATE_VOLUME", period: 24 },
    { kind: "PROJECTED_VOLUME_AT_TIME", period: 24 },
    { kind: "MARKET_FACILITATION" },
  ],
  "Projection": [
    { kind: "ICHIMOKU", tenkan_period: 9, kijun_period: 26, senkou_b_period: 52 },
    { kind: "TIME_SERIES_FORECAST", period: 14 },
  ],
};

// ─── Generate test data ───────────────────────────────────────────────
function makeBars(count) {
  const bars = new Array(count);
  let close = 100;
  for (let i = 0; i < count; i += 1) {
    close += Math.sin(i / 17) * 1.2 + Math.cos(i / 31) * 0.8 + 0.05;
    const high = close + 2 + (i % 7) * 0.15;
    const low = close - 2 - (i % 5) * 0.12;
    bars[i] = {
      time: 1_700_000_000 + i * 60,
      open: close - 0.4,
      high,
      low,
      close,
      volume: 50 + (i % 100) * 1.5,
    };
  }
  return bars;
}

function toColumns(bars) {
  const time = new Uint32Array(bars.length);
  const open = new Float64Array(bars.length);
  const high = new Float64Array(bars.length);
  const low = new Float64Array(bars.length);
  const close = new Float64Array(bars.length);
  const volume = new Float64Array(bars.length);
  for (let i = 0; i < bars.length; i += 1) {
    time[i] = bars[i].time;
    open[i] = bars[i].open;
    high[i] = bars[i].high;
    low[i] = bars[i].low;
    close[i] = bars[i].close;
    volume[i] = bars[i].volume;
  }
  return { time, open, high, low, close, volume };
}

// ─── Benchmark helpers ────────────────────────────────────────────────
function benchmarkGroup(name, groupIndicators, columns, bars) {
  // Full compute: ingest + add all indicators
  const computeSamples = [];
  for (let r = 0; r < runs; r += 1) {
    const engine = new ChartEngine();
    engine.ingest_columns_fast(
      columns.time, columns.open, columns.high, columns.low, columns.close, columns.volume,
    );
    const start = performance.now();
    for (const config of groupIndicators) {
      engine.add_indicator_config(config);
    }
    computeSamples.push(performance.now() - start);
  }

  // Incremental: upsert one bar with all indicators active
  const incrementalSamples = [];
  for (let r = 0; r < runs; r += 1) {
    const engine = new ChartEngine();
    engine.ingest_columns_fast(
      columns.time, columns.open, columns.high, columns.low, columns.close, columns.volume,
    );
    const ids = groupIndicators.map((config) => engine.add_indicator_config(config));
    const lastBar = bars[bars.length - 1];
    const newClose = lastBar.close + 2.5;
    const start = performance.now();
    engine.upsert_bar_fast(
      lastBar.time + 60,
      newClose - 1,
      newClose + 5,
      newClose - 4,
      newClose,
      120,
    );
    for (const id of ids) {
      engine.latest_indicator_values_fast(id);
    }
    incrementalSamples.push(performance.now() - start);
  }

  computeSamples.sort((a, b) => a - b);
  incrementalSamples.sort((a, b) => a - b);

  return {
    category: name,
    indicatorCount: groupIndicators.length,
    compute: {
      medianMs: round3(computeSamples[Math.floor(computeSamples.length / 2)]),
      meanMs: round3(computeSamples.reduce((s, v) => s + v, 0) / computeSamples.length),
    },
    incremental: {
      medianMs: round3(incrementalSamples[Math.floor(incrementalSamples.length / 2)]),
      meanMs: round3(incrementalSamples.reduce((s, v) => s + v, 0) / incrementalSamples.length),
    },
  };
}

// ─── Run benchmarks ───────────────────────────────────────────────────
const bars = makeBars(size);
const columns = toColumns(bars);

console.log(`Benchmarking ${Object.keys(indicatorGroups).length} categories, ${size} bars, ${runs} runs\n`);

const results = {
  createdAt: new Date().toISOString(),
  size,
  runs,
  categories: [],
};

for (const [name, groupIndicators] of Object.entries(indicatorGroups)) {
  const result = benchmarkGroup(name, groupIndicators, columns, bars);
  results.categories.push(result);
  console.log(
    `${name.padEnd(22)} (${String(result.indicatorCount).padStart(2)} indicators)  ` +
    `compute: ${result.compute.medianMs.toFixed(3)}ms  ` +
    `incremental: ${result.incremental.medianMs.toFixed(3)}ms`,
  );
}

// Total: all indicators at once
const allIndicators = Object.values(indicatorGroups).flat();
const totalResult = benchmarkGroup("ALL INDICATORS", allIndicators, columns, bars);
results.total = totalResult;
console.log(
  `\n${"ALL INDICATORS".padEnd(22)} (${String(totalResult.indicatorCount).padStart(2)} indicators)  ` +
  `compute: ${totalResult.compute.medianMs.toFixed(3)}ms  ` +
  `incremental: ${totalResult.incremental.medianMs.toFixed(3)}ms`,
);

const filename = `bench-indicators-${new Date().toISOString().replaceAll(":", "-")}.json`;
const outputPath = path.join(outDir, filename);
await writeFile(outputPath, `${JSON.stringify(results, null, 2)}\n`);
console.log(`\nWrote ${path.relative(rootDir, outputPath)}`);

function parsePositiveInt(value, label) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) throw new Error(`${label} must be positive`);
  return parsed;
}

function round3(value) {
  return Math.round(value * 1000) / 1000;
}
