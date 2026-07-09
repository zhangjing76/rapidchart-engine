# Formula script reference

Formula indicators use a small Lua-style expression language implemented by the
engine. It is not general Lua and does not execute JavaScript.

## Complete example

```ts
const id = engine.addFormulaIndicator({
  name: "EMA Trend Strength",
  pane: "separate",
  params: {
    fast: 12,
    slow: 34,
    signal: 9,
  },
  outputs: [
    {
      name: "spread",
      renderer: "line",
      pane: "separate",
      color: "#38bdf8",
    },
    {
      name: "histogram",
      renderer: "histogram",
      pane: "separate",
      color: "#f59e0b",
    },
  ],
  script: `
    -- Variables not listed in outputs are intermediate values.
    fast_ma = ema(close, fast)
    slow_ma = ema(close, slow)
    spread = fast_ma - slow_ma
    signal_line = ema(spread, signal)
    histogram = spread - signal_line
  `,
});
```

Every name listed in `outputs` must be assigned by the script. Assigned names
not listed in `outputs` remain internal.

## Statements and names

A script contains assignments:

```lua
name = expression
```

Statements may be separated by whitespace, newlines, or semicolons. `--` starts
a comment that continues to the end of the line.

Names may contain ASCII letters, digits, and underscores, but must start with a
letter or underscore. Names are case-sensitive. An expression may reference:

1. a variable assigned earlier in the script
2. a value supplied in `params`
3. a built-in market series or constant

Assignments are evaluated in order for every candle. A variable cannot refer to
itself or to a variable assigned later.

## Market data

| Name | Value |
|---|---|
| `time` | Unix timestamp in seconds |
| `open` | Candle open |
| `high` | Candle high |
| `low` | Candle low |
| `close` | Candle close |
| `volume` | Candle volume |
| `hl2` | `(high + low) / 2` |
| `hlc3` | `(high + low + close) / 3` |
| `ohlc4` | `(open + high + low + close) / 4` |

Constants:

| Name | Value |
|---|---|
| `true` | `1` |
| `false` | `0` |
| `na`, `nil` | Missing value (`NaN`) |

## Numbers and operators

Numbers use decimal or scientific notation, such as `12`, `0.5`, and `1e-3`.

Operators, from highest to lowest precedence:

| Operators | Meaning |
|---|---|
| `-value`, `not value` | Negation and logical NOT |
| `*`, `/`, `%` | Multiply, divide, remainder |
| `+`, `-` | Add, subtract |
| `==`, `~=`, `<`, `<=`, `>`, `>=` | Comparison |
| `and` | Logical AND |
| `or` | Logical OR |

Parentheses may be used to control evaluation order. Comparisons and logical
operators return `1` for true and `0` for false. Zero and missing values are
false; every other number is true. `and`, `or`, and `iff()` evaluate all of
their operands; they do not short-circuit.

## Functions

| Function | Result |
|---|---|
| `sma(source, period)` | Simple moving average |
| `ema(source, period)` | Exponential moving average |
| `rsi(source, period)` | Wilder RSI |
| `atr(period)` | Wilder average true range using candle OHLC |
| `highest(source, period)` | Highest value in the rolling window |
| `lowest(source, period)` | Lowest value in the rolling window |
| `stdev(source, period)` | Population standard deviation |
| `abs(value)` | Absolute value |
| `min(left, right)` | Smaller value |
| `max(left, right)` | Larger value |
| `iff(condition, yes, no)` | Conditional value |
| `nz(value)` | Replace a missing value with `0` |
| `nz(value, fallback)` | Replace a missing value with `fallback` |
| `cross(left, right)` | Either series crosses the other |
| `crossover(left, right)` | Left crosses above right |
| `crossunder(left, right)` | Left crosses below right |

Periods must evaluate to a positive finite number and are rounded to the nearest
integer. A call's period must stay constant while the indicator is running, so
use a literal or parameter rather than a changing market series.

Rolling functions return `na` until they have enough input. `ema()` starts at
the first source value. Crossing functions return `na` until a previous pair of
values exists.

## Outputs and rendering

The script calculates values; the `outputs` configuration declares which values
are public and how the application should render them:

```ts
outputs: [
  {
    name: "signal",
    renderer: "line",       // "line" or "histogram"
    pane: "overlay",        // "overlay" or "separate"
    color: "#22c55e",
  },
]
```

The output `name` must exactly match an assignment in the script. Output order
is the order returned by `indicatorValueSeries()`, `latestIndicatorValues()`,
and `latestIndicatorPoints()`.

## Incremental updates

The same script handles historical loads and live updates. Stateful function
calls retain their rolling state when a new candle is appended. Replacing the
latest candle or changing history causes the engine to recompute the formula
from the beginning.

Each function call site has independent state:

```lua
fast = ema(close, 12)
slow = ema(close, 26)
```

These two calls do not share EMA state.

## Current limitations

The language currently has:

- assignments and expressions only
- no loops, functions, tables, strings, imports, or file/network access
- no `if ... then`; use `iff(condition, yes, no)`
- no historical indexing such as `close[1]`
- no direct calls to arbitrary built-in Rust indicators
- no user-defined persistent variables outside stateful built-in functions

Unknown names, unknown functions, missing required arguments, missing output
assignments, and dynamic periods produce an engine error.
