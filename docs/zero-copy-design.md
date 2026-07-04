# Zero-Copy Engine Design

This document describes the zero-copy architecture of the Rust/WASM chart engine and provides a guide for developers adding new indicators while preserving zero-copy semantics.

## Architecture Overview

The engine achieves zero-copy data flow from ingest to rendering by eliminating unnecessary allocations and memcpys at the WASM/JS boundary.

```text
Binance REST/WebSocket
        |
        v
JavaScript app
  - fetch history → ingestColumnsZeroCopy()  [one copy: JS heap → WASM heap]
  - live kline    → upsertBarFast()          [scalar args, no copy]
        |
        v
ChartEngine (Rust/WASM)
  - CandleStore: columnar Vec<f64> with pre-reserved capacity
  - IndicatorArena: packed contiguous f64 buffer per indicator
  - NodeCache: Rc<Vec<f64>> shared series (free Rc::clone on cache hits)
        |
        v
JavaScript reads
  - candleColumnsFast()            → Float64Array.view() over WASM memory [zero-copy]
  - indicatorOutputValuesFast()    → Float64Array.view() over arena data  [zero-copy]
  - latestIndicatorValuesFast()    → Float64Array.view() over scratch     [near zero-copy]
```

## Data Flow: Copy Budget

| Stage | Copies | Mechanism |
|-------|--------|-----------|
| **Ingest** (JS → WASM) | 1 per column | `TypedArray.set()` into WASM linear memory via `alloc_candle_buffer` |
| **Recompute** (compute indicators) | 0 cache hits | `Rc::clone` returns shared pointer, no data copy |
| **Recompute** (cache insert) | 0 | `Rc::new(values)` wraps computed Vec, `Rc::clone` for cache |
| **Incremental update** | 0 | Arena `upsert_last` writes one f64 in-place |
| **Read candles** (WASM → JS) | 0 | `Float64Array::view()` over CandleStore Vecs |
| **Read indicators** (WASM → JS) | 0 | `Float64Array::view()` over IndicatorArena data |
| **Read latest values** (WASM → JS) | 0 | `Float64Array::view()` over scratch buffer |

## Core Design Decisions

### 1. NaN Sentinel Storage

All indicator output values are stored as `Vec<f64>` with `f64::NAN` representing missing/not-yet-computed values.

```rust
type Series = Vec<f64>;  // NaN = missing value
```

**Why:** `Vec<Option<f64>>` uses 16 bytes per value (8 byte f64 + 8 byte discriminant). `Vec<f64>` uses 8 bytes — half the memory, and the raw slice is directly viewable as a JavaScript `Float64Array` without any transformation.

**Convention:** Use `nan_to_none(v)` when you need `Option<f64>` semantics (e.g., for conditional logic in incremental updates).

### 2. IndicatorArena

Each indicator's outputs are packed into a single contiguous `Vec<f64>`:

```rust
struct IndicatorArena {
    data: Vec<f64>,      // [slot_0: N values][slot_1: N values]...
    slots: Vec<String>,  // slot names: "value", "signal", "histogram", etc.
    slot_len: usize,     // N = bars count
}
```

**Why:** One allocation per indicator instead of N separate Vecs. The `Float64Array::view()` can point directly into any slot's contiguous slice.

### 3. Rc<Series> NodeCache

Shared intermediate computations (EMA, ATR, SMA) are stored in the cache as `Rc<Vec<f64>>`:

```rust
type RcSeries = Rc<Series>;
type NodeCache = HashMap<String, RcSeries>;
```

**Why:** When MACD, Bollinger, and Keltner all need `ema:close:20`, the first computation wraps the result in `Rc::new(values)`. Subsequent cache hits return `Rc::clone(values)` — a pointer bump, not a 400KB memcpy.

### 4. Capacity Pre-Reservation

Both `CandleStore` and `IndicatorArena` reserve 256 extra slots on creation:

```rust
fn from_raw_columns(mut time: Vec<u32>, ...) -> Self {
    time.reserve(256);  // room for 256 live appends before realloc
    ...
}
```

**Why:** WASM linear memory never shrinks. After many engine create/destroy cycles, the allocator fragments. Pre-reserving avoids expensive reallocation on the first live bar append.

