# rapidchart-engine

Rust/WASM chart engine with a Vite test app using TradingView Lightweight Charts and Binance market data.

## Library usage

`rapidchart-engine` now exposes a small typed TypeScript wrapper from [src/index.ts](/Users/jingzhang/Projects/chart/src/index.ts). The Rust engine computes candles and indicators; your app renders the returned series.

### Build the WASM package

```bash
npm install
npm run build:wasm
```

This generates the browser-facing package in `pkg/`.

### Import and initialize

```ts
import { RapidChartEngine, initEngine } from "rapidchart-engine";

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
const smaSeries = engine.indicatorSeries(smaId);
const rsiSeries = engine.indicatorSeries(rsiId);
const macdValues = engine.indicatorValueSeries(macdId);
```

`indicatorSeries()` returns visible outputs only. Multi-output indicators such as `MACD`, `BOLLINGER`, `KELTNER`, `DONCHIAN`, `ADX`, and `STOCHASTIC` return one series per output.

The Rust/WASM layer returns raw indicator values. The TypeScript wrapper maps those values onto candle timestamps and applies visual shifts for chart-specific outputs such as `ICHIMOKU`.

`candles()` remains the canonical market-data API. There is no separate `timeline()` API: Rust owns the full OHLCV history, and the TypeScript side reads candle times from `candles()` when it needs to build chart-ready points.

For zero-copy groundwork, the wrapper also exposes columnar read APIs:

- `candleColumns()` returns typed arrays for `time`, `open`, `high`, `low`, `close`, and `volume`
- `indicatorValueSeries(id)` returns typed arrays of raw visible output values, using `NaN` for gaps

Those accessors still materialize typed arrays today, but they define the API shape we can later back with shared memory or zero-copy views.

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

### Push live updates

```ts
engine.upsertBarFast({
  time: 1719837600,
  open: 62450,
  high: 62700,
  low: 62350,
  close: 62620,
  volume: 1100,
});

const latestRsi = engine.latestIndicatorPoints(rsiId);
```

### Render with Lightweight Charts

```ts
import {
  CandlestickSeries,
  LineSeries,
  createChart,
  type Time,
} from "lightweight-charts";

const chart = createChart(document.getElementById("chart")!);
const candleSeries = chart.addSeries(CandlestickSeries);
const smaLine = chart.addSeries(LineSeries);

candleSeries.setData(
  engine.candles().map((bar: any) => ({
    ...bar,
    time: bar.time as Time,
  })),
);

const smaOutput = engine.indicatorSeries(smaId)[0];
smaLine.setData(
  smaOutput.points
    .filter((point: any) => point.value !== null)
    .map((point: any) => ({
      time: point.time as Time,
      value: point.value,
    })),
);
```

### Wrapper API

- `new RapidChartEngine()`
- `ingestColumnsFast(columns)`
- `candles()`
- `candleColumns()`
- `addIndicator(config)`
- `indicatorValueSeries(id)`
- `indicatorSeries(id)`
- `latestIndicatorValues(id)`
- `latestIndicatorTime(id, output)`
- `latestIndicatorPoints(id)`
- `removeIndicator(id)`
- `indicatorDescriptors()`
- `dagDebug()`
- `upsertBarFast(bar)`

The local implementation lives in [src/engine.ts](/Users/jingzhang/Projects/chart/src/engine.ts), but consumers should import from the package root instead of reaching into `src/`.

### Supported indicator configs

All configs require `kind`.

