import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

import initWasm, { ChartEngine } from "../pkg/rapidchart_engine.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const wasmPath = path.join(rootDir, "pkg", "rapidchart_engine_bg.wasm");
const outDir = path.join(rootDir, "benchmarks", "results");

const sizes = parseSizes(process.argv[2] ?? "1000,10000,50000");
const runs = parsePositiveInt(process.argv[3] ?? "7", "runs");
const upserts = parsePositiveInt(process.argv[4] ?? "250", "upserts");

const wasmOutput = await initWasm({
  module_or_path: await readFile(wasmPath),
});
const wasmMemory = wasmOutput.memory;

await mkdir(outDir, { recursive: true });

const indicators = [
  { kind: "SMA", period: 20 },
  { kind: "EMA", period: 50 },
  { kind: "RSI", period: 14 },
  { kind: "MACD", fast: 12, slow: 26, signal: 9 },
  { kind: "BB", period: 20, multiplier: 2 },
  { kind: "ATR", period: 14 },
];

const results = {
  createdAt: new Date().toISOString(),
  sizes,
  runs,
  upserts,
  indicators,
  scenarios: [],
};

for (const size of sizes) {
  const bars = makeBars(size);
  const columns = toColumns(bars);
  const { engine: outputEngine, indicatorIds } = buildEngine(columns, indicators);
  const outputIndicatorId = indicatorIds[3];

  results.scenarios.push(
    benchmarkScenario(`ingestBars:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_bars(bars);
    }),
    benchmarkScenario(`ingestColumns:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns(columns);
    }),
    benchmarkScenario(`ingestColumnsFast:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
    }),
    benchmarkScenario(`ingestColumnsZeroCopy:${size}`, runs, () => {
      const engine = new ChartEngine();
      const ptrs = engine.alloc_candle_buffer(size);
      new Uint32Array(wasmMemory.buffer, ptrs.time_ptr, size).set(columns.time);
      new Float64Array(wasmMemory.buffer, ptrs.open_ptr, size).set(columns.open);
      new Float64Array(wasmMemory.buffer, ptrs.high_ptr, size).set(columns.high);
      new Float64Array(wasmMemory.buffer, ptrs.low_ptr, size).set(columns.low);
      new Float64Array(wasmMemory.buffer, ptrs.close_ptr, size).set(columns.close);
      new Float64Array(wasmMemory.buffer, ptrs.volume_ptr, size).set(columns.volume);
      engine.finalize_candle_buffer();
    }),
    benchmarkScenario(`ingestColumnsZeroCopy+indicators:${size}`, runs, () => {
      const engine = new ChartEngine();
      const ptrs = engine.alloc_candle_buffer(size);
      new Uint32Array(wasmMemory.buffer, ptrs.time_ptr, size).set(columns.time);
      new Float64Array(wasmMemory.buffer, ptrs.open_ptr, size).set(columns.open);
      new Float64Array(wasmMemory.buffer, ptrs.high_ptr, size).set(columns.high);
      new Float64Array(wasmMemory.buffer, ptrs.low_ptr, size).set(columns.low);
      new Float64Array(wasmMemory.buffer, ptrs.close_ptr, size).set(columns.close);
      new Float64Array(wasmMemory.buffer, ptrs.volume_ptr, size).set(columns.volume);
      engine.finalize_candle_buffer();
      addIndicators(engine, indicators);
    }),
    benchmarkScenario(`ingestBars+indicators:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_bars(bars);
      addIndicators(engine, indicators);
    }),
    benchmarkScenario(`ingestColumns+indicators:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns(columns);
      addIndicators(engine, indicators);
    }),
    benchmarkScenario(`ingestColumnsFast+indicators:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      addIndicators(engine, indicators);
    }),
    benchmarkScenarioWithSetup(`upsertBar:${size}+${upserts}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      addIndicators(engine, indicators);
      return engine;
    }, (engine) => {
      let time = bars[bars.length - 1].time;
      let close = bars[bars.length - 1].close;
      for (let i = 0; i < upserts; i += 1) {
        time += 60;
        close += Math.sin(i / 7) * 8 + 0.5;
        const low = close - 6 - (i % 3);
        const high = close + 6 + (i % 5);
        engine.upsert_bar_fast(
          time,
          close - 1.5,
          high,
          low,
          close,
          100 + (i % 20),
        );
      }
    }),
    benchmarkScenarioWithSetup(`appendLiveBar:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      addIndicators(engine, indicators);
      return engine;
    }, (engine) => {
      const bar = nextBar(bars[bars.length - 1], 1);
      engine.upsert_bar_fast(bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume);
    }),
    benchmarkScenarioWithSetup(`replaceLastBar:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      addIndicators(engine, indicators);
      return engine;
    }, (engine) => {
      const bar = replaceBar(bars[bars.length - 1]);
      engine.upsert_bar_fast(bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume);
    }),
    benchmarkScenarioWithSetup(`appendLiveBar+latestOutputs:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      const ids = addIndicators(engine, indicators);
      return { engine, ids };
    }, ({ engine, ids }) => {
      const bar = nextBar(bars[bars.length - 1], 1);
      engine.upsert_bar_fast(bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume);
      for (const id of ids) {
        engine.latest_indicator_values_fast(id);
      }
    }),
    benchmarkScenarioWithSetup(`replaceLastBar+latestOutputs:${size}`, runs, () => {
      const engine = new ChartEngine();
      engine.ingest_columns_fast(
        columns.time,
        columns.open,
        columns.high,
        columns.low,
        columns.close,
        columns.volume,
      );
      const ids = addIndicators(engine, indicators);
      return { engine, ids };
    }, ({ engine, ids }) => {
      const bar = replaceBar(bars[bars.length - 1]);
      engine.upsert_bar_fast(bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume);
      for (const id of ids) {
        engine.latest_indicator_values_fast(id);
      }
    }),
    benchmarkScenario(`candles:${size}`, runs, () => {
      outputEngine.candles();
    }),
    benchmarkScenario(`candleColumnsFast:${size}`, runs, () => {
      outputEngine.candle_columns_fast();
    }),
    benchmarkScenario(`indicatorOutputsAll:${size}`, runs, () => {
      outputEngine.indicator_outputs_all(outputIndicatorId);
    }),
    benchmarkScenario(`indicatorOutputValuesFast:${size}`, runs, () => {
      outputEngine.indicator_output_values_fast(outputIndicatorId);
    }),
  );
}