### 5. Unsafe Float64Array::view()

Output functions return ephemeral views into WASM linear memory:

```rust
pub fn candle_columns_fast(&self) -> Result<JsValue, JsValue> {
    js_set(&out, "close", unsafe { Float64Array::view(&self.bars.close) })?;
    ...
}
```

**Safety contract:** The caller (TypeScript wrapper) must not retain the view across any engine mutation. The wrapper reads the view immediately and passes data to Lightweight Charts.

## Zero-Copy Ingest Path

The `alloc_candle_buffer` / `finalize_candle_buffer` API eliminates the `.to_vec()` copy:

```
┌─────────────────┐    alloc_candle_buffer(50000)     ┌──────────────────────┐
│  JavaScript     │ ──────────────────────────────────>│  Rust allocates Vecs │
│                 │ <──────────── { ptrs, len } ───────│  returns pointers    │
│                 │                                    └──────────────────────┘
│  TypedArray.set │ ─── writes directly into ──────────> WASM linear memory
│                 │    (one memcpy per column)
│                 │                                    ┌──────────────────────┐
│                 │ ──── finalize_candle_buffer() ────>│  Rust recomputes     │
│                 │                                    │  indicators          │
└─────────────────┘                                    └──────────────────────┘
```

Traditional `ingest_columns_fast` does: JS TypedArray → wasm-bindgen `.to_vec()` → Rust Vec. That's two allocations (wasm-bindgen temp + final Vec). The zero-copy path writes directly into the final Vec.

---

## Developer Guide: Adding a New Indicator with Zero-Copy

### Checklist

When adding a new indicator, follow these patterns to maintain zero-copy:

1. **Return `RcSeries` from store functions** (not `Series`)
2. **Use `f64::NAN`** for missing values (not `Option`)
3. **Use `Rc::clone`** on cache hits (not `(**values).clone()`)
4. **Use `Rc::new` + `Rc::clone`** on cache insert (not `values.clone()`)
5. **Use `rc_one_output`** for single-output indicators
6. **Use `rc_into_owned`** only when you need a mutable owned Vec

### Template: Single-Output Store Function

```rust
fn my_indicator_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    // 1. Cache key
    let key = format!("my_indicator:close:{period}");

    // 2. Cache hit → free Rc::clone (no data copy)
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }

    // 3. Compute output using f64::NAN for missing values
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    for index in period - 1..store.len() {
        // ... your computation ...
        out[index] = computed_value;
    }

    // 4. Wrap in Rc, insert clone into cache, return Rc
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
```

### Template: Using Shared Cached Series

If your indicator depends on EMA, ATR, or another cached series:

```rust
fn my_composite_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> RcSeries {
    let key = format!("my_composite:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }

    // Get shared EMA — free if already cached by another indicator
    let ema = ema_close_store(store, period, nodes);  // returns RcSeries

    // Iterate via Deref (no copy — Rc<Vec<f64>> derefs to &[f64])
    let out: Vec<_> = ema.iter().enumerate().map(|(i, &val)| {
        if val.is_nan() { f64::NAN } else { val * 2.0 }
    }).collect();

    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}
```

**Key point:** `ema_close_store()` returns `Rc<Vec<f64>>`. Calling `.iter()` on it works via Deref — no clone, no copy. The EMA data stays shared in memory.

### Template: Multi-Output Indicator

For indicators with multiple outputs (like MACD, Bollinger, ADX):

```rust
fn my_multi_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    // Check if all outputs are cached
    let upper_key = format!("my_multi:upper:{period}");
    let lower_key = format!("my_multi:lower:{period}");
    if let (Some(upper), Some(lower)) = (nodes.get(&upper_key), nodes.get(&lower_key)) {
        return vec![
            IndicatorOutput { name: "upper".to_string(), values: (**upper).clone() },
            IndicatorOutput { name: "lower".to_string(), values: (**lower).clone() },
        ];
    }

    // Compute
    let base = rc_into_owned(ema_close_store(store, period, nodes));
    let upper: Vec<_> = base.iter().map(|&v| if v.is_nan() { f64::NAN } else { v + 10.0 }).collect();
    let lower: Vec<_> = base.iter().map(|&v| if v.is_nan() { f64::NAN } else { v - 10.0 }).collect();

    // Cache each output
    nodes.insert(upper_key, Rc::new(upper.clone()));
    nodes.insert(lower_key, Rc::new(lower.clone()));

    vec![
        IndicatorOutput { name: "upper".to_string(), values: upper },
        IndicatorOutput { name: "lower".to_string(), values: lower },
    ]
}
```