- `SMA`: `{ kind: "SMA", period }`
- `EMA`: `{ kind: "EMA", period }`
- `RSI`: `{ kind: "RSI", period }`
- `CCI`: `{ kind: "CCI", period }`
- `MFI`: `{ kind: "MFI", period }`
- `WILLIAMS_R`: `{ kind: "WILLIAMS_R", period }`
- `ATR`: `{ kind: "ATR", period }`
- `ADX`: `{ kind: "ADX", period }`
- `TRIX`: `{ kind: "TRIX", period }`
- `DEMA`: `{ kind: "DEMA", period }`
- `TEMA`: `{ kind: "TEMA", period }`
- `TRIMA`: `{ kind: "TRIMA", period }`
- `STDDEV`: `{ kind: "STDDEV", period }`
- `ENVELOPE`: `{ kind: "ENVELOPE", period, multiplier }`
- `TSI`: `{ kind: "TSI", period, stoch_period }`
- `KST`: `{ kind: "KST" }`
- `BOP`: `{ kind: "BOP" }`
- `DPO`: `{ kind: "DPO", period }`
- `MOMENTUM`: `{ kind: "MOMENTUM", period }`
- `ULTIMATE_OSCILLATOR`: `{ kind: "ULTIMATE_OSCILLATOR", period, stoch_period, smooth }`
- `CHAIKIN_OSCILLATOR`: `{ kind: "CHAIKIN_OSCILLATOR", fast, slow }`
- `FORCE_INDEX`: `{ kind: "FORCE_INDEX", period }`
- `VWMA`: `{ kind: "VWMA", period }`
- `WILLIAMS_AD`: `{ kind: "WILLIAMS_AD" }`
- `CHAIKIN_VOLATILITY`: `{ kind: "CHAIKIN_VOLATILITY", period }`
- `PRICE_CHANNEL`: `{ kind: "PRICE_CHANNEL", period }`
- `STARC`: `{ kind: "STARC", period, multiplier }`
- `STOCHASTIC`: `{ kind: "STOCHASTIC", period, smooth }`
- `STOCH_RSI`: `{ kind: "STOCH_RSI", period, stoch_period, smooth, signal }`
- `MACD`: `{ kind: "MACD", fast, slow, signal }`
- `PPO`: `{ kind: "PPO", fast, slow, signal }`
- `BB`: `{ kind: "BB", period, multiplier }`
- `SUPERTREND`: `{ kind: "SUPERTREND", period, multiplier }`
- `KELTNER`: `{ kind: "KELTNER", period, multiplier }`
- `DONCHIAN`: `{ kind: "DONCHIAN", period }`
- `PARABOLIC_SAR`: `{ kind: "PARABOLIC_SAR", psar_step, psar_max_step }`
- `ICHIMOKU`: `{ kind: "ICHIMOKU", tenkan_period, kijun_period, senkou_b_period }`
- `PIVOT_POINTS`: `{ kind: "PIVOT_POINTS" }`
- `ROC`: `{ kind: "ROC", period }`
- `AROON`: `{ kind: "AROON", period }`
- `CMF`: `{ kind: "CMF", period }`
- `ADL`: `{ kind: "ADL" }`
- `WMA`: `{ kind: "WMA", period }`
- `HMA`: `{ kind: "HMA", period }`
- `LINEAR_REGRESSION`: `{ kind: "LINEAR_REGRESSION", period }`
- `OBV`: `{ kind: "OBV" }`
- `VWAP`: `{ kind: "VWAP" }`

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

The main runtime object is [`ChartEngine`](/Users/jingzhang/Projects/chart/src/lib.rs:110). It stores:

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
5. Each live kline update is passed to `engine.upsert_bar_fast(...)`.
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

Hidden outputs stay in Rust and are filtered out by [`is_visible_output()`](/Users/jingzhang/Projects/chart/src/lib.rs:528) when JavaScript asks for renderable series.

### Full recompute vs incremental update

The engine has two update modes.

Full recompute:

- used after `ingestColumnsFast()` or `ingestColumnsZeroCopy()`
- used after adding or removing an indicator
- rebuilds outputs for all indicators from the full bar history

Incremental update:

- used after `upsert_bar_fast()`
- updates only the latest value for each indicator
- avoids rebuilding the full history on every live tick/bar

The incremental path lives in [`update_indicators_incremental()`](/Users/jingzhang/Projects/chart/src/lib.rs:340). Each supported indicator has a `latest_*` function that computes just the next value from prior outputs plus the newest bar.

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

The current boundary is intentionally simple:

