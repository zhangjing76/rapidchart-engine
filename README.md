# rapidchart-engine

Rust/WASM chart engine with a Vite test app using TradingView Lightweight Charts and Binance market data.

## Library usage

`rapidchart-engine` exposes a typed TypeScript wrapper from [src/index.ts](src/index.ts). The Rust engine computes candles and indicators; your app renders the returned series.

### Build the WASM package

```bash
npm install
npm run build:wasm
```

This generates the browser-facing package in `pkg/`.

### Import and initialize

```ts
import { RapidChartEngine, initEngine, type Bar } from "rapidchart-engine";

await initEngine();

const engine = new RapidChartEngine();
```

### Load candles

Bars use Unix seconds for `time`.

```ts
const bars = [
  {
    time: 1719830400,
    open: 62000,
    high: 62500,
    low: 61800,
    close: 62300,
    volume: 1200,
  },
  {
    time: 1719834000,
    open: 62300,
    high: 62600,
    low: 62100,
    close: 62450,
    volume: 980,
  },
];

engine.ingestColumnsFast({
  time: new Uint32Array(bars.map((bar) => bar.time)),
  open: new Float64Array(bars.map((bar) => bar.open)),
  high: new Float64Array(bars.map((bar) => bar.high)),
  low: new Float64Array(bars.map((bar) => bar.low)),
  close: new Float64Array(bars.map((bar) => bar.close)),
  volume: new Float64Array(bars.map((bar) => bar.volume)),
});
```

### Add indicators

```ts
const smaId = engine.addIndicator({
  kind: "SMA",
  period: 20,
});

const rsiId = engine.addIndicator({
  kind: "RSI",
  period: 14,
});

const macdId = engine.addIndicator({
  kind: "MACD",
  fast: 12,
  slow: 26,
  signal: 9,
});
```

### Read computed series

```ts
const candles = engine.candles();
const candleColumns = engine.candleColumns();
const smaValues = engine.indicatorValueSeries(smaId);
const rsiValues = engine.indicatorValueSeries(rsiId);
const macdValues = engine.indicatorValueSeries(macdId);
```

`indicatorValueSeries()` returns visible outputs only. Multi-output indicators
such as `MACD`, `BB`, `KELTNER`, `DONCHIAN`, `ADX`, and `STOCHASTIC` return one
entry per output.

Rust owns the OHLCV history. Use `candles()` for convenient bar objects or
`candleColumns()` for typed arrays; there is no separate `timeline()` API.

For zero-copy groundwork, the wrapper also exposes columnar read APIs:

- `candleColumns()` returns typed arrays for `time`, `open`, `high`, `low`, `close`, and `volume`
- `indicatorValueSeries(id)` returns typed arrays of raw visible output values, using `NaN` for gaps

Those accessors return zero-copy views into WASM memory. Do not retain them
across engine mutations or WASM memory growth; read them again after calls such
as `upsertBarFast()`.

## Internal indicator contract

Indicator store functions now use a consistent internal shape:

- single-output `*_store(...)` functions return `RcSeries`
- multi-output `*_store(...)` functions return `Vec<NamedSeries>`
- `dispatch.rs` converts both shapes into `Vec<IndicatorOutput>` via `.into_outputs()`

This keeps the cache/DAG layer shared and zero-copy-friendly while preserving the existing public output API.

### Zero-copy rule

Inside indicator code, series data should stay in shared `Rc<Vec<f64>>` form for as long as possible.

- use `Rc::clone(...)` on cache hits
- use `named_series(name, values)` for multi-output internal results
- do not build `IndicatorOutput` directly inside store functions
- only convert to owned `Vec<f64>` at the final output boundary

At that boundary, `rc_into_owned(...)` moves the underlying `Vec<f64>` when the `Rc` is uniquely owned, and only clones when the series is still shared elsewhere.

### Render with Lightweight Charts

```ts
import {
  CandlestickSeries,
  HistogramSeries,
  LineSeries,
  createChart,
  type Time,
} from "lightweight-charts";

const chart = createChart(document.getElementById("chart")!);
const candleSeries = chart.addSeries(CandlestickSeries);
const volumeSeries = chart.addSeries(HistogramSeries, {
  priceFormat: { type: "volume" },
  priceScaleId: "",
});
const smaLine = chart.addSeries(LineSeries);

const bars = engine.candles();
candleSeries.setData(bars.map((bar) => ({ ...bar, time: bar.time as Time })));
volumeSeries.setData(
  bars.map((bar) => ({
    time: bar.time as Time,
    value: bar.volume,
    color: bar.close >= bar.open ? "#26a69a" : "#ef5350",
  })),
);

const smaOutput = engine.indicatorValueSeries(smaId)[0];
if (smaOutput) {
  const points = [];
  for (let index = 0; index < candleColumns.time.length; index += 1) {
    const value = smaOutput.values[index];
    if (value !== undefined && !Number.isNaN(value)) {
      points.push({ time: candleColumns.time[index] as Time, value });
    }
  }
  smaLine.setData(points);
}
```