### Template: Incremental Update

The incremental path uses `upsert_output` which writes directly into the `IndicatorArena`:

```rust
// In update_indicators_incremental():
"MY_INDICATOR" => {
    let value = latest_my_indicator_store(&self.bars, indicator.period);
    upsert_output(&mut indicator.outputs, "value", target_len, value);
}
```

`upsert_output` calls `arena.upsert_last(name, target_len, val)` which:
- If slot exists: writes one f64 at the end (O(1), no allocation)
- If slot is new: extends the arena data (rare, only on first bar)
- If bars grew: resizes all slots once (amortized via pre-reservation)

### Template: Latest Value Function

```rust
fn latest_my_indicator_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    // O(1) computation using only the last bar + previous state
    let window = &store.close[store.len() - period..];
    Some(window.iter().sum::<f64>() / period as f64)
}
```

### Wiring into compute_indicator_store

```rust
// In the match block:
"MY_INDICATOR" => rc_one_output(my_indicator_store(store, period, nodes)),
```

Use `rc_one_output` (not `one_output`) for functions returning `RcSeries`. This extracts the Vec from the Rc without cloning if the indicator is the sole consumer of that cached value.

### Key Helpers

| Helper | Use when |
|--------|----------|
| `Rc::clone(values)` | Returning from cache hit (free) |
| `Rc::new(values)` | Wrapping a newly computed Vec |
| `rc_into_owned(rc)` | You need a mutable `Vec<f64>` from an `RcSeries` |
| `rc_one_output(rc)` | Building `Vec<IndicatorOutput>` from a single RcSeries |
| `nan_to_none(v)` | Converting f64 to Option<f64> for conditional logic |
| `output_at(arena, name, idx)` | Reading previous output value in incremental path |

### Anti-Patterns to Avoid

```rust
// ❌ BAD: Cloning the Vec on cache hit
if let Some(values) = nodes.get(&key) {
    return (**values).clone();  // 400KB memcpy at 50K bars!
}

// ✅ GOOD: Free Rc pointer bump
if let Some(values) = nodes.get(&key) {
    return Rc::clone(values);
}
```

```rust
// ❌ BAD: Double allocation on insert
let values = compute(...);
nodes.insert(key, Rc::new(values.clone()));  // clone + Rc::new = 2 allocations
values

// ✅ GOOD: Single allocation
let rc = Rc::new(compute(...));
nodes.insert(key, Rc::clone(&rc));  // Rc::clone = free
rc
```

```rust
// ❌ BAD: Using Option for missing values
let mut out: Vec<Option<f64>> = vec![None; len];
out[i] = Some(value);

// ✅ GOOD: NaN sentinel enables zero-copy views
let mut out: Vec<f64> = vec![f64::NAN; len];
out[i] = value;
```

```rust
// ❌ BAD: Copying data for JS output
Float64Array::from(self.bars.close.as_slice())  // allocates new TypedArray + copies

// ✅ GOOD: View into existing WASM memory
unsafe { Float64Array::view(&self.bars.close) }  // zero allocation, zero copy
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Cache hit (Rc::clone) | O(1) | Pointer increment |
| Cache insert (Rc::new) | O(1) | Single allocation |
| Incremental append | O(1) per indicator | One f64 write per output slot |
| Candle read | O(1) | View pointer, no data movement |
| Indicator read | O(1) | View pointer into arena |
| Full recompute | O(N × I) | N = bars, I = indicator count |

At 50K bars with 6 indicators:
- Live tick (append + read all outputs): **< 1ms**
- Candle column read: **0.005ms**
- Indicator output read: **0.006ms**
- Full initial load + recompute: **~45ms** (one-time)