const filename = `bench-${timestamp()}.json`;
const outputPath = path.join(outDir, filename);
await writeFile(outputPath, `${JSON.stringify(results, null, 2)}\n`);

printSummary(results, outputPath);

function benchmarkScenario(name, scenarioRuns, fn) {
  const samples = [];
  for (let i = 0; i < scenarioRuns; i += 1) {
    const start = performance.now();
    fn();
    samples.push(performance.now() - start);
  }
  samples.sort((a, b) => a - b);
  const median = samples[Math.floor(samples.length / 2)];
  const mean = samples.reduce((sum, value) => sum + value, 0) / samples.length;
  return {
    name,
    samplesMs: samples.map(round3),
    medianMs: round3(median),
    meanMs: round3(mean),
    minMs: round3(samples[0]),
    maxMs: round3(samples[samples.length - 1]),
  };
}

function benchmarkScenarioWithSetup(name, scenarioRuns, setup, fn) {
  const samples = [];
  for (let i = 0; i < scenarioRuns; i += 1) {
    const state = setup();
    const start = performance.now();
    fn(state);
    samples.push(performance.now() - start);
  }
  samples.sort((a, b) => a - b);
  const median = samples[Math.floor(samples.length / 2)];
  const mean = samples.reduce((sum, value) => sum + value, 0) / samples.length;
  return {
    name,
    samplesMs: samples.map(round3),
    medianMs: round3(median),
    meanMs: round3(mean),
    minMs: round3(samples[0]),
    maxMs: round3(samples[samples.length - 1]),
  };
}

function addIndicators(engine, configs) {
  return configs.map((config) => engine.add_indicator_config(config));
}

function buildEngine(columns, configs) {
  const engine = new ChartEngine();
  engine.ingest_columns_fast(
    columns.time,
    columns.open,
    columns.high,
    columns.low,
    columns.close,
    columns.volume,
  );
  const indicatorIds = addIndicators(engine, configs);
  return { engine, indicatorIds };
}

function makeBars(size) {
  const bars = new Array(size);
  let close = 100;
  for (let i = 0; i < size; i += 1) {
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
    const bar = bars[i];
    time[i] = bar.time;
    open[i] = bar.open;
    high[i] = bar.high;
    low[i] = bar.low;
    close[i] = bar.close;
    volume[i] = bar.volume;
  }
  return { time, open, high, low, close, volume };
}

function nextBar(lastBar, step) {
  const close = lastBar.close + Math.sin(step / 7) * 8 + 0.5;
  return {
    time: lastBar.time + 60,
    open: close - 1.5,
    high: close + 6 + (step % 5),
    low: close - 6 - (step % 3),
    close,
    volume: 100 + (step % 20),
  };
}

function replaceBar(lastBar) {
  const close = lastBar.close + 1.25;
  return {
    time: lastBar.time,
    open: lastBar.open,
    high: Math.max(lastBar.high, close + 1.5),
    low: Math.min(lastBar.low, close - 1.5),
    close,
    volume: lastBar.volume + 12,
  };
}

function printSummary(results, outputPath) {
  console.log(`Wrote ${path.relative(rootDir, outputPath)}`);
  for (const scenario of results.scenarios) {
    console.log(
      `${scenario.name.padEnd(30)} median=${scenario.medianMs.toFixed(3)}ms mean=${scenario.meanMs.toFixed(3)}ms min=${scenario.minMs.toFixed(3)}ms max=${scenario.maxMs.toFixed(3)}ms`,
    );
  }
}

function parseSizes(value) {
  return value
    .split(",")
    .map((part) => parsePositiveInt(part.trim(), "size"))
    .filter((part, index, array) => array.indexOf(part) === index);
}

function parsePositiveInt(value, label) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`${label} must be a positive integer`);
  }
  return parsed;
}

function round3(value) {
  return Math.round(value * 1000) / 1000;
}

function timestamp() {
  return new Date().toISOString().replaceAll(":", "-");
}