For live data, expose one app-level function that updates the engine first and
then fans the result out to the independent Lightweight Charts series:

```ts
function updateChart(bar: Bar) {
  engine.upsertBarFast(bar);

  candleSeries.update({ ...bar, time: bar.time as Time });
  volumeSeries.update({
    time: bar.time as Time,
    value: bar.volume,
    color: bar.close >= bar.open ? "#26a69a" : "#ef5350",
  });

  for (const point of engine.latestIndicatorPoints(smaId)) {
    if (point.value !== null) {
      smaLine.update({ time: point.time as Time, value: point.value });
    }
  }
}
```

Candles, volume, and indicators are separate Lightweight Charts series because
they use different data shapes and renderers. `updateChart()` is the unified
application boundary; the individual `.update()` calls are the renderer
boundary.

### Wrapper API

- `initEngine(moduleOrPath?)`
- `new RapidChartEngine()`
- `ingestColumnsFast(columns)`
- `ingestColumnsZeroCopy(columns)`
- `candles()`
- `candleColumns()`
- `addIndicator(config)`
- `addIndicators(configs)`
- `addFormulaIndicator(config)`
- `addFormulaIndicators(configs)`
- `indicatorValueSeries(id)`
- `latestIndicatorValues(id)`
- `latestIndicatorTime(id, output)`
- `latestIndicatorPoints(id)`
- `removeIndicator(id)`
- `indicatorDescriptors()`
- `dagDebug()`
- `upsertBarFast(bar)`

The local implementation lives in [src/engine.ts](src/engine.ts), but consumers should import from the package root instead of reaching into `src/`.

### Supported indicator configs

Every built-in config requires `kind`; the remaining fields are typed by
`IndicatorConfig`. Read `engine.indicatorDescriptors()` to build a picker or
parameter form from the current defaults and constraints instead of maintaining
a second indicator catalog in application code.

## What it does

- Loads Binance crypto candles by symbol and timeframe
- Streams live kline updates from Binance WebSocket
- Renders candles, volume, and indicator series with Lightweight Charts
- Uses a Rust engine compiled to WebAssembly for indicator computation
- Shows a DAG debug view so you can inspect source, computed, and indicator nodes
- Saves named test-app layouts in local storage and applies built-in indicator presets

## Performance workflow

The repo now has three benchmark layers:

1. Engine bulk benchmarks
2. Engine comparison reports
3. Real browser live-update sampling

### Engine benchmarks

Run the Rust/WASM benchmark harness:

```bash
npm run bench
```

It writes JSON snapshots to `benchmarks/results/`.

Compare two or more snapshots:

```bash
npm run bench:compare benchmarks/results/bench-a.json benchmarks/results/bench-b.json
```

### Browser benchmark

Run a repeatable browser sample against the built app:

```bash
npm run bench:browser
```

Defaults:

- preset: `Trend Stack`
- samples: `5`
- local Chrome executable: `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`

Override the preset by passing a label:

```bash
npm run bench:browser -- "Momentum Stack"
```

The script:

- builds the app
- serves `dist/` locally
- opens the app in headless Chrome through `playwright-core`
- applies the preset
- samples the live footer perf text
- prints parsed JSON

If Chrome lives elsewhere, set `CHROME_EXECUTABLE`.

### Footer metrics

The test app footer shows `latest/avg` timings for the live kline path:

- `tick`: total time spent handling one websocket update
- `engine`: Rust/WASM `upsert` and latest-value reads
- `candle`: candlestick series update cost
- `volume`: volume histogram update cost
- `ind`: indicator series update cost

For the current implementation, `ind` is usually the largest browser-side slice once engine transport overhead is reduced.

## Engine design

The engine follows one hard boundary:

- JavaScript owns transport and rendering
- Rust/WASM owns market state and indicator computation

That split is deliberate. Networking, UI events, and chart drawing stay in JavaScript. Candle storage, indicator state, incremental updates, and dependency tracking stay inside the Rust engine.