JavaScript sends plain objects into WASM:

- bars
- indicator config objects

Rust returns plain objects back to JavaScript:

- candles
- indicator output series
- latest indicator points
- indicator descriptors
- DAG debug data

This is not zero-copy. It is the simple version that is easy to debug and good enough for the current test app. The design docs aim for typed-array and lower-copy flows later, but the repo today optimizes first for correctness and incremental compute behavior.

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

- `SMA`
- `EMA`
- `RSI`
- `MACD`
- `BOLLINGER`
- `OBV`
- `ATR`
- `VWAP`
- `STOCHASTIC`
- `ADX` with `ADX`, `+DI`, and `-DI`
- `TRIX`
- `DEMA`
- `TEMA`
- `TRIMA`
- `STDDEV`
- `ENVELOPE`
- `TSI`
- `KST`
- `BOP`
- `DPO`
- `MOMENTUM`
- `ULTIMATE OSCILLATOR`
- `CHAIKIN OSCILLATOR`
- `FORCE INDEX`
- `VWMA`
- `WILLIAMS A/D`
- `CHAIKIN VOLATILITY`
- `PRICE CHANNEL`
- `STARC`
- `SUPERTREND`
- `KELTNER`
- `DONCHIAN`
- `CCI`
- `MFI`
- `WILLIAMS %R`
- `STOCH RSI`
- `PARABOLIC SAR`
- `ICHIMOKU`
- `PIVOT POINTS`
- `ROC`
- `AROON`
- `CMF`
- `ADL`
- `WMA`
- `HMA`
- `LINEAR REGRESSION`
- `PPO`

## Project layout

- [src/lib.rs](/Users/jingzhang/Projects/chart/src/lib.rs): Rust/WASM chart engine
- [src/main.ts](/Users/jingzhang/Projects/chart/src/main.ts): test app UI and Binance integration
- [src/style.css](/Users/jingzhang/Projects/chart/src/style.css): app styles
- [Cargo.toml](/Users/jingzhang/Projects/chart/Cargo.toml): Rust crate config
- [package.json](/Users/jingzhang/Projects/chart/package.json): frontend scripts

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

The dev server builds the WASM package first, then starts Vite on `127.0.0.1`.

## Build

```bash
npm run build
```

## Deploy to GitHub Pages

The repo includes a GitHub Actions workflow at [.github/workflows/deploy-pages.yml](/Users/jingzhang/Projects/chart/.github/workflows/deploy-pages.yml).

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

The Vite config uses a relative asset base, so the built app works under the GitHub Pages repo path without extra repo-name-specific config.

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
| `src/dispatch.rs` | Match arm in `compute_indicator_store()` |
| `src/lib.rs` | Match arm in `update_indicators_incremental()` |

If the indicator has hidden intermediate state used only for incremental updates, keep those outputs out of the UI by adding them to `is_visible_output()` in `src/dag.rs`.

If the indicator has custom parameters that need validation, add a check in `validate_indicator()` in `src/dag.rs`.

### 4. Expose it in the test app

The UI mostly comes from Rust descriptors, but `src/main.ts` still matters.

- Add a label in `indicatorLabel()` if the indicator has custom formatting.
- Extend `IndicatorConfig` if the indicator needs new parameters.
- Update `setConfigParam()` and `readIndicatorConfig()` if the new params need parsing or validation.

If the descriptor is enough, the picker and series rendering work without extra UI code.

### 5. Add one small test set

Add the smallest checks that prove the indicator is wired correctly in the test module in `src/tests.rs`:

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
- add match arm in `compute_indicator_store()` in `src/dispatch.rs`
- add match arm in `update_indicators_incremental()` in `src/lib.rs`
- hide state outputs in `is_visible_output()` if needed
- add `main.ts` label/param handling if needed
- add 2-3 focused tests in `src/tests.rs`

## Notes

- The repo ignores generated output such as `node_modules/`, `target/`, `dist/`, and `pkg/`.
- The two design docs are intentionally excluded from git.
