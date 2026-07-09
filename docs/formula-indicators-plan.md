# Lua-Style Formula Indicators

## Summary
Add user-created indicators through a safe Lua-style formula DSL executed by Rust/WASM. This is not embedded Lua and not arbitrary JS. Users write familiar assignment/math formulas, Rust parses them into a small AST, evaluates them against `CandleStore`, and stores visible outputs in the existing indicator output path.

## Public API
- Add `add_formula_indicator(def)` / `addFormulaIndicator(def)` where `def` contains `name`, `pane`, `params`, `outputs`, and `script`.
- Outputs are explicit: assigned variables are intermediate unless their name appears in `outputs`.
- Keep built-ins unchanged. Formula indicators get their own ids and are returned by the same read APIs: `indicatorValueSeries`, `indicatorSeries`, `latestIndicatorValues`, and `latestIndicatorPoints`.

Example:

```ts
engine.addFormulaIndicator({
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

## DSL V1
- Support: assignments, identifiers, numbers, parentheses, function calls, `+ - * / %`, comparisons, `and or not`.
- Built-ins: `open`, `high`, `low`, `close`, `volume`, `time`, `hl2`, `hlc3`, `ohlc4`, `na`.
- Functions: `sma`, `ema`, `rsi`, `atr`, `highest`, `lowest`, `stdev`, `abs`, `min`, `max`, `iff`, `nz`, `cross`, `crossover`, `crossunder`.
- Skip in v1: loops, tables, user functions, imports, strings, drawing commands, network/DOM/IO.
- Parser: use a small Rust parser crate, preferably `pest`, instead of hand-rolling precedence/error handling.

## Engine Changes
- Add an internal indicator source enum: built-in `IndicatorKind` or formula definition.
- Add a formula module that parses once on registration, validates output names, validates function calls/arity, and evaluates formulas into `IndicatorArena`.
- Formula evaluation computes full series and caches assignment results by variable name during one evaluation.
- Live update v1 recomputes formula outputs after `upsertBarFast`. This is intentionally simple and correct; add incremental formula evaluation only if benchmarks show it matters.
- Preserve existing renderer path: formula outputs use the same visible output APIs as built-ins.

## Frontend/App Changes
- Add TS types for `FormulaIndicatorDefinition`, formula params, and formula outputs.
- Store formula configs in the wrapper so `indicatorSeries` and latest update methods work by id like built-ins.
- Example app adds a minimal “custom formula indicator” preset, not a full editor yet.
- README documents the DSL, explicit output rule, examples, and limitations.

## Tests
- Parser tests for assignment, precedence, function calls, comparisons, booleans, and invalid syntax.
- Evaluator tests for `sma`, `ema`, `rsi`, `atr`, `cross*`, `iff`, and `nz` against known data.
- Engine tests that intermediate variables are hidden and explicit outputs render.
- Live update test: after `upsertBarFast`, formula latest value matches a full recompute.
- Run `cargo test`, `npm run build`, and compare benchmark/build size before/after.