### Current architecture

Today the engine is centered around one `ChartEngine` instance per symbol and timeframe.

At a high level:

```text
Binance REST/WebSocket
        |
        v
JavaScript app
  - fetch history
  - subscribe to live klines
  - render Lightweight Charts
        |
        v
ChartEngine (Rust/WASM)
  - store bars
  - compute indicators
  - update indicators incrementally
  - expose visible output series
  - expose DAG debug data
```

The main runtime object is [`ChartEngine`](src/lib.rs). It stores:

- `symbol`
- `timeframe`
- `bars`
- `indicators`
- `next_indicator_id`
- `dag`

### Data model

The canonical market input is an OHLCV bar:

```text
time, open, high, low, close, volume
```

This is the current truth inside the engine. The JavaScript side fetches Binance klines and converts them into this bar shape before ingesting them into WASM.

Current flow:

1. JavaScript fetches historical bars.
2. JavaScript writes the data into typed arrays and calls `engine.ingestColumnsFast(...)` or `engine.ingestColumnsZeroCopy(...)`.
3. Rust stores the bars and recomputes indicator state.
4. JavaScript opens a WebSocket stream.
5. Each live kline update is passed to the public wrapper method `engine.upsertBarFast(...)`.
6. Rust updates the last bar or appends a new one, then incrementally updates indicators.

### Indicator model

Every indicator instance is stored as:

- identity: `id`, `kind`
- parameters: `period`, `smooth`, `signal`, `multiplier`, `macd`
- outputs: `Vec<IndicatorOutput>`

An indicator may produce one visible series or many:

- single-output: `SMA`, `EMA`, `RSI`, `ATR`, `VWAP`
- multi-output: `MACD`, `ADX`, `STOCHASTIC`, `STOCH_RSI`, `BB`, `KELTNER`, `DONCHIAN`

The engine keeps both:

- visible outputs returned to JavaScript
- hidden state outputs used for incremental updates

Examples of hidden outputs:

- RSI: `avg_gain`, `avg_loss`
- ADX: `tr_avg`, `plus_dm_avg`, `minus_dm_avg`, `dx`
- MACD: `fast_ema`, `slow_ema`
- VWAP: `cumulative_pv`, `cumulative_volume`
- SUPERTREND: `upper_band`, `lower_band`, `trend`

Hidden outputs stay in Rust and are filtered out by `is_visible_output()` in `src/dag.rs` when JavaScript asks for renderable series.

### Full recompute vs incremental update

The engine has two update modes.

Full recompute:

- used after `ingestColumnsFast()` or `ingestColumnsZeroCopy()`
- used after adding or removing an indicator
- rebuilds outputs for all indicators from the full bar history

Incremental update:

- used after `upsertBarFast()` (which calls the internal WASM method `upsert_bar_fast()`)
- updates only the latest value for each indicator
- avoids rebuilding the full history on every live tick/bar

The engine calls `update_indicators_incremental()` from `src/lib.rs`; indicator-specific incremental dispatch lives in `update_indicator_incremental()` in `src/dispatch.rs`. Each supported indicator has a `latest_*` function that computes just the next value from prior outputs plus the newest bar.

That is the main performance rule in the current engine:

- historical load can scan
- live updates should be O(1) per indicator wherever practical

### Shared computation and node reuse

The engine already deduplicates shared intermediate computations inside a recompute pass through `NodeCache`:

- type: `HashMap<String, Series>`
- examples:
  - `ema:close:20`
  - `atr:ohlc:14`
  - `rsi:close:14`
  - `stoch:rsi:14:14:3:3`

This matters because different indicators reuse the same base series:

- `MACD` reuses EMA nodes
- `STOCH_RSI` reuses RSI nodes
- `KELTNER` reuses EMA and ATR nodes
- `SUPERTREND` reuses ATR nodes

So the current design already follows the “compute shared series once” rule, even though the storage model is still simple.

### DAG model

The DAG currently serves two purposes:

1. make dependencies explicit
2. let the test app show what the engine created

For each indicator, the engine declares:

- `indicator_nodes()`
- `indicator_edges()`

Source nodes are things like:

- `close`
- `high`
- `low`
- `volume`

Computed nodes are things like:

- `ema:close:20`
- `atr:ohlc:14`
- `adx:ohlc:14`
- `keltner:upper:20:2`

Indicator instance nodes are things like:

- `RSI#1`
- `MACD(12,26,9)#4`

The DAG debug output is exposed through `dag_debug()` and rendered by the test app so you can see whether indicators are sharing upstream computation as expected.

### JavaScript/WASM boundary

The public TypeScript wrapper hides the generated snake-case WASM methods.
Applications call methods such as `upsertBarFast()`; only the wrapper calls
`upsert_bar_fast()`.

Historical OHLCV input and raw series output use typed arrays:

- `ingestColumnsFast()` passes typed arrays through `wasm-bindgen`
- `ingestColumnsZeroCopy()` writes typed arrays directly into allocated WASM memory
- `candleColumns()`, `indicatorValueSeries()`, and `latestIndicatorValues()` return typed arrays

Convenience and metadata APIs such as `candles()`, `latestIndicatorPoints()`,
`indicatorDescriptors()`, and `dagDebug()` allocate JavaScript objects.
`candleColumns()`, `indicatorValueSeries()`, and `latestIndicatorValues()` expose
zero-copy typed-array views into WASM memory. Treat those views as ephemeral and
read them again after mutating the engine.

### Rendering model

The engine does not render.

Instead, Rust returns data plus lightweight metadata:

- descriptor `pane`
- output `renderer`
- output `color`

The test app uses that metadata to decide whether an output should become:

- a line series
- a histogram series
- an overlay pane series
- a separate pane series

That keeps the engine renderer-agnostic. The current UI uses TradingView Lightweight Charts, but the computation layer is not tied to that library.

### Why the current engine performs reasonably well

The current implementation is fast for the present scope because it does a few important things and skips a lot of unnecessary machinery:

- bars are stored once in Rust
- indicators are updated incrementally on live updates
- shared series are reused within recomputation
- hidden rolling state stays in the engine
- JavaScript does not recompute indicators
- rendering is separate from computation

For a browser-side indicator engine, those choices matter more than adding a large framework.

### Current limits

The current engine is intentionally narrower than the original architecture docs.

What it does not have yet:

- multi-view shared sessions
- multi-timeframe aggregation graph
- tick-level canonical storage
- event queue / poll model
- zero-copy typed array output views
- plugin SDK
- script engine
- backtesting runtime
- persistent storage layer

Right now it is one in-memory chart engine instance with one canonical bar series and a growing indicator set.

### Target architecture

The two design docs in the repo describe the larger direction:

- JavaScript continues to own transport
- WASM continues to own truth
- everything becomes a named series
- indicators become a first-class dependency graph
- one canonical session can feed multiple chart views
- aggregation moves from “input timeframe only” toward a reducer graph
- outputs move toward lower-copy access patterns

That target is still useful, but the current repo takes the lazy path first:

- get the data model stable
- prove incremental indicators
- prove shared node reuse
- prove renderer separation
- then add bigger runtime pieces only when they are actually needed

## Implemented indicators

Call `engine.indicatorDescriptors()` for the current list, parameter defaults,
output names, renderers, panes, and colors. The compile-time TypeScript union
`IndicatorKind` in `src/engine.ts` is the source of truth for accepted built-in
indicator names.

## Project layout

- [src/lib.rs](src/lib.rs): Rust/WASM chart engine
- [src/engine.ts](src/engine.ts): npm-facing TypeScript wrapper
- [src/index.ts](src/index.ts): package entrypoint
- [examples/app](examples/app): GitHub Pages example app UI, fixtures, and styles
- [Cargo.toml](Cargo.toml): Rust crate config
- [package.json](package.json): package metadata and build scripts

## Requirements

- Node.js
- npm
- Rust toolchain
- `wasm-pack`

## Run locally

```bash
npm install
npm run dev
```

The dev server builds the WASM package first, then starts Vite on `127.0.0.1` using `examples/app` as the app root.

## Build

```bash
npm run build
```

## Deploy to GitHub Pages

The repo includes a GitHub Actions workflow at [.github/workflows/deploy-pages.yml](.github/workflows/deploy-pages.yml).

To enable auto-deploy:

1. Push the repo to GitHub.
2. Open `Settings -> Pages`.
3. Set `Source` to `GitHub Actions`.
4. Push to `main`.

The workflow will:

- install Node dependencies
- install Rust and the `wasm32-unknown-unknown` target
- install `wasm-pack`
- run `npm run build`
- publish `dist/` to GitHub Pages

The Vite config builds the example app from `examples/app` into the root `dist/` folder and uses a relative asset base, so the built app works under the GitHub Pages repo path without extra repo-name-specific config.

## Test

```bash
cargo test
```

## Add a new indicator

The shortest path is to copy the shape of an existing indicator with similar behavior.

### 1. Create the indicator file

Create `src/indicators/new_indicator.rs` with:

- `new_indicator_store(store: &CandleStore, ..., nodes: &mut NodeCache) -> RcSeries` for one visible output, or `Vec<NamedSeries>` for multiple outputs/state
- `latest_new_indicator_store(store: &CandleStore, ...) -> Option<f64>` — incremental single-value update
- `descriptor()` — UI metadata for a single-indicator module

The full-series function should return series data, not JS-facing output objects. `src/dispatch.rs` converts it with `.into_outputs()`. Visible lines use names like `value`, `signal`, `upper`, `plus_di`, and so on. Hidden state outputs can also live in the same return value.

If the indicator reuses computed series, cache them in `NodeCache`. Examples already in the repo:

- `ema:close:{period}`
- `atr:ohlc:{period}`
- `adx:ohlc:{period}`
- `vwap:hlcv`

### 2. Register the module

In `src/indicators/mod.rs`, add:

```rust
pub mod new_indicator;
pub use new_indicator::*;
```

Also add the module descriptor to `indicator_descriptors()` in the same file:

```rust
new_indicator::descriptor(),
```

### 3. Wire into the engine

Touch these core files:

| File | What to add |
|------|-------------|
| `src/types.rs` | Add the kind once in `indicator_kinds!`, plus any parameter flags |
| `src/dag.rs` | Add `indicator_nodes()`, `indicator_edges()`, visibility, and validation if needed |
| `src/dispatch.rs` | Match arms in `compute_indicator_store()` and `update_indicator_incremental()` |

If the indicator has hidden intermediate state used only for incremental updates, keep those outputs out of the UI by adding them to `is_visible_output()` in `src/dag.rs`.

If the indicator has custom parameters that need validation, add a check in `validate_indicator()` in `src/dag.rs`.

### 4. Expose it in the example app

The UI mostly comes from Rust descriptors, but `examples/app/main.ts` still matters.

- Add a label in `indicatorLabel()` if the indicator has custom formatting.
- Extend `IndicatorConfig` if the indicator needs new parameters.
- Update `setConfigParam()` and `readIndicatorConfig()` if the new params need parsing or validation.

If the descriptor is enough, the picker and series rendering work without extra UI code.

### 5. Add one small test set

Add the smallest checks that prove the indicator is wired correctly in a
`#[cfg(test)] mod tests` beside the implementation:

- DAG node test
- Full-series output shape test
- Incremental-last-value matches full-series-last-value test

That is enough to catch most mistakes in this repo.

### 6. Rebuild and verify

```bash
cargo test
npm run build
```

### Template

Use this checklist:

- create `src/indicators/new_indicator.rs` with `_store` compute and `latest_*_store` functions
- add `descriptor()` in the indicator file
- add `pub mod`, `pub use`, and the descriptor call in `src/indicators/mod.rs`
- add kind once to `indicator_kinds!` in `src/types.rs`
- add `indicator_nodes()` and `indicator_edges()` in `src/dag.rs`
- add batch and incremental match arms in `src/dispatch.rs`
- hide state outputs in `is_visible_output()` if needed
- add `main.ts` label/param handling if needed
- add 2-3 focused tests beside the indicator implementation

## Add a formula indicator

Use the formula API when you want end users to define indicators from the browser without shipping arbitrary JavaScript execution.

### 1. Register the formula

```ts
const id = engine.addFormulaIndicator({
  name: "EMA Trend Strength",
  pane: "separate",
  params: { fast: 12, slow: 34, signal: 9 },
  outputs: [
    { name: "spread", renderer: "line", pane: "separate", color: "#38bdf8" },
    { name: "histogram", renderer: "histogram", pane: "separate", color: "#f59e0b" },
  ],
  script: `
    fast_ma = ema(close, fast)
    slow_ma = ema(close, slow)
    spread = fast_ma - slow_ma
    signal_line = ema(spread, signal)
    histogram = spread - signal_line
  `,
});
```

### 2. Read outputs the same way as built-ins

- `indicatorValueSeries(id)`
- `latestIndicatorValues(id)`
- `latestIndicatorPoints(id)`

### 3. Keep the script small

- Assignments are intermediate unless their name appears in `outputs`
- Supported helpers are the built-in numeric/series functions documented in the code
- No loops, imports, or arbitrary JavaScript execution

## Notes

- The repo ignores generated output such as `node_modules/`, `target/`, `dist/`, and `pkg/`.
