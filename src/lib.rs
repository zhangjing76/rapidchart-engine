use js_sys::{Array, Float64Array, Object, Reflect, Uint32Array};
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Deserialize, Serialize)]
pub struct Bar {
    pub time: u32,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Default)]
struct CandleStore {
    time: Vec<u32>,
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<f64>,
}

impl CandleStore {
    fn from_bars(bars: Vec<Bar>) -> Self {
        let mut store = Self {
            time: Vec::with_capacity(bars.len()),
            open: Vec::with_capacity(bars.len()),
            high: Vec::with_capacity(bars.len()),
            low: Vec::with_capacity(bars.len()),
            close: Vec::with_capacity(bars.len()),
            volume: Vec::with_capacity(bars.len()),
        };
        for bar in bars {
            store.push(bar);
        }
        store
    }

    fn from_columns(columns: CandleColumnsInput) -> Result<Self, &'static str> {
        let len = columns.time.len();
        if columns.open.len() != len
            || columns.high.len() != len
            || columns.low.len() != len
            || columns.close.len() != len
            || columns.volume.len() != len
        {
            return Err("candle column lengths must match for time/open/high/low/close/volume");
        }
        Ok(Self::from_raw_columns(
            columns.time,
            columns.open,
            columns.high,
            columns.low,
            columns.close,
            columns.volume,
        ))
    }

    fn from_raw_columns(
        mut time: Vec<u32>,
        mut open: Vec<f64>,
        mut high: Vec<f64>,
        mut low: Vec<f64>,
        mut close: Vec<f64>,
        mut volume: Vec<f64>,
    ) -> Self {
        time.reserve(256);
        open.reserve(256);
        high.reserve(256);
        low.reserve(256);
        close.reserve(256);
        volume.reserve(256);
        Self { time, open, high, low, close, volume }
    }

    fn len(&self) -> usize {
        self.time.len()
    }

    fn is_empty(&self) -> bool {
        self.time.is_empty()
    }

    fn push(&mut self, bar: Bar) {
        self.time.push(bar.time);
        self.open.push(bar.open);
        self.high.push(bar.high);
        self.low.push(bar.low);
        self.close.push(bar.close);
        self.volume.push(bar.volume);
    }

    fn set_last(&mut self, bar: Bar) {
        let index = self.len() - 1;
        self.time[index] = bar.time;
        self.open[index] = bar.open;
        self.high[index] = bar.high;
        self.low[index] = bar.low;
        self.close[index] = bar.close;
        self.volume[index] = bar.volume;
    }

    fn last_time(&self) -> Option<u32> {
        self.time.last().copied()
    }

    fn last_close(&self) -> Option<f64> {
        self.close.last().copied()
    }

    fn to_bars(&self) -> Vec<Bar> {
        (0..self.len())
            .map(|index| Bar {
                time: self.time[index],
                open: self.open[index],
                high: self.high[index],
                low: self.low[index],
                close: self.close[index],
                volume: self.volume[index],
            })
            .collect()
    }
}

struct Indicator {
    id: u32,
    kind: String,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    outputs: IndicatorArena,
}

#[derive(Clone, Copy)]
struct MacdParams {
    fast: usize,
    slow: usize,
    signal: usize,
}

#[derive(Deserialize)]
struct IndicatorConfig {
    kind: String,
    period: Option<usize>,
    stoch_period: Option<usize>,
    smooth: Option<usize>,
    fast: Option<usize>,
    slow: Option<usize>,
    signal: Option<usize>,
    multiplier: Option<f64>,
    tenkan_period: Option<usize>,
    kijun_period: Option<usize>,
    senkou_b_period: Option<usize>,
    psar_step: Option<f64>,
    psar_max_step: Option<f64>,
}

#[derive(Deserialize)]
struct CandleColumnsInput {
    time: Vec<u32>,
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<f64>,
}

#[derive(Serialize)]
struct IndicatorDescriptor {
    kind: &'static str,
    name: &'static str,
    pane: &'static str,
    params: Vec<ParamDescriptor>,
    outputs: Vec<OutputDescriptor>,
}

#[derive(Serialize)]
struct ParamDescriptor {
    name: &'static str,
    label: &'static str,
    default: f64,
    min: f64,
    step: &'static str,
}

#[derive(Serialize)]
struct OutputDescriptor {
    name: &'static str,
    renderer: &'static str,
    pane: &'static str,
    color: &'static str,
}

#[derive(Clone, Serialize)]
struct IndicatorOutput {
    name: String,
    values: Vec<f64>,
}

/// Packed contiguous storage for all outputs of a single indicator.
/// All output slots share the same length (= bars count). The backing
/// Vec stores them sequentially: [slot_0: len values][slot_1: len values]...
struct IndicatorArena {
    data: Vec<f64>,
    /// Metadata per slot: (name, slot_index) — slot_index is used to compute offset as slot_index * slot_len.
    slots: Vec<String>,
    /// Number of f64 values per slot (= bars count for this indicator).
    slot_len: usize,
}

impl IndicatorArena {
    /// Create from a Vec<IndicatorOutput>, packing all outputs into one allocation.
    fn from_outputs(outputs: Vec<IndicatorOutput>) -> Self {
        if outputs.is_empty() {
            return Self { data: Vec::new(), slots: Vec::new(), slot_len: 0 };
        }
        let slot_len = outputs[0].values.len();
        let num_slots = outputs.len();
        let mut data = Vec::with_capacity(num_slots * (slot_len + 256));
        let mut slots = Vec::with_capacity(num_slots);

        for output in outputs {
            debug_assert_eq!(output.values.len(), slot_len);
            data.extend_from_slice(&output.values);
            slots.push(output.name);
        }

        Self { data, slots, slot_len }
    }

    /// Get the slice for a named output.
    fn get(&self, name: &str) -> Option<&[f64]> {
        let idx = self.slots.iter().position(|s| s == name)?;
        let start = idx * self.slot_len;
        Some(&self.data[start..start + self.slot_len])
    }

    /// Get the mutable slice for a named output.
    fn get_mut(&mut self, name: &str) -> Option<&mut [f64]> {
        let idx = self.slots.iter().position(|s| s == name)?;
        let start = idx * self.slot_len;
        Some(&mut self.data[start..start + self.slot_len])
    }

    /// Get value at a specific index for a named output.
    fn value_at(&self, name: &str, index: usize) -> Option<f64> {
        let idx = self.slots.iter().position(|s| s == name)?;
        let start = idx * self.slot_len;
        self.data.get(start + index).copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
    }

    /// Get the last value for a named output.
    fn last_value(&self, name: &str) -> Option<f64> {
        if self.slot_len == 0 { return None; }
        self.value_at(name, self.slot_len - 1)
    }

    /// Resize all slots to a new length (for incremental append).
    /// Fills new slots with NaN. If the slot doesn't exist, creates it.
    fn resize_all(&mut self, new_len: usize) {
        if new_len == self.slot_len {
            return;
        }
        let old_len = self.slot_len;
        let num_slots = self.slots.len();
        let mut new_data = Vec::with_capacity(num_slots * (new_len + 256));

        for i in 0..num_slots {
            let old_start = i * old_len;
            let copy_len = old_len.min(new_len);
            new_data.extend_from_slice(&self.data[old_start..old_start + copy_len]);
            if new_len > old_len {
                new_data.resize(new_data.len() + (new_len - old_len), f64::NAN);
            }
        }

        self.data = new_data;
        self.slot_len = new_len;
    }

    /// Set the last value for a named output. If the slot doesn't exist, create it.
    fn upsert_last(&mut self, name: &str, target_len: usize, value: f64) {
        if target_len != self.slot_len {
            self.resize_all(target_len);
        }
        if let Some(idx) = self.slots.iter().position(|s| s == name) {
            let offset = idx * self.slot_len + self.slot_len - 1;
            self.data[offset] = value;
        } else {
            // New slot — append NaN-filled slice, set last value
            self.slots.push(name.to_string());
            let start = self.data.len();
            self.data.resize(start + self.slot_len, f64::NAN);
            if self.slot_len > 0 {
                self.data[start + self.slot_len - 1] = value;
            }
        }
    }

    /// Number of output slots.
    fn num_slots(&self) -> usize {
        self.slots.len()
    }

    /// Iterate over (name, slice) for all slots.
    fn iter_slots(&self) -> impl Iterator<Item = (&str, &[f64])> {
        self.slots.iter().enumerate().map(move |(i, name)| {
            let start = i * self.slot_len;
            (name.as_str(), &self.data[start..start + self.slot_len])
        })
    }
}

#[derive(Serialize)]
struct IndicatorLatestValue {
    output: String,
    value: Option<f64>,
}

#[derive(Default, Serialize)]
struct DagDebug {
    nodes: Vec<String>,
    edges: Vec<DagEdge>,
}

#[derive(Serialize)]
struct DagEdge {
    from: String,
    to: String,
}

#[wasm_bindgen]
pub struct ChartEngine {
    bars: CandleStore,
    indicators: Vec<Indicator>,
    next_indicator_id: u32,
    dag: DagDebug,
    indicator_values_scratch: Vec<Vec<f64>>,
    latest_values_scratch: Vec<f64>,
}

#[wasm_bindgen]
impl ChartEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            bars: CandleStore::default(),
            indicators: Vec::new(),
            next_indicator_id: 1,
            dag: DagDebug::default(),
            indicator_values_scratch: Vec::new(),
            latest_values_scratch: Vec::new(),
        }
    }

    pub fn ingest_bars(&mut self, bars: JsValue) -> Result<(), JsValue> {
        let bars: Vec<Bar> = serde_wasm_bindgen::from_value(bars)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.bars = CandleStore::from_bars(bars);
        self.recompute_indicators();
        Ok(())
    }

    pub fn ingest_columns(&mut self, columns: JsValue) -> Result<(), JsValue> {
        let columns: CandleColumnsInput = serde_wasm_bindgen::from_value(columns)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.bars = CandleStore::from_columns(columns).map_err(JsValue::from_str)?;
        self.recompute_indicators();
        Ok(())
    }

    pub fn ingest_columns_fast(
        &mut self,
        time: Uint32Array,
        open: Float64Array,
        high: Float64Array,
        low: Float64Array,
        close: Float64Array,
        volume: Float64Array,
    ) -> Result<(), JsValue> {
        let len = time.length();
        if open.length() != len
            || high.length() != len
            || low.length() != len
            || close.length() != len
            || volume.length() != len
        {
            return Err(JsValue::from_str(
                "candle column lengths must match for time/open/high/low/close/volume",
            ));
        }
        self.bars = CandleStore::from_raw_columns(
            time.to_vec(),
            open.to_vec(),
            high.to_vec(),
            low.to_vec(),
            close.to_vec(),
            volume.to_vec(),
        );
        self.recompute_indicators();
        Ok(())
    }

    /// Allocate candle column buffers of the given length and return their byte offsets
    /// into WASM linear memory. JS can then write directly into these buffers using
    /// TypedArray views over wasm.memory.buffer, eliminating the copy in ingest_columns_fast.
    pub fn alloc_candle_buffer(&mut self, len: u32) -> Result<JsValue, JsValue> {
        let len = len as usize;
        let mut time = Vec::<u32>::with_capacity(len + 256);
        let mut open = Vec::<f64>::with_capacity(len + 256);
        let mut high = Vec::<f64>::with_capacity(len + 256);
        let mut low = Vec::<f64>::with_capacity(len + 256);
        let mut close = Vec::<f64>::with_capacity(len + 256);
        let mut volume = Vec::<f64>::with_capacity(len + 256);

        // SAFETY: We set the length to `len` so the memory is addressable.
        // JS will write all values before finalize_candle_buffer is called.
        unsafe {
            time.set_len(len);
            open.set_len(len);
            high.set_len(len);
            low.set_len(len);
            close.set_len(len);
            volume.set_len(len);
        }

        let out = Object::new();
        js_set(&out, "time_ptr", JsValue::from_f64(time.as_ptr() as u32 as f64))?;
        js_set(&out, "open_ptr", JsValue::from_f64(open.as_ptr() as u32 as f64))?;
        js_set(&out, "high_ptr", JsValue::from_f64(high.as_ptr() as u32 as f64))?;
        js_set(&out, "low_ptr", JsValue::from_f64(low.as_ptr() as u32 as f64))?;
        js_set(&out, "close_ptr", JsValue::from_f64(close.as_ptr() as u32 as f64))?;
        js_set(&out, "volume_ptr", JsValue::from_f64(volume.as_ptr() as u32 as f64))?;
        js_set(&out, "len", JsValue::from_f64(len as f64))?;

        self.bars = CandleStore { time, open, high, low, close, volume };
        Ok(out.into())
    }

    /// Finalize a previously allocated candle buffer. Call this after writing data
    /// via the pointers returned by alloc_candle_buffer. Triggers indicator recompute.
    pub fn finalize_candle_buffer(&mut self) {
        self.recompute_indicators();
    }

    pub fn upsert_bar(&mut self, bar: JsValue) -> Result<(), JsValue> {
        let bar: Bar = serde_wasm_bindgen::from_value(bar)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.upsert_bar_inner(bar)
    }

    pub fn upsert_bar_fast(
        &mut self,
        time: u32,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<(), JsValue> {
        self.upsert_bar_inner(Bar {
            time,
            open,
            high,
            low,
            close,
            volume,
        })
    }

    fn upsert_bar_inner(&mut self, bar: Bar) -> Result<(), JsValue> {
        if upsert_candle_store(&mut self.bars, bar) && !self.update_indicators_incremental() {
            return Err(JsValue::from_str(
                "indicator does not support incremental updates",
            ));
        }
        Ok(())
    }

    pub fn add_indicator_config(&mut self, config: JsValue) -> Result<u32, JsValue> {
        let config: IndicatorConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let id = self.add_indicator_from_config(config)?;
        self.recompute_indicators();
        Ok(id)
    }

    pub fn add_indicator_configs(&mut self, configs: JsValue) -> Result<JsValue, JsValue> {
        let configs: Vec<IndicatorConfig> = serde_wasm_bindgen::from_value(configs)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        let mut ids = Vec::with_capacity(configs.len());
        for config in configs {
            ids.push(self.add_indicator_from_config(config)?);
        }
        if !ids.is_empty() {
            self.recompute_indicators();
        }
        serde_wasm_bindgen::to_value(&ids).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn indicator_descriptors(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&indicator_descriptors())
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    fn add_indicator_from_config(&mut self, config: IndicatorConfig) -> Result<u32, JsValue> {
        let kind = config.kind.to_uppercase();
        if kind != "SMA"
            && kind != "EMA"
            && kind != "RSI"
            && kind != "STOCH_RSI"
            && kind != "CCI"
            && kind != "MACD"
            && kind != "PPO"
            && kind != "BB"
            && kind != "OBV"
            && kind != "ATR"
            && kind != "ADX"
            && kind != "SUPERTREND"
            && kind != "KELTNER"
            && kind != "DONCHIAN"
            && kind != "PARABOLIC_SAR"
            && kind != "ICHIMOKU"
            && kind != "PIVOT_POINTS"
            && kind != "ROC"
            && kind != "AROON"
            && kind != "CMF"
            && kind != "ADL"
            && kind != "WMA"
            && kind != "HMA"
            && kind != "LINEAR_REGRESSION"
            && kind != "DEMA"
            && kind != "TEMA"
            && kind != "TRIMA"
            && kind != "STDDEV"
            && kind != "ENVELOPE"
            && kind != "TRIX"
            && kind != "TSI"
            && kind != "KST"
            && kind != "BOP"
            && kind != "DPO"
            && kind != "MOMENTUM"
            && kind != "ULTIMATE_OSCILLATOR"
            && kind != "CHAIKIN_OSCILLATOR"
            && kind != "FORCE_INDEX"
            && kind != "VWMA"
            && kind != "WILLIAMS_AD"
            && kind != "CHAIKIN_VOLATILITY"
            && kind != "PRICE_CHANNEL"
            && kind != "STARC"
            && kind != "VWAP"
            && kind != "STOCHASTIC"
            && kind != "WILLIAMS_R"
            && kind != "MFI"
        {
            return Err(JsValue::from_str(
                "indicator kind must be SMA, EMA, RSI, STOCH_RSI, CCI, MACD, PPO, BB, OBV, ATR, ADX, SUPERTREND, KELTNER, DONCHIAN, PARABOLIC_SAR, ICHIMOKU, PIVOT_POINTS, ROC, AROON, CMF, ADL, WMA, HMA, LINEAR_REGRESSION, DEMA, TEMA, TRIMA, STDDEV, ENVELOPE, TRIX, TSI, KST, BOP, DPO, MOMENTUM, ULTIMATE_OSCILLATOR, CHAIKIN_OSCILLATOR, FORCE_INDEX, VWMA, WILLIAMS_AD, CHAIKIN_VOLATILITY, PRICE_CHANNEL, STARC, VWAP, STOCHASTIC, WILLIAMS_R, or MFI",
            ));
        }
        let macd = if kind == "MACD" || kind == "PPO" || kind == "CHAIKIN_OSCILLATOR" {
            Some(MacdParams {
                fast: config
                    .fast
                    .unwrap_or(if kind == "CHAIKIN_OSCILLATOR" { 3 } else { 12 }),
                slow: config
                    .slow
                    .unwrap_or(if kind == "CHAIKIN_OSCILLATOR" { 10 } else { 26 }),
                signal: config.signal.unwrap_or(9),
            })
        } else {
            None
        };
        let period = config.period.unwrap_or(0);
        let stoch_period = config.stoch_period.unwrap_or(period);
        let smooth = config.smooth.unwrap_or(3);
        let signal = config.signal.unwrap_or(3);
        let tenkan_period = config.tenkan_period.unwrap_or(9);
        let kijun_period = config.kijun_period.unwrap_or(26);
        let senkou_b_period = config.senkou_b_period.unwrap_or(52);
        let multiplier = config.multiplier.unwrap_or(2.0);
        let psar_step = config.psar_step.unwrap_or(0.02);
        let psar_max_step = config.psar_max_step.unwrap_or(0.2);
        validate_indicator(
            &kind,
            period,
            stoch_period,
            smooth,
            signal,
            tenkan_period,
            kijun_period,
            senkou_b_period,
            macd,
            multiplier,
            psar_step,
            psar_max_step,
        )?;

        let id = self.next_indicator_id;
        self.next_indicator_id += 1;
        let indicator = Indicator {
            id,
            kind,
            period,
            stoch_period,
            smooth,
            signal,
            tenkan_period,
            kijun_period,
            senkou_b_period,
            macd,
            multiplier,
            psar_step,
            psar_max_step,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };
        self.indicators.push(indicator);
        Ok(id)
    }

    pub fn remove_indicator(&mut self, id: u32) -> bool {
        let old_len = self.indicators.len();
        self.indicators.retain(|indicator| indicator.id != id);
        let removed = self.indicators.len() != old_len;
        if removed {
            self.recompute_indicators();
        }
        removed
    }

    pub fn candles(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.bars.to_bars())
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn candle_columns_fast(&self) -> Result<JsValue, JsValue> {
        let out = Object::new();
        js_set(&out, "time", unsafe { Uint32Array::view(&self.bars.time) })?;
        js_set(&out, "open", unsafe { Float64Array::view(&self.bars.open) })?;
        js_set(&out, "high", unsafe { Float64Array::view(&self.bars.high) })?;
        js_set(&out, "low", unsafe { Float64Array::view(&self.bars.low) })?;
        js_set(&out, "close", unsafe { Float64Array::view(&self.bars.close) })?;
        js_set(&out, "volume", unsafe { Float64Array::view(&self.bars.volume) })?;
        Ok(out.into())
    }

    pub fn indicator_outputs_all(&self, id: u32) -> Result<JsValue, JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let outputs: Vec<_> = indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .map(|(name, values)| IndicatorOutput { name: name.to_string(), values: values.to_vec() })
            .collect();
        serde_wasm_bindgen::to_value(&outputs).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn indicator_output_values_fast(&self, id: u32) -> Result<JsValue, JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let outputs = Array::new();
        for (name, values) in indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
        {
            let item = Object::new();
            js_set(&item, "name", JsValue::from_str(name))?;
            js_set(
                &item,
                "values",
                unsafe { Float64Array::view(values) },
            )?;
            outputs.push(&item);
        }
        Ok(outputs.into())
    }

    pub fn latest_indicator_values(&self, id: u32) -> Result<JsValue, JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let points: Vec<_> = indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .map(|(name, values)| IndicatorLatestValue {
                output: name.to_string(),
                value: values.last().copied().and_then(nan_to_none),
            })
            .collect();
        serde_wasm_bindgen::to_value(&points).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn latest_indicator_values_fast(&mut self, id: u32) -> Result<JsValue, JsValue> {
        let values = self.latest_indicator_values_slice(id)?;
        // ponytail: ephemeral wasm-memory view for the live hot path; switch to a stable shared buffer API if callers need to retain it.
        Ok(unsafe { Float64Array::view(values) }.into())
    }

    pub fn dag_debug(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.dag).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    fn recompute_indicators(&mut self) {
        // ponytail: full recompute is enough for snapshot testing; switch to incremental state for live streams.
        let mut nodes = HashMap::new();
        let mut bars_snapshot = None;
        let mut dag = DagDebug {
            nodes: vec![
                "close".to_string(),
                "high".to_string(),
                "low".to_string(),
                "volume".to_string(),
            ],
            edges: Vec::new(),
        };
        for indicator in &mut self.indicators {
            indicator.outputs = IndicatorArena::from_outputs(compute_indicator_store(
                &self.bars,
                &indicator.kind,
                indicator.period,
                indicator.stoch_period,
                indicator.smooth,
                indicator.signal,
                indicator.tenkan_period,
                indicator.kijun_period,
                indicator.senkou_b_period,
                indicator.macd,
                indicator.multiplier,
                indicator.psar_step,
                indicator.psar_max_step,
                &mut nodes,
                &mut bars_snapshot,
            ));
            let indicator_node = indicator_node(indicator);
            dag.nodes.push(indicator_node.clone());
            for node in indicator_nodes(indicator) {
                if !dag.nodes.contains(&node) {
                    dag.nodes.push(node.clone());
                }
            }
            dag.edges
                .extend(indicator_edges(indicator, &indicator_node));
        }
        self.dag = dag;
    }

    fn update_indicators_incremental(&mut self) -> bool {
        if !self
            .indicators
            .iter()
            .all(|indicator| supports_incremental(indicator.kind.as_str()))
        {
            return false;
        }
        let target_len = self.bars.len();
        for indicator in &mut self.indicators {
            match indicator.kind.as_str() {
                "SMA" => upsert_output(
                    &mut indicator.outputs,
                    "value",
                    target_len,
                    latest_sma_store(&self.bars, indicator.period),
                ),
                "EMA" => {
                    let value =
                        latest_ema_store(&self.bars, indicator.period, indicator.outputs.get("value"));
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "RSI" => {
                    let (value, avg_gain, avg_loss) =
                        latest_rsi_store(&self.bars, indicator.period, &indicator.outputs);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                    upsert_output(&mut indicator.outputs, "avg_gain", target_len, avg_gain);
                    upsert_output(&mut indicator.outputs, "avg_loss", target_len, avg_loss);
                }
                "ROC" => {
                    let value = latest_roc_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "CCI" => {
                    let value = latest_cci_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "WILLIAMS_R" => {
                    let value = latest_williams_r_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "MFI" => {
                    let value = latest_mfi_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "CMF" => {
                    let value = latest_cmf_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "STOCH_RSI" => {
                    let (k, d) = latest_stoch_rsi_store(
                        &self.bars,
                        indicator.period,
                        indicator.stoch_period,
                        indicator.smooth,
                        indicator.signal,
                    );
                    upsert_output(&mut indicator.outputs, "k", target_len, k);
                    upsert_output(&mut indicator.outputs, "d", target_len, d);
                }
                "OBV" => {
                    let value = latest_obv_store(&self.bars, indicator.outputs.get("value"));
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "ATR" => {
                    let value =
                        latest_atr_store(&self.bars, indicator.period, indicator.outputs.get("value"));
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "ADX" => {
                    let (value, plus_di, minus_di, tr_avg, plus_dm_avg, minus_dm_avg, dx) =
                        latest_adx_store(&self.bars, indicator.period, &indicator.outputs);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                    upsert_output(&mut indicator.outputs, "plus_di", target_len, plus_di);
                    upsert_output(&mut indicator.outputs, "minus_di", target_len, minus_di);
                    upsert_output(&mut indicator.outputs, "tr_avg", target_len, tr_avg);
                    upsert_output(
                        &mut indicator.outputs,
                        "plus_dm_avg",
                        target_len,
                        plus_dm_avg,
                    );
                    upsert_output(
                        &mut indicator.outputs,
                        "minus_dm_avg",
                        target_len,
                        minus_dm_avg,
                    );
                    upsert_output(&mut indicator.outputs, "dx", target_len, dx);
                }
                "SUPERTREND" => {
                    let (value, upper_band, lower_band, trend) = latest_supertrend_store(
                        &self.bars,
                        indicator.period,
                        indicator.multiplier,
                        &indicator.outputs,
                    );
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                    upsert_output(&mut indicator.outputs, "upper_band", target_len, upper_band);
                    upsert_output(&mut indicator.outputs, "lower_band", target_len, lower_band);
                    upsert_output(&mut indicator.outputs, "trend", target_len, trend);
                }
                "KELTNER" => {
                    let (upper, middle, lower) = latest_keltner_store(
                        &self.bars,
                        indicator.period,
                        indicator.multiplier,
                        &indicator.outputs,
                    );
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "DONCHIAN" => {
                    let (upper, middle, lower) =
                        latest_donchian_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "PARABOLIC_SAR" => {
                    let (value, ep, af, trend) = latest_parabolic_sar_store(
                        &self.bars,
                        indicator.psar_step,
                        indicator.psar_max_step,
                        &indicator.outputs,
                    );
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                    upsert_output(&mut indicator.outputs, "ep", target_len, ep);
                    upsert_output(&mut indicator.outputs, "af", target_len, af);
                    upsert_output(&mut indicator.outputs, "trend", target_len, trend);
                }
                "ICHIMOKU" => {
                    let (tenkan, kijun, senkou_a, senkou_b, chikou) = latest_ichimoku_store(
                        &self.bars,
                        indicator.tenkan_period,
                        indicator.kijun_period,
                        indicator.senkou_b_period,
                    );
                    upsert_output(&mut indicator.outputs, "tenkan", target_len, tenkan);
                    upsert_output(&mut indicator.outputs, "kijun", target_len, kijun);
                    upsert_output(&mut indicator.outputs, "senkou_a", target_len, senkou_a);
                    upsert_output(&mut indicator.outputs, "senkou_b", target_len, senkou_b);
                    upsert_output(&mut indicator.outputs, "chikou", target_len, chikou);
                }
                "PIVOT_POINTS" => {
                    let (pp, r1, s1, r2, s2) = latest_pivot_points_store(&self.bars);
                    upsert_output(&mut indicator.outputs, "pp", target_len, pp);
                    upsert_output(&mut indicator.outputs, "r1", target_len, r1);
                    upsert_output(&mut indicator.outputs, "s1", target_len, s1);
                    upsert_output(&mut indicator.outputs, "r2", target_len, r2);
                    upsert_output(&mut indicator.outputs, "s2", target_len, s2);
                }
                "AROON" => {
                    let (up, down, oscillator) = latest_aroon_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "up", target_len, up);
                    upsert_output(&mut indicator.outputs, "down", target_len, down);
                    upsert_output(&mut indicator.outputs, "oscillator", target_len, oscillator);
                }
                "ADL" => {
                    let value = latest_adl_store(&self.bars, indicator.outputs.get("value"));
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "WMA" => {
                    let value = latest_wma_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "HMA" => {
                    let value = latest_hma_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "LINEAR_REGRESSION" => {
                    let value = latest_linear_regression_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "DEMA" => {
                    let value = latest_dema_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "TEMA" => {
                    let value = latest_tema_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "TRIMA" => {
                    let value = latest_trima_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "STDDEV" => {
                    let value = latest_stddev_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "ENVELOPE" => {
                    let (upper, middle, lower) =
                        latest_envelope_store(&self.bars, indicator.period, indicator.multiplier);
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "TRIX" => {
                    let value = latest_trix_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "TSI" => {
                    let value =
                        latest_tsi_store(&self.bars, indicator.period, indicator.stoch_period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "KST" => {
                    let value = latest_kst_store(&self.bars);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "BOP" => {
                    let value = latest_bop_store(&self.bars);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "DPO" => {
                    let value = latest_dpo_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "MOMENTUM" => {
                    let value = latest_momentum_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "ULTIMATE_OSCILLATOR" => {
                    let value = latest_ultimate_oscillator_store(
                        &self.bars,
                        indicator.period,
                        indicator.stoch_period,
                        indicator.smooth,
                    );
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "CHAIKIN_OSCILLATOR" => {
                    let params = indicator.macd.unwrap_or(MacdParams {
                        fast: 3,
                        slow: 10,
                        signal: 9,
                    });
                    let value = latest_chaikin_oscillator_store(&self.bars, params);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "FORCE_INDEX" => {
                    let value = latest_force_index_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "VWMA" => {
                    let value = latest_vwma_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "WILLIAMS_AD" => {
                    let value = latest_williams_ad_store(&self.bars, indicator.outputs.get("value"));
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "CHAIKIN_VOLATILITY" => {
                    let value = latest_chaikin_volatility_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                }
                "PRICE_CHANNEL" => {
                    let (upper, middle, lower) =
                        latest_price_channel_store(&self.bars, indicator.period);
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "STARC" => {
                    let (upper, middle, lower) =
                        latest_starc_store(&self.bars, indicator.period, indicator.multiplier);
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "VWAP" => {
                    let (value, cumulative_pv, cumulative_volume) =
                        latest_vwap_store(&self.bars, &indicator.outputs);
                    upsert_output(&mut indicator.outputs, "value", target_len, value);
                    upsert_output(
                        &mut indicator.outputs,
                        "cumulative_pv",
                        target_len,
                        cumulative_pv,
                    );
                    upsert_output(
                        &mut indicator.outputs,
                        "cumulative_volume",
                        target_len,
                        cumulative_volume,
                    );
                }
                "BB" => {
                    let (upper, middle, lower) =
                        latest_bollinger_store(&self.bars, indicator.period, indicator.multiplier);
                    upsert_output(&mut indicator.outputs, "upper", target_len, upper);
                    upsert_output(&mut indicator.outputs, "middle", target_len, middle);
                    upsert_output(&mut indicator.outputs, "lower", target_len, lower);
                }
                "STOCHASTIC" => {
                    let (k, d) = latest_stochastic_store(
                        &self.bars,
                        indicator.period,
                        indicator.smooth,
                        &indicator.outputs,
                    );
                    upsert_output(&mut indicator.outputs, "k", target_len, k);
                    upsert_output(&mut indicator.outputs, "d", target_len, d);
                }
                "MACD" => {
                    let macd = indicator.macd.unwrap_or(MacdParams {
                        fast: 12,
                        slow: 26,
                        signal: 9,
                    });
                    let (macd_line, signal, histogram, fast_ema, slow_ema) =
                        latest_macd_store(&self.bars, macd, &indicator.outputs);
                    upsert_output(&mut indicator.outputs, "macd", target_len, macd_line);
                    upsert_output(&mut indicator.outputs, "signal", target_len, signal);
                    upsert_output(&mut indicator.outputs, "histogram", target_len, histogram);
                    upsert_output(&mut indicator.outputs, "fast_ema", target_len, fast_ema);
                    upsert_output(&mut indicator.outputs, "slow_ema", target_len, slow_ema);
                }
                "PPO" => {
                    let params = indicator.macd.unwrap_or(MacdParams {
                        fast: 12,
                        slow: 26,
                        signal: 9,
                    });
                    let (ppo, signal, histogram) =
                        latest_ppo_store(&self.bars, params, &indicator.outputs);
                    upsert_output(&mut indicator.outputs, "ppo", target_len, ppo);
                    upsert_output(&mut indicator.outputs, "signal", target_len, signal);
                    upsert_output(&mut indicator.outputs, "histogram", target_len, histogram);
                }
                _ => return false,
            }
        }
        true
    }

    fn latest_indicator_values_slice(&mut self, id: u32) -> Result<&[f64], JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        self.latest_values_scratch.clear();
        self.latest_values_scratch.extend(indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .map(|(_, values)| values.last().copied().unwrap_or(f64::NAN))
        );
        Ok(self.latest_values_scratch.as_slice())
    }

    fn populate_indicator_output_values_scratch(&mut self, id: u32) -> Result<(), JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let visible_count = indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .count();
        if self.indicator_values_scratch.len() < visible_count {
            self.indicator_values_scratch
                .resize_with(visible_count, Vec::new);
        }
        for (index, (_, values)) in indicator.outputs.iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .enumerate()
        {
            let scratch = &mut self.indicator_values_scratch[index];
            scratch.clear();
            scratch.extend(values.iter().copied());
        }
        Ok(())
    }
}

fn supports_incremental(kind: &str) -> bool {
    matches!(
        kind,
        "SMA"
            | "EMA"
            | "RSI"
            | "STOCH_RSI"
            | "CCI"
            | "OBV"
            | "BB"
            | "MACD"
            | "ATR"
            | "ADX"
            | "SUPERTREND"
            | "KELTNER"
            | "DONCHIAN"
            | "PARABOLIC_SAR"
            | "ICHIMOKU"
            | "PIVOT_POINTS"
            | "ROC"
            | "AROON"
            | "CMF"
            | "ADL"
            | "WMA"
            | "HMA"
            | "LINEAR_REGRESSION"
            | "TRIX"
            | "DEMA"
            | "TEMA"
            | "TRIMA"
            | "STDDEV"
            | "ENVELOPE"
            | "TSI"
            | "KST"
            | "BOP"
            | "DPO"
            | "MOMENTUM"
            | "ULTIMATE_OSCILLATOR"
            | "CHAIKIN_OSCILLATOR"
            | "FORCE_INDEX"
            | "VWMA"
            | "WILLIAMS_AD"
            | "CHAIKIN_VOLATILITY"
            | "PRICE_CHANNEL"
            | "STARC"
            | "VWAP"
            | "STOCHASTIC"
            | "WILLIAMS_R"
            | "MFI"
            | "PPO"
    )
}

fn is_visible_output(name: &str) -> bool {
    !matches!(
        name,
        "fast_ema"
            | "slow_ema"
            | "avg_gain"
            | "avg_loss"
            | "tr_avg"
            | "plus_dm_avg"
            | "minus_dm_avg"
            | "dx"
            | "upper_band"
            | "lower_band"
            | "trend"
            | "ep"
            | "af"
            | "cumulative_pv"
            | "cumulative_volume"
    )
}

type Series = Vec<f64>;
type RcSeries = Rc<Series>;
type NodeCache = HashMap<String, RcSeries>;

/// Unwrap an Rc<Vec<f64>> into owned Vec<f64>. Zero-cost if sole owner, clones if shared.
#[inline]
fn rc_into_owned(rc: RcSeries) -> Series {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

#[inline(always)]
fn nan_to_none(v: f64) -> Option<f64> {
    if v.is_nan() { None } else { Some(v) }
}

fn js_set(target: &Object, key: &str, value: impl Into<JsValue>) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(key), &value.into())
        .map(|_| ())
        .map_err(|err| err)
}

fn indicator_descriptors() -> Vec<IndicatorDescriptor> {
    vec![
        period_descriptor("SMA", "SMA", "overlay", 20),
        period_descriptor("EMA", "EMA", "overlay", 20),
        period_descriptor("WMA", "WMA", "overlay", 20),
        period_descriptor("HMA", "HMA", "overlay", 20),
        period_descriptor("LINEAR_REGRESSION", "LINEAR REGRESSION", "overlay", 20),
        period_descriptor("DEMA", "DEMA", "overlay", 20),
        period_descriptor("TEMA", "TEMA", "overlay", 20),
        period_descriptor("TRIMA", "TRIMA", "overlay", 20),
        period_descriptor("STDDEV", "STDDEV", "separate", 20),
        period_descriptor("TRIX", "TRIX", "separate", 15),
        IndicatorDescriptor {
            kind: "ENVELOPE",
            name: "ENVELOPE",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier %",
                    default: 2.0,
                    min: 0.1,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#2563eb"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "TSI",
            name: "TSI",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Long",
                    default: 25.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Short",
                    default: 13.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        period_descriptor("DPO", "DPO", "separate", 20),
        period_descriptor("MOMENTUM", "MOMENTUM", "separate", 10),
        period_descriptor("RSI", "RSI", "separate", 14),
        period_descriptor("ROC", "ROC", "separate", 14),
        period_descriptor("CCI", "CCI", "separate", 20),
        period_descriptor("MFI", "MFI", "separate", 14),
        period_descriptor("CMF", "CMF", "separate", 20),
        period_descriptor("FORCE_INDEX", "FORCE INDEX", "separate", 13),
        period_descriptor("VWMA", "VWMA", "overlay", 20),
        IndicatorDescriptor {
            kind: "WILLIAMS_AD",
            name: "WILLIAMS A/D",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        period_descriptor("CHAIKIN_VOLATILITY", "CHAIKIN VOLATILITY", "separate", 10),
        IndicatorDescriptor {
            kind: "PRICE_CHANNEL",
            name: "PRICE CHANNEL",
            pane: "overlay",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 20.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#f59e0b"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#f59e0b"),
            ],
        },
        IndicatorDescriptor {
            kind: "STARC",
            name: "STARC",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 15.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 0.1,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#0f766e"),
                output_descriptor("middle", "line", "overlay", "#2563eb"),
                output_descriptor("lower", "line", "overlay", "#0f766e"),
            ],
        },
        period_descriptor("WILLIAMS_R", "WILLIAMS %R", "separate", 14),
        IndicatorDescriptor {
            kind: "PARABOLIC_SAR",
            name: "PARABOLIC SAR",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "psar_step",
                    label: "Step",
                    default: 0.02,
                    min: 0.001,
                    step: "0.001",
                },
                ParamDescriptor {
                    name: "psar_max_step",
                    label: "Max",
                    default: 0.2,
                    min: 0.01,
                    step: "0.01",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "overlay", "#059669")],
        },
        IndicatorDescriptor {
            kind: "ICHIMOKU",
            name: "ICHIMOKU",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "tenkan_period",
                    label: "Tenkan",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "kijun_period",
                    label: "Kijun",
                    default: 26.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "senkou_b_period",
                    label: "Senkou B",
                    default: 52.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("tenkan", "line", "overlay", "#2563eb"),
                output_descriptor("kijun", "line", "overlay", "#dc2626"),
                output_descriptor("senkou_a", "line", "overlay", "#059669"),
                output_descriptor("senkou_b", "line", "overlay", "#ea580c"),
                output_descriptor("chikou", "line", "overlay", "#64748b"),
            ],
        },
        IndicatorDescriptor {
            kind: "PIVOT_POINTS",
            name: "PIVOT POINTS",
            pane: "overlay",
            params: Vec::new(),
            outputs: vec![
                output_descriptor("pp", "line", "overlay", "#64748b"),
                output_descriptor("r1", "line", "overlay", "#059669"),
                output_descriptor("s1", "line", "overlay", "#dc2626"),
                output_descriptor("r2", "line", "overlay", "#16a34a"),
                output_descriptor("s2", "line", "overlay", "#b91c1c"),
            ],
        },
        IndicatorDescriptor {
            kind: "AROON",
            name: "AROON",
            pane: "separate",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("up", "line", "separate", "#059669"),
                output_descriptor("down", "line", "separate", "#dc2626"),
                output_descriptor("oscillator", "line", "separate", "#2563eb"),
            ],
        },
        IndicatorDescriptor {
            kind: "ADL",
            name: "ADL",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        IndicatorDescriptor {
            kind: "KST",
            name: "KST",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        IndicatorDescriptor {
            kind: "BOP",
            name: "BOP",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
        IndicatorDescriptor {
            kind: "ULTIMATE_OSCILLATOR",
            name: "ULTIMATE OSCILLATOR",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Short",
                    default: 7.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Medium",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "Long",
                    default: 28.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#2563eb")],
        },
        IndicatorDescriptor {
            kind: "SUPERTREND",
            name: "SUPERTREND",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 10.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 3.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "overlay", "#0f766e")],
        },
        IndicatorDescriptor {
            kind: "KELTNER",
            name: "KELTNER",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#0f766e"),
                output_descriptor("middle", "line", "overlay", "#2563eb"),
                output_descriptor("lower", "line", "overlay", "#0f766e"),
            ],
        },
        IndicatorDescriptor {
            kind: "DONCHIAN",
            name: "DONCHIAN",
            pane: "overlay",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 20.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#f59e0b"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#f59e0b"),
            ],
        },
        IndicatorDescriptor {
            kind: "STOCH_RSI",
            name: "STOCH RSI",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "stoch_period",
                    label: "Stoch",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "%K",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "%D",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("k", "line", "separate", "#2563eb"),
                output_descriptor("d", "line", "separate", "#dc2626"),
            ],
        },
        period_descriptor("ATR", "ATR", "separate", 14),
        IndicatorDescriptor {
            kind: "ADX",
            name: "ADX",
            pane: "separate",
            params: vec![ParamDescriptor {
                name: "period",
                label: "Period",
                default: 14.0,
                min: 1.0,
                step: "1",
            }],
            outputs: vec![
                output_descriptor("value", "line", "separate", "#2563eb"),
                output_descriptor("plus_di", "line", "separate", "#059669"),
                output_descriptor("minus_di", "line", "separate", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "VWAP",
            name: "VWAP",
            pane: "overlay",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "overlay", "#0f766e")],
        },
        IndicatorDescriptor {
            kind: "STOCHASTIC",
            name: "STOCHASTIC",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 14.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "smooth",
                    label: "Smooth",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("k", "line", "separate", "#2563eb"),
                output_descriptor("d", "line", "separate", "#dc2626"),
            ],
        },
        IndicatorDescriptor {
            kind: "OBV",
            name: "OBV",
            pane: "separate",
            params: Vec::new(),
            outputs: vec![output_descriptor("value", "line", "separate", "#059669")],
        },
        IndicatorDescriptor {
            kind: "BB",
            name: "BOLLINGER",
            pane: "overlay",
            params: vec![
                ParamDescriptor {
                    name: "period",
                    label: "Period",
                    default: 20.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "multiplier",
                    label: "Multiplier",
                    default: 2.0,
                    min: 1.0,
                    step: "0.1",
                },
            ],
            outputs: vec![
                output_descriptor("upper", "line", "overlay", "#9333ea"),
                output_descriptor("middle", "line", "overlay", "#64748b"),
                output_descriptor("lower", "line", "overlay", "#9333ea"),
            ],
        },
        IndicatorDescriptor {
            kind: "MACD",
            name: "MACD",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 12.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 26.0,
                    min: 2.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "Signal",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("macd", "line", "separate", "#2563eb"),
                output_descriptor("signal", "line", "separate", "#dc2626"),
                output_descriptor("histogram", "histogram", "separate", "#86efac"),
            ],
        },
        IndicatorDescriptor {
            kind: "PPO",
            name: "PPO",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 12.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 26.0,
                    min: 2.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "signal",
                    label: "Signal",
                    default: 9.0,
                    min: 1.0,
                    step: "1",
                },
            ],
            outputs: vec![
                output_descriptor("ppo", "line", "separate", "#2563eb"),
                output_descriptor("signal", "line", "separate", "#dc2626"),
                output_descriptor("histogram", "histogram", "separate", "#86efac"),
            ],
        },
        IndicatorDescriptor {
            kind: "CHAIKIN_OSCILLATOR",
            name: "CHAIKIN OSCILLATOR",
            pane: "separate",
            params: vec![
                ParamDescriptor {
                    name: "fast",
                    label: "Fast",
                    default: 3.0,
                    min: 1.0,
                    step: "1",
                },
                ParamDescriptor {
                    name: "slow",
                    label: "Slow",
                    default: 10.0,
                    min: 2.0,
                    step: "1",
                },
            ],
            outputs: vec![output_descriptor("value", "line", "separate", "#9333ea")],
        },
    ]
}

fn period_descriptor(
    kind: &'static str,
    name: &'static str,
    pane: &'static str,
    default: usize,
) -> IndicatorDescriptor {
    IndicatorDescriptor {
        kind,
        name,
        pane,
        params: vec![ParamDescriptor {
            name: "period",
            label: "Period",
            default: default as f64,
            min: 1.0,
            step: "1",
        }],
        outputs: vec![output_descriptor("value", "line", pane, "#2563eb")],
    }
}

fn output_descriptor(
    name: &'static str,
    renderer: &'static str,
    pane: &'static str,
    color: &'static str,
) -> OutputDescriptor {
    OutputDescriptor {
        name,
        renderer,
        pane,
        color,
    }
}

fn materialized_bars<'a>(store: &CandleStore, snapshot: &'a mut Option<Vec<Bar>>) -> &'a [Bar] {
    if snapshot.is_none() {
        *snapshot = Some(store.to_bars());
    }
    snapshot.as_deref().expect("bars snapshot initialized")
}

fn compute_indicator(
    bars: &[Bar],
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd_params: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    match kind {
        "SMA" => one_output(sma_close(bars, period, nodes)),
        "EMA" => one_output(ema_close(bars, period, nodes)),
        "WMA" => one_output(wma_close(bars, period, nodes)),
        "HMA" => one_output(hma(bars, period, nodes)),
        "LINEAR_REGRESSION" => one_output(linear_regression_node(bars, period, nodes)),
        "TRIX" => one_output(trix_node(bars, period, nodes)),
        "TSI" => one_output(tsi_node(bars, period, stoch_period, nodes)),
        "DPO" => one_output(dpo_node(bars, period, nodes)),
        "MOMENTUM" => one_output(momentum_node(bars, period, nodes)),
        "RSI" => rsi_outputs(bars, period),
        "ROC" => one_output(roc_node(bars, period, nodes)),
        "CCI" => one_output(cci_node(bars, period, nodes)),
        "MFI" => one_output(mfi_node(bars, period, nodes)),
        "CMF" => one_output(cmf_node(bars, period, nodes)),
        "WILLIAMS_R" => one_output(williams_r_node(bars, period, nodes)),
        "PARABOLIC_SAR" => parabolic_sar(bars, psar_step, psar_max_step, nodes),
        "ICHIMOKU" => ichimoku(bars, tenkan_period, kijun_period, senkou_b_period, nodes),
        "PIVOT_POINTS" => pivot_points(bars, nodes),
        "AROON" => aroon(bars, period, nodes),
        "ADL" => one_output(adl_node(bars, nodes)),
        "KST" => one_output(kst_node(bars, nodes)),
        "BOP" => one_output(bop_node(bars, nodes)),
        "ULTIMATE_OSCILLATOR" => one_output(ultimate_oscillator_node(
            bars,
            period,
            stoch_period,
            smooth,
            nodes,
        )),
        "CHAIKIN_OSCILLATOR" => one_output(chaikin_oscillator_node(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )),
        "FORCE_INDEX" => one_output(force_index_node(bars, period, nodes)),
        "VWMA" => one_output(vwma_node(bars, period, nodes)),
        "WILLIAMS_AD" => one_output(williams_ad_node(bars, nodes)),
        "CHAIKIN_VOLATILITY" => one_output(chaikin_volatility_node(bars, period, nodes)),
        "PRICE_CHANNEL" => price_channel(bars, period, nodes),
        "STARC" => starc(bars, period, multiplier, nodes),
        "SUPERTREND" => supertrend(bars, period, multiplier, nodes),
        "KELTNER" => keltner(bars, period, multiplier, nodes),
        "DONCHIAN" => donchian(bars, period, nodes),
        "STOCH_RSI" => stoch_rsi(bars, period, stoch_period, smooth, signal, nodes),
        "OBV" => one_output(obv_node(bars, nodes)),
        "ATR" => one_output(atr_node(bars, period, nodes)),
        "ADX" => adx(bars, period, nodes),
        "VWAP" => vwap(bars, nodes),
        "STOCHASTIC" => stochastic(bars, period, smooth, nodes),
        "BB" => bollinger(bars, period, multiplier, nodes),
        "MACD" => macd(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        "PPO" => ppo(
            bars,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        _ => Vec::new(),
    }
}

fn one_output(values: Vec<f64>) -> Vec<IndicatorOutput> {
    vec![IndicatorOutput {
        name: "value".to_string(),
        values,
    }]
}

fn rc_one_output(rc: RcSeries) -> Vec<IndicatorOutput> {
    vec![IndicatorOutput {
        name: "value".to_string(),
        values: rc_into_owned(rc),
    }]
}

fn compute_indicator_store(
    store: &CandleStore,
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd_params: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    nodes: &mut NodeCache,
    bars_snapshot: &mut Option<Vec<Bar>>,
) -> Vec<IndicatorOutput> {
    match kind {
        "SMA" => one_output(rc_into_owned(sma_close_store(store, period, nodes))),
        "EMA" => one_output(rc_into_owned(ema_close_store(store, period, nodes))),
        "RSI" => rsi_outputs_store(store, period, nodes),
        "ROC" => rc_one_output(roc_store(store, period, nodes)),
        "CCI" => rc_one_output(cci_store(store, period, nodes)),
        "MFI" => rc_one_output(mfi_store(store, period, nodes)),
        "CMF" => rc_one_output(cmf_store(store, period, nodes)),
        "WILLIAMS_R" => rc_one_output(williams_r_store(store, period, nodes)),
        "OBV" => one_output(rc_into_owned(obv_store(store, nodes))),
        "ADL" => one_output(rc_into_owned(adl_store(store, nodes))),
        "VWAP" => vwap_store(store, nodes),
        "VWMA" => rc_one_output(vwma_store(store, period, nodes)),
        "WILLIAMS_AD" => rc_one_output(williams_ad_store(store, nodes)),
        "ATR" => one_output(rc_into_owned(atr_store(store, period, nodes))),
        "ADX" => adx_store(store, period, nodes),
        "SUPERTREND" => supertrend_store(store, period, multiplier, nodes),
        "KELTNER" => keltner_store(store, period, multiplier, nodes),
        "STARC" => starc_store(store, period, multiplier, nodes),
        "WMA" => one_output(rc_into_owned(wma_store(store, period, nodes))),
        "HMA" => one_output(rc_into_owned(hma_store(store, period, nodes))),
        "LINEAR_REGRESSION" => rc_one_output(linear_regression_store(store, period, nodes)),
        "DEMA" => rc_one_output(dema_store(store, period, nodes)),
        "TEMA" => rc_one_output(tema_store(store, period, nodes)),
        "TRIMA" => rc_one_output(trima_store(store, period, nodes)),
        "STDDEV" => one_output(rc_into_owned(stddev_store(store, period, nodes))),
        "ENVELOPE" => envelope_store(store, period, multiplier, nodes),
        "TRIX" => rc_one_output(trix_store(store, period, nodes)),
        "TSI" => rc_one_output(tsi_store(store, period, stoch_period, nodes)),
        "KST" => rc_one_output(kst_store(store, nodes)),
        "BOP" => rc_one_output(bop_store(store, nodes)),
        "MOMENTUM" => rc_one_output(momentum_store(store, period, nodes)),
        "DPO" => rc_one_output(dpo_store(store, period, nodes)),
        "FORCE_INDEX" => rc_one_output(force_index_store(store, period, nodes)),
        "PRICE_CHANNEL" => price_channel_store(store, period, nodes),
        "STOCHASTIC" => stochastic_store(store, period, smooth, nodes),
        "BB" => bollinger_store(store, period, multiplier, nodes),
        "DONCHIAN" => donchian_store(store, period, nodes),
        "PARABOLIC_SAR" => parabolic_sar_store(store, psar_step, psar_max_step, nodes),
        "ICHIMOKU" => ichimoku_store(store, tenkan_period, kijun_period, senkou_b_period, nodes),
        "PIVOT_POINTS" => pivot_points_store(store, nodes),
        "AROON" => aroon_store(store, period, nodes),
        "ULTIMATE_OSCILLATOR" => one_output(ultimate_oscillator_store(
            store,
            period,
            stoch_period,
            smooth,
            nodes,
        )),
        "CHAIKIN_VOLATILITY" => rc_one_output(chaikin_volatility_store(store, period, nodes)),
        "STOCH_RSI" => stoch_rsi_store(store, period, stoch_period, smooth, signal, nodes),
        "CHAIKIN_OSCILLATOR" => one_output(chaikin_oscillator_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )),
        "MACD" => macd_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        "PPO" => ppo_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        ),
        _ => compute_indicator(
            materialized_bars(store, bars_snapshot),
            kind,
            period,
            stoch_period,
            smooth,
            signal,
            tenkan_period,
            kijun_period,
            senkou_b_period,
            macd_params,
            multiplier,
            psar_step,
            psar_max_step,
            nodes,
        ),
    }
}

fn indicator_nodes(indicator: &Indicator) -> Vec<String> {
    match indicator.kind.as_str() {
        "SMA" => vec![format!("sma:close:{}", indicator.period)],
        "EMA" => vec![format!("ema:close:{}", indicator.period)],
        "WMA" => vec![format!("wma:close:{}", indicator.period)],
        "HMA" => vec![
            format!("wma:close:{}", indicator.period / 2),
            format!("wma:close:{}", indicator.period),
            format!("hma:close:{}", indicator.period),
        ],
        "LINEAR_REGRESSION" => vec![format!("linreg:close:{}", indicator.period)],
        "DEMA" => vec![
            format!("ema:close:{}", indicator.period),
            format!("dema:ema2:{}", indicator.period),
            format!("dema:value:{}", indicator.period),
        ],
        "TEMA" => vec![
            format!("ema:close:{}", indicator.period),
            format!("tema:ema2:{}", indicator.period),
            format!("tema:ema3:{}", indicator.period),
            format!("tema:value:{}", indicator.period),
        ],
        "TRIMA" => vec![
            format!("sma:close:{}", indicator.period),
            format!("trima:value:{}", indicator.period),
        ],
        "STDDEV" => vec![format!("stddev:close:{}", indicator.period)],
        "ENVELOPE" => vec![
            format!("sma:close:{}", indicator.period),
            format!(
                "envelope:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "envelope:middle:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "envelope:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        "TRIX" => vec![
            format!("ema:close:{}", indicator.period),
            format!("trix:ema2:{}", indicator.period),
            format!("trix:value:{}", indicator.period),
        ],
        "TSI" => vec![format!(
            "tsi:{}:{}",
            indicator.period, indicator.stoch_period
        )],
        "DPO" => vec![
            format!("sma:close:{}", indicator.period),
            format!("dpo:close:{}", indicator.period),
        ],
        "MOMENTUM" => vec![format!("momentum:close:{}", indicator.period)],
        "RSI" => vec![format!("rsi:close:{}", indicator.period)],
        "ROC" => vec![format!("roc:close:{}", indicator.period)],
        "CCI" => vec![format!("cci:hlc:{}", indicator.period)],
        "CMF" => vec![format!("cmf:hlcv:{}", indicator.period)],
        "MFI" => vec![format!("mfi:hlcv:{}", indicator.period)],
        "WILLIAMS_R" => vec![format!("willr:hlc:{}", indicator.period)],
        "VWMA" => vec![format!("vwma:close:volume:{}", indicator.period)],
        "WILLIAMS_AD" => vec!["wad:ohlc".to_string()],
        "CHAIKIN_VOLATILITY" => vec![
            format!("cvol:ema:{}", indicator.period),
            format!("cvol:value:{}", indicator.period),
        ],
        "PRICE_CHANNEL" => vec![
            format!("price_channel:upper:{}", indicator.period),
            format!("price_channel:middle:{}", indicator.period),
            format!("price_channel:lower:{}", indicator.period),
        ],
        "STARC" => vec![
            format!("sma:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!("starc:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("starc:middle:{}:{}", indicator.period, indicator.multiplier),
            format!("starc:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "PARABOLIC_SAR" => vec![format!(
            "psar:ohlc:{}:{}",
            indicator.psar_step, indicator.psar_max_step
        )],
        "ICHIMOKU" => vec![
            format!("ichimoku:tenkan:{}", indicator.tenkan_period),
            format!("ichimoku:kijun:{}", indicator.kijun_period),
            format!(
                "ichimoku:senkou_a:{}:{}",
                indicator.tenkan_period, indicator.kijun_period
            ),
            format!("ichimoku:senkou_b:{}", indicator.senkou_b_period),
            "ichimoku:chikou".to_string(),
        ],
        "PIVOT_POINTS" => vec![
            "pivot:pp".to_string(),
            "pivot:r1".to_string(),
            "pivot:s1".to_string(),
            "pivot:r2".to_string(),
            "pivot:s2".to_string(),
        ],
        "AROON" => vec![format!("aroon:hl:{}", indicator.period)],
        "ADL" => vec!["adl:hlcv".to_string()],
        "KST" => vec![
            "roc:close:10".to_string(),
            "roc:close:15".to_string(),
            "roc:close:20".to_string(),
            "roc:close:30".to_string(),
            "kst:value".to_string(),
        ],
        "BOP" => vec!["bop:ohlc".to_string()],
        "ULTIMATE_OSCILLATOR" => vec![format!(
            "uo:{}:{}:{}",
            indicator.period, indicator.stoch_period, indicator.smooth
        )],
        "CHAIKIN_OSCILLATOR" => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            });
            vec![
                "adl:hlcv".to_string(),
                format!("chaikin:{}:{}", params.fast, params.slow),
            ]
        }
        "FORCE_INDEX" => vec![format!("force:close:volume:{}", indicator.period)],
        "SUPERTREND" => vec![
            format!("atr:ohlc:{}", indicator.period),
            format!("supertrend:{}:{}", indicator.period, indicator.multiplier),
        ],
        "KELTNER" => vec![
            format!("ema:close:{}", indicator.period),
            format!("atr:ohlc:{}", indicator.period),
            format!(
                "keltner:upper:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "keltner:middle:{}:{}",
                indicator.period, indicator.multiplier
            ),
            format!(
                "keltner:lower:{}:{}",
                indicator.period, indicator.multiplier
            ),
        ],
        "DONCHIAN" => vec![
            format!("donchian:upper:{}", indicator.period),
            format!("donchian:middle:{}", indicator.period),
            format!("donchian:lower:{}", indicator.period),
        ],
        "STOCH_RSI" => vec![
            format!("rsi:close:{}", indicator.period),
            format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            ),
        ],
        "ATR" => vec![format!("atr:ohlc:{}", indicator.period)],
        "ADX" => vec![format!("adx:ohlc:{}", indicator.period)],
        "VWAP" => vec!["vwap:hlcv".to_string()],
        "STOCHASTIC" => vec![format!(
            "stoch:hlc:{}:{}",
            indicator.period, indicator.smooth
        )],
        "BB" => vec![
            format!("sma:close:{}", indicator.period),
            format!("bb:upper:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:middle:{}:{}", indicator.period, indicator.multiplier),
            format!("bb:lower:{}:{}", indicator.period, indicator.multiplier),
        ],
        "OBV" => vec!["obv:close:volume".to_string()],
        "MACD" => {
            let macd = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema:close:{}", macd.fast),
                format!("ema:close:{}", macd.slow),
            ]
        }
        "PPO" => {
            let macd = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            vec![
                format!("ema:close:{}", macd.fast),
                format!("ema:close:{}", macd.slow),
                format!("ppo:{}:{}:{}", macd.fast, macd.slow, macd.signal),
            ]
        }
        _ => vec!["close".to_string()],
    }
}

fn indicator_edges(indicator: &Indicator, indicator_node: &str) -> Vec<DagEdge> {
    match indicator.kind.as_str() {
        "BB" => {
            let sma = format!("sma:close:{}", indicator.period);
            let upper = format!("bb:upper:{}:{}", indicator.period, indicator.multiplier);
            let middle = format!("bb:middle:{}:{}", indicator.period, indicator.multiplier);
            let lower = format!("bb:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &sma),
                edge(&sma, &upper),
                edge(&sma, &middle),
                edge(&sma, &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "OBV" => vec![
            edge("close", "obv:close:volume"),
            edge("volume", "obv:close:volume"),
            edge("obv:close:volume", indicator_node),
        ],
        "ATR" => {
            let atr = format!("atr:ohlc:{}", indicator.period);
            vec![
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&atr, indicator_node),
            ]
        }
        "CCI" => {
            let cci = format!("cci:hlc:{}", indicator.period);
            vec![
                edge("high", &cci),
                edge("low", &cci),
                edge("close", &cci),
                edge(&cci, indicator_node),
            ]
        }
        "VWMA" => {
            let vwma = format!("vwma:close:volume:{}", indicator.period);
            vec![
                edge("close", &vwma),
                edge("volume", &vwma),
                edge(&vwma, indicator_node),
            ]
        }
        "WILLIAMS_AD" => vec![
            edge("high", "wad:ohlc"),
            edge("low", "wad:ohlc"),
            edge("close", "wad:ohlc"),
            edge("wad:ohlc", indicator_node),
        ],
        "CHAIKIN_VOLATILITY" => {
            let ema = format!("cvol:ema:{}", indicator.period);
            let value = format!("cvol:value:{}", indicator.period);
            vec![
                edge("high", &ema),
                edge("low", &ema),
                edge(&ema, &value),
                edge(&value, indicator_node),
            ]
        }
        "PRICE_CHANNEL" => {
            let upper = format!("price_channel:upper:{}", indicator.period);
            let middle = format!("price_channel:middle:{}", indicator.period);
            let lower = format!("price_channel:lower:{}", indicator.period);
            vec![
                edge("high", &upper),
                edge("low", &upper),
                edge("high", &middle),
                edge("low", &middle),
                edge("high", &lower),
                edge("low", &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "STARC" => {
            let sma = format!("sma:close:{}", indicator.period);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let upper = format!("starc:upper:{}:{}", indicator.period, indicator.multiplier);
            let middle = format!("starc:middle:{}:{}", indicator.period, indicator.multiplier);
            let lower = format!("starc:lower:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("close", &sma),
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&sma, &upper),
                edge(&atr, &upper),
                edge(&sma, &middle),
                edge(&sma, &lower),
                edge(&atr, &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "DEMA" => {
            let ema1 = format!("ema:close:{}", indicator.period);
            let ema2 = format!("dema:ema2:{}", indicator.period);
            let dema = format!("dema:value:{}", indicator.period);
            vec![
                edge("close", &ema1),
                edge(&ema1, &ema2),
                edge(&ema2, &dema),
                edge(&dema, indicator_node),
            ]
        }
        "TEMA" => {
            let ema1 = format!("ema:close:{}", indicator.period);
            let ema2 = format!("tema:ema2:{}", indicator.period);
            let ema3 = format!("tema:ema3:{}", indicator.period);
            let tema = format!("tema:value:{}", indicator.period);
            vec![
                edge("close", &ema1),
                edge(&ema1, &ema2),
                edge(&ema2, &ema3),
                edge(&ema3, &tema),
                edge(&tema, indicator_node),
            ]
        }
        "TRIMA" => {
            let sma1 = format!("sma:close:{}", indicator.period);
            let trima = format!("trima:value:{}", indicator.period);
            vec![
                edge("close", &sma1),
                edge(&sma1, &trima),
                edge(&trima, indicator_node),
            ]
        }
        "STDDEV" => {
            let stddev = format!("stddev:close:{}", indicator.period);
            vec![edge("close", &stddev), edge(&stddev, indicator_node)]
        }
        "ENVELOPE" => {
            let sma = format!("sma:close:{}", indicator.period);
            let upper = format!(
                "envelope:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let middle = format!(
                "envelope:middle:{}:{}",
                indicator.period, indicator.multiplier
            );
            let lower = format!(
                "envelope:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &sma),
                edge(&sma, &upper),
                edge(&sma, &middle),
                edge(&sma, &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "WMA" => {
            let wma = format!("wma:close:{}", indicator.period);
            vec![edge("close", &wma), edge(&wma, indicator_node)]
        }
        "HMA" => {
            let half = format!("wma:close:{}", indicator.period / 2);
            let full = format!("wma:close:{}", indicator.period);
            let hma = format!("hma:close:{}", indicator.period);
            vec![
                edge("close", &half),
                edge("close", &full),
                edge(&half, &hma),
                edge(&full, &hma),
                edge(&hma, indicator_node),
            ]
        }
        "LINEAR_REGRESSION" => {
            let linreg = format!("linreg:close:{}", indicator.period);
            vec![edge("close", &linreg), edge(&linreg, indicator_node)]
        }
        "TRIX" => {
            let ema1 = format!("ema:close:{}", indicator.period);
            let ema2 = format!("trix:ema2:{}", indicator.period);
            let trix = format!("trix:value:{}", indicator.period);
            vec![
                edge("close", &ema1),
                edge(&ema1, &ema2),
                edge(&ema2, &trix),
                edge(&trix, indicator_node),
            ]
        }
        "TSI" => {
            let tsi = format!("tsi:{}:{}", indicator.period, indicator.stoch_period);
            vec![edge("close", &tsi), edge(&tsi, indicator_node)]
        }
        "DPO" => {
            let sma = format!("sma:close:{}", indicator.period);
            let dpo = format!("dpo:close:{}", indicator.period);
            vec![
                edge("close", &sma),
                edge("close", &dpo),
                edge(&sma, &dpo),
                edge(&dpo, indicator_node),
            ]
        }
        "MOMENTUM" => {
            let momentum = format!("momentum:close:{}", indicator.period);
            vec![edge("close", &momentum), edge(&momentum, indicator_node)]
        }
        "ROC" => {
            let roc = format!("roc:close:{}", indicator.period);
            vec![edge("close", &roc), edge(&roc, indicator_node)]
        }
        "MFI" => {
            let mfi = format!("mfi:hlcv:{}", indicator.period);
            vec![
                edge("high", &mfi),
                edge("low", &mfi),
                edge("close", &mfi),
                edge("volume", &mfi),
                edge(&mfi, indicator_node),
            ]
        }
        "CMF" => {
            let cmf = format!("cmf:hlcv:{}", indicator.period);
            vec![
                edge("high", &cmf),
                edge("low", &cmf),
                edge("close", &cmf),
                edge("volume", &cmf),
                edge(&cmf, indicator_node),
            ]
        }
        "SUPERTREND" => {
            let atr = format!("atr:ohlc:{}", indicator.period);
            let supertrend = format!("supertrend:{}:{}", indicator.period, indicator.multiplier);
            vec![
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge("high", &supertrend),
                edge("low", &supertrend),
                edge("close", &supertrend),
                edge(&atr, &supertrend),
                edge(&supertrend, indicator_node),
            ]
        }
        "PARABOLIC_SAR" => {
            let psar = format!(
                "psar:ohlc:{}:{}",
                indicator.psar_step, indicator.psar_max_step
            );
            vec![
                edge("high", &psar),
                edge("low", &psar),
                edge("close", &psar),
                edge(&psar, indicator_node),
            ]
        }
        "ICHIMOKU" => {
            let tenkan = format!("ichimoku:tenkan:{}", indicator.tenkan_period);
            let kijun = format!("ichimoku:kijun:{}", indicator.kijun_period);
            let senkou_a = format!(
                "ichimoku:senkou_a:{}:{}",
                indicator.tenkan_period, indicator.kijun_period
            );
            let senkou_b = format!("ichimoku:senkou_b:{}", indicator.senkou_b_period);
            vec![
                edge("high", &tenkan),
                edge("low", &tenkan),
                edge("high", &kijun),
                edge("low", &kijun),
                edge("high", &senkou_b),
                edge("low", &senkou_b),
                edge("close", "ichimoku:chikou"),
                edge(&tenkan, &senkou_a),
                edge(&kijun, &senkou_a),
                edge(&tenkan, indicator_node),
                edge(&kijun, indicator_node),
                edge(&senkou_a, indicator_node),
                edge(&senkou_b, indicator_node),
                edge("ichimoku:chikou", indicator_node),
            ]
        }
        "PIVOT_POINTS" => vec![
            edge("high", "pivot:pp"),
            edge("low", "pivot:pp"),
            edge("close", "pivot:pp"),
            edge("pivot:pp", "pivot:r1"),
            edge("pivot:pp", "pivot:s1"),
            edge("pivot:pp", "pivot:r2"),
            edge("pivot:pp", "pivot:s2"),
            edge("pivot:pp", indicator_node),
            edge("pivot:r1", indicator_node),
            edge("pivot:s1", indicator_node),
            edge("pivot:r2", indicator_node),
            edge("pivot:s2", indicator_node),
        ],
        "AROON" => {
            let aroon = format!("aroon:hl:{}", indicator.period);
            vec![
                edge("high", &aroon),
                edge("low", &aroon),
                edge(&aroon, indicator_node),
            ]
        }
        "ADL" => vec![
            edge("high", "adl:hlcv"),
            edge("low", "adl:hlcv"),
            edge("close", "adl:hlcv"),
            edge("volume", "adl:hlcv"),
            edge("adl:hlcv", indicator_node),
        ],
        "KST" => vec![
            edge("close", "roc:close:10"),
            edge("close", "roc:close:15"),
            edge("close", "roc:close:20"),
            edge("close", "roc:close:30"),
            edge("roc:close:10", "kst:value"),
            edge("roc:close:15", "kst:value"),
            edge("roc:close:20", "kst:value"),
            edge("roc:close:30", "kst:value"),
            edge("kst:value", indicator_node),
        ],
        "BOP" => vec![
            edge("high", "bop:ohlc"),
            edge("low", "bop:ohlc"),
            edge("close", "bop:ohlc"),
            edge("bop:ohlc", indicator_node),
        ],
        "ULTIMATE_OSCILLATOR" => {
            let uo = format!(
                "uo:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth
            );
            vec![
                edge("high", &uo),
                edge("low", &uo),
                edge("close", &uo),
                edge(&uo, indicator_node),
            ]
        }
        "CHAIKIN_OSCILLATOR" => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            });
            let chaikin = format!("chaikin:{}:{}", params.fast, params.slow);
            vec![
                edge("high", "adl:hlcv"),
                edge("low", "adl:hlcv"),
                edge("close", "adl:hlcv"),
                edge("volume", "adl:hlcv"),
                edge("adl:hlcv", &chaikin),
                edge(&chaikin, indicator_node),
            ]
        }
        "FORCE_INDEX" => {
            let force = format!("force:close:volume:{}", indicator.period);
            vec![
                edge("close", &force),
                edge("volume", &force),
                edge(&force, indicator_node),
            ]
        }
        "KELTNER" => {
            let ema = format!("ema:close:{}", indicator.period);
            let atr = format!("atr:ohlc:{}", indicator.period);
            let upper = format!(
                "keltner:upper:{}:{}",
                indicator.period, indicator.multiplier
            );
            let middle = format!(
                "keltner:middle:{}:{}",
                indicator.period, indicator.multiplier
            );
            let lower = format!(
                "keltner:lower:{}:{}",
                indicator.period, indicator.multiplier
            );
            vec![
                edge("close", &ema),
                edge("high", &atr),
                edge("low", &atr),
                edge("close", &atr),
                edge(&ema, &upper),
                edge(&atr, &upper),
                edge(&ema, &middle),
                edge(&ema, &lower),
                edge(&atr, &lower),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "DONCHIAN" => {
            let upper = format!("donchian:upper:{}", indicator.period);
            let middle = format!("donchian:middle:{}", indicator.period);
            let lower = format!("donchian:lower:{}", indicator.period);
            vec![
                edge("high", &upper),
                edge("low", &lower),
                edge(&upper, &middle),
                edge(&lower, &middle),
                edge(&upper, indicator_node),
                edge(&middle, indicator_node),
                edge(&lower, indicator_node),
            ]
        }
        "WILLIAMS_R" => {
            let willr = format!("willr:hlc:{}", indicator.period);
            vec![
                edge("high", &willr),
                edge("low", &willr),
                edge("close", &willr),
                edge(&willr, indicator_node),
            ]
        }
        "STOCH_RSI" => {
            let rsi = format!("rsi:close:{}", indicator.period);
            let stoch = format!(
                "stoch:rsi:{}:{}:{}:{}",
                indicator.period, indicator.stoch_period, indicator.smooth, indicator.signal
            );
            vec![
                edge("close", &rsi),
                edge(&rsi, &stoch),
                edge(&stoch, indicator_node),
            ]
        }
        "ADX" => {
            let adx = format!("adx:ohlc:{}", indicator.period);
            vec![
                edge("high", &adx),
                edge("low", &adx),
                edge("close", &adx),
                edge(&adx, indicator_node),
            ]
        }
        "VWAP" => vec![
            edge("high", "vwap:hlcv"),
            edge("low", "vwap:hlcv"),
            edge("close", "vwap:hlcv"),
            edge("volume", "vwap:hlcv"),
            edge("vwap:hlcv", indicator_node),
        ],
        "PPO" => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let fast = format!("ema:close:{}", params.fast);
            let slow = format!("ema:close:{}", params.slow);
            let ppo = format!("ppo:{}:{}:{}", params.fast, params.slow, params.signal);
            vec![
                edge("close", &fast),
                edge("close", &slow),
                edge(&fast, &ppo),
                edge(&slow, &ppo),
                edge(&ppo, indicator_node),
            ]
        }
        "STOCHASTIC" => {
            let stoch = format!("stoch:hlc:{}:{}", indicator.period, indicator.smooth);
            vec![
                edge("high", &stoch),
                edge("low", &stoch),
                edge("close", &stoch),
                edge(&stoch, indicator_node),
            ]
        }
        _ => indicator_nodes(indicator)
            .into_iter()
            .map(|node| edge(&node, indicator_node))
            .collect(),
    }
}

fn edge(from: &str, to: &str) -> DagEdge {
    DagEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn indicator_node(indicator: &Indicator) -> String {
    match (indicator.kind.as_str(), indicator.macd) {
        ("MACD", Some(macd)) | ("PPO", Some(macd)) => format!(
            "{}({},{},{})#{}",
            indicator.kind, macd.fast, macd.slow, macd.signal, indicator.id
        ),
        ("CHAIKIN_OSCILLATOR", Some(macd)) => format!(
            "CHAIKIN_OSCILLATOR({},{})#{}",
            macd.fast, macd.slow, indicator.id
        ),
        _ => format!("{}#{}", indicator.kind, indicator.id),
    }
}

fn validate_indicator(
    kind: &str,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
) -> Result<(), JsValue> {
    if kind == "MACD" || kind == "PPO" || kind == "CHAIKIN_OSCILLATOR" {
        let macd = macd.expect("MACD params are built before validation");
        if macd.fast == 0 || macd.slow <= macd.fast || macd.signal == 0 {
            return Err(JsValue::from_str(
                "fast/slow params must satisfy fast > 0, slow > fast, signal > 0",
            ));
        }
    } else if kind == "ICHIMOKU"
        && (tenkan_period == 0
            || kijun_period == 0
            || senkou_b_period == 0
            || kijun_period < tenkan_period
            || senkou_b_period < kijun_period)
    {
        return Err(JsValue::from_str(
            "ICHIMOKU params must satisfy tenkan > 0, kijun >= tenkan, senkou_b >= kijun",
        ));
    } else if kind == "PARABOLIC_SAR"
        && (!psar_step.is_finite()
            || !psar_max_step.is_finite()
            || psar_step <= 0.0
            || psar_max_step <= 0.0
            || psar_max_step < psar_step)
    {
        return Err(JsValue::from_str(
            "PARABOLIC_SAR params must satisfy step > 0 and max_step >= step",
        ));
    } else if kind != "OBV"
        && kind != "VWAP"
        && kind != "PARABOLIC_SAR"
        && kind != "ICHIMOKU"
        && kind != "PIVOT_POINTS"
        && kind != "ADL"
        && kind != "WILLIAMS_AD"
        && kind != "DEMA"
        && kind != "TEMA"
        && kind != "TRIMA"
        && kind != "STDDEV"
        && kind != "ENVELOPE"
        && kind != "KST"
        && kind != "BOP"
        && period == 0
    {
        return Err(JsValue::from_str("period must be greater than zero"));
    } else if kind == "STOCH_RSI" && stoch_period == 0 {
        return Err(JsValue::from_str("stoch_period must be greater than zero"));
    } else if kind == "TSI" && stoch_period == 0 {
        return Err(JsValue::from_str("stoch_period must be greater than zero"));
    } else if (kind == "STOCHASTIC" || kind == "STOCH_RSI") && smooth == 0 {
        return Err(JsValue::from_str("smooth must be greater than zero"));
    } else if kind == "STOCH_RSI" && signal == 0 {
        return Err(JsValue::from_str("signal must be greater than zero"));
    } else if kind == "ULTIMATE_OSCILLATOR" && (stoch_period < period || smooth < stoch_period) {
        return Err(JsValue::from_str(
            "ULTIMATE_OSCILLATOR params must satisfy short <= medium <= long",
        ));
    } else if (kind == "BB"
        || kind == "SUPERTREND"
        || kind == "KELTNER"
        || kind == "ENVELOPE"
        || kind == "STARC")
        && (!multiplier.is_finite() || multiplier <= 0.0)
    {
        return Err(JsValue::from_str("multiplier must be greater than zero"));
    }
    Ok(())
}

fn upsert_bar(bars: &mut Vec<Bar>, bar: Bar) -> bool {
    match bars.last_mut() {
        Some(last) if last.time == bar.time => {
            *last = bar;
            true
        }
        Some(last) if last.time < bar.time => {
            bars.push(bar);
            true
        }
        None => {
            bars.push(bar);
            true
        }
        _ => false,
    }
}

fn upsert_candle_store(bars: &mut CandleStore, bar: Bar) -> bool {
    match bars.last_time() {
        Some(time) if time == bar.time => {
            bars.set_last(bar);
            true
        }
        Some(time) if time < bar.time => {
            bars.push(bar);
            true
        }
        None => {
            bars.push(bar);
            true
        }
        _ => false,
    }
}

fn upsert_output(
    outputs: &mut IndicatorArena,
    name: &str,
    target_len: usize,
    value: Option<f64>,
) {
    let val = value.unwrap_or(f64::NAN);
    outputs.upsert_last(name, target_len, val);
}

fn output_at(outputs: &IndicatorArena, name: &str, index: usize) -> Option<f64> {
    outputs.value_at(name, index)
}

/// Same as output_at but for Vec<IndicatorOutput> used in internal compute functions.
fn output_at_vec(outputs: &[IndicatorOutput], name: &str, index: usize) -> Option<f64> {
    outputs
        .iter()
        .find(|output| output.name == name)
        .and_then(|output| output.values.get(index))
        .copied()
        .and_then(|v| if v.is_nan() { None } else { Some(v) })
}

fn sma(bars: &[Bar], period: usize) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.close;
        if i >= period {
            sum -= bars[i - period].close;
        }
        out.push(if i + 1 >= period { sum / period as f64 } else { f64::NAN });
    }
    out
}

fn sma_close_values(values: &[f64], period: usize) -> Series {
    let mut out = Vec::with_capacity(values.len());
    let mut sum = 0.0;
    for (index, value) in values.iter().copied().enumerate() {
        sum += value;
        if index >= period {
            sum -= values[index - period];
        }
        out.push(if period > 0 && index + 1 >= period { sum / period as f64 } else { f64::NAN });
    }
    out
}

fn latest_sma(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    Some(
        bars[bars.len() - period..]
            .iter()
            .map(|bar| bar.close)
            .sum::<f64>()
            / period as f64,
    )
}

fn latest_sma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    Some(store.close[store.len() - period..].iter().sum::<f64>() / period as f64)
}

fn sma_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = sma(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn sma_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("sma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(sma_close_values(&store.close, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn ema(bars: &[Bar], period: usize) -> Series {
    ema_values(bars.iter().map(|bar| bar.close), period)
}

fn ema_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("ema:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = ema(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn ema_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("ema:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(ema_values(store.close.iter().copied(), period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn ema_values(values: impl IntoIterator<Item = f64>, period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::new();
    for value in values {
        let next = match current {
            Some(previous) => alpha * value + (1.0 - alpha) * previous,
            None => value,
        };
        current = Some(next);
        out.push(next);
    }
    out
}

fn ema_series(values: &[f64], period: usize) -> Series {
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut current = None::<f64>;
    let mut out = Vec::with_capacity(values.len());
    for &value in values {
        if value.is_nan() {
            out.push(f64::NAN);
        } else {
            let next = match current {
                Some(previous) => alpha * value + (1.0 - alpha) * previous,
                None => value,
            };
            current = Some(next);
            out.push(next);
        }
    }
    out
}

fn latest_ema(bars: &[Bar], period: usize, output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    if period == 0 || bars.len() == 1 {
        return Some(last.close);
    }

    let previous = output
        .and_then(|values| values.get(bars.len() - 2))
        .copied().and_then(nan_to_none)
        .unwrap_or(bars[bars.len() - 2].close);
    let alpha = 2.0 / (period as f64 + 1.0);
    Some(alpha * last.close + (1.0 - alpha) * previous)
}

fn latest_ema_store(
    store: &CandleStore,
    period: usize,
    output: Option<&[f64]>,
) -> Option<f64> {
    let last = store.last_close()?;
    if period == 0 || store.len() == 1 {
        return Some(last);
    }

    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied().and_then(nan_to_none)
        .unwrap_or(store.close[store.len() - 2]);
    Some(ema_next(last, previous, period))
}

fn wma_from_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    let denominator = (period * (period + 1) / 2) as f64;
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let weighted_sum = window
            .iter()
            .enumerate()
            .map(|(offset, value)| (offset + 1) as f64 * value)
            .sum::<f64>();
        out[index] = weighted_sum / denominator;
    }
    out
}

fn wma(bars: &[Bar], period: usize) -> Series {
    let values: Vec<_> = bars.iter().map(|bar| bar.close).collect();
    wma_from_values(&values, period)
}

fn wma_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("wma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = wma(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_wma(bars: &[Bar], period: usize) -> Option<f64> {
    wma(bars, period).last().copied().and_then(nan_to_none)
}

fn wma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("wma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values: Vec<_> = store.close.iter().copied().collect();
    let rc = Rc::new(wma_from_values(&values, period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_wma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let denominator = (period * (period + 1) / 2) as f64;
    let start = store.len() - period;
    let weighted_sum = store.close[start..]
        .iter()
        .enumerate()
        .map(|(offset, value)| (offset + 1) as f64 * value)
        .sum::<f64>();
    Some(weighted_sum / denominator)
}

fn hma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("hma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    if period == 0 {
        return Rc::new(vec![f64::NAN; store.len()]);
    }
    let half_period = (period / 2).max(1);
    let sqrt_period = ((period as f64).sqrt().round() as usize).max(1);
    let half = wma_store(store, half_period, nodes);
    let full = wma_store(store, period, nodes);
    let raw: Vec<_> = half
        .iter()
        .zip(full.iter())
        .map(|(half, full)| match (half, full) {
            (half, full) if !half.is_nan() && !full.is_nan() => 2.0 * *half - *full,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(wma_from_values(&raw, sqrt_period));
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_hma_store(store: &CandleStore, period: usize) -> Option<f64> {
    hma_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn hma(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("hma:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    if period == 0 {
        return vec![f64::NAN; bars.len()];
    }
    let half_period = (period / 2).max(1);
    let sqrt_period = ((period as f64).sqrt().round() as usize).max(1);
    let half = wma_close(bars, half_period, nodes);
    let full = wma_close(bars, period, nodes);
    let raw: Vec<_> = half
        .iter()
        .zip(full.iter())
        .map(|(half, full)| match (half, full) {
            (half, full) if !half.is_nan() && !full.is_nan() => 2.0 * *half - *full,
            _ => f64::NAN,
        })
        .collect();
    let values = wma_from_values(&raw, sqrt_period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_hma(bars: &[Bar], period: usize) -> Option<f64> {
    hma(bars, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn linear_regression(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let sum_y = window.iter().map(|bar| bar.close).sum::<f64>();
        let sum_xy = window
            .iter()
            .enumerate()
            .map(|(offset, bar)| offset as f64 * bar.close)
            .sum::<f64>();
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        out[index] = intercept + slope * (period - 1) as f64;
    }
    out
}

fn linear_regression_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("linreg:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = linear_regression(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_linear_regression(bars: &[Bar], period: usize) -> Option<f64> {
    linear_regression(bars, period).last().copied().and_then(nan_to_none)
}

fn linear_regression_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("linreg:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    let n = period as f64;
    let sum_x = (0..period).map(|x| x as f64).sum::<f64>();
    let sum_xx = (0..period).map(|x| (x * x) as f64).sum::<f64>();
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let window = &store.close[index + 1 - period..=index];
        let sum_y = window.iter().sum::<f64>();
        let sum_xy = window
            .iter()
            .enumerate()
            .map(|(offset, close)| offset as f64 * close)
            .sum::<f64>();
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        out[index] = intercept + slope * (period - 1) as f64;
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_linear_regression_store(store: &CandleStore, period: usize) -> Option<f64> {
    linear_regression_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn dema(bars: &[Bar], period: usize) -> Series {
    let ema1 = ema_close(bars, period, &mut HashMap::new());
    let ema2 = ema_series(&ema1, period);
    ema1.iter()
        .zip(ema2.iter())
        .map(|(first, second)| match (first, second) {
            (first, second) if !first.is_nan() && !second.is_nan() => 2.0 * *first - *second,
            _ => f64::NAN,
        })
        .collect()
}

fn dema_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("dema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let ema1 = ema_close(bars, period, nodes);
    let ema2_key = format!("dema:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .map(|(first, second)| match (first, second) {
            (first, second) if !first.is_nan() && !second.is_nan() => 2.0 * *first - *second,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_dema(bars: &[Bar], period: usize) -> Option<f64> {
    dema(bars, period).last().copied().and_then(nan_to_none)
}

fn dema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("dema:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .map(|(first, second)| match (first, second) {
            (first, second) if !first.is_nan() && !second.is_nan() => 2.0 * *first - *second,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_dema_store(store: &CandleStore, period: usize) -> Option<f64> {
    dema_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn tema(bars: &[Bar], period: usize) -> Series {
    let ema1 = ema_close(bars, period, &mut HashMap::new());
    let ema2 = ema_series(&ema1, period);
    let ema3 = ema_series(&ema2, period);
    ema1.iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| match (first, second, third) {
            (first, second, third) if !first.is_nan() && !second.is_nan() && !third.is_nan() => 3.0 * *first - 3.0 * *second + *third,
            _ => f64::NAN,
        })
        .collect()
}

fn tema_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("tema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let ema1 = ema_close(bars, period, nodes);
    let ema2_key = format!("tema:ema2:{period}");
    let ema3_key = format!("tema:ema3:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = nodes
        .get(&ema3_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema2, period));
    nodes.insert(ema3_key, Rc::new(ema3.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| match (first, second, third) {
            (first, second, third) if !first.is_nan() && !second.is_nan() && !third.is_nan() => 3.0 * *first - 3.0 * *second + *third,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_tema(bars: &[Bar], period: usize) -> Option<f64> {
    tema(bars, period).last().copied().and_then(nan_to_none)
}

fn tema_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tema:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("tema:ema2:{period}");
    let ema3_key = format!("tema:ema3:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = nodes
        .get(&ema3_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema2, period));
    nodes.insert(ema3_key, Rc::new(ema3.clone()));
    let values: Vec<_> = ema1
        .iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((first, second), third)| match (first, second, third) {
            (first, second, third) if !first.is_nan() && !second.is_nan() && !third.is_nan() => 3.0 * *first - 3.0 * *second + *third,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_tema_store(store: &CandleStore, period: usize) -> Option<f64> {
    tema_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn trima(bars: &[Bar], period: usize) -> Series {
    sma_from_series(&sma_close(bars, period, &mut HashMap::new()), period)
}

fn trima_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = sma_from_series(&sma_close(bars, period, nodes), period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_trima(bars: &[Bar], period: usize) -> Option<f64> {
    trima(bars, period).last().copied().and_then(nan_to_none)
}

fn trima_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trima:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values = sma_from_series(&sma_close_store(store, period, nodes), period);
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_trima_store(store: &CandleStore, period: usize) -> Option<f64> {
    trima_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn stddev(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let mean = window.iter().map(|bar| bar.close).sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|bar| {
                let diff = bar.close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        out[index] = variance.sqrt();
    }
    out
}

fn stddev_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("stddev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = stddev(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_stddev(bars: &[Bar], period: usize) -> Option<f64> {
    stddev(bars, period).last().copied().and_then(nan_to_none)
}

fn stddev_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("stddev:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let window = &store.close[index + 1 - period..=index];
        let mean = window.iter().sum::<f64>() / period as f64;
        let variance = window
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        out[index] = variance.sqrt();
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_stddev_store(store: &CandleStore, period: usize) -> Option<f64> {
    stddev_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn envelope(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = sma_close(bars, period, nodes);
    let key_base = format!("envelope:{period}:{multiplier}");
    let upper_key = format!("envelope:upper:{period}:{multiplier}");
    let middle_key = format!("envelope:middle:{period}:{multiplier}");
    let lower_key = format!("envelope:lower:{period}:{multiplier}");
    if let (Some(upper), Some(middle), Some(lower)) = (
        nodes.get(&upper_key),
        nodes.get(&middle_key),
        nodes.get(&lower_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "upper".to_string(),
                values: (**upper).clone(),
            },
            IndicatorOutput {
                name: "middle".to_string(),
                values: (**middle).clone(),
            },
            IndicatorOutput {
                name: "lower".to_string(),
                values: (**lower).clone(),
            },
        ];
    }
    let upper: Vec<_> = middle
        .iter()
        .map(|&value| if value.is_nan() { f64::NAN } else { value * (1.0 + multiplier / 100.0) })
        .collect();
    let lower: Vec<_> = middle
        .iter()
        .map(|&value| if value.is_nan() { f64::NAN } else { value * (1.0 - multiplier / 100.0) })
        .collect();
    nodes.insert(upper_key, Rc::new(upper.clone()));
    nodes.insert(middle_key, Rc::new(middle.clone()));
    nodes.insert(lower_key, Rc::new(lower.clone()));
    nodes.insert(key_base, Rc::new(middle.clone()));
    vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ]
}

fn latest_envelope(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma(bars, period);
    let band = middle.map(|middle| middle * multiplier / 100.0);
    (
        middle.zip(band).map(|(middle, band)| middle + band),
        middle,
        middle.zip(band).map(|(middle, band)| middle - band),
    )
}

fn envelope_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let key_base = format!("envelope:{period}:{multiplier}");
    let upper_key = format!("envelope:upper:{period}:{multiplier}");
    let middle_key = format!("envelope:middle:{period}:{multiplier}");
    let lower_key = format!("envelope:lower:{period}:{multiplier}");
    if let (Some(upper), Some(middle), Some(lower)) = (
        nodes.get(&upper_key),
        nodes.get(&middle_key),
        nodes.get(&lower_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "upper".to_string(),
                values: (**upper).clone(),
            },
            IndicatorOutput {
                name: "middle".to_string(),
                values: (**middle).clone(),
            },
            IndicatorOutput {
                name: "lower".to_string(),
                values: (**lower).clone(),
            },
        ];
    }
    let upper: Vec<_> = middle
        .iter()
        .map(|&value| if value.is_nan() { f64::NAN } else { value * (1.0 + multiplier / 100.0) })
        .collect();
    let lower: Vec<_> = middle
        .iter()
        .map(|&value| if value.is_nan() { f64::NAN } else { value * (1.0 - multiplier / 100.0) })
        .collect();
    nodes.insert(upper_key, Rc::new(upper.clone()));
    nodes.insert(middle_key, Rc::new(middle.clone()));
    nodes.insert(lower_key, Rc::new(lower.clone()));
    nodes.insert(key_base, Rc::new(middle.clone()));
    vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ]
}

fn latest_envelope_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma_store(store, period);
    let band = middle.map(|middle| middle * multiplier / 100.0);
    (
        middle.zip(band).map(|(middle, band)| middle + band),
        middle,
        middle.zip(band).map(|(middle, band)| middle - band),
    )
}

fn trix(bars: &[Bar], period: usize) -> Series {
    let ema1 = ema_close(bars, period, &mut HashMap::new());
    let ema2 = ema_series(&ema1, period);
    let ema3 = ema_series(&ema2, period);
    let mut out = vec![f64::NAN; bars.len()];
    for index in 1..bars.len() {
        { let previous = ema3[index - 1]; let current = ema3[index]; if !previous.is_nan() && !current.is_nan() { out[index] = if previous != 0.0 { 100.0 * (current / previous - 1.0) } else { 0.0 }; } }
    }
    out
}

fn trix_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("trix:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let ema1 = ema_close(bars, period, nodes);
    let ema2_key = format!("trix:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = ema_series(&ema2, period);
    let mut out = vec![f64::NAN; bars.len()];
    for index in 1..bars.len() {
        { let previous = ema3[index - 1]; let current = ema3[index]; if !previous.is_nan() && !current.is_nan() { out[index] = if previous != 0.0 { 100.0 * (current / previous - 1.0) } else { 0.0 }; } }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

fn latest_trix(bars: &[Bar], period: usize) -> Option<f64> {
    trix(bars, period).last().copied().and_then(nan_to_none)
}

fn trix_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("trix:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema1 = rc_into_owned(ema_close_store(store, period, nodes));
    let ema2_key = format!("trix:ema2:{period}");
    let ema2 = nodes
        .get(&ema2_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| ema_series(&ema1, period));
    nodes.insert(ema2_key, Rc::new(ema2.clone()));
    let ema3 = ema_series(&ema2, period);
    let mut out = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        { let previous = ema3[index - 1]; let current = ema3[index]; if !previous.is_nan() && !current.is_nan() { out[index] = if previous != 0.0 { 100.0 * (current / previous - 1.0) } else { 0.0 }; } }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_trix_store(store: &CandleStore, period: usize) -> Option<f64> {
    trix_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn tsi(bars: &[Bar], long: usize, short: usize) -> Series {
    let mut momentum = vec![f64::NAN; bars.len()];
    let mut abs_momentum = vec![f64::NAN; bars.len()];
    for index in 1..bars.len() {
        let value = bars[index].close - bars[index - 1].close;
        momentum[index] = value;
        abs_momentum[index] = value.abs();
    }
    let ema1 = ema_series(&momentum, long);
    let ema2 = ema_series(&ema1, short);
    let abs_ema1 = ema_series(&abs_momentum, long);
    let abs_ema2 = ema_series(&abs_ema1, short);
    ema2.iter()
        .zip(abs_ema2.iter())
        .map(|(num, den)| match (num, den) {
            (num, den) if !num.is_nan() && !den.is_nan() && *den != 0.0 => 100.0 * *num / *den,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect()
}

fn tsi_node(bars: &[Bar], long: usize, short: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("tsi:{long}:{short}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = tsi(bars, long, short);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_tsi(bars: &[Bar], long: usize, short: usize) -> Option<f64> {
    tsi(bars, long, short).last().copied().and_then(nan_to_none)
}

fn tsi_store(store: &CandleStore, long: usize, short: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("tsi:{long}:{short}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut momentum = vec![f64::NAN; store.len()];
    let mut abs_momentum = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        let value = store.close[index] - store.close[index - 1];
        momentum[index] = value;
        abs_momentum[index] = value.abs();
    }
    let ema1 = ema_series(&momentum, long);
    let ema2 = ema_series(&ema1, short);
    let abs_ema1 = ema_series(&abs_momentum, long);
    let abs_ema2 = ema_series(&abs_ema1, short);
    let values: Series = ema2
        .iter()
        .zip(abs_ema2.iter())
        .map(|(num, den)| match (num, den) {
            (num, den) if !num.is_nan() && !den.is_nan() && *den != 0.0 => 100.0 * *num / *den,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_tsi_store(store: &CandleStore, long: usize, short: usize) -> Option<f64> {
    tsi_store(store, long, short, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn momentum(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if bars.len() <= period {
        return out;
    }
    for index in period..bars.len() {
        out[index] = bars[index].close - bars[index - period].close;
    }
    out
}

fn momentum_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("momentum:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = momentum(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_momentum(bars: &[Bar], period: usize) -> Option<f64> {
    momentum(bars, period).last().copied().and_then(nan_to_none)
}

fn momentum_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("momentum:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if store.len() <= period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period..store.len() {
        out[index] = store.close[index] - store.close[index - period];
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_momentum_store(store: &CandleStore, period: usize) -> Option<f64> {
    momentum_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn dpo(bars: &[Bar], period: usize) -> Series {
    let sma_values = sma(bars, period);
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; bars.len()];
    for index in 0..bars.len() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        let mean = sma_values[index]; if !mean.is_nan() { out[index] = bars[index - shift].close - mean; }
    }
    out
}

fn dpo_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let sma_key = format!("sma:close:{period}");
    let sma_values = nodes
        .get(&sma_key)
        .map(|rc| (**rc).clone())
        .unwrap_or_else(|| sma_close(bars, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; bars.len()];
    for index in 0..bars.len() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        let mean = sma_values[index]; if !mean.is_nan() { out[index] = bars[index - shift].close - mean; }
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

fn latest_dpo(bars: &[Bar], period: usize) -> Option<f64> {
    dpo(bars, period).last().copied().and_then(nan_to_none)
}

fn dpo_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("dpo:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let sma_values = rc_into_owned(sma_close_store(store, period, nodes));
    let shift = period / 2 + 1;
    let mut out = vec![f64::NAN; store.len()];
    for index in 0..store.len() {
        if index < period.saturating_sub(1) || index < shift {
            continue;
        }
        let mean = sma_values[index]; if !mean.is_nan() { out[index] = store.close[index - shift] - mean; }
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_dpo_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let shift = period / 2 + 1;
    let index = store.len() - 1;
    if index < shift || index < period.saturating_sub(1) {
        return None;
    }
    latest_sma_store(store, period).map(|mean| store.close[index - shift] - mean)
}

fn kst(bars: &[Bar]) -> Series {
    let roc1 = roc(bars, 10);
    let roc2 = roc(bars, 15);
    let roc3 = roc(bars, 20);
    let roc4 = roc(bars, 30);
    let sma1 = sma_from_series(&roc1, 10);
    let sma2 = sma_from_series(&roc2, 10);
    let sma3 = sma_from_series(&roc3, 10);
    let sma4 = sma_from_series(&roc4, 15);
    sma1.iter()
        .zip(sma2.iter())
        .zip(sma3.iter())
        .zip(sma4.iter())
        .map(|(((a, b), c), d)| match (a, b, c, d) {
            (a, b, c, d) if !a.is_nan() && !b.is_nan() && !c.is_nan() && !d.is_nan() => *a + 2.0 * *b + 3.0 * *c + 4.0 * *d,
            _ => f64::NAN,
        })
        .collect()
}

fn sma_from_series(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }
    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        out[index] = window.iter().sum::<f64>() / period as f64;
    }
    out
}

fn kst_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "kst:value".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let roc1 = roc_node(bars, 10, nodes);
    let roc2 = roc_node(bars, 15, nodes);
    let roc3 = roc_node(bars, 20, nodes);
    let roc4 = roc_node(bars, 30, nodes);
    let sma1 = sma_from_series(&roc1, 10);
    let sma2 = sma_from_series(&roc2, 10);
    let sma3 = sma_from_series(&roc3, 10);
    let sma4 = sma_from_series(&roc4, 15);
    let values: Vec<_> = sma1
        .iter()
        .zip(sma2.iter())
        .zip(sma3.iter())
        .zip(sma4.iter())
        .map(|(((a, b), c), d)| match (a, b, c, d) {
            (a, b, c, d) if !a.is_nan() && !b.is_nan() && !c.is_nan() && !d.is_nan() => *a + 2.0 * *b + 3.0 * *c + 4.0 * *d,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_kst(bars: &[Bar]) -> Option<f64> {
    kst(bars).last().copied().and_then(nan_to_none)
}

fn kst_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "kst:value".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let roc1 = roc_store(store, 10, nodes);
    let roc2 = roc_store(store, 15, nodes);
    let roc3 = roc_store(store, 20, nodes);
    let roc4 = roc_store(store, 30, nodes);
    let sma1 = sma_from_series(&roc1, 10);
    let sma2 = sma_from_series(&roc2, 10);
    let sma3 = sma_from_series(&roc3, 10);
    let sma4 = sma_from_series(&roc4, 15);
    let values: Vec<_> = sma1
        .iter()
        .zip(sma2.iter())
        .zip(sma3.iter())
        .zip(sma4.iter())
        .map(|(((a, b), c), d)| match (a, b, c, d) {
            (a, b, c, d) if !a.is_nan() && !b.is_nan() && !c.is_nan() && !d.is_nan() => *a + 2.0 * *b + 3.0 * *c + 4.0 * *d,
            _ => f64::NAN,
        })
        .collect();
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_kst_store(store: &CandleStore) -> Option<f64> {
    kst_store(store, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn bop(bars: &[Bar]) -> Series {
    bars.iter()
        .map(|bar| {
            let range = bar.high - bar.low;
            if range == 0.0 { 0.0 } else { (bar.close - bar.open) / range }
        })
        .collect()
}

fn bop_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "bop:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = bop(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_bop(bars: &[Bar]) -> Option<f64> {
    bop(bars).last().copied().and_then(nan_to_none)
}

fn bop_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "bop:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let values: Vec<_> = (0..store.len()).map(|index| { let range = store.high[index] - store.low[index]; if range == 0.0 { 0.0 } else { (store.close[index] - store.open[index]) / range } })
        .collect();
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_bop_store(store: &CandleStore) -> Option<f64> {
    bop_store(store, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn ultimate_oscillator(bars: &[Bar], short: usize, medium: usize, long: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if short == 0 || medium == 0 || long == 0 || bars.len() <= long {
        return out;
    }
    let mut bp = vec![0.0; bars.len()];
    let mut tr = vec![0.0; bars.len()];
    for index in 1..bars.len() {
        let previous_close = bars[index - 1].close;
        let min_low = bars[index].low.min(previous_close);
        let max_high = bars[index].high.max(previous_close);
        bp[index] = bars[index].close - min_low;
        tr[index] = max_high - min_low;
    }
    for index in long..bars.len() {
        let avg = |period: usize| {
            let start = index + 1 - period;
            let bp_sum = bp[start..=index].iter().sum::<f64>();
            let tr_sum = tr[start..=index].iter().sum::<f64>();
            if tr_sum == 0.0 {
                0.0
            } else {
                bp_sum / tr_sum
            }
        };
        out[index] = 100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0;
    }
    out
}

fn ultimate_oscillator_node(
    bars: &[Bar],
    short: usize,
    medium: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("uo:{short}:{medium}:{long}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = ultimate_oscillator(bars, short, medium, long);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_ultimate_oscillator(
    bars: &[Bar],
    short: usize,
    medium: usize,
    long: usize,
) -> Option<f64> {
    ultimate_oscillator(bars, short, medium, long)
        .last()
        .copied().and_then(nan_to_none)
}

fn ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("uo:{short}:{medium}:{long}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let mut out = vec![f64::NAN; store.len()];
    if short == 0 || medium == 0 || long == 0 || store.len() <= long {
        nodes.insert(key, Rc::new(out.clone()));
        return out;
    }
    let mut bp = vec![0.0; store.len()];
    let mut tr = vec![0.0; store.len()];
    for index in 1..store.len() {
        let previous_close = store.close[index - 1];
        let min_low = store.low[index].min(previous_close);
        let max_high = store.high[index].max(previous_close);
        bp[index] = store.close[index] - min_low;
        tr[index] = max_high - min_low;
    }
    for index in long..store.len() {
        let avg = |period: usize| {
            let start = index + 1 - period;
            let bp_sum = bp[start..=index].iter().sum::<f64>();
            let tr_sum = tr[start..=index].iter().sum::<f64>();
            if tr_sum == 0.0 {
                0.0
            } else {
                bp_sum / tr_sum
            }
        };
        out[index] = 100.0 * (4.0 * avg(short) + 2.0 * avg(medium) + avg(long)) / 7.0;
    }
    nodes.insert(key, Rc::new(out.clone()));
    out
}

fn latest_ultimate_oscillator_store(
    store: &CandleStore,
    short: usize,
    medium: usize,
    long: usize,
) -> Option<f64> {
    ultimate_oscillator_store(store, short, medium, long, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn force_index(bars: &[Bar], period: usize) -> Series {
    let mut raw = vec![f64::NAN; bars.len()];
    for index in 1..bars.len() {
        raw[index] = (bars[index].close - bars[index - 1].close) * bars[index].volume;
    }
    ema_series(&raw, period)
}

fn force_index_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("force:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = force_index(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_force_index(bars: &[Bar], period: usize) -> Option<f64> {
    force_index(bars, period).last().copied().and_then(nan_to_none)
}

fn force_index_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("force:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut raw = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        raw[index] = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    let values = ema_series(&raw, period);
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_force_index_store(store: &CandleStore, period: usize) -> Option<f64> {
    if store.len() < 2 {
        return None;
    }
    let mut raw = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        raw[index] = (store.close[index] - store.close[index - 1]) * store.volume[index];
    }
    ema_series(&raw, period).last().copied().and_then(nan_to_none)
}

#[cfg(test)]
fn rsi(bars: &[Bar], period: usize) -> Series {
    rsi_outputs(bars, period).remove(0).values
}

fn rsi_outputs(bars: &[Bar], period: usize) -> Vec<IndicatorOutput> {
    let mut values = vec![f64::NAN; bars.len()];
    let mut avg_gains = vec![f64::NAN; bars.len()];
    let mut avg_losses = vec![f64::NAN; bars.len()];
    if bars.len() <= period {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "avg_gain".to_string(),
                values: avg_gains,
            },
            IndicatorOutput {
                name: "avg_loss".to_string(),
                values: avg_losses,
            },
        ];
    }

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..=period {
        let change = bars[i].close - bars[i - 1].close;
        if change >= 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    values[period] = rsi_value(avg_gain, avg_loss);
    avg_gains[period] = avg_gain;
    avg_losses[period] = avg_loss;

    for i in period + 1..bars.len() {
        let change = bars[i].close - bars[i - 1].close;
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        values[i] = rsi_value(avg_gain, avg_loss);
        avg_gains[i] = avg_gain;
        avg_losses[i] = avg_loss;
    }

    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "avg_gain".to_string(),
            values: avg_gains,
        },
        IndicatorOutput {
            name: "avg_loss".to_string(),
            values: avg_losses,
        },
    ]
}

fn rsi_outputs_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let key = format!("rsi:close:{period}");
    let gain_key = format!("rsi:avg_gain:{period}");
    let loss_key = format!("rsi:avg_loss:{period}");
    if let (Some(values), Some(avg_gains), Some(avg_losses)) =
        (nodes.get(&key), nodes.get(&gain_key), nodes.get(&loss_key))
    {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "avg_gain".to_string(),
                values: (**avg_gains).clone(),
            },
            IndicatorOutput {
                name: "avg_loss".to_string(),
                values: (**avg_losses).clone(),
            },
        ];
    }

    let mut values = vec![f64::NAN; store.len()];
    let mut avg_gains = vec![f64::NAN; store.len()];
    let mut avg_losses = vec![f64::NAN; store.len()];
    if store.len() <= period {
        nodes.insert(key, Rc::new(values.clone()));
        nodes.insert(gain_key, Rc::new(avg_gains.clone()));
        nodes.insert(loss_key, Rc::new(avg_losses.clone()));
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "avg_gain".to_string(),
                values: avg_gains,
            },
            IndicatorOutput {
                name: "avg_loss".to_string(),
                values: avg_losses,
            },
        ];
    }

    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for index in 1..=period {
        let change = store.close[index] - store.close[index - 1];
        if change >= 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    values[period] = rsi_value(avg_gain, avg_loss);
    avg_gains[period] = avg_gain;
    avg_losses[period] = avg_loss;

    for index in period + 1..store.len() {
        let change = store.close[index] - store.close[index - 1];
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        values[index] = rsi_value(avg_gain, avg_loss);
        avg_gains[index] = avg_gain;
        avg_losses[index] = avg_loss;
    }

    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(gain_key, Rc::new(avg_gains.clone()));
    nodes.insert(loss_key, Rc::new(avg_losses.clone()));
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "avg_gain".to_string(),
            values: avg_gains,
        },
        IndicatorOutput {
            name: "avg_loss".to_string(),
            values: avg_losses,
        },
    ]
}

fn rsi_value(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    }
}

fn latest_rsi(
    bars: &[Bar],
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || bars.len() <= period {
        return (None, None, None);
    }

    if bars.len() == period + 1 {
        let outputs = rsi_outputs(bars, period);
        let index = bars.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "avg_gain", index),
            output_at_vec(&outputs, "avg_loss", index),
        );
    }

    let previous_index = bars.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "avg_gain", previous_index).is_some()
        && output_at(outputs, "avg_loss", previous_index).is_some()
    {
        outputs
    } else {
        previous_outputs = IndicatorArena::from_outputs(rsi_outputs(&bars[..bars.len() - 1], period));
        &previous_outputs
    };
    let previous_gain = output_at(source_outputs, "avg_gain", previous_index).unwrap_or(0.0);
    let previous_loss = output_at(source_outputs, "avg_loss", previous_index).unwrap_or(0.0);
    let change = bars.last().expect("checked non-empty").close - bars[previous_index].close;
    let gain = change.max(0.0);
    let loss = (-change).max(0.0);
    let avg_gain = (previous_gain * (period - 1) as f64 + gain) / period as f64;
    let avg_loss = (previous_loss * (period - 1) as f64 + loss) / period as f64;
    (
        Some(rsi_value(avg_gain, avg_loss)),
        Some(avg_gain),
        Some(avg_loss),
    )
}

fn latest_rsi_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None, None);
    }

    if store.len() == period + 1 {
        let outputs = rsi_outputs_store(store, period, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "avg_gain", index),
            output_at_vec(&outputs, "avg_loss", index),
        );
    }

    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "avg_gain", previous_index).is_some()
        && output_at(outputs, "avg_loss", previous_index).is_some()
    {
        outputs
    } else {
        let previous = CandleStore {
            time: store.time[..store.len() - 1].to_vec(),
            open: store.open[..store.len() - 1].to_vec(),
            high: store.high[..store.len() - 1].to_vec(),
            low: store.low[..store.len() - 1].to_vec(),
            close: store.close[..store.len() - 1].to_vec(),
            volume: store.volume[..store.len() - 1].to_vec(),
        };
        previous_outputs = IndicatorArena::from_outputs(rsi_outputs_store(&previous, period, &mut HashMap::new()));
        &previous_outputs
    };
    let previous_gain = output_at(source_outputs, "avg_gain", previous_index).unwrap_or(0.0);
    let previous_loss = output_at(source_outputs, "avg_loss", previous_index).unwrap_or(0.0);
    let change = store.close[store.len() - 1] - store.close[previous_index];
    let gain = change.max(0.0);
    let loss = (-change).max(0.0);
    let avg_gain = (previous_gain * (period - 1) as f64 + gain) / period as f64;
    let avg_loss = (previous_loss * (period - 1) as f64 + loss) / period as f64;
    (
        Some(rsi_value(avg_gain, avg_loss)),
        Some(avg_gain),
        Some(avg_loss),
    )
}

fn rsi_close(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("rsi:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = rsi_outputs(bars, period).remove(0).values;
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn rsi_close_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("rsi:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let rc = Rc::new(rsi_outputs_store(store, period, nodes).remove(0).values);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn obv(bars: &[Bar]) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut current = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        if i > 0 {
            let previous = bars[i - 1].close;
            if bar.close > previous {
                current += bar.volume;
            } else if bar.close < previous {
                current -= bar.volume;
            }
        }
        out.push(current);
    }
    out
}

fn latest_obv(bars: &[Bar], output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    if bars.len() == 1 {
        return Some(0.0);
    }

    let previous = output
        .and_then(|values| values.get(bars.len() - 2))
        .copied().and_then(nan_to_none)
        .unwrap_or(0.0);
    let previous_close = bars[bars.len() - 2].close;
    if last.close > previous_close {
        Some(previous + last.volume)
    } else if last.close < previous_close {
        Some(previous - last.volume)
    } else {
        Some(previous)
    }
}

fn obv_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "obv:close:volume".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = obv(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn obv_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "obv:close:volume".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for index in 0..store.len() {
        if index > 0 {
            let previous = store.close[index - 1];
            let close = store.close[index];
            if close > previous {
                current += store.volume[index];
            } else if close < previous {
                current -= store.volume[index];
            }
        }
        out.push(current);
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_obv_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let last = store.last_close()?;
    if store.len() == 1 {
        return Some(0.0);
    }

    let previous = output
        .and_then(|values| values.get(store.len() - 2))
        .copied().and_then(nan_to_none)
        .unwrap_or(0.0);
    let previous_close = store.close[store.len() - 2];
    if last > previous_close {
        Some(previous + store.volume[store.len() - 1])
    } else if last < previous_close {
        Some(previous - store.volume[store.len() - 1])
    } else {
        Some(previous)
    }
}

fn money_flow_multiplier(bar: &Bar) -> f64 {
    money_flow_multiplier_parts(bar.high, bar.low, bar.close)
}

fn money_flow_multiplier_parts(high: f64, low: f64, close: f64) -> f64 {
    let range = high - low;
    if range == 0.0 {
        0.0
    } else {
        ((close - low) - (high - close)) / range
    }
}

fn adl(bars: &[Bar]) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut current = 0.0;
    for bar in bars {
        current += money_flow_multiplier(bar) * bar.volume;
        out.push(current);
    }
    out
}

fn adl_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "adl:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = adl(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn adl_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "adl:hlcv".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for index in 0..store.len() {
        current +=
            money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index];
        out.push(current);
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_adl(bars: &[Bar], output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    let previous = bars
        .len()
        .checked_sub(2)
        .and_then(|index| {
            output
                .and_then(|values| values.get(index))
                .copied().and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(previous + money_flow_multiplier(last) * last.volume)
}

fn latest_adl_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    let previous = index
        .checked_sub(1)
        .and_then(|previous_index| {
            output
                .and_then(|values| values.get(previous_index))
                .copied().and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(
        previous
            + money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index],
    )
}

fn williams_ad_step(previous_close: f64, bar: &Bar) -> f64 {
    williams_ad_step_parts(previous_close, bar.high, bar.low, bar.close)
}

fn williams_ad_step_parts(previous_close: f64, high: f64, low: f64, close: f64) -> f64 {
    if close > previous_close {
        close - previous_close.min(low)
    } else if close < previous_close {
        close - previous_close.max(high)
    } else {
        0.0
    }
}

fn williams_ad(bars: &[Bar]) -> Series {
    let mut out = Vec::with_capacity(bars.len());
    let mut current = 0.0;
    for (index, bar) in bars.iter().enumerate() {
        if index > 0 {
            current += williams_ad_step(bars[index - 1].close, bar);
        }
        out.push(current);
    }
    out
}

fn williams_ad_node(bars: &[Bar], nodes: &mut NodeCache) -> Series {
    let key = "wad:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = williams_ad(bars);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn williams_ad_store(store: &CandleStore, nodes: &mut NodeCache) -> RcSeries {
    let key = "wad:ohlc".to_string();
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = Vec::with_capacity(store.len());
    let mut current = 0.0;
    for index in 0..store.len() {
        if index > 0 {
            current += williams_ad_step_parts(
                store.close[index - 1],
                store.high[index],
                store.low[index],
                store.close[index],
            );
        }
        out.push(current);
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_williams_ad(bars: &[Bar], output: Option<&[f64]>) -> Option<f64> {
    let last = bars.last()?;
    if bars.len() == 1 {
        return Some(0.0);
    }
    let previous = bars
        .len()
        .checked_sub(2)
        .and_then(|index| {
            output
                .and_then(|values| values.get(index))
                .copied().and_then(nan_to_none)
        })
        .unwrap_or(0.0);
    Some(previous + williams_ad_step(bars[bars.len() - 2].close, last))
}

fn latest_williams_ad_store(store: &CandleStore, output: Option<&[f64]>) -> Option<f64> {
    let index = store.len().checked_sub(1)?;
    if index == 0 {
        return Some(0.0);
    }
    let previous = output
        .and_then(|values| values.get(index - 1))
        .copied().and_then(nan_to_none)
        .unwrap_or(0.0);
    Some(
        previous
            + williams_ad_step_parts(
                store.close[index - 1],
                store.high[index],
                store.low[index],
                store.close[index],
            ),
    )
}

fn typical_price(bar: &Bar) -> f64 {
    typical_price_parts(bar.high, bar.low, bar.close)
}

fn typical_price_parts(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}

fn vwap(bars: &[Bar], nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    if let Some(values) = nodes.get("vwap:hlcv") {
        return vwap_outputs(
            (**values).clone(),
            nodes.get("vwap:cumulative_pv").map(|rc| (**rc).clone()).unwrap_or_default(),
            nodes
                .get("vwap:cumulative_volume")
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
        );
    }

    let mut values = Vec::with_capacity(bars.len());
    let mut cumulative_pv_values = Vec::with_capacity(bars.len());
    let mut cumulative_volume_values = Vec::with_capacity(bars.len());
    let mut cumulative_pv = 0.0;
    let mut cumulative_volume = 0.0;
    for bar in bars {
        cumulative_pv += typical_price(bar) * bar.volume;
        cumulative_volume += bar.volume;
        values.push(if cumulative_volume > 0.0 { cumulative_pv / cumulative_volume } else { f64::NAN });
        cumulative_pv_values.push(cumulative_pv);
        cumulative_volume_values.push(cumulative_volume);
    }

    nodes.insert("vwap:hlcv".to_string(), Rc::new(values.clone()));
    nodes.insert(
        "vwap:cumulative_pv".to_string(),
        Rc::new(cumulative_pv_values.clone()),
    );
    nodes.insert(
        "vwap:cumulative_volume".to_string(),
        Rc::new(cumulative_volume_values.clone()),
    );
    vwap_outputs(values, cumulative_pv_values, cumulative_volume_values)
}

fn vwap_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    if let Some(values) = nodes.get("vwap:hlcv") {
        return vwap_outputs(
            (**values).clone(),
            nodes.get("vwap:cumulative_pv").map(|rc| (**rc).clone()).unwrap_or_default(),
            nodes
                .get("vwap:cumulative_volume")
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
        );
    }

    let mut values = Vec::with_capacity(store.len());
    let mut cumulative_pv_values = Vec::with_capacity(store.len());
    let mut cumulative_volume_values = Vec::with_capacity(store.len());
    let mut cumulative_pv = 0.0;
    let mut cumulative_volume = 0.0;
    for index in 0..store.len() {
        cumulative_pv +=
            typical_price_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index];
        cumulative_volume += store.volume[index];
        values.push(if cumulative_volume > 0.0 { cumulative_pv / cumulative_volume } else { f64::NAN });
        cumulative_pv_values.push(cumulative_pv);
        cumulative_volume_values.push(cumulative_volume);
    }

    nodes.insert("vwap:hlcv".to_string(), Rc::new(values.clone()));
    nodes.insert(
        "vwap:cumulative_pv".to_string(),
        Rc::new(cumulative_pv_values.clone()),
    );
    nodes.insert(
        "vwap:cumulative_volume".to_string(),
        Rc::new(cumulative_volume_values.clone()),
    );
    vwap_outputs(values, cumulative_pv_values, cumulative_volume_values)
}

fn vwap_outputs(
    values: Series,
    cumulative_pv: Series,
    cumulative_volume: Series,
) -> Vec<IndicatorOutput> {
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "cumulative_pv".to_string(),
            values: cumulative_pv,
        },
        IndicatorOutput {
            name: "cumulative_volume".to_string(),
            values: cumulative_volume,
        },
    ]
}

fn latest_vwap(
    bars: &[Bar],
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(last) = bars.last() else {
        return (None, None, None);
    };
    let previous_index = bars.len().checked_sub(2);
    let previous_pv = previous_index
        .and_then(|index| output_at(outputs, "cumulative_pv", index))
        .unwrap_or(0.0);
    let previous_volume = previous_index
        .and_then(|index| output_at(outputs, "cumulative_volume", index))
        .unwrap_or(0.0);
    let cumulative_pv = previous_pv + typical_price(last) * last.volume;
    let cumulative_volume = previous_volume + last.volume;
    (
        (cumulative_volume > 0.0).then_some(cumulative_pv / cumulative_volume),
        Some(cumulative_pv),
        Some(cumulative_volume),
    )
}

fn latest_vwap_store(
    store: &CandleStore,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(index) = store.len().checked_sub(1) else {
        return (None, None, None);
    };
    let previous_index = index.checked_sub(1);
    let previous_pv = previous_index
        .and_then(|previous_index| output_at(outputs, "cumulative_pv", previous_index))
        .unwrap_or(0.0);
    let previous_volume = previous_index
        .and_then(|previous_index| output_at(outputs, "cumulative_volume", previous_index))
        .unwrap_or(0.0);
    let cumulative_pv = previous_pv
        + typical_price_parts(store.high[index], store.low[index], store.close[index])
            * store.volume[index];
    let cumulative_volume = previous_volume + store.volume[index];
    (
        (cumulative_volume > 0.0).then_some(cumulative_pv / cumulative_volume),
        Some(cumulative_pv),
        Some(cumulative_volume),
    )
}

fn vwma(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let volume_sum = window.iter().map(|bar| bar.volume).sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = window.iter().map(|bar| bar.close * bar.volume).sum::<f64>();
        out[index] = weighted_sum / volume_sum;
    }
    out
}

fn vwma_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("vwma:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = vwma(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn vwma_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("vwma:close:volume:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let start = index + 1 - period;
        let volume_sum = store.volume[start..=index].iter().sum::<f64>();
        if volume_sum == 0.0 {
            continue;
        }
        let weighted_sum = (start..=index)
            .map(|window_index| store.close[window_index] * store.volume[window_index])
            .sum::<f64>();
        out[index] = weighted_sum / volume_sum;
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_vwma(bars: &[Bar], period: usize) -> Option<f64> {
    vwma(bars, period).last().copied().and_then(nan_to_none)
}

fn latest_vwma_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let volume_sum = store.volume[start..].iter().sum::<f64>();
    if volume_sum == 0.0 {
        return None;
    }
    let weighted_sum = (start..store.len())
        .map(|index| store.close[index] * store.volume[index])
        .sum::<f64>();
    Some(weighted_sum / volume_sum)
}

fn cmf_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cmf:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period - 1..store.len() {
        let start = index + 1 - period;
        let mut mfv_sum = 0.0;
        let mut volume_sum = 0.0;
        for window_index in start..=index {
            mfv_sum += money_flow_multiplier_parts(
                store.high[window_index],
                store.low[window_index],
                store.close[window_index],
            ) * store.volume[window_index];
            volume_sum += store.volume[window_index];
        }
        out[index] = if volume_sum != 0.0 { mfv_sum / volume_sum } else { f64::NAN };
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_cmf_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let mut mfv_sum = 0.0;
    let mut volume_sum = 0.0;
    for index in start..store.len() {
        mfv_sum +=
            money_flow_multiplier_parts(store.high[index], store.low[index], store.close[index])
                * store.volume[index];
        volume_sum += store.volume[index];
    }
    (volume_sum != 0.0).then_some(mfv_sum / volume_sum)
}

fn cmf(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }
    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let mfv_sum = window
            .iter()
            .map(|bar| money_flow_multiplier(bar) * bar.volume)
            .sum::<f64>();
        let volume_sum = window.iter().map(|bar| bar.volume).sum::<f64>();
        out[index] = if volume_sum != 0.0 { mfv_sum / volume_sum } else { f64::NAN };
    }
    out
}

fn cmf_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cmf:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = cmf(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_cmf(bars: &[Bar], period: usize) -> Option<f64> {
    cmf(bars, period).last().copied().and_then(nan_to_none)
}

fn typical_price_at(store: &CandleStore, index: usize) -> f64 {
    typical_price_parts(store.high[index], store.low[index], store.close[index])
}

fn cci(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }

    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let typical_prices: Vec<_> = window.iter().map(typical_price).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        out[index] = if mean_deviation == 0.0 { 0.0 } else { (typical_price(&bars[index]) - sma) / (0.015 * mean_deviation) };
    }
    out
}

fn cci_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cci:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = cci(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_cci(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let window = &bars[bars.len() - period..];
    let typical_prices: Vec<_> = window.iter().map(typical_price).collect();
    let sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - sma).abs())
        .sum::<f64>()
        / period as f64;
    Some(if mean_deviation == 0.0 {
        0.0
    } else {
        (typical_price(bars.last().expect("checked non-empty")) - sma) / (0.015 * mean_deviation)
    })
}

fn cci_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cci:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    for index in period - 1..store.len() {
        let window = index + 1 - period..=index;
        let typical_prices: Vec<_> = window.clone().map(|i| typical_price_at(store, i)).collect();
        let sma = typical_prices.iter().sum::<f64>() / period as f64;
        let mean_deviation = typical_prices
            .iter()
            .map(|value| (value - sma).abs())
            .sum::<f64>()
            / period as f64;
        out[index] = if mean_deviation == 0.0 { 0.0 } else { (typical_price_at(store, index) - sma) / (0.015 * mean_deviation) };
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_cci_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let typical_prices: Vec<_> = (start..store.len())
        .map(|index| typical_price_at(store, index))
        .collect();
    let sma = typical_prices.iter().sum::<f64>() / period as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|value| (value - sma).abs())
        .sum::<f64>()
        / period as f64;
    Some(if mean_deviation == 0.0 {
        0.0
    } else {
        (typical_price_at(store, store.len() - 1) - sma) / (0.015 * mean_deviation)
    })
}

fn williams_r(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }

    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let highest_high = window.iter().map(|bar| bar.high).fold(f64::MIN, f64::max);
        let lowest_low = window.iter().map(|bar| bar.low).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        out[index] = if range == 0.0 { 0.0 } else { -100.0 * (highest_high - bars[index].close) / range };
    }
    out
}

fn williams_r_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("willr:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = williams_r(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_williams_r(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period {
        return None;
    }
    let window = &bars[bars.len() - period..];
    let highest_high = window.iter().map(|bar| bar.high).fold(f64::MIN, f64::max);
    let lowest_low = window.iter().map(|bar| bar.low).fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    Some(if range == 0.0 {
        0.0
    } else {
        -100.0 * (highest_high - bars.last().expect("checked non-empty").close) / range
    })
}

fn williams_r_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("willr:hlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    for index in period - 1..store.len() {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        out[index] = if range == 0.0 { 0.0 } else { -100.0 * (highest_high - store.close[index]) / range };
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_williams_r_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() < period {
        return None;
    }
    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    Some(if range == 0.0 {
        0.0
    } else {
        -100.0 * (highest_high - store.close[store.len() - 1]) / range
    })
}

fn mfi(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return out;
    }

    for index in period..bars.len() {
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;
        for current in index + 1 - period..=index {
            let previous = current - 1;
            let previous_tp = typical_price(&bars[previous]);
            let current_tp = typical_price(&bars[current]);
            let raw_flow = current_tp * bars[current].volume;
            if current_tp > previous_tp {
                positive_flow += raw_flow;
            } else if current_tp < previous_tp {
                negative_flow += raw_flow;
            }
        }
        out[index] = mfi_value(positive_flow, negative_flow);
    }
    out
}

fn mfi_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("mfi:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = mfi(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn mfi_value(positive_flow: f64, negative_flow: f64) -> f64 {
    if negative_flow == 0.0 {
        100.0
    } else {
        let money_ratio = positive_flow / negative_flow;
        100.0 - 100.0 / (1.0 + money_ratio)
    }
}

fn latest_mfi(bars: &[Bar], period: usize) -> Option<f64> {
    if period == 0 || bars.len() <= period {
        return None;
    }

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;
    for current in bars.len() - period..bars.len() {
        let previous = current - 1;
        let previous_tp = typical_price(&bars[previous]);
        let current_tp = typical_price(&bars[current]);
        let raw_flow = current_tp * bars[current].volume;
        if current_tp > previous_tp {
            positive_flow += raw_flow;
        } else if current_tp < previous_tp {
            negative_flow += raw_flow;
        }
    }
    Some(mfi_value(positive_flow, negative_flow))
}

fn mfi_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("mfi:hlcv:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    for index in period..store.len() {
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;
        for current in index + 1 - period..=index {
            let previous = current - 1;
            let previous_tp = typical_price_at(store, previous);
            let current_tp = typical_price_at(store, current);
            let raw_flow = current_tp * store.volume[current];
            if current_tp > previous_tp {
                positive_flow += raw_flow;
            } else if current_tp < previous_tp {
                negative_flow += raw_flow;
            }
        }
        out[index] = mfi_value(positive_flow, negative_flow);
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_mfi_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;
    for current in store.len() - period..store.len() {
        let previous = current - 1;
        let previous_tp = typical_price_at(store, previous);
        let current_tp = typical_price_at(store, current);
        let raw_flow = current_tp * store.volume[current];
        if current_tp > previous_tp {
            positive_flow += raw_flow;
        } else if current_tp < previous_tp {
            negative_flow += raw_flow;
        }
    }
    Some(mfi_value(positive_flow, negative_flow))
}

fn roc(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return out;
    }
    for index in period..bars.len() {
        let previous = bars[index - period].close;
        out[index] = if previous == 0.0 { 0.0 } else { 100.0 * (bars[index].close / previous - 1.0) };
    }
    out
}

fn roc_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("roc:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = roc(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_roc(bars: &[Bar], period: usize) -> Option<f64> {
    roc(bars, period).last().copied().and_then(nan_to_none)
}

fn roc_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("roc:close:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
        return rc;
    }
    for index in period..store.len() {
        let previous = store.close[index - period];
        out[index] = if previous == 0.0 { 0.0 } else { 100.0 * (store.close[index] / previous - 1.0) };
    }
    let rc = Rc::new(out); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_roc_store(store: &CandleStore, period: usize) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    let previous = store.close[store.len() - 1 - period];
    Some(if previous == 0.0 {
        0.0
    } else {
        100.0 * (store.close[store.len() - 1] / previous - 1.0)
    })
}

fn aroon(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let key = format!("aroon:hl:{period}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            IndicatorOutput {
                name: "up".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "down".to_string(),
                values: nodes
                    .get(&format!("aroon:down:{period}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "oscillator".to_string(),
                values: nodes
                    .get(&format!("aroon:oscillator:{period}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }

    let mut up = vec![f64::NAN; bars.len()];
    let mut down = vec![f64::NAN; bars.len()];
    let mut oscillator = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return vec![
            IndicatorOutput {
                name: "up".to_string(),
                values: up,
            },
            IndicatorOutput {
                name: "down".to_string(),
                values: down,
            },
            IndicatorOutput {
                name: "oscillator".to_string(),
                values: oscillator,
            },
        ];
    }

    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let highest_index = window
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.high.total_cmp(&b.high))
            .map(|(offset, _)| offset)
            .unwrap_or(0);
        let lowest_index = window
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.low.total_cmp(&b.low))
            .map(|(offset, _)| offset)
            .unwrap_or(0);
        let periods_since_high = period - 1 - highest_index;
        let periods_since_low = period - 1 - lowest_index;
        let up_value = 100.0 * (period - periods_since_high) as f64 / period as f64;
        let down_value = 100.0 * (period - periods_since_low) as f64 / period as f64;
        up[index] = up_value;
        down[index] = down_value;
        oscillator[index] = up_value - down_value;
    }

    nodes.insert(key, Rc::new(up.clone()));
    nodes.insert(format!("aroon:down:{period}"), Rc::new(down.clone()));
    nodes.insert(format!("aroon:oscillator:{period}"), Rc::new(oscillator.clone()));
    vec![
        IndicatorOutput {
            name: "up".to_string(),
            values: up,
        },
        IndicatorOutput {
            name: "down".to_string(),
            values: down,
        },
        IndicatorOutput {
            name: "oscillator".to_string(),
            values: oscillator,
        },
    ]
}

fn latest_aroon(bars: &[Bar], period: usize) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = aroon(bars, period, &mut HashMap::new());
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "up", index),
        output_at_vec(&outputs, "down", index),
        output_at_vec(&outputs, "oscillator", index),
    )
}

fn aroon_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let key = format!("aroon:hl:{period}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            IndicatorOutput {
                name: "up".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "down".to_string(),
                values: nodes
                    .get(&format!("aroon:down:{period}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "oscillator".to_string(),
                values: nodes
                    .get(&format!("aroon:oscillator:{period}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }

    let mut up = vec![f64::NAN; store.len()];
    let mut down = vec![f64::NAN; store.len()];
    let mut oscillator = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return vec![
            IndicatorOutput {
                name: "up".to_string(),
                values: up,
            },
            IndicatorOutput {
                name: "down".to_string(),
                values: down,
            },
            IndicatorOutput {
                name: "oscillator".to_string(),
                values: oscillator,
            },
        ];
    }

    for index in period - 1..store.len() {
        let mut highest_index = 0;
        let mut highest = f64::NEG_INFINITY;
        let mut lowest_index = 0;
        let mut lowest = f64::INFINITY;
        for offset in 0..period {
            let window_index = index + 1 - period + offset;
            if store.high[window_index] > highest {
                highest = store.high[window_index];
                highest_index = offset;
            }
            if store.low[window_index] < lowest {
                lowest = store.low[window_index];
                lowest_index = offset;
            }
        }
        let periods_since_high = period - 1 - highest_index;
        let periods_since_low = period - 1 - lowest_index;
        let up_value = 100.0 * (period - periods_since_high) as f64 / period as f64;
        let down_value = 100.0 * (period - periods_since_low) as f64 / period as f64;
        up[index] = up_value;
        down[index] = down_value;
        oscillator[index] = up_value - down_value;
    }

    nodes.insert(key, Rc::new(up.clone()));
    nodes.insert(format!("aroon:down:{period}"), Rc::new(down.clone()));
    nodes.insert(format!("aroon:oscillator:{period}"), Rc::new(oscillator.clone()));
    vec![
        IndicatorOutput {
            name: "up".to_string(),
            values: up,
        },
        IndicatorOutput {
            name: "down".to_string(),
            values: down,
        },
        IndicatorOutput {
            name: "oscillator".to_string(),
            values: oscillator,
        },
    ]
}

fn latest_aroon_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = aroon_store(store, period, &mut HashMap::new());
    let index = store.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "up", index),
        output_at_vec(&outputs, "down", index),
        output_at_vec(&outputs, "oscillator", index),
    )
}

fn stochastic_k(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return out;
    }

    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let highest_high = window.iter().map(|bar| bar.high).fold(f64::MIN, f64::max);
        let lowest_low = window.iter().map(|bar| bar.low).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        out[index] = if range == 0.0 { 0.0 } else { 100.0 * (bars[index].close - lowest_low) / range };
    }
    out
}

fn stochastic_k_values(values: &[f64], period: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }

    for index in period - 1..values.len() {
        let window = &values[index + 1 - period..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        let highest = window.iter().copied().fold(f64::MIN, f64::max);
        let lowest = window.iter().copied().fold(f64::MAX, f64::min);
        let range = highest - lowest;
        let current = values[index];
        out[index] = if range == 0.0 { 0.0 } else { 100.0 * (current - lowest) / range };
    }
    out
}

fn smooth_series(values: &[f64], smooth: usize) -> Series {
    let mut out = vec![f64::NAN; values.len()];
    if smooth == 0 {
        return out;
    }
    for index in 0..values.len() {
        if index + 1 < smooth {
            continue;
        }
        let window = &values[index + 1 - smooth..=index];
        if window.iter().any(|value| value.is_nan()) {
            continue;
        }
        out[index] = window.iter().sum::<f64>() / smooth as f64;
    }
    out
}

fn stochastic(
    bars: &[Bar],
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let k = stochastic_k(bars, period);
    let d = smooth_series(&k, smooth);

    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:hlc:{period}:{smooth}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}

fn stochastic_k_store(store: &CandleStore, period: usize) -> Series {
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return out;
    }

    for index in period - 1..store.len() {
        let window = index + 1 - period..=index;
        let highest_high = window
            .clone()
            .map(|i| store.high[i])
            .fold(f64::MIN, f64::max);
        let lowest_low = window.map(|i| store.low[i]).fold(f64::MAX, f64::min);
        let range = highest_high - lowest_low;
        out[index] = if range == 0.0 { 0.0 } else { 100.0 * (store.close[index] - lowest_low) / range };
    }
    out
}

fn stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let k = stochastic_k_store(store, period);
    let d = smooth_series(&k, smooth);
    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:hlc:{period}:{smooth}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}

fn stoch_rsi(
    bars: &[Bar],
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let rsi = rsi_close(bars, period, nodes);
    let raw_k = stochastic_k_values(&rsi, stoch_period);
    let k = smooth_series(&raw_k, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:rsi:{period}:{stoch_period}:{smooth}:{signal}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}

fn stoch_rsi_store(
    store: &CandleStore,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let rsi = rsi_close_store(store, period, nodes);
    let raw_k = stochastic_k_values(&rsi, stoch_period);
    let k = smooth_series(&raw_k, smooth);
    let d = smooth_series(&k, signal);
    let outputs = vec![
        IndicatorOutput {
            name: "k".to_string(),
            values: k,
        },
        IndicatorOutput {
            name: "d".to_string(),
            values: d,
        },
    ];
    nodes.insert(
        format!("stoch:rsi:{period}:{stoch_period}:{smooth}:{signal}"),
        Rc::new(outputs[0].values.clone()),
    );
    outputs
}

fn latest_stochastic(
    bars: &[Bar],
    period: usize,
    smooth: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    let Some(last) = bars.last() else {
        return (None, None);
    };
    if period == 0 || bars.len() < period {
        return (None, None);
    }

    let window = &bars[bars.len() - period..];
    let highest_high = window.iter().map(|bar| bar.high).fold(f64::MIN, f64::max);
    let lowest_low = window.iter().map(|bar| bar.low).fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    let k = if range == 0.0 {
        0.0
    } else {
        100.0 * (last.close - lowest_low) / range
    };

    if smooth == 0 || bars.len() < period + smooth - 1 {
        return (Some(k), None);
    }

    let mut values = Vec::with_capacity(smooth);
    for index in bars.len() - smooth..bars.len() - 1 {
        let Some(value) = output_at(outputs, "k", index) else {
            return (Some(k), None);
        };
        values.push(value);
    }
    values.push(k);
    (Some(k), Some(values.iter().sum::<f64>() / smooth as f64))
}

fn latest_stochastic_store(
    store: &CandleStore,
    period: usize,
    smooth: usize,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None);
    }

    let start = store.len() - period;
    let highest_high = store.high[start..].iter().copied().fold(f64::MIN, f64::max);
    let lowest_low = store.low[start..].iter().copied().fold(f64::MAX, f64::min);
    let range = highest_high - lowest_low;
    let k = if range == 0.0 {
        0.0
    } else {
        100.0 * (store.close[store.len() - 1] - lowest_low) / range
    };

    if smooth == 0 || store.len() < period + smooth - 1 {
        return (Some(k), None);
    }

    let mut values = Vec::with_capacity(smooth);
    for index in store.len() - smooth..store.len() - 1 {
        let Some(value) = output_at(outputs, "k", index) else {
            return (Some(k), None);
        };
        values.push(value);
    }
    values.push(k);
    (Some(k), Some(values.iter().sum::<f64>() / smooth as f64))
}

fn latest_stoch_rsi(
    bars: &[Bar],
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
) -> (Option<f64>, Option<f64>) {
    let outputs = stoch_rsi(
        bars,
        period,
        stoch_period,
        smooth,
        signal,
        &mut HashMap::new(),
    );
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "k", index),
        output_at_vec(&outputs, "d", index),
    )
}

fn latest_stoch_rsi_store(
    store: &CandleStore,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
) -> (Option<f64>, Option<f64>) {
    let outputs = stoch_rsi_store(
        store,
        period,
        stoch_period,
        smooth,
        signal,
        &mut HashMap::new(),
    );
    let index = store.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "k", index),
        output_at_vec(&outputs, "d", index),
    )
}

fn true_range(bars: &[Bar], index: usize) -> f64 {
    if index == 0 {
        return bars[0].high - bars[0].low;
    }
    let previous_close = bars[index - 1].close;
    (bars[index].high - bars[index].low)
        .max((bars[index].high - previous_close).abs())
        .max((bars[index].low - previous_close).abs())
}

fn true_range_store(store: &CandleStore, index: usize) -> f64 {
    if index == 0 {
        return store.high[0] - store.low[0];
    }
    let previous_close = store.close[index - 1];
    (store.high[index] - store.low[index])
        .max((store.high[index] - previous_close).abs())
        .max((store.low[index] - previous_close).abs())
}

fn atr(bars: &[Bar], period: usize) -> Series {
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return out;
    }

    let mut current = (1..=period)
        .map(|index| true_range(bars, index))
        .sum::<f64>()
        / period as f64;
    out[period] = current;

    for index in period + 1..bars.len() {
        current = (current * (period - 1) as f64 + true_range(bars, index)) / period as f64;
        out[index] = current;
    }
    out
}

fn atr_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("atr:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let values = atr(bars, period);
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn atr_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("atr:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let mut out = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        let rc = Rc::new(out);
        nodes.insert(key, Rc::clone(&rc));
        return rc;
    }

    let mut current = (1..=period)
        .map(|index| true_range_store(store, index))
        .sum::<f64>()
        / period as f64;
    out[period] = current;

    for index in period + 1..store.len() {
        current = (current * (period - 1) as f64 + true_range_store(store, index)) / period as f64;
        out[index] = current;
    }
    let rc = Rc::new(out);
    nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_atr(bars: &[Bar], period: usize, output: Option<&[f64]>) -> Option<f64> {
    if period == 0 || bars.len() <= period {
        return None;
    }
    if bars.len() == period + 1 {
        return atr(bars, period).last().copied().and_then(nan_to_none);
    }

    let previous_index = bars.len() - 2;
    let previous = output
        .and_then(|values| values.get(previous_index))
        .copied().and_then(nan_to_none)
        .unwrap_or_else(|| atr(&bars[..bars.len() - 1], period)[previous_index]);
    Some((previous * (period - 1) as f64 + true_range(bars, bars.len() - 1)) / period as f64)
}

fn latest_atr_store(
    store: &CandleStore,
    period: usize,
    output: Option<&[f64]>,
) -> Option<f64> {
    if period == 0 || store.len() <= period {
        return None;
    }
    if store.len() == period + 1 {
        return atr_store(store, period, &mut HashMap::new())
            .last()
            .copied().and_then(nan_to_none);
    }

    let previous_index = store.len() - 2;
    let previous = output
        .and_then(|values| values.get(previous_index))
        .copied().and_then(nan_to_none)
        .unwrap_or_else(|| {
            let previous = CandleStore {
                time: store.time[..store.len() - 1].to_vec(),
                open: store.open[..store.len() - 1].to_vec(),
                high: store.high[..store.len() - 1].to_vec(),
                low: store.low[..store.len() - 1].to_vec(),
                close: store.close[..store.len() - 1].to_vec(),
                volume: store.volume[..store.len() - 1].to_vec(),
            };
            atr_store(&previous, period, &mut HashMap::new())[previous_index]
        });
    Some(
        (previous * (period - 1) as f64 + true_range_store(store, store.len() - 1)) / period as f64,
    )
}

fn supertrend(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let atr = atr_node(bars, period, nodes);
    let mut values = vec![f64::NAN; bars.len()];
    let mut upper_band = vec![f64::NAN; bars.len()];
    let mut lower_band = vec![f64::NAN; bars.len()];
    let mut trend = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return supertrend_outputs(values, upper_band, lower_band, trend);
    }

    for index in period..bars.len() {
        let atr_value = atr[index]; if atr_value.is_nan() {
            continue;
        };
        let hl2 = (bars[index].high + bars[index].low) / 2.0;
        let basic_upper = hl2 + multiplier * atr_value;
        let basic_lower = hl2 - multiplier * atr_value;
        let previous_close = bars[index - 1].close;

        let current_upper = if index == period {
            basic_upper
        } else {
            let previous_upper = { let __v = upper_band[index - 1]; if __v.is_nan() { basic_upper } else { __v } };
            if basic_upper < previous_upper || previous_close > previous_upper {
                basic_upper
            } else {
                previous_upper
            }
        };
        let current_lower = if index == period {
            basic_lower
        } else {
            let previous_lower = { let __v = lower_band[index - 1]; if __v.is_nan() { basic_lower } else { __v } };
            if basic_lower > previous_lower || previous_close < previous_lower {
                basic_lower
            } else {
                previous_lower
            }
        };

        upper_band[index] = current_upper;
        lower_band[index] = current_lower;

        let current_trend = if index == period {
            if bars[index].close >= hl2 {
                1.0
            } else {
                -1.0
            }
        } else {
            let previous_trend = { let __v = trend[index - 1]; if __v.is_nan() { 1.0 } else { __v } };
            if previous_trend < 0.0 {
                if bars[index].close > current_upper {
                    1.0
                } else {
                    -1.0
                }
            } else if bars[index].close < current_lower {
                -1.0
            } else {
                1.0
            }
        };
        trend[index] = current_trend;
        values[index] = if current_trend < 0.0 { current_upper } else { current_lower };
    }

    nodes.insert(format!("supertrend:{period}:{multiplier}"), Rc::new(values.clone()));
    supertrend_outputs(values, upper_band, lower_band, trend)
}

fn supertrend_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut values = vec![f64::NAN; store.len()];
    let mut upper_band = vec![f64::NAN; store.len()];
    let mut lower_band = vec![f64::NAN; store.len()];
    let mut trend = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        return supertrend_outputs(values, upper_band, lower_band, trend);
    }

    for index in period..store.len() {
        let atr_value = atr[index]; if atr_value.is_nan() {
            continue;
        };
        let hl2 = (store.high[index] + store.low[index]) / 2.0;
        let basic_upper = hl2 + multiplier * atr_value;
        let basic_lower = hl2 - multiplier * atr_value;
        let previous_close = store.close[index - 1];

        let current_upper = if index == period {
            basic_upper
        } else {
            let previous_upper = { let __v = upper_band[index - 1]; if __v.is_nan() { basic_upper } else { __v } };
            if basic_upper < previous_upper || previous_close > previous_upper {
                basic_upper
            } else {
                previous_upper
            }
        };
        let current_lower = if index == period {
            basic_lower
        } else {
            let previous_lower = { let __v = lower_band[index - 1]; if __v.is_nan() { basic_lower } else { __v } };
            if basic_lower > previous_lower || previous_close < previous_lower {
                basic_lower
            } else {
                previous_lower
            }
        };

        upper_band[index] = current_upper;
        lower_band[index] = current_lower;

        let current_trend = if index == period {
            if store.close[index] >= hl2 {
                1.0
            } else {
                -1.0
            }
        } else {
            let previous_trend = { let __v = trend[index - 1]; if __v.is_nan() { 1.0 } else { __v } };
            if previous_trend < 0.0 {
                if store.close[index] > current_upper {
                    1.0
                } else {
                    -1.0
                }
            } else if store.close[index] < current_lower {
                -1.0
            } else {
                1.0
            }
        };
        trend[index] = current_trend;
        values[index] = if current_trend < 0.0 { current_upper } else { current_lower };
    }

    nodes.insert(format!("supertrend:{period}:{multiplier}"), Rc::new(values.clone()));
    supertrend_outputs(values, upper_band, lower_band, trend)
}

fn supertrend_outputs(
    values: Series,
    upper_band: Series,
    lower_band: Series,
    trend: Series,
) -> Vec<IndicatorOutput> {
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "upper_band".to_string(),
            values: upper_band,
        },
        IndicatorOutput {
            name: "lower_band".to_string(),
            values: lower_band,
        },
        IndicatorOutput {
            name: "trend".to_string(),
            values: trend,
        },
    ]
}

fn latest_supertrend(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || bars.len() <= period {
        return (None, None, None, None);
    }
    if bars.len() == period + 1 {
        let outputs = supertrend(bars, period, multiplier, &mut HashMap::new());
        let index = bars.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "upper_band", index),
            output_at_vec(&outputs, "lower_band", index),
            output_at_vec(&outputs, "trend", index),
        );
    }

    let Some(atr_value) = latest_atr(bars, period, None) else {
        return (None, None, None, None);
    };
    let index = bars.len() - 1;
    let hl2 = (bars[index].high + bars[index].low) / 2.0;
    let basic_upper = hl2 + multiplier * atr_value;
    let basic_lower = hl2 - multiplier * atr_value;
    let previous_close = bars[index - 1].close;
    let previous_upper = output_at(outputs, "upper_band", index - 1).unwrap_or(basic_upper);
    let previous_lower = output_at(outputs, "lower_band", index - 1).unwrap_or(basic_lower);
    let previous_trend = output_at(outputs, "trend", index - 1).unwrap_or(1.0);

    let upper = if basic_upper < previous_upper || previous_close > previous_upper {
        basic_upper
    } else {
        previous_upper
    };
    let lower = if basic_lower > previous_lower || previous_close < previous_lower {
        basic_lower
    } else {
        previous_lower
    };
    let trend = if previous_trend < 0.0 {
        if bars[index].close > upper {
            1.0
        } else {
            -1.0
        }
    } else if bars[index].close < lower {
        -1.0
    } else {
        1.0
    };
    let value = if trend < 0.0 { upper } else { lower };
    (Some(value), Some(upper), Some(lower), Some(trend))
}

fn latest_supertrend_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() <= period {
        return (None, None, None, None);
    }
    if store.len() == period + 1 {
        let outputs = supertrend_store(store, period, multiplier, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "upper_band", index),
            output_at_vec(&outputs, "lower_band", index),
            output_at_vec(&outputs, "trend", index),
        );
    }

    let Some(atr_value) = latest_atr_store(store, period, None) else {
        return (None, None, None, None);
    };
    let index = store.len() - 1;
    let hl2 = (store.high[index] + store.low[index]) / 2.0;
    let basic_upper = hl2 + multiplier * atr_value;
    let basic_lower = hl2 - multiplier * atr_value;
    let previous_close = store.close[index - 1];
    let previous_upper = output_at(outputs, "upper_band", index - 1).unwrap_or(basic_upper);
    let previous_lower = output_at(outputs, "lower_band", index - 1).unwrap_or(basic_lower);
    let previous_trend = output_at(outputs, "trend", index - 1).unwrap_or(1.0);
    let upper = if basic_upper < previous_upper || previous_close > previous_upper {
        basic_upper
    } else {
        previous_upper
    };
    let lower = if basic_lower > previous_lower || previous_close < previous_lower {
        basic_lower
    } else {
        previous_lower
    };
    let trend = if previous_trend < 0.0 {
        if store.close[index] > upper {
            1.0
        } else {
            -1.0
        }
    } else if store.close[index] < lower {
        -1.0
    } else {
        1.0
    };
    let value = if trend < 0.0 { upper } else { lower };
    (Some(value), Some(upper), Some(lower), Some(trend))
}

fn parabolic_sar(
    bars: &[Bar],
    step: f64,
    max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let key = format!("psar:ohlc:{step}:{max_step}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: nodes
                    .get(&format!("psar:ep:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: nodes
                    .get(&format!("psar:af:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: nodes
                    .get(&format!("psar:trend:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }

    let mut values = vec![f64::NAN; bars.len()];
    let mut ep_values = vec![f64::NAN; bars.len()];
    let mut af_values = vec![f64::NAN; bars.len()];
    let mut trend_values = vec![f64::NAN; bars.len()];
    if bars.len() < 2 {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: ep_values,
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: af_values,
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: trend_values,
            },
        ];
    }

    let mut trend = if bars[1].close >= bars[0].close {
        1.0
    } else {
        -1.0
    };
    let mut sar = if trend > 0.0 {
        bars[0].low
    } else {
        bars[0].high
    };
    let mut ep = if trend > 0.0 {
        bars[1].high
    } else {
        bars[1].low
    };
    let mut af = step;

    values[1] = sar;
    ep_values[1] = ep;
    af_values[1] = af;
    trend_values[1] = trend;

    for index in 2..bars.len() {
        let mut next_sar = sar + af * (ep - sar);
        if trend > 0.0 {
            next_sar = next_sar.min(bars[index - 1].low).min(bars[index - 2].low);
            if bars[index].low < next_sar {
                trend = -1.0;
                next_sar = ep;
                ep = bars[index].low;
                af = step;
            } else if bars[index].high > ep {
                ep = bars[index].high;
                af = (af + step).min(max_step);
            }
        } else {
            next_sar = next_sar.max(bars[index - 1].high).max(bars[index - 2].high);
            if bars[index].high > next_sar {
                trend = 1.0;
                next_sar = ep;
                ep = bars[index].high;
                af = step;
            } else if bars[index].low < ep {
                ep = bars[index].low;
                af = (af + step).min(max_step);
            }
        }
        sar = next_sar;
        values[index] = sar;
        ep_values[index] = ep;
        af_values[index] = af;
        trend_values[index] = trend;
    }

    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(format!("psar:ep:{step}:{max_step}"), Rc::new(ep_values.clone()));
    nodes.insert(format!("psar:af:{step}:{max_step}"), Rc::new(af_values.clone()));
    nodes.insert(
        format!("psar:trend:{step}:{max_step}"),
        Rc::new(trend_values.clone()),
    );
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "ep".to_string(),
            values: ep_values,
        },
        IndicatorOutput {
            name: "af".to_string(),
            values: af_values,
        },
        IndicatorOutput {
            name: "trend".to_string(),
            values: trend_values,
        },
    ]
}

fn latest_parabolic_sar(
    bars: &[Bar],
    step: f64,
    max_step: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if bars.len() < 2 {
        return (None, None, None, None);
    }
    if bars.len() == 2 {
        let outputs = parabolic_sar(bars, step, max_step, &mut HashMap::new());
        let index = bars.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "ep", index),
            output_at_vec(&outputs, "af", index),
            output_at_vec(&outputs, "trend", index),
        );
    }

    let previous_index = bars.len() - 2;
    let previous_sar = output_at(outputs, "value", previous_index).unwrap_or_else(|| {
        parabolic_sar(&bars[..bars.len() - 1], step, max_step, &mut HashMap::new())[0].values
            [previous_index]
    });
    let previous_ep = output_at(outputs, "ep", previous_index).unwrap_or(previous_sar);
    let previous_af = output_at(outputs, "af", previous_index).unwrap_or(step);
    let previous_trend = output_at(outputs, "trend", previous_index).unwrap_or(1.0);
    let index = bars.len() - 1;

    let mut trend = previous_trend;
    let mut sar = previous_sar + previous_af * (previous_ep - previous_sar);
    let mut ep = previous_ep;
    let mut af = previous_af;

    if trend > 0.0 {
        sar = sar.min(bars[index - 1].low).min(bars[index - 2].low);
        if bars[index].low < sar {
            trend = -1.0;
            sar = previous_ep;
            ep = bars[index].low;
            af = step;
        } else if bars[index].high > ep {
            ep = bars[index].high;
            af = (af + step).min(max_step);
        }
    } else {
        sar = sar.max(bars[index - 1].high).max(bars[index - 2].high);
        if bars[index].high > sar {
            trend = 1.0;
            sar = previous_ep;
            ep = bars[index].high;
            af = step;
        } else if bars[index].low < ep {
            ep = bars[index].low;
            af = (af + step).min(max_step);
        }
    }

    (Some(sar), Some(ep), Some(af), Some(trend))
}

fn parabolic_sar_store(
    store: &CandleStore,
    step: f64,
    max_step: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let key = format!("psar:ohlc:{step}:{max_step}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: nodes
                    .get(&format!("psar:ep:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: nodes
                    .get(&format!("psar:af:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: nodes
                    .get(&format!("psar:trend:{step}:{max_step}"))
                    .map(|rc| (**rc).clone())
                    .unwrap_or_default(),
            },
        ];
    }

    let mut values = vec![f64::NAN; store.len()];
    let mut ep_values = vec![f64::NAN; store.len()];
    let mut af_values = vec![f64::NAN; store.len()];
    let mut trend_values = vec![f64::NAN; store.len()];
    if store.len() < 2 {
        return vec![
            IndicatorOutput {
                name: "value".to_string(),
                values,
            },
            IndicatorOutput {
                name: "ep".to_string(),
                values: ep_values,
            },
            IndicatorOutput {
                name: "af".to_string(),
                values: af_values,
            },
            IndicatorOutput {
                name: "trend".to_string(),
                values: trend_values,
            },
        ];
    }

    let mut trend = if store.close[1] >= store.close[0] {
        1.0
    } else {
        -1.0
    };
    let mut sar = if trend > 0.0 {
        store.low[0]
    } else {
        store.high[0]
    };
    let mut ep = if trend > 0.0 {
        store.high[1]
    } else {
        store.low[1]
    };
    let mut af = step;

    values[1] = sar;
    ep_values[1] = ep;
    af_values[1] = af;
    trend_values[1] = trend;

    for index in 2..store.len() {
        let mut next_sar = sar + af * (ep - sar);
        if trend > 0.0 {
            next_sar = next_sar.min(store.low[index - 1]).min(store.low[index - 2]);
            if store.low[index] < next_sar {
                trend = -1.0;
                next_sar = ep;
                ep = store.low[index];
                af = step;
            } else if store.high[index] > ep {
                ep = store.high[index];
                af = (af + step).min(max_step);
            }
        } else {
            next_sar = next_sar
                .max(store.high[index - 1])
                .max(store.high[index - 2]);
            if store.high[index] > next_sar {
                trend = 1.0;
                next_sar = ep;
                ep = store.high[index];
                af = step;
            } else if store.low[index] < ep {
                ep = store.low[index];
                af = (af + step).min(max_step);
            }
        }
        sar = next_sar;
        values[index] = sar;
        ep_values[index] = ep;
        af_values[index] = af;
        trend_values[index] = trend;
    }

    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(format!("psar:ep:{step}:{max_step}"), Rc::new(ep_values.clone()));
    nodes.insert(format!("psar:af:{step}:{max_step}"), Rc::new(af_values.clone()));
    nodes.insert(
        format!("psar:trend:{step}:{max_step}"),
        Rc::new(trend_values.clone()),
    );
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "ep".to_string(),
            values: ep_values,
        },
        IndicatorOutput {
            name: "af".to_string(),
            values: af_values,
        },
        IndicatorOutput {
            name: "trend".to_string(),
            values: trend_values,
        },
    ]
}

fn latest_parabolic_sar_store(
    store: &CandleStore,
    step: f64,
    max_step: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    if store.len() < 2 {
        return (None, None, None, None);
    }
    if store.len() == 2 {
        let outputs = parabolic_sar_store(store, step, max_step, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "ep", index),
            output_at_vec(&outputs, "af", index),
            output_at_vec(&outputs, "trend", index),
        );
    }

    let previous_index = store.len() - 2;
    let previous_sar = output_at(outputs, "value", previous_index).unwrap_or_else(|| {
        parabolic_sar_store(
            &CandleStore {
                time: store.time[..store.len() - 1].to_vec(),
                open: store.open[..store.len() - 1].to_vec(),
                high: store.high[..store.len() - 1].to_vec(),
                low: store.low[..store.len() - 1].to_vec(),
                close: store.close[..store.len() - 1].to_vec(),
                volume: store.volume[..store.len() - 1].to_vec(),
            },
            step,
            max_step,
            &mut HashMap::new(),
        )[0]
        .values[previous_index]
    });
    let previous_ep = output_at(outputs, "ep", previous_index).unwrap_or(previous_sar);
    let previous_af = output_at(outputs, "af", previous_index).unwrap_or(step);
    let previous_trend = output_at(outputs, "trend", previous_index).unwrap_or(1.0);
    let index = store.len() - 1;

    let mut trend = previous_trend;
    let mut sar = previous_sar + previous_af * (previous_ep - previous_sar);
    let mut ep = previous_ep;
    let mut af = previous_af;

    if trend > 0.0 {
        sar = sar.min(store.low[index - 1]).min(store.low[index - 2]);
        if store.low[index] < sar {
            trend = -1.0;
            sar = previous_ep;
            ep = store.low[index];
            af = step;
        } else if store.high[index] > ep {
            ep = store.high[index];
            af = (af + step).min(max_step);
        }
    } else {
        sar = sar.max(store.high[index - 1]).max(store.high[index - 2]);
        if store.high[index] > sar {
            trend = 1.0;
            sar = previous_ep;
            ep = store.high[index];
            af = step;
        } else if store.low[index] < ep {
            ep = store.low[index];
            af = (af + step).min(max_step);
        }
    }

    (Some(sar), Some(ep), Some(af), Some(trend))
}

fn midpoint(window: &[Bar]) -> f64 {
    let high = window
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window
        .iter()
        .map(|bar| bar.low)
        .fold(f64::INFINITY, f64::min);
    (high + low) / 2.0
}

fn midpoint_store(store: &CandleStore, start: usize, end: usize) -> f64 {
    let high = store.high[start..=end]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[start..=end]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (high + low) / 2.0
}

fn ichimoku(
    bars: &[Bar],
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let tenkan_key = format!("ichimoku:tenkan:{tenkan_period}");
    let kijun_key = format!("ichimoku:kijun:{kijun_period}");
    let senkou_a_key = format!("ichimoku:senkou_a:{tenkan_period}:{kijun_period}");
    let senkou_b_key = format!("ichimoku:senkou_b:{senkou_b_period}");
    if let Some(values) = nodes.get(&tenkan_key) {
        return vec![
            IndicatorOutput {
                name: "tenkan".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "kijun".to_string(),
                values: nodes.get(&kijun_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_a".to_string(),
                values: nodes.get(&senkou_a_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_b".to_string(),
                values: nodes.get(&senkou_b_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "chikou".to_string(),
                values: nodes.get("ichimoku:chikou").map(|rc| (**rc).clone()).unwrap_or_default(),
            },
        ];
    }

    let mut tenkan = vec![f64::NAN; bars.len()];
    let mut kijun = vec![f64::NAN; bars.len()];
    let mut senkou_a = vec![f64::NAN; bars.len()];
    let mut senkou_b = vec![f64::NAN; bars.len()];
    let chikou: Vec<_> = bars.iter().map(|bar| bar.close).collect();

    for index in 0..bars.len() {
        if index + 1 >= tenkan_period {
            tenkan[index] = midpoint(&bars[index + 1 - tenkan_period..=index]);
        }
        if index + 1 >= kijun_period {
            kijun[index] = midpoint(&bars[index + 1 - kijun_period..=index]);
        }
        let tenkan_value = tenkan[index]; let kijun_value = kijun[index]; if !tenkan_value.is_nan() && !kijun_value.is_nan() {
            senkou_a[index] = (tenkan_value + kijun_value) / 2.0;
        }
        if index + 1 >= senkou_b_period {
            senkou_b[index] = midpoint(&bars[index + 1 - senkou_b_period..=index]);
        }
    }

    nodes.insert(tenkan_key, Rc::new(tenkan.clone()));
    nodes.insert(kijun_key, Rc::new(kijun.clone()));
    nodes.insert(senkou_a_key, Rc::new(senkou_a.clone()));
    nodes.insert(senkou_b_key, Rc::new(senkou_b.clone()));
    nodes.insert("ichimoku:chikou".to_string(), Rc::new(chikou.clone()));
    vec![
        IndicatorOutput {
            name: "tenkan".to_string(),
            values: tenkan,
        },
        IndicatorOutput {
            name: "kijun".to_string(),
            values: kijun,
        },
        IndicatorOutput {
            name: "senkou_a".to_string(),
            values: senkou_a,
        },
        IndicatorOutput {
            name: "senkou_b".to_string(),
            values: senkou_b,
        },
        IndicatorOutput {
            name: "chikou".to_string(),
            values: chikou,
        },
    ]
}

fn ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let tenkan_key = format!("ichimoku:tenkan:{tenkan_period}");
    let kijun_key = format!("ichimoku:kijun:{kijun_period}");
    let senkou_a_key = format!("ichimoku:senkou_a:{tenkan_period}:{kijun_period}");
    let senkou_b_key = format!("ichimoku:senkou_b:{senkou_b_period}");
    if let Some(values) = nodes.get(&tenkan_key) {
        return vec![
            IndicatorOutput {
                name: "tenkan".to_string(),
                values: (**values).clone(),
            },
            IndicatorOutput {
                name: "kijun".to_string(),
                values: nodes.get(&kijun_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_a".to_string(),
                values: nodes.get(&senkou_a_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "senkou_b".to_string(),
                values: nodes.get(&senkou_b_key).map(|rc| (**rc).clone()).unwrap_or_default(),
            },
            IndicatorOutput {
                name: "chikou".to_string(),
                values: nodes.get("ichimoku:chikou").map(|rc| (**rc).clone()).unwrap_or_default(),
            },
        ];
    }

    let mut tenkan = vec![f64::NAN; store.len()];
    let mut kijun = vec![f64::NAN; store.len()];
    let mut senkou_a = vec![f64::NAN; store.len()];
    let mut senkou_b = vec![f64::NAN; store.len()];
    let chikou: Vec<_> = store.close.iter().copied().collect();

    for index in 0..store.len() {
        if index + 1 >= tenkan_period {
            tenkan[index] = midpoint_store(store, index + 1 - tenkan_period, index);
        }
        if index + 1 >= kijun_period {
            kijun[index] = midpoint_store(store, index + 1 - kijun_period, index);
        }
        let tenkan_value = tenkan[index]; let kijun_value = kijun[index]; if !tenkan_value.is_nan() && !kijun_value.is_nan() {
            senkou_a[index] = (tenkan_value + kijun_value) / 2.0;
        }
        if index + 1 >= senkou_b_period {
            senkou_b[index] = midpoint_store(store, index + 1 - senkou_b_period, index);
        }
    }

    nodes.insert(tenkan_key, Rc::new(tenkan.clone()));
    nodes.insert(kijun_key, Rc::new(kijun.clone()));
    nodes.insert(senkou_a_key, Rc::new(senkou_a.clone()));
    nodes.insert(senkou_b_key, Rc::new(senkou_b.clone()));
    nodes.insert("ichimoku:chikou".to_string(), Rc::new(chikou.clone()));
    vec![
        IndicatorOutput {
            name: "tenkan".to_string(),
            values: tenkan,
        },
        IndicatorOutput {
            name: "kijun".to_string(),
            values: kijun,
        },
        IndicatorOutput {
            name: "senkou_a".to_string(),
            values: senkou_a,
        },
        IndicatorOutput {
            name: "senkou_b".to_string(),
            values: senkou_b,
        },
        IndicatorOutput {
            name: "chikou".to_string(),
            values: chikou,
        },
    ]
}

fn latest_ichimoku(
    bars: &[Bar],
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let outputs = ichimoku(
        bars,
        tenkan_period,
        kijun_period,
        senkou_b_period,
        &mut HashMap::new(),
    );
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "tenkan", index),
        output_at_vec(&outputs, "kijun", index),
        output_at_vec(&outputs, "senkou_a", index),
        output_at_vec(&outputs, "senkou_b", index),
        output_at_vec(&outputs, "chikou", index),
    )
}

fn latest_ichimoku_store(
    store: &CandleStore,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let outputs = ichimoku_store(
        store,
        tenkan_period,
        kijun_period,
        senkou_b_period,
        &mut HashMap::new(),
    );
    let index = store.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "tenkan", index),
        output_at_vec(&outputs, "kijun", index),
        output_at_vec(&outputs, "senkou_a", index),
        output_at_vec(&outputs, "senkou_b", index),
        output_at_vec(&outputs, "chikou", index),
    )
}

fn pivot_points(bars: &[Bar], nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut pp = vec![f64::NAN; bars.len()];
    let mut r1 = vec![f64::NAN; bars.len()];
    let mut s1 = vec![f64::NAN; bars.len()];
    let mut r2 = vec![f64::NAN; bars.len()];
    let mut s2 = vec![f64::NAN; bars.len()];
    for index in 1..bars.len() {
        let previous = &bars[index - 1];
        let pivot = (previous.high + previous.low + previous.close) / 3.0;
        let range = previous.high - previous.low;
        pp[index] = pivot;
        r1[index] = 2.0 * pivot - previous.low;
        s1[index] = 2.0 * pivot - previous.high;
        r2[index] = pivot + range;
        s2[index] = pivot - range;
    }
    nodes.insert("pivot:pp".to_string(), Rc::new(pp.clone()));
    nodes.insert("pivot:r1".to_string(), Rc::new(r1.clone()));
    nodes.insert("pivot:s1".to_string(), Rc::new(s1.clone()));
    nodes.insert("pivot:r2".to_string(), Rc::new(r2.clone()));
    nodes.insert("pivot:s2".to_string(), Rc::new(s2.clone()));
    vec![
        IndicatorOutput {
            name: "pp".to_string(),
            values: pp,
        },
        IndicatorOutput {
            name: "r1".to_string(),
            values: r1,
        },
        IndicatorOutput {
            name: "s1".to_string(),
            values: s1,
        },
        IndicatorOutput {
            name: "r2".to_string(),
            values: r2,
        },
        IndicatorOutput {
            name: "s2".to_string(),
            values: s2,
        },
    ]
}

fn pivot_points_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut pp = vec![f64::NAN; store.len()];
    let mut r1 = vec![f64::NAN; store.len()];
    let mut s1 = vec![f64::NAN; store.len()];
    let mut r2 = vec![f64::NAN; store.len()];
    let mut s2 = vec![f64::NAN; store.len()];
    for index in 1..store.len() {
        let pivot = (store.high[index - 1] + store.low[index - 1] + store.close[index - 1]) / 3.0;
        let range = store.high[index - 1] - store.low[index - 1];
        pp[index] = pivot;
        r1[index] = 2.0 * pivot - store.low[index - 1];
        s1[index] = 2.0 * pivot - store.high[index - 1];
        r2[index] = pivot + range;
        s2[index] = pivot - range;
    }
    nodes.insert("pivot:pp".to_string(), Rc::new(pp.clone()));
    nodes.insert("pivot:r1".to_string(), Rc::new(r1.clone()));
    nodes.insert("pivot:s1".to_string(), Rc::new(s1.clone()));
    nodes.insert("pivot:r2".to_string(), Rc::new(r2.clone()));
    nodes.insert("pivot:s2".to_string(), Rc::new(s2.clone()));
    vec![
        IndicatorOutput {
            name: "pp".to_string(),
            values: pp,
        },
        IndicatorOutput {
            name: "r1".to_string(),
            values: r1,
        },
        IndicatorOutput {
            name: "s1".to_string(),
            values: s1,
        },
        IndicatorOutput {
            name: "r2".to_string(),
            values: r2,
        },
        IndicatorOutput {
            name: "s2".to_string(),
            values: s2,
        },
    ]
}

fn latest_pivot_points(
    bars: &[Bar],
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let previous = match bars.iter().nth_back(1) {
        Some(previous) => previous,
        None => return (None, None, None, None, None),
    };
    let pivot = (previous.high + previous.low + previous.close) / 3.0;
    let range = previous.high - previous.low;
    (
        Some(pivot),
        Some(2.0 * pivot - previous.low),
        Some(2.0 * pivot - previous.high),
        Some(pivot + range),
        Some(pivot - range),
    )
}

fn latest_pivot_points_store(
    store: &CandleStore,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    if store.len() < 2 {
        return (None, None, None, None, None);
    }
    let index = store.len() - 2;
    let pivot = (store.high[index] + store.low[index] + store.close[index]) / 3.0;
    let range = store.high[index] - store.low[index];
    (
        Some(pivot),
        Some(2.0 * pivot - store.low[index]),
        Some(2.0 * pivot - store.high[index]),
        Some(pivot + range),
        Some(pivot - range),
    )
}

fn keltner(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = ema_close(bars, period, nodes);
    let atr = atr_node(bars, period, nodes);
    let mut upper = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];

    for index in 0..bars.len() {
        let mid = middle[index]; let atr_value = atr[index]; if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
    }

    let outputs = vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ];

    for output in &outputs {
        nodes.insert(
            format!("keltner:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_keltner(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_ema(
        bars,
        period,
        outputs.get("middle"),
    );
    let atr = latest_atr(bars, period, None);

    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}

fn starc(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = sma_close(bars, period, nodes);
    let atr = atr_node(bars, period, nodes);
    let mut upper = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];

    for index in 0..bars.len() {
        let mid = middle[index]; let atr_value = atr[index]; if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
    }

    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("starc:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_starc(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma(bars, period);
    let atr = latest_atr(bars, period, None);

    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}

fn keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = rc_into_owned(ema_close_store(store, period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for index in 0..store.len() {
        let mid = middle[index]; let atr_value = atr[index]; if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
    }
    let outputs = vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ];
    for output in &outputs {
        nodes.insert(
            format!("keltner:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_keltner_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_ema_store(
        store,
        period,
        outputs.get("middle"),
    );
    let atr = latest_atr_store(store, period, None);
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}

fn starc_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    let atr = rc_into_owned(atr_store(store, period, nodes));
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    for index in 0..store.len() {
        let mid = middle[index]; let atr_value = atr[index]; if mid.is_nan() || atr_value.is_nan() {
            continue;
        };
        upper[index] = mid + multiplier * atr_value;
        lower[index] = mid - multiplier * atr_value;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("starc:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_starc_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let middle = latest_sma_store(store, period);
    let atr = latest_atr_store(store, period, None);
    match (middle, atr) {
        (Some(middle), Some(atr)) => (
            Some(middle + multiplier * atr),
            Some(middle),
            Some(middle - multiplier * atr),
        ),
        _ => (None, middle, None),
    }
}

fn donchian(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; bars.len()];
    let mut middle = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }

    for index in period - 1..bars.len() {
        let window = &bars[index + 1 - period..=index];
        let high = window
            .iter()
            .map(|bar| bar.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let low = window
            .iter()
            .map(|bar| bar.low)
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }

    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("donchian:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_donchian(bars: &[Bar], period: usize) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || bars.len() < period {
        return (None, None, None);
    }
    let window = &bars[bars.len() - period..];
    let high = window
        .iter()
        .map(|bar| bar.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let low = window
        .iter()
        .map(|bar| bar.low)
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}

fn donchian_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }

    for index in period - 1..store.len() {
        let high = store.high[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = store.low[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }

    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("donchian:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_donchian_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let high = store.high[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}

fn price_channel(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let outputs = donchian(bars, period, &mut HashMap::new());
    for output in &outputs {
        nodes.insert(
            format!("price_channel:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_price_channel(bars: &[Bar], period: usize) -> (Option<f64>, Option<f64>, Option<f64>) {
    latest_donchian(bars, period)
}

fn price_channel_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut middle = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return bollinger_outputs(upper, middle, lower);
    }
    for index in period - 1..store.len() {
        let high = store.high[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let low = store.low[index + 1 - period..=index]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        upper[index] = high;
        middle[index] = (high + low) / 2.0;
        lower[index] = low;
    }
    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("price_channel:{}:{}", output.name, period),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_price_channel_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let high = store.high[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let low = store.low[store.len() - period..]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    (Some(high), Some((high + low) / 2.0), Some(low))
}

fn directional_movement(bars: &[Bar], index: usize) -> (f64, f64) {
    if index == 0 {
        return (0.0, 0.0);
    }
    let up_move = bars[index].high - bars[index - 1].high;
    let down_move = bars[index - 1].low - bars[index].low;
    let plus_dm = if up_move > down_move && up_move > 0.0 {
        up_move
    } else {
        0.0
    };
    let minus_dm = if down_move > up_move && down_move > 0.0 {
        down_move
    } else {
        0.0
    };
    (plus_dm, minus_dm)
}

fn directional_movement_store(store: &CandleStore, index: usize) -> (f64, f64) {
    if index == 0 {
        return (0.0, 0.0);
    }
    let up_move = store.high[index] - store.high[index - 1];
    let down_move = store.low[index - 1] - store.low[index];
    let plus_dm = if up_move > down_move && up_move > 0.0 {
        up_move
    } else {
        0.0
    };
    let minus_dm = if down_move > up_move && down_move > 0.0 {
        down_move
    } else {
        0.0
    };
    (plus_dm, minus_dm)
}

fn adx(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let key = format!("adx:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return adx_outputs(
            (**values).clone(),
            nodes
                .get(&format!("adx:plus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:tr_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:plus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:dx:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
        );
    }

    let mut values = vec![f64::NAN; bars.len()];
    let mut plus_di_values = vec![f64::NAN; bars.len()];
    let mut minus_di_values = vec![f64::NAN; bars.len()];
    let mut tr_avg_values = vec![f64::NAN; bars.len()];
    let mut plus_dm_avg_values = vec![f64::NAN; bars.len()];
    let mut minus_dm_avg_values = vec![f64::NAN; bars.len()];
    let mut dx_values = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return adx_outputs(
            values,
            plus_di_values,
            minus_di_values,
            tr_avg_values,
            plus_dm_avg_values,
            minus_dm_avg_values,
            dx_values,
        );
    }

    let mut tr_avg = (1..=period)
        .map(|index| true_range(bars, index))
        .sum::<f64>()
        / period as f64;
    let mut plus_dm_avg = (1..=period)
        .map(|index| directional_movement(bars, index).0)
        .sum::<f64>()
        / period as f64;
    let mut minus_dm_avg = (1..=period)
        .map(|index| directional_movement(bars, index).1)
        .sum::<f64>()
        / period as f64;
    plus_di_values[period] = di_value(tr_avg, plus_dm_avg);
    minus_di_values[period] = di_value(tr_avg, minus_dm_avg);
    tr_avg_values[period] = tr_avg;
    plus_dm_avg_values[period] = plus_dm_avg;
    minus_dm_avg_values[period] = minus_dm_avg;
    dx_values[period] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);

    for index in period + 1..bars.len() {
        let (plus_dm, minus_dm) = directional_movement(bars, index);
        tr_avg = (tr_avg * (period - 1) as f64 + true_range(bars, index)) / period as f64;
        plus_dm_avg = (plus_dm_avg * (period - 1) as f64 + plus_dm) / period as f64;
        minus_dm_avg = (minus_dm_avg * (period - 1) as f64 + minus_dm) / period as f64;
        plus_di_values[index] = di_value(tr_avg, plus_dm_avg);
        minus_di_values[index] = di_value(tr_avg, minus_dm_avg);
        tr_avg_values[index] = tr_avg;
        plus_dm_avg_values[index] = plus_dm_avg;
        minus_dm_avg_values[index] = minus_dm_avg;
        dx_values[index] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    }

    if bars.len() > period * 2 {
        let mut adx = dx_values[period + 1..=period * 2]
            .iter()
            .copied()
            .sum::<f64>()
            / period as f64;
        values[period * 2] = adx;
        for index in period * 2 + 1..bars.len() {
            adx = (adx * (period - 1) as f64 + { let __v = dx_values[index]; if __v.is_nan() { 0.0 } else { __v } }) / period as f64;
            values[index] = adx;
        }
    }

    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(format!("adx:plus_di:{period}"), Rc::new(plus_di_values.clone()));
    nodes.insert(format!("adx:minus_di:{period}"), Rc::new(minus_di_values.clone()));
    nodes.insert(format!("adx:tr_avg:{period}"), Rc::new(tr_avg_values.clone()));
    nodes.insert(
        format!("adx:plus_dm_avg:{period}"),
        Rc::new(plus_dm_avg_values.clone()),
    );
    nodes.insert(
        format!("adx:minus_dm_avg:{period}"),
        Rc::new(minus_dm_avg_values.clone()),
    );
    nodes.insert(format!("adx:dx:{period}"), Rc::new(dx_values.clone()));
    adx_outputs(
        values,
        plus_di_values,
        minus_di_values,
        tr_avg_values,
        plus_dm_avg_values,
        minus_dm_avg_values,
        dx_values,
    )
}

fn adx_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let key = format!("adx:ohlc:{period}");
    if let Some(values) = nodes.get(&key) {
        return adx_outputs(
            (**values).clone(),
            nodes
                .get(&format!("adx:plus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_di:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:tr_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:plus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:minus_dm_avg:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
            nodes
                .get(&format!("adx:dx:{period}"))
                .map(|rc| (**rc).clone())
                .unwrap_or_default(),
        );
    }

    let mut values = vec![f64::NAN; store.len()];
    let mut plus_di_values = vec![f64::NAN; store.len()];
    let mut minus_di_values = vec![f64::NAN; store.len()];
    let mut tr_avg_values = vec![f64::NAN; store.len()];
    let mut plus_dm_avg_values = vec![f64::NAN; store.len()];
    let mut minus_dm_avg_values = vec![f64::NAN; store.len()];
    let mut dx_values = vec![f64::NAN; store.len()];
    if period == 0 || store.len() <= period {
        return adx_outputs(
            values,
            plus_di_values,
            minus_di_values,
            tr_avg_values,
            plus_dm_avg_values,
            minus_dm_avg_values,
            dx_values,
        );
    }

    let mut tr_avg = (1..=period)
        .map(|index| true_range_store(store, index))
        .sum::<f64>()
        / period as f64;
    let mut plus_dm_avg = (1..=period)
        .map(|index| directional_movement_store(store, index).0)
        .sum::<f64>()
        / period as f64;
    let mut minus_dm_avg = (1..=period)
        .map(|index| directional_movement_store(store, index).1)
        .sum::<f64>()
        / period as f64;
    plus_di_values[period] = di_value(tr_avg, plus_dm_avg);
    minus_di_values[period] = di_value(tr_avg, minus_dm_avg);
    tr_avg_values[period] = tr_avg;
    plus_dm_avg_values[period] = plus_dm_avg;
    minus_dm_avg_values[period] = minus_dm_avg;
    dx_values[period] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);

    for index in period + 1..store.len() {
        let (plus_dm, minus_dm) = directional_movement_store(store, index);
        tr_avg = (tr_avg * (period - 1) as f64 + true_range_store(store, index)) / period as f64;
        plus_dm_avg = (plus_dm_avg * (period - 1) as f64 + plus_dm) / period as f64;
        minus_dm_avg = (minus_dm_avg * (period - 1) as f64 + minus_dm) / period as f64;
        plus_di_values[index] = di_value(tr_avg, plus_dm_avg);
        minus_di_values[index] = di_value(tr_avg, minus_dm_avg);
        tr_avg_values[index] = tr_avg;
        plus_dm_avg_values[index] = plus_dm_avg;
        minus_dm_avg_values[index] = minus_dm_avg;
        dx_values[index] = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    }

    if store.len() > period * 2 {
        let mut adx = dx_values[period + 1..=period * 2]
            .iter()
            .copied()
            .sum::<f64>()
            / period as f64;
        values[period * 2] = adx;
        for index in period * 2 + 1..store.len() {
            adx = (adx * (period - 1) as f64 + { let __v = dx_values[index]; if __v.is_nan() { 0.0 } else { __v } }) / period as f64;
            values[index] = adx;
        }
    }

    nodes.insert(key, Rc::new(values.clone()));
    nodes.insert(format!("adx:plus_di:{period}"), Rc::new(plus_di_values.clone()));
    nodes.insert(format!("adx:minus_di:{period}"), Rc::new(minus_di_values.clone()));
    nodes.insert(format!("adx:tr_avg:{period}"), Rc::new(tr_avg_values.clone()));
    nodes.insert(
        format!("adx:plus_dm_avg:{period}"),
        Rc::new(plus_dm_avg_values.clone()),
    );
    nodes.insert(
        format!("adx:minus_dm_avg:{period}"),
        Rc::new(minus_dm_avg_values.clone()),
    );
    nodes.insert(format!("adx:dx:{period}"), Rc::new(dx_values.clone()));
    adx_outputs(
        values,
        plus_di_values,
        minus_di_values,
        tr_avg_values,
        plus_dm_avg_values,
        minus_dm_avg_values,
        dx_values,
    )
}

fn di_value(tr_avg: f64, dm_avg: f64) -> f64 {
    if tr_avg == 0.0 {
        0.0
    } else {
        100.0 * dm_avg / tr_avg
    }
}

fn dx_value(tr_avg: f64, plus_dm_avg: f64, minus_dm_avg: f64) -> f64 {
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let sum = plus_di + minus_di;
    if sum == 0.0 {
        0.0
    } else {
        100.0 * (plus_di - minus_di).abs() / sum
    }
}

fn adx_outputs(
    values: Series,
    plus_di: Series,
    minus_di: Series,
    tr_avg: Series,
    plus_dm_avg: Series,
    minus_dm_avg: Series,
    dx: Series,
) -> Vec<IndicatorOutput> {
    vec![
        IndicatorOutput {
            name: "value".to_string(),
            values,
        },
        IndicatorOutput {
            name: "plus_di".to_string(),
            values: plus_di,
        },
        IndicatorOutput {
            name: "minus_di".to_string(),
            values: minus_di,
        },
        IndicatorOutput {
            name: "tr_avg".to_string(),
            values: tr_avg,
        },
        IndicatorOutput {
            name: "plus_dm_avg".to_string(),
            values: plus_dm_avg,
        },
        IndicatorOutput {
            name: "minus_dm_avg".to_string(),
            values: minus_dm_avg,
        },
        IndicatorOutput {
            name: "dx".to_string(),
            values: dx,
        },
    ]
}

fn latest_adx(
    bars: &[Bar],
    period: usize,
    outputs: &IndicatorArena,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    if period == 0 || bars.len() <= period {
        return (None, None, None, None, None, None, None);
    }
    if bars.len() <= period * 2 {
        let outputs = adx(bars, period, &mut HashMap::new());
        let index = bars.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "plus_di", index),
            output_at_vec(&outputs, "minus_di", index),
            output_at_vec(&outputs, "tr_avg", index),
            output_at_vec(&outputs, "plus_dm_avg", index),
            output_at_vec(&outputs, "minus_dm_avg", index),
            output_at_vec(&outputs, "dx", index),
        );
    }

    let previous_index = bars.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "tr_avg", previous_index).is_some()
        && output_at(outputs, "plus_dm_avg", previous_index).is_some()
        && output_at(outputs, "minus_dm_avg", previous_index).is_some()
        && output_at(outputs, "dx", previous_index).is_some()
    {
        outputs
    } else {
        previous_outputs = IndicatorArena::from_outputs(adx(&bars[..bars.len() - 1], period, &mut HashMap::new()));
        &previous_outputs
    };

    let tr_avg = (output_at(source_outputs, "tr_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + true_range(bars, bars.len() - 1))
        / period as f64;
    let (plus_dm, minus_dm) = directional_movement(bars, bars.len() - 1);
    let plus_dm_avg = (output_at(source_outputs, "plus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + plus_dm)
        / period as f64;
    let minus_dm_avg = (output_at(source_outputs, "minus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + minus_dm)
        / period as f64;
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let dx = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    let value = if bars.len() == period * 2 + 1 {
        let prior_dx_sum = (period + 1..=previous_index)
            .map(|index| output_at(source_outputs, "dx", index).unwrap_or(0.0))
            .sum::<f64>();
        Some((prior_dx_sum + dx) / period as f64)
    } else {
        let previous_adx = output_at(source_outputs, "value", previous_index).unwrap_or(0.0);
        Some((previous_adx * (period - 1) as f64 + dx) / period as f64)
    };
    (
        value,
        Some(plus_di),
        Some(minus_di),
        Some(tr_avg),
        Some(plus_dm_avg),
        Some(minus_dm_avg),
        Some(dx),
    )
}

fn latest_adx_store(
    store: &CandleStore,
    period: usize,
    outputs: &IndicatorArena,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    if period == 0 || store.len() <= period {
        return (None, None, None, None, None, None, None);
    }
    if store.len() <= period * 2 {
        let outputs = adx_store(store, period, &mut HashMap::new());
        let index = store.len() - 1;
        return (
            output_at_vec(&outputs, "value", index),
            output_at_vec(&outputs, "plus_di", index),
            output_at_vec(&outputs, "minus_di", index),
            output_at_vec(&outputs, "tr_avg", index),
            output_at_vec(&outputs, "plus_dm_avg", index),
            output_at_vec(&outputs, "minus_dm_avg", index),
            output_at_vec(&outputs, "dx", index),
        );
    }

    let previous_index = store.len() - 2;
    let previous_outputs;
    let source_outputs = if output_at(outputs, "tr_avg", previous_index).is_some()
        && output_at(outputs, "plus_dm_avg", previous_index).is_some()
        && output_at(outputs, "minus_dm_avg", previous_index).is_some()
        && output_at(outputs, "dx", previous_index).is_some()
    {
        outputs
    } else {
        let previous = CandleStore {
            time: store.time[..store.len() - 1].to_vec(),
            open: store.open[..store.len() - 1].to_vec(),
            high: store.high[..store.len() - 1].to_vec(),
            low: store.low[..store.len() - 1].to_vec(),
            close: store.close[..store.len() - 1].to_vec(),
            volume: store.volume[..store.len() - 1].to_vec(),
        };
        previous_outputs = IndicatorArena::from_outputs(adx_store(&previous, period, &mut HashMap::new()));
        &previous_outputs
    };

    let tr_avg = (output_at(source_outputs, "tr_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + true_range_store(store, store.len() - 1))
        / period as f64;
    let (plus_dm, minus_dm) = directional_movement_store(store, store.len() - 1);
    let plus_dm_avg = (output_at(source_outputs, "plus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + plus_dm)
        / period as f64;
    let minus_dm_avg = (output_at(source_outputs, "minus_dm_avg", previous_index).unwrap_or(0.0)
        * (period - 1) as f64
        + minus_dm)
        / period as f64;
    let plus_di = di_value(tr_avg, plus_dm_avg);
    let minus_di = di_value(tr_avg, minus_dm_avg);
    let dx = dx_value(tr_avg, plus_dm_avg, minus_dm_avg);
    let value = if store.len() == period * 2 + 1 {
        let prior_dx_sum = (period + 1..=previous_index)
            .map(|index| output_at(source_outputs, "dx", index).unwrap_or(0.0))
            .sum::<f64>();
        Some((prior_dx_sum + dx) / period as f64)
    } else {
        let previous_adx = output_at(source_outputs, "value", previous_index).unwrap_or(0.0);
        Some((previous_adx * (period - 1) as f64 + dx) / period as f64)
    };
    (
        value,
        Some(plus_di),
        Some(minus_di),
        Some(tr_avg),
        Some(plus_dm_avg),
        Some(minus_dm_avg),
        Some(dx),
    )
}

fn bollinger(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; bars.len()];
    let mut lower = vec![f64::NAN; bars.len()];
    let middle = sma_close(bars, period, nodes);
    if period == 0 {
        return bollinger_outputs(upper, middle, lower);
    }

    for i in period - 1..bars.len() {
        let window = &bars[i + 1 - period..=i];
        let mean = middle[i]; if mean.is_nan() {
            continue;
        };
        let variance = window
            .iter()
            .map(|bar| {
                let diff = bar.close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let band = variance.sqrt() * multiplier;
        upper[i] = mean + band;
        lower[i] = mean - band;
    }

    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("bb:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let mut upper = vec![f64::NAN; store.len()];
    let mut lower = vec![f64::NAN; store.len()];
    let middle = rc_into_owned(sma_close_store(store, period, nodes));
    if period == 0 {
        return bollinger_outputs(upper, middle, lower);
    }

    for index in period - 1..store.len() {
        let mean = middle[index]; if mean.is_nan() {
            continue;
        };
        let variance = store.close[index + 1 - period..=index]
            .iter()
            .map(|close| {
                let diff = close - mean;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;
        let band = variance.sqrt() * multiplier;
        upper[index] = mean + band;
        lower[index] = mean - band;
    }

    let outputs = bollinger_outputs(upper, middle, lower);
    for output in &outputs {
        nodes.insert(
            format!("bb:{}:{}:{}", output.name, period, multiplier),
            Rc::new(output.values.clone()),
        );
    }
    outputs
}

fn latest_bollinger(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || bars.len() < period {
        return (None, None, None);
    }
    let window = &bars[bars.len() - period..];
    let mean = window.iter().map(|bar| bar.close).sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|bar| {
            let diff = bar.close - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    let band = variance.sqrt() * multiplier;
    (Some(mean + band), Some(mean), Some(mean - band))
}

fn latest_bollinger_store(
    store: &CandleStore,
    period: usize,
    multiplier: f64,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if period == 0 || store.len() < period {
        return (None, None, None);
    }
    let window = &store.close[store.len() - period..];
    let mean = window.iter().sum::<f64>() / period as f64;
    let variance = window
        .iter()
        .map(|close| {
            let diff = close - mean;
            diff * diff
        })
        .sum::<f64>()
        / period as f64;
    let band = variance.sqrt() * multiplier;
    (Some(mean + band), Some(mean), Some(mean - band))
}

fn bollinger_outputs(upper: Series, middle: Series, lower: Series) -> Vec<IndicatorOutput> {
    vec![
        IndicatorOutput {
            name: "upper".to_string(),
            values: upper,
        },
        IndicatorOutput {
            name: "middle".to_string(),
            values: middle,
        },
        IndicatorOutput {
            name: "lower".to_string(),
            values: lower,
        },
    ]
}

fn macd(bars: &[Bar], params: MacdParams, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let fast = ema_close(bars, params.fast, nodes);
    let slow = ema_close(bars, params.slow, nodes);
    let macd_line: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| *fast - *slow)
        .collect();
    let signal = ema_values(
        macd_line.iter().copied(),
        params.signal,
    );
    let histogram: Vec<_> = macd_line
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| *macd - *signal)
        .collect();

    vec![
        IndicatorOutput {
            name: "macd".to_string(),
            values: macd_line,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow,
        },
    ]
}

fn ppo(bars: &[Bar], params: MacdParams, nodes: &mut NodeCache) -> Vec<IndicatorOutput> {
    let fast = ema_close(bars, params.fast, nodes);
    let slow = ema_close(bars, params.slow, nodes);
    let ppo_line: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() && *slow != 0.0 => 100.0 * (*fast - *slow) / *slow,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&ppo_line, params.signal);
    let histogram: Vec<_> = ppo_line
        .iter()
        .zip(signal.iter())
        .map(|(ppo, signal)| match (ppo, signal) {
            (ppo, signal) if !ppo.is_nan() && !signal.is_nan() => *ppo - *signal,
            _ => f64::NAN,
        })
        .collect();

    vec![
        IndicatorOutput {
            name: "ppo".to_string(),
            values: ppo_line.clone(),
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow,
        },
    ]
}

fn chaikin_oscillator(bars: &[Bar], params: MacdParams) -> Series {
    let adl = adl(bars);
    let fast = ema_series(&adl, params.fast);
    let slow = ema_series(&adl, params.slow);
    fast.iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect()
}

fn chaikin_oscillator_node(bars: &[Bar], params: MacdParams, nodes: &mut NodeCache) -> Series {
    let key = format!("chaikin:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let adl_values = adl_node(bars, nodes);
    let fast = ema_series(&adl_values, params.fast);
    let slow = ema_series(&adl_values, params.slow);
    let values: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn chaikin_volatility(bars: &[Bar], period: usize) -> Series {
    let ranges: Vec<_> = bars.iter().map(|bar| bar.high - bar.low).collect();
    let ema = ema_series(&ranges, period);
    let mut out = vec![f64::NAN; bars.len()];
    if period == 0 || bars.len() <= period {
        return out;
    }
    for index in period..bars.len() {
        match (ema[index], ema[index - period]) {
            (current, previous) if !current.is_nan() && !previous.is_nan() && previous != 0.0 => {
                out[index] = 100.0 * (current - previous) / previous;
            }
            (current2, previous2) if !current2.is_nan() && !previous2.is_nan() => out[index] = 0.0,
            _ => {}
        }
    }
    out
}

fn chaikin_volatility_node(bars: &[Bar], period: usize, nodes: &mut NodeCache) -> Series {
    let key = format!("cvol:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let ema_key = format!("cvol:ema:{period}");
    let ranges: Vec<_> = bars.iter().map(|bar| bar.high - bar.low).collect();
    let ema = ema_series(&ranges, period);
    nodes.insert(ema_key, Rc::new(ema.clone()));
    let mut values = vec![f64::NAN; bars.len()];
    if period != 0 && bars.len() > period {
        for index in period..bars.len() {
            match (ema[index], ema[index - period]) {
                (current, previous) if !current.is_nan() && !previous.is_nan() && previous != 0.0 => {
                    values[index] = 100.0 * (current - previous) / previous;
                }
                (current2, previous2) if !current2.is_nan() && !previous2.is_nan() => values[index] = 0.0,
                _ => {}
            }
        }
    }
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_chaikin_volatility(bars: &[Bar], period: usize) -> Option<f64> {
    chaikin_volatility(bars, period).last().copied().and_then(nan_to_none)
}

fn chaikin_volatility_store(store: &CandleStore, period: usize, nodes: &mut NodeCache) -> RcSeries {
    let key = format!("cvol:value:{period}");
    if let Some(values) = nodes.get(&key) {
        return Rc::clone(values);
    }
    let ema_key = format!("cvol:ema:{period}");
    let ranges: Vec<_> = (0..store.len())
        .map(|index| store.high[index] - store.low[index])
        .collect();
    let ema = ema_series(&ranges, period);
    nodes.insert(ema_key, Rc::new(ema.clone()));
    let mut values = vec![f64::NAN; store.len()];
    if period != 0 && store.len() > period {
        for index in period..store.len() {
            match (ema[index], ema[index - period]) {
                (current, previous) if !current.is_nan() && !previous.is_nan() && previous != 0.0 => {
                    values[index] = 100.0 * (current - previous) / previous;
                }
                (current2, previous2) if !current2.is_nan() && !previous2.is_nan() => values[index] = 0.0,
                _ => {}
            }
        }
    }
    let rc = Rc::new(values); nodes.insert(key, Rc::clone(&rc));
    rc
}

fn latest_chaikin_volatility_store(store: &CandleStore, period: usize) -> Option<f64> {
    chaikin_volatility_store(store, period, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn latest_macd(
    bars: &[Bar],
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let last = match bars.last() {
        Some(last) => last,
        None => return (None, None, None, None, None),
    };
    if bars.len() == 1 {
        return (
            Some(0.0),
            Some(0.0),
            Some(0.0),
            Some(last.close),
            Some(last.close),
        );
    }

    let previous_index = bars.len() - 2;
    let previous_close = bars[previous_index].close;
    let previous_fast = output_at(outputs, "fast_ema", previous_index).unwrap_or(previous_close);
    let previous_slow = output_at(outputs, "slow_ema", previous_index).unwrap_or(previous_close);
    let fast = ema_next(last.close, previous_fast, params.fast);
    let slow = ema_next(last.close, previous_slow, params.slow);
    let macd_line = fast - slow;
    let previous_signal = output_at(outputs, "signal", previous_index).unwrap_or(0.0);
    let signal = ema_next(macd_line, previous_signal, params.signal);
    let histogram = macd_line - signal;
    (
        Some(macd_line),
        Some(signal),
        Some(histogram),
        Some(fast),
        Some(slow),
    )
}

fn macd_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let macd_key = format!(
        "macd:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "macd:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "macd:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let fast_key = format!("ema:close:{}", params.fast);
    let slow_key = format!("ema:close:{}", params.slow);
    if let (Some(macd), Some(signal), Some(histogram), Some(fast_ema), Some(slow_ema)) = (
        nodes.get(&macd_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
        nodes.get(&fast_key),
        nodes.get(&slow_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "macd".to_string(),
                values: (**macd).clone(),
            },
            IndicatorOutput {
                name: "signal".to_string(),
                values: (**signal).clone(),
            },
            IndicatorOutput {
                name: "histogram".to_string(),
                values: (**histogram).clone(),
            },
            IndicatorOutput {
                name: "fast_ema".to_string(),
                values: (**fast_ema).clone(),
            },
            IndicatorOutput {
                name: "slow_ema".to_string(),
                values: (**slow_ema).clone(),
            },
        ];
    }

    let fast_ema = rc_into_owned(ema_close_store(store, params.fast, nodes));
    let slow_ema = rc_into_owned(ema_close_store(store, params.slow, nodes));
    let macd: Series = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&macd, params.signal);
    let histogram: Series = macd
        .iter()
        .zip(signal.iter())
        .map(|(macd, signal)| match (macd, signal) {
            (macd, signal) if !macd.is_nan() && !signal.is_nan() => *macd - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(macd_key, Rc::new(macd.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        IndicatorOutput {
            name: "macd".to_string(),
            values: macd,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
        IndicatorOutput {
            name: "fast_ema".to_string(),
            values: fast_ema,
        },
        IndicatorOutput {
            name: "slow_ema".to_string(),
            values: slow_ema,
        },
    ]
}

fn latest_macd_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
) {
    let last = match store.last_close() {
        Some(last) => last,
        None => return (None, None, None, None, None),
    };
    if store.len() == 1 {
        return (Some(0.0), Some(0.0), Some(0.0), Some(last), Some(last));
    }

    let previous_index = store.len() - 2;
    let previous_close = store.close[previous_index];
    let previous_fast = output_at(outputs, "fast_ema", previous_index).unwrap_or(previous_close);
    let previous_slow = output_at(outputs, "slow_ema", previous_index).unwrap_or(previous_close);
    let fast = ema_next(last, previous_fast, params.fast);
    let slow = ema_next(last, previous_slow, params.slow);
    let macd_line = fast - slow;
    let previous_signal = output_at(outputs, "signal", previous_index).unwrap_or(0.0);
    let signal = ema_next(macd_line, previous_signal, params.signal);
    let histogram = macd_line - signal;
    (
        Some(macd_line),
        Some(signal),
        Some(histogram),
        Some(fast),
        Some(slow),
    )
}

fn latest_ppo(bars: &[Bar], params: MacdParams) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = ppo(bars, params, &mut HashMap::new());
    let index = bars.len().saturating_sub(1);
    (
        output_at_vec(&outputs, "ppo", index),
        output_at_vec(&outputs, "signal", index),
        output_at_vec(&outputs, "histogram", index),
    )
}

fn latest_chaikin_oscillator(bars: &[Bar], params: MacdParams) -> Option<f64> {
    chaikin_oscillator(bars, params).last().copied().and_then(nan_to_none)
}

fn chaikin_oscillator_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Series {
    let key = format!("chaikin:{}:{}", params.fast, params.slow);
    if let Some(values) = nodes.get(&key) {
        return (**values).clone();
    }
    let adl_values = adl_store(store, nodes);
    let fast = ema_series(&adl_values, params.fast);
    let slow = ema_series(&adl_values, params.slow);
    let values: Vec<_> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| match (fast, slow) {
            (fast, slow) if !fast.is_nan() && !slow.is_nan() => *fast - *slow,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(key, Rc::new(values.clone()));
    values
}

fn latest_chaikin_oscillator_store(store: &CandleStore, params: MacdParams) -> Option<f64> {
    chaikin_oscillator_store(store, params, &mut HashMap::new())
        .last()
        .copied().and_then(nan_to_none)
}

fn ppo_store(
    store: &CandleStore,
    params: MacdParams,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    let ppo_key = format!(
        "ppo:value:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let signal_key = format!(
        "ppo:signal:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    let histogram_key = format!(
        "ppo:histogram:{}:{}:{}",
        params.fast, params.slow, params.signal
    );
    if let (Some(ppo), Some(signal), Some(histogram)) = (
        nodes.get(&ppo_key),
        nodes.get(&signal_key),
        nodes.get(&histogram_key),
    ) {
        return vec![
            IndicatorOutput {
                name: "ppo".to_string(),
                values: (**ppo).clone(),
            },
            IndicatorOutput {
                name: "signal".to_string(),
                values: (**signal).clone(),
            },
            IndicatorOutput {
                name: "histogram".to_string(),
                values: (**histogram).clone(),
            },
        ];
    }

    let macd_outputs = macd_store(store, params, nodes);
    let macd_line = macd_outputs
        .iter()
        .find(|output| output.name == "macd")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| vec![f64::NAN; store.len()]);
    let slow_ema = macd_outputs
        .iter()
        .find(|output| output.name == "slow_ema")
        .map(|output| output.values.clone())
        .unwrap_or_else(|| vec![f64::NAN; store.len()]);
    let ppo: Series = macd_line
        .iter()
        .zip(slow_ema.iter())
        .map(|(macd, slow)| match (macd, slow) {
            (macd, slow) if !macd.is_nan() && !slow.is_nan() && *slow != 0.0 => 100.0 * *macd / *slow,
            (a, b) if !a.is_nan() && !b.is_nan() => 0.0,
            _ => f64::NAN,
        })
        .collect();
    let signal = ema_series(&ppo, params.signal);
    let histogram: Series = ppo
        .iter()
        .zip(signal.iter())
        .map(|(ppo, signal)| match (ppo, signal) {
            (ppo, signal) if !ppo.is_nan() && !signal.is_nan() => *ppo - *signal,
            _ => f64::NAN,
        })
        .collect();
    nodes.insert(ppo_key, Rc::new(ppo.clone()));
    nodes.insert(signal_key, Rc::new(signal.clone()));
    nodes.insert(histogram_key, Rc::new(histogram.clone()));
    vec![
        IndicatorOutput {
            name: "ppo".to_string(),
            values: ppo,
        },
        IndicatorOutput {
            name: "signal".to_string(),
            values: signal,
        },
        IndicatorOutput {
            name: "histogram".to_string(),
            values: histogram,
        },
    ]
}

fn latest_ppo_store(
    store: &CandleStore,
    params: MacdParams,
    outputs: &IndicatorArena,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let (macd_line, _, _, _, slow_ema) = latest_macd_store(store, params, outputs);
    let ppo = match (macd_line, slow_ema) {
        (Some(macd_line), Some(slow_ema)) if slow_ema != 0.0 => Some(100.0 * macd_line / slow_ema),
        (Some(_), Some(_)) => Some(0.0),
        _ => None,
    };
    let previous_signal = store
        .len()
        .checked_sub(2)
        .and_then(|index| output_at(outputs, "signal", index))
        .unwrap_or(ppo.unwrap_or(0.0));
    let signal = ppo.map(|ppo| ema_next(ppo, previous_signal, params.signal));
    let histogram = ppo.zip(signal).map(|(ppo, signal)| ppo - signal);
    (ppo, signal, histogram)
}

fn ema_next(value: f64, previous: f64, period: usize) -> f64 {
    let alpha = 2.0 / (period as f64 + 1.0);
    alpha * value + (1.0 - alpha) * previous
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_series_eq(left: &[f64], right: &[f64]) {
        assert_eq!(left.len(), right.len(), "series lengths differ: {} vs {}", left.len(), right.len());
        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            if l.is_nan() && r.is_nan() { continue; }
            assert!((l - r).abs() < 1e-10, "series differ at index {}: left={}, right={}", i, l, r);
        }
    }

    macro_rules! assert_vec_eq {
        ($left:expr, $right:expr) => { assert_series_eq(&$left, &$right); };
    }

    fn assert_outputs_eq(left: &[IndicatorOutput], right: &[IndicatorOutput], names: &[&str]) {
        for name in names {
            let l = left.iter().find(|o| o.name == *name).expect(&format!("left missing {}", name));
            let r = right.iter().find(|o| o.name == *name).expect(&format!("right missing {}", name));
            assert_series_eq(&l.values, &r.values);
        }
    }

    fn bars(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, close)| Bar {
                time: i as u32,
                open: *close,
                high: *close,
                low: *close,
                close: *close,
                volume: 1.0,
            })
            .collect()
    }

    fn store_from_bars(bars: Vec<Bar>) -> CandleStore {
        CandleStore::from_bars(bars)
    }

    #[test]
    fn candle_store_from_columns_matches_from_bars() {
        let bars = bars(&[10.0, 11.0, 12.0]);
        let from_bars = CandleStore::from_bars(bars.clone());
        let from_columns = CandleStore::from_columns(CandleColumnsInput {
            time: bars.iter().map(|bar| bar.time).collect(),
            open: bars.iter().map(|bar| bar.open).collect(),
            high: bars.iter().map(|bar| bar.high).collect(),
            low: bars.iter().map(|bar| bar.low).collect(),
            close: bars.iter().map(|bar| bar.close).collect(),
            volume: bars.iter().map(|bar| bar.volume).collect(),
        })
        .unwrap();

        assert_eq!(from_columns.time, from_bars.time);
        assert_eq!(from_columns.open, from_bars.open);
        assert_eq!(from_columns.high, from_bars.high);
        assert_eq!(from_columns.low, from_bars.low);
        assert_eq!(from_columns.close, from_bars.close);
        assert_eq!(from_columns.volume, from_bars.volume);
    }

    #[test]
    fn candle_store_from_columns_rejects_mismatched_lengths() {
        let error = CandleStore::from_columns(CandleColumnsInput {
            time: vec![0, 1],
            open: vec![1.0],
            high: vec![1.0, 2.0],
            low: vec![1.0, 2.0],
            close: vec![1.0, 2.0],
            volume: vec![1.0, 2.0],
        })
        .err()
        .unwrap();

        assert_eq!(
            Some(error),
            Some("candle column lengths must match for time/open/high/low/close/volume")
        );
    }

    fn ohlc(values: &[(f64, f64, f64)]) -> Vec<Bar> {
        values
            .iter()
            .enumerate()
            .map(|(i, (high, low, close))| Bar {
                time: i as u32,
                open: *close,
                high: *high,
                low: *low,
                close: *close,
                volume: 1.0,
            })
            .collect()
    }

    fn indicator_stub(kind: &str) -> Indicator {
        Indicator {
            id: 1,
            kind: kind.to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 28,
            signal: 9,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        }
    }

    #[test]
    fn sma_waits_for_period_then_rolls() {
        assert_vec_eq!(
            sma(&bars(&[1.0, 2.0, 3.0, 4.0]), 3),
            vec![f64::NAN, f64::NAN, 2.0, 3.0]
        );
    }

    #[test]
    fn ema_updates_from_first_close() {
        assert_eq!(
            ema(&bars(&[10.0, 12.0, 14.0]), 3),
            vec![10.0, 11.0, 12.5]
        );
    }

    #[test]
    fn latest_sma_matches_full_series_last_value() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            latest_sma(&bars, 3),
            sma(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn latest_ema_uses_previous_output_value() {
        let previous_bars = bars(&[10.0, 12.0]);
        let mut next_bars = previous_bars.clone();
        next_bars.push(Bar {
            time: 2,
            open: 14.0,
            high: 14.0,
            low: 14.0,
            close: 14.0,
            volume: 1.0,
        });
        let output = IndicatorOutput {
            name: "value".to_string(),
            values: ema(&previous_bars, 3),
        };

        assert_eq!(latest_ema(&next_bars, 3, Some(&output.values[..])), Some(12.5));
    }

    #[test]
    fn store_sma_matches_row_sma() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(
            sma(&bars, 3),
            sma_close_store(&store, 3, &mut HashMap::new())
        );
        assert_eq!(latest_sma(&bars, 3), latest_sma_store(&store, 3));
    }

    #[test]
    fn store_ema_matches_row_ema() {
        let bars = bars(&[10.0, 12.0, 14.0, 13.0]);
        let store = store_from_bars(bars.clone());
        let previous_output = IndicatorOutput {
            name: "value".to_string(),
            values: ema(&bars[..bars.len() - 1], 3),
        };

        assert_eq!(
            ema(&bars, 3),
            *ema_close_store(&store, 3, &mut HashMap::new())
        );
        assert_eq!(
            latest_ema(&bars, 3, Some(&previous_output.values[..])),
            latest_ema_store(&store, 3, Some(&previous_output.values[..]))
        );
    }

    #[test]
    fn rsi_waits_for_period_then_uses_wilder_smoothing() {
        let values = rsi(&bars(&[1.0, 2.0, 1.0, 3.0, 2.0]), 3);
        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(values[2].is_nan());
        assert_eq!(values[3], 75.0);
        assert!((values[4] - 54.54545454545455).abs() < 0.000001);
    }

    #[test]
    fn latest_rsi_matches_full_series_last_value() {
        let previous_bars = bars(&[1.0, 2.0, 1.0, 3.0]);
        let all_bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0]);
        let previous_outputs = rsi_outputs(&previous_bars, 3);
        let full_outputs = rsi_outputs(&all_bars, 3);
        let latest = latest_rsi(&all_bars, 3, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(
            latest.0,
            output_at_vec(&full_outputs, "value", all_bars.len() - 1)
        );
        assert_eq!(
            latest.1,
            output_at_vec(&full_outputs, "avg_gain", all_bars.len() - 1)
        );
        assert_eq!(
            latest.2,
            output_at_vec(&full_outputs, "avg_loss", all_bars.len() - 1)
        );
    }

    #[test]
    fn hidden_state_outputs_are_not_visible() {
        assert!(is_visible_output("value"));
        assert!(is_visible_output("histogram"));
        assert!(!is_visible_output("avg_gain"));
        assert!(!is_visible_output("tr_avg"));
        assert!(!is_visible_output("fast_ema"));
        assert!(!is_visible_output("cumulative_pv"));
    }

    #[test]
    fn all_exposed_indicators_support_incremental_updates() {
        for descriptor in indicator_descriptors() {
            assert!(
                supports_incremental(descriptor.kind),
                "{} must be handled incrementally or intentionally hidden",
                descriptor.kind
            );
        }
    }

    #[test]
    fn unknown_indicator_does_not_support_incremental_updates() {
        assert!(!supports_incremental("UNKNOWN"));
    }

    #[test]
    fn rsi_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "RSI".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["rsi:close:14"]);
    }

    #[test]
    fn cci_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "CCI".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["cci:hlc:20"]);
    }

    #[test]
    fn supertrend_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "SUPERTREND".to_string(),
            period: 10,
            stoch_period: 10,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 3.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec!["atr:ohlc:10", "supertrend:10:3"]
        );
    }

    #[test]
    fn keltner_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "KELTNER".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "ema:close:20",
                "atr:ohlc:20",
                "keltner:upper:20:2",
                "keltner:middle:20:2",
                "keltner:lower:20:2",
            ]
        );
    }

    #[test]
    fn parabolic_sar_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "PARABOLIC_SAR".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["psar:ohlc:0.02:0.2"]);
    }

    #[test]
    fn ichimoku_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "ICHIMOKU".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "ichimoku:tenkan:9",
                "ichimoku:kijun:26",
                "ichimoku:senkou_a:9:26",
                "ichimoku:senkou_b:52",
                "ichimoku:chikou",
            ]
        );
    }

    #[test]
    fn pivot_points_have_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "PIVOT_POINTS".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec!["pivot:pp", "pivot:r1", "pivot:s1", "pivot:r2", "pivot:s2"]
        );
    }

    #[test]
    fn roc_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "ROC".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["roc:close:14"]);
    }

    #[test]
    fn aroon_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "AROON".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["aroon:hl:14"]);
    }

    #[test]
    fn cmf_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "CMF".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["cmf:hlcv:20"]);
    }

    #[test]
    fn adl_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "ADL".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["adl:hlcv"]);
    }

    #[test]
    fn vwma_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "VWMA".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["vwma:close:volume:20"]);
    }

    #[test]
    fn williams_ad_has_a_computed_dag_node() {
        let indicator = indicator_stub("WILLIAMS_AD");
        assert_eq!(indicator_nodes(&indicator), vec!["wad:ohlc"]);
    }

    #[test]
    fn chaikin_volatility_has_computed_dag_nodes() {
        let indicator = indicator_stub("CHAIKIN_VOLATILITY");
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["cvol:ema:14", "cvol:value:14"]
        );
    }

    #[test]
    fn price_channel_has_computed_dag_nodes() {
        let indicator = indicator_stub("PRICE_CHANNEL");
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "price_channel:upper:14",
                "price_channel:middle:14",
                "price_channel:lower:14",
            ]
        );
    }

    #[test]
    fn starc_has_computed_dag_nodes() {
        let indicator = indicator_stub("STARC");
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "sma:close:14",
                "atr:ohlc:14",
                "starc:upper:14:2",
                "starc:middle:14:2",
                "starc:lower:14:2",
            ]
        );
    }

    #[test]
    fn wma_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "WMA".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["wma:close:20"]);
    }

    #[test]
    fn hma_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "HMA".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec!["wma:close:10", "wma:close:20", "hma:close:20"]
        );
    }

    #[test]
    fn linear_regression_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "LINEAR_REGRESSION".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["linreg:close:20"]);
    }

    #[test]
    fn dema_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("DEMA");
        indicator.period = 15;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["ema:close:15", "dema:ema2:15", "dema:value:15"]
        );
    }

    #[test]
    fn tema_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("TEMA");
        indicator.period = 15;
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "ema:close:15",
                "tema:ema2:15",
                "tema:ema3:15",
                "tema:value:15"
            ]
        );
    }

    #[test]
    fn trima_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("TRIMA");
        indicator.period = 20;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["sma:close:20", "trima:value:20"]
        );
    }

    #[test]
    fn stddev_has_a_computed_dag_node() {
        let mut indicator = indicator_stub("STDDEV");
        indicator.period = 20;
        assert_eq!(indicator_nodes(&indicator), vec!["stddev:close:20"]);
    }

    #[test]
    fn envelope_has_computed_dag_nodes() {
        let indicator = indicator_stub("ENVELOPE");
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "sma:close:14",
                "envelope:upper:14:2",
                "envelope:middle:14:2",
                "envelope:lower:14:2",
            ]
        );
    }

    #[test]
    fn trix_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("TRIX");
        indicator.period = 15;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["ema:close:15", "trix:ema2:15", "trix:value:15"]
        );
    }

    #[test]
    fn tsi_has_a_computed_dag_node() {
        let mut indicator = indicator_stub("TSI");
        indicator.period = 25;
        indicator.stoch_period = 13;
        assert_eq!(indicator_nodes(&indicator), vec!["tsi:25:13"]);
    }

    #[test]
    fn kst_has_computed_dag_nodes() {
        let indicator = indicator_stub("KST");
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "roc:close:10",
                "roc:close:15",
                "roc:close:20",
                "roc:close:30",
                "kst:value",
            ]
        );
    }

    #[test]
    fn bop_has_a_computed_dag_node() {
        let indicator = indicator_stub("BOP");
        assert_eq!(indicator_nodes(&indicator), vec!["bop:ohlc"]);
    }

    #[test]
    fn dpo_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("DPO");
        indicator.period = 20;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["sma:close:20", "dpo:close:20"]
        );
    }

    #[test]
    fn momentum_has_a_computed_dag_node() {
        let mut indicator = indicator_stub("MOMENTUM");
        indicator.period = 10;
        assert_eq!(indicator_nodes(&indicator), vec!["momentum:close:10"]);
    }

    #[test]
    fn ultimate_oscillator_has_a_computed_dag_node() {
        let mut indicator = indicator_stub("ULTIMATE_OSCILLATOR");
        indicator.period = 7;
        indicator.stoch_period = 14;
        indicator.smooth = 28;
        assert_eq!(indicator_nodes(&indicator), vec!["uo:7:14:28"]);
    }

    #[test]
    fn chaikin_oscillator_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("CHAIKIN_OSCILLATOR");
        indicator.macd = Some(MacdParams {
            fast: 3,
            slow: 10,
            signal: 9,
        });
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["adl:hlcv", "chaikin:3:10"]
        );
    }

    #[test]
    fn force_index_has_a_computed_dag_node() {
        let mut indicator = indicator_stub("FORCE_INDEX");
        indicator.period = 13;
        assert_eq!(indicator_nodes(&indicator), vec!["force:close:volume:13"]);
    }

    #[test]
    fn ppo_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("PPO");
        indicator.macd = Some(MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        });
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["ema:close:12", "ema:close:26", "ppo:12:26:9"]
        );
    }

    #[test]
    fn donchian_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "DONCHIAN".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "donchian:upper:20",
                "donchian:middle:20",
                "donchian:lower:20",
            ]
        );
    }

    #[test]
    fn mfi_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "MFI".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["mfi:hlcv:14"]);
    }

    #[test]
    fn williams_r_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "WILLIAMS_R".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["willr:hlc:14"]);
    }

    #[test]
    fn stoch_rsi_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "STOCH_RSI".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec!["rsi:close:14", "stoch:rsi:14:14:3:3"]
        );
    }

    #[test]
    fn atr_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "ATR".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["atr:ohlc:14"]);
    }

    #[test]
    fn adx_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "ADX".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["adx:ohlc:14"]);
    }

    #[test]
    fn vwap_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "VWAP".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["vwap:hlcv"]);
    }

    #[test]
    fn stochastic_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "STOCHASTIC".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["stoch:hlc:14:3"]);
    }

    #[test]
    fn obv_adds_or_subtracts_volume_by_close_direction() {
        let mut bars = bars(&[10.0, 11.0, 9.0, 9.0, 12.0]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0]) {
            bar.volume = volume;
        }

        assert_eq!(
            obv(&bars),
            vec![0.0, 2.0, -1.0, -1.0, 4.0]
        );
    }

    #[test]
    fn latest_obv_uses_previous_output_value() {
        let mut previous_bars = bars(&[10.0, 11.0]);
        previous_bars[1].volume = 2.0;
        let mut next_bars = previous_bars.clone();
        next_bars.push(Bar {
            time: 2,
            open: 9.0,
            high: 9.0,
            low: 9.0,
            close: 9.0,
            volume: 3.0,
        });
        let output = IndicatorOutput {
            name: "value".to_string(),
            values: obv(&previous_bars),
        };

        assert_eq!(latest_obv(&next_bars, Some(&output.values[..])), Some(-1.0));
    }

    #[test]
    fn store_volume_indicators_match_row_versions() {
        let mut bars = ohlc(&[(3.0, 0.0, 0.0), (6.0, 0.0, 0.0), (8.0, 1.0, 5.0)]);
        bars[0].volume = 2.0;
        bars[1].volume = 4.0;
        bars[2].volume = 3.0;
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(obv(&bars), &obv_store(&store, &mut HashMap::new()));
        assert_vec_eq!(adl(&bars), &adl_store(&store, &mut HashMap::new()));
        assert_vec_eq!(vwma(&bars, 2), &vwma_store(&store, 2, &mut HashMap::new()));
        let row_vwap = vwap(&bars, &mut HashMap::new());
        let store_vwap = vwap_store(&store, &mut HashMap::new());
        assert_outputs_eq(&row_vwap, &store_vwap, &["value", "cumulative_pv", "cumulative_volume"]);
    }

    #[test]
    fn store_window_indicators_match_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(roc(&bars, 2), &roc_store(&store, 2, &mut HashMap::new()));
        assert_vec_eq!(cmf(&bars, 3), &cmf_store(&store, 3, &mut HashMap::new()));
        let row_bb = bollinger(&bars, 3, 2.0, &mut HashMap::new());
        let store_bb = bollinger_store(&store, 3, 2.0, &mut HashMap::new());
        assert_outputs_eq(&row_bb, &store_bb, &["upper", "middle", "lower"]);
    }

    #[test]
    fn store_oscillator_windows_match_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(cci(&bars, 3), &cci_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(
            williams_r(&bars, 3),
            williams_r_store(&store, 3, &mut HashMap::new())
        );
        assert_vec_eq!(mfi(&bars, 3), &mfi_store(&store, 3, &mut HashMap::new()));
        let row_stoch = stochastic(&bars, 3, 2, &mut HashMap::new());
        let store_stoch = stochastic_store(&store, 3, 2, &mut HashMap::new());
        assert_outputs_eq(&row_stoch, &store_stoch, &["k", "d"]);
    }

    #[test]
    fn store_stateful_ohlc_indicators_match_row_versions() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
            (16.0, 14.0, 15.0),
            (17.0, 15.0, 16.0),
            (18.0, 16.0, 17.0),
        ]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(atr(&bars, 3), &atr_store(&store, 3, &mut HashMap::new()));
        let row_adx = adx(&bars, 3, &mut HashMap::new());
        let store_adx = adx_store(&store, 3, &mut HashMap::new());
        assert_outputs_eq(&row_adx, &store_adx, &[
            "value",
            "plus_di",
            "minus_di",
            "tr_avg",
            "plus_dm_avg",
            "minus_dm_avg",
            "dx",
        ]);
        for (row, store_output) in [
            (
                keltner(&bars, 3, 2.0, &mut HashMap::new()),
                keltner_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
            (
                starc(&bars, 3, 2.0, &mut HashMap::new()),
                starc_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
            (
                supertrend(&bars, 3, 2.0, &mut HashMap::new()),
                supertrend_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
        ] {
            for row_output in &row {
                let store_vals = store_output.iter().find(|o| o.name == row_output.name).map(|o| &o.values).unwrap();
                assert_series_eq(&row_output.values, store_vals);
            }
        }
    }

    #[test]
    fn store_misc_incremental_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(wma(&bars, 3), &wma_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(dpo(&bars, 4), &dpo_store(&store, 4, &mut HashMap::new()));
        assert_vec_eq!(
            force_index(&bars, 2),
            force_index_store(&store, 2, &mut HashMap::new())
        );
        let row_channel = price_channel(&bars, 3, &mut HashMap::new());
        let store_channel = price_channel_store(&store, 3, &mut HashMap::new());
        assert_outputs_eq(&row_channel, &store_channel, &["upper", "middle", "lower"]);
    }

    #[test]
    fn store_math_batch_matches_row_versions() {
        let bars = bars(&[10.0, 12.0, 11.0, 13.0, 15.0, 14.0, 16.0, 18.0]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(
            hma(&bars, 5, &mut HashMap::new()),
            hma_store(&store, 5, &mut HashMap::new())
        );
        assert_vec_eq!(linear_regression(&bars, 4), linear_regression_store(&store, 4, &mut HashMap::new()));
        assert_vec_eq!(
            stddev(&bars, 4),
            stddev_store(&store, 4, &mut HashMap::new())
        );
        assert_vec_eq!(trix(&bars, 3), &trix_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(tsi(&bars, 4, 2), tsi_store(&store, 4, 2, &mut HashMap::new()));
        assert_vec_eq!(momentum(&bars, 3), momentum_store(&store, 3, &mut HashMap::new()));
    }

    #[test]
    fn store_remaining_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 9.5, 12.0),
            (14.0, 10.0, 13.0),
            (15.0, 11.0, 14.0),
            (16.0, 12.0, 15.0),
            (17.0, 13.0, 16.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip(1..=8) {
            bar.volume = volume as f64;
        }
        let store = store_from_bars(bars.clone());

        for (row, store_output) in [
            (
                donchian(&bars, 3, &mut HashMap::new()),
                donchian_store(&store, 3, &mut HashMap::new()),
            ),
            (
                parabolic_sar(&bars, 0.02, 0.2, &mut HashMap::new()),
                parabolic_sar_store(&store, 0.02, 0.2, &mut HashMap::new()),
            ),
            (
                ichimoku(&bars, 3, 4, 5, &mut HashMap::new()),
                ichimoku_store(&store, 3, 4, 5, &mut HashMap::new()),
            ),
            (
                pivot_points(&bars, &mut HashMap::new()),
                pivot_points_store(&store, &mut HashMap::new()),
            ),
            (
                aroon(&bars, 3, &mut HashMap::new()),
                aroon_store(&store, 3, &mut HashMap::new()),
            ),
            (
                stoch_rsi(&bars, 3, 3, 2, 2, &mut HashMap::new()),
                stoch_rsi_store(&store, 3, 3, 2, 2, &mut HashMap::new()),
            ),
        ] {
            for row_output in &row {
                let store_vals = store_output.iter().find(|o| o.name == row_output.name).map(|o| &o.values).unwrap();
                assert_series_eq(&row_output.values, store_vals);
            }
        }

        assert_vec_eq!(ultimate_oscillator(&bars, 2, 3, 4), ultimate_oscillator_store(&store, 2, 3, 4, &mut HashMap::new()));
        assert_vec_eq!(chaikin_volatility(&bars, 3), chaikin_volatility_store(&store, 3, &mut HashMap::new()));
    }

    #[test]
    fn store_final_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
            (16.0, 14.0, 15.0),
            (17.0, 15.0, 16.0),
            (18.0, 16.0, 17.0),
            (19.0, 17.0, 18.0),
            (20.0, 18.0, 19.0),
            (21.0, 19.0, 20.0),
            (22.0, 20.0, 21.0),
            (23.0, 21.0, 22.0),
            (24.0, 22.0, 23.0),
            (25.0, 23.0, 24.0),
            (26.0, 24.0, 25.0),
            (27.0, 25.0, 26.0),
            (28.0, 26.0, 27.0),
            (29.0, 27.0, 28.0),
            (30.0, 28.0, 29.0),
            (31.0, 29.0, 30.0),
            (32.0, 30.0, 31.0),
            (33.0, 31.0, 32.0),
            (34.0, 32.0, 33.0),
            (35.0, 33.0, 34.0),
            (36.0, 34.0, 35.0),
            (37.0, 35.0, 36.0),
            (38.0, 36.0, 37.0),
            (39.0, 37.0, 38.0),
            (40.0, 38.0, 39.0),
        ]);
        for (i, bar) in bars.iter_mut().enumerate() {
            bar.volume = (i + 1) as f64;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(dema(&bars, 5), &dema_store(&store, 5, &mut HashMap::new()));
        assert_vec_eq!(tema(&bars, 5), &tema_store(&store, 5, &mut HashMap::new()));
        assert_vec_eq!(trima(&bars, 5), &trima_store(&store, 5, &mut HashMap::new()));
        assert_vec_eq!(kst(&bars), &kst_store(&store, &mut HashMap::new()));
        assert_vec_eq!(bop(&bars), &bop_store(&store, &mut HashMap::new()));
        assert_eq!(
            chaikin_oscillator(
                &bars,
                MacdParams {
                    fast: 3,
                    slow: 10,
                    signal: 9,
                }
            ),
            chaikin_oscillator_store(
                &store,
                MacdParams {
                    fast: 3,
                    slow: 10,
                    signal: 9,
                },
                &mut HashMap::new(),
            )
        );
        let row_envelope = envelope(&bars, 5, 2.0, &mut HashMap::new());
        let store_envelope = envelope_store(&store, 5, 2.0, &mut HashMap::new());
        assert_outputs_eq(&row_envelope, &store_envelope, &["upper", "middle", "lower"]);
    }

    #[test]
    fn vwap_uses_cumulative_price_volume() {
        let mut bars = ohlc(&[(3.0, 0.0, 0.0), (6.0, 0.0, 0.0)]);
        bars[0].volume = 2.0;
        bars[1].volume = 4.0;
        let outputs = vwap(&bars, &mut HashMap::new());

        assert_eq!(outputs[0].name, "value");
        assert_series_eq(&outputs[0].values, &[1.0, 10.0 / 6.0]);
    }

    #[test]
    fn latest_vwap_matches_full_series_last_value() {
        let mut previous_bars = ohlc(&[(3.0, 0.0, 0.0)]);
        previous_bars[0].volume = 2.0;
        let mut all_bars = ohlc(&[(3.0, 0.0, 0.0), (6.0, 0.0, 0.0)]);
        all_bars[0].volume = 2.0;
        all_bars[1].volume = 4.0;
        let previous_outputs = vwap(&previous_bars, &mut HashMap::new());
        let full_outputs = vwap(&all_bars, &mut HashMap::new());
        let latest = latest_vwap(&all_bars, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(latest.0, full_outputs[0].values.last().copied().and_then(nan_to_none));
        assert_eq!(
            latest.1,
            output_at_vec(&full_outputs, "cumulative_pv", all_bars.len() - 1)
        );
        assert_eq!(
            latest.2,
            output_at_vec(&full_outputs, "cumulative_volume", all_bars.len() - 1)
        );
    }

    #[test]
    fn cci_returns_one_value_series() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
        ]);
        let values = cci(&bars, 3);

        assert_eq!(values.len(), bars.len());
        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(!values[3].is_nan());
    }

    #[test]
    fn latest_cci_matches_full_series_last_value() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
        ]);

        assert_eq!(
            latest_cci(&bars, 3),
            cci(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn latest_indicator_values_fast_reuses_visible_output_scratch() {
        let mut engine = ChartEngine::new();
        engine.indicators.push(Indicator {
            id: 7,
            kind: "RSI".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 0,
            signal: 0,
            tenkan_period: 0,
            kijun_period: 0,
            senkou_b_period: 0,
            macd: None,
            multiplier: 0.0,
            psar_step: 0.0,
            psar_max_step: 0.0,
            outputs: IndicatorArena::from_outputs(vec![
                IndicatorOutput {
                    name: "value".to_string(),
                    values: vec![1.25],
                },
                IndicatorOutput {
                    name: "avg_gain".to_string(),
                    values: vec![f64::NAN],
                },
                IndicatorOutput {
                    name: "avg_loss".to_string(),
                    values: vec![-0.5],
                },
            ]),
        });

        let values = engine.latest_indicator_values_slice(7).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 1.25);
        assert_eq!(engine.latest_values_scratch.len(), 1);
    }

    #[test]
    fn indicator_output_values_fast_reuses_visible_output_scratch() {
        let mut engine = ChartEngine::new();
        engine.indicators.push(Indicator {
            id: 9,
            kind: "MACD".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 0,
            signal: 0,
            tenkan_period: 0,
            kijun_period: 0,
            senkou_b_period: 0,
            macd: None,
            multiplier: 0.0,
            psar_step: 0.0,
            psar_max_step: 0.0,
            outputs: IndicatorArena::from_outputs(vec![
                IndicatorOutput {
                    name: "macd".to_string(),
                    values: vec![1.0, f64::NAN],
                },
                IndicatorOutput {
                    name: "signal".to_string(),
                    values: vec![2.0, 3.0],
                },
                IndicatorOutput {
                    name: "histogram".to_string(),
                    values: vec![-1.0, 0.5],
                },
                IndicatorOutput {
                    name: "fast_ema".to_string(),
                    values: vec![99.0, 100.0],
                },
            ]),
        });

        engine.populate_indicator_output_values_scratch(9).unwrap();

        assert_eq!(engine.indicator_values_scratch.len(), 3);
        assert_eq!(engine.indicator_values_scratch[0][0], 1.0);
        assert!(engine.indicator_values_scratch[0][1].is_nan());
        assert_eq!(engine.indicator_values_scratch[1].as_slice(), &[2.0, 3.0]);
        assert_eq!(engine.indicator_values_scratch[2].as_slice(), &[-1.0, 0.5]);
    }

    #[test]
    fn mfi_returns_one_value_series() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
        ]);
        bars[0].volume = 1.0;
        bars[1].volume = 2.0;
        bars[2].volume = 3.0;
        bars[3].volume = 4.0;
        let values = mfi(&bars, 3);

        assert_eq!(values.len(), bars.len());
        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(values[2].is_nan());
        assert!(!values[3].is_nan());
    }

    #[test]
    fn latest_mfi_matches_full_series_last_value() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
        ]);
        bars[0].volume = 1.0;
        bars[1].volume = 2.0;
        bars[2].volume = 3.0;
        bars[3].volume = 4.0;

        assert_eq!(
            latest_mfi(&bars, 3),
            mfi(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn williams_r_returns_one_value_series() {
        let bars = ohlc(&[
            (10.0, 5.0, 6.0),
            (11.0, 4.0, 7.0),
            (12.0, 3.0, 8.0),
            (13.0, 2.0, 9.0),
        ]);
        let values = williams_r(&bars, 3);

        assert_eq!(values.len(), bars.len());
        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(!values[3].is_nan());
    }

    #[test]
    fn latest_williams_r_matches_full_series_last_value() {
        let bars = ohlc(&[
            (10.0, 5.0, 6.0),
            (11.0, 4.0, 7.0),
            (12.0, 3.0, 8.0),
            (13.0, 2.0, 9.0),
        ]);

        assert_eq!(
            latest_williams_r(&bars, 3),
            williams_r(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn stochastic_returns_k_and_d() {
        let bars = ohlc(&[
            (10.0, 5.0, 6.0),
            (11.0, 4.0, 7.0),
            (12.0, 3.0, 8.0),
            (13.0, 2.0, 9.0),
            (14.0, 1.0, 10.0),
        ]);
        let outputs = stochastic(&bars, 3, 2, &mut HashMap::new());

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].name, "k");
        assert_eq!(outputs[1].name, "d");
        assert_eq!(outputs[0].values.len(), bars.len());
    }

    #[test]
    fn latest_stochastic_matches_full_series_last_values() {
        let previous_bars = ohlc(&[
            (10.0, 5.0, 6.0),
            (11.0, 4.0, 7.0),
            (12.0, 3.0, 8.0),
            (13.0, 2.0, 9.0),
        ]);
        let all_bars = ohlc(&[
            (10.0, 5.0, 6.0),
            (11.0, 4.0, 7.0),
            (12.0, 3.0, 8.0),
            (13.0, 2.0, 9.0),
            (14.0, 1.0, 10.0),
        ]);
        let previous_outputs = stochastic(&previous_bars, 3, 2, &mut HashMap::new());
        let full_outputs = stochastic(&all_bars, 3, 2, &mut HashMap::new());
        let latest = latest_stochastic(&all_bars, 3, 2, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(latest.0, output_at_vec(&full_outputs, "k", all_bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&full_outputs, "d", all_bars.len() - 1));
    }

    #[test]
    fn stoch_rsi_returns_k_and_d() {
        let bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0, 4.0, 3.0]);
        let outputs = stoch_rsi(&bars, 3, 3, 2, 2, &mut HashMap::new());

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].name, "k");
        assert_eq!(outputs[1].name, "d");
        assert_eq!(outputs[0].values.len(), bars.len());
    }

    #[test]
    fn latest_stoch_rsi_matches_full_series_last_values() {
        let previous_bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0, 4.0]);
        let all_bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0, 4.0, 3.0]);
        let full_outputs = stoch_rsi(&all_bars, 3, 3, 2, 2, &mut HashMap::new());
        let latest = latest_stoch_rsi(&all_bars, 3, 3, 2, 2);

        assert_eq!(latest.0, output_at_vec(&full_outputs, "k", all_bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&full_outputs, "d", all_bars.len() - 1));
        assert!(
            stoch_rsi(&previous_bars, 3, 3, 2, 2, &mut HashMap::new())[0]
                .values
                .len()
                < all_bars.len()
        );
    }

    #[test]
    fn atr_waits_for_period_then_uses_wilder_smoothing() {
        let values = atr(
            &ohlc(&[
                (10.0, 9.0, 9.5),
                (11.0, 9.0, 10.0),
                (12.0, 10.0, 11.0),
                (14.0, 10.0, 13.0),
                (15.0, 12.0, 14.0),
            ]),
            3,
        );

        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(values[2].is_nan());
        assert!((values[3] - 2.6666666666666665).abs() < 0.000001);
        assert!((values[4] - 2.7777777777777777).abs() < 0.000001);
    }

    #[test]
    fn latest_atr_matches_full_series_last_value() {
        let previous_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
        ]);
        let all_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let output = IndicatorOutput {
            name: "value".to_string(),
            values: atr(&previous_bars, 3),
        };

        assert_eq!(
            latest_atr(&all_bars, 3, Some(&output.values[..])),
            atr(&all_bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn supertrend_returns_value_and_state_outputs() {
        let bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let outputs = supertrend(&bars, 3, 2.0, &mut HashMap::new());

        assert_eq!(
            outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec!["value", "upper_band", "lower_band", "trend"]
        );
        assert_eq!(outputs[0].values.len(), bars.len());
    }

    #[test]
    fn latest_supertrend_matches_full_series_last_values() {
        let previous_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
        ]);
        let all_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let previous_outputs = supertrend(&previous_bars, 3, 2.0, &mut HashMap::new());
        let full_outputs = supertrend(&all_bars, 3, 2.0, &mut HashMap::new());
        let latest = latest_supertrend(&all_bars, 3, 2.0, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(
            latest.0,
            output_at_vec(&full_outputs, "value", all_bars.len() - 1)
        );
        assert_eq!(
            latest.1,
            output_at_vec(&full_outputs, "upper_band", all_bars.len() - 1)
        );
        assert_eq!(
            latest.2,
            output_at_vec(&full_outputs, "lower_band", all_bars.len() - 1)
        );
        assert_eq!(
            latest.3,
            output_at_vec(&full_outputs, "trend", all_bars.len() - 1)
        );
    }

    #[test]
    fn adx_returns_value_and_state_outputs() {
        let bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.5, 10.5),
            (12.0, 10.0, 11.5),
            (12.5, 10.5, 12.0),
            (13.0, 11.0, 12.5),
            (13.5, 11.5, 13.0),
            (14.0, 12.0, 13.5),
        ]);
        let outputs = adx(&bars, 3, &mut HashMap::new());

        assert_eq!(
            outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "value",
                "plus_di",
                "minus_di",
                "tr_avg",
                "plus_dm_avg",
                "minus_dm_avg",
                "dx"
            ]
        );
        assert_eq!(outputs[0].values.len(), bars.len());
    }

    #[test]
    fn latest_adx_matches_full_series_last_values() {
        let previous_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.5, 10.5),
            (12.0, 10.0, 11.5),
            (12.5, 10.5, 12.0),
            (13.0, 11.0, 12.5),
            (13.5, 11.5, 13.0),
        ]);
        let all_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.5, 10.5),
            (12.0, 10.0, 11.5),
            (12.5, 10.5, 12.0),
            (13.0, 11.0, 12.5),
            (13.5, 11.5, 13.0),
            (14.0, 12.0, 13.5),
        ]);
        let previous_outputs = adx(&previous_bars, 3, &mut HashMap::new());
        let full_outputs = adx(&all_bars, 3, &mut HashMap::new());
        let latest = latest_adx(&all_bars, 3, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(
            latest.0,
            output_at_vec(&full_outputs, "value", all_bars.len() - 1)
        );
        assert_eq!(
            latest.1,
            output_at_vec(&full_outputs, "plus_di", all_bars.len() - 1)
        );
        assert_eq!(
            latest.2,
            output_at_vec(&full_outputs, "minus_di", all_bars.len() - 1)
        );
        assert_eq!(
            latest.3,
            output_at_vec(&full_outputs, "tr_avg", all_bars.len() - 1)
        );
        assert_eq!(
            latest.4,
            output_at_vec(&full_outputs, "plus_dm_avg", all_bars.len() - 1)
        );
        assert_eq!(
            latest.5,
            output_at_vec(&full_outputs, "minus_dm_avg", all_bars.len() - 1)
        );
        assert_eq!(latest.6, output_at_vec(&full_outputs, "dx", all_bars.len() - 1));
    }

    #[test]
    fn macd_returns_line_signal_and_histogram() {
        let bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        let outputs = macd(
            &bars,
            MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            },
            &mut HashMap::new(),
        );

        assert_eq!(
            outputs[0..3]
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec!["macd", "signal", "histogram"]
        );
        assert_eq!(outputs[0].name, "macd");
        assert_eq!(outputs[1].name, "signal");
        assert_eq!(outputs[2].name, "histogram");
        assert_eq!(outputs[0].values.len(), bars.len());
        assert_eq!(
            outputs[2].values[29],
            outputs[0].values[29] - outputs[1].values[29]
        );
    }

    #[test]
    fn latest_macd_matches_full_series_last_values() {
        let previous_bars = bars(&(1..=29).map(|value| value as f64).collect::<Vec<_>>());
        let all_bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        let params = MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        };
        let previous_outputs = macd(&previous_bars, params, &mut HashMap::new());
        let full_outputs = macd(&all_bars, params, &mut HashMap::new());
        let latest = latest_macd(&all_bars, params, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(latest.0, full_outputs[0].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.1, full_outputs[1].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.2, full_outputs[2].values.last().copied().and_then(nan_to_none));
        assert_eq!(
            latest.3,
            output_at_vec(&full_outputs, "fast_ema", all_bars.len() - 1)
        );
        assert_eq!(
            latest.4,
            output_at_vec(&full_outputs, "slow_ema", all_bars.len() - 1)
        );
    }

    #[test]
    fn ema_nodes_are_reused_by_macd() {
        let bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        let mut nodes = HashMap::new();

        let ema12 = compute_indicator(
            &bars, "EMA", 12, 0, 0, 0, 9, 26, 52, None, 2.0, 0.02, 0.2, &mut nodes,
        )[0]
        .values
        .clone();
        let macd = compute_indicator(
            &bars,
            "MACD",
            0,
            0,
            0,
            0,
            9,
            26,
            52,
            Some(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            2.0,
            0.02,
            0.2,
            &mut nodes,
        );

        assert_eq!(nodes.len(), 2);
        assert_vec_eq!(nodes["ema:close:12"], ema12);
        assert_eq!(
            macd[0].values[29],
            nodes["ema:close:12"][29] - nodes["ema:close:26"][29]
        );
    }

    #[test]
    fn rsi_nodes_are_reused_by_stoch_rsi() {
        let bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0, 4.0, 3.0, 5.0]);
        let mut nodes = HashMap::new();

        let rsi = rsi_close(&bars, 3, &mut nodes);
        let stoch_rsi_outputs = stoch_rsi(&bars, 3, 3, 2, 2, &mut nodes);

        assert_vec_eq!(nodes["rsi:close:3"], rsi);
        assert_vec_eq!(nodes["stoch:rsi:3:3:2:2"], stoch_rsi_outputs[0].values);
    }

    #[test]
    fn bollinger_returns_upper_middle_lower() {
        let outputs = bollinger(&bars(&[1.0, 2.0, 3.0]), 3, 2.0, &mut HashMap::new());

        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[0].name, "upper");
        assert_eq!(outputs[1].name, "middle");
        assert_eq!(outputs[2].name, "lower");
        assert_series_eq(&outputs[1].values, &[f64::NAN, f64::NAN, 2.0]);
        assert!((outputs[0].values[2] - 3.632993161855452).abs() < 0.000001);
        assert!((outputs[2].values[2] - 0.367006838144548).abs() < 0.000001);
    }

    #[test]
    fn latest_bollinger_matches_full_series_last_values() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0]);
        let outputs = bollinger(&bars, 3, 2.0, &mut HashMap::new());
        let (upper, middle, lower) = latest_bollinger(&bars, 3, 2.0);

        assert_eq!(upper, outputs[0].values.last().copied().and_then(nan_to_none));
        assert_eq!(middle, outputs[1].values.last().copied().and_then(nan_to_none));
        assert_eq!(lower, outputs[2].values.last().copied().and_then(nan_to_none));
    }

    #[test]
    fn keltner_returns_upper_middle_lower() {
        let bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let outputs = keltner(&bars, 3, 2.0, &mut HashMap::new());

        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[0].name, "upper");
        assert_eq!(outputs[1].name, "middle");
        assert_eq!(outputs[2].name, "lower");
        assert_eq!(outputs[0].values.len(), bars.len());
        assert!(!outputs[0].values[4].is_nan());
        assert!(!outputs[1].values[4].is_nan());
        assert!(!outputs[2].values[4].is_nan());
    }

    #[test]
    fn latest_keltner_matches_full_series_last_values() {
        let previous_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
        ]);
        let all_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let previous_outputs = keltner(&previous_bars, 3, 2.0, &mut HashMap::new());
        let full_outputs = keltner(&all_bars, 3, 2.0, &mut HashMap::new());
        let latest = latest_keltner(&all_bars, 3, 2.0, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(
            latest.0,
            output_at_vec(&full_outputs, "upper", all_bars.len() - 1)
        );
        assert_eq!(
            latest.1,
            output_at_vec(&full_outputs, "middle", all_bars.len() - 1)
        );
        assert_eq!(
            latest.2,
            output_at_vec(&full_outputs, "lower", all_bars.len() - 1)
        );
    }

    #[test]
    fn donchian_returns_upper_middle_lower() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 7.0, 10.0),
            (12.0, 6.0, 11.0),
            (13.0, 5.0, 12.0),
        ]);
        let outputs = donchian(&bars, 3, &mut HashMap::new());

        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[0].name, "upper");
        assert_eq!(outputs[1].name, "middle");
        assert_eq!(outputs[2].name, "lower");
        assert_eq!(outputs[0].values[3], 13.0);
        assert_eq!(outputs[1].values[3], 9.0);
        assert_eq!(outputs[2].values[3], 5.0);
    }

    #[test]
    fn latest_donchian_matches_full_series_last_values() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 7.0, 10.0),
            (12.0, 6.0, 11.0),
            (13.0, 5.0, 12.0),
        ]);
        let outputs = donchian(&bars, 3, &mut HashMap::new());
        let latest = latest_donchian(&bars, 3);

        assert_eq!(latest.0, outputs[0].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.1, outputs[1].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.2, outputs[2].values.last().copied().and_then(nan_to_none));
    }

    #[test]
    fn parabolic_sar_matches_latest_values() {
        let previous_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 10.0, 10.5),
            (12.0, 11.0, 11.5),
            (13.0, 12.0, 12.5),
        ]);
        let all_bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 10.0, 10.5),
            (12.0, 11.0, 11.5),
            (13.0, 12.0, 12.5),
            (14.0, 13.0, 13.5),
        ]);
        let previous_outputs = parabolic_sar(&previous_bars, 0.02, 0.2, &mut HashMap::new());
        let full_outputs = parabolic_sar(&all_bars, 0.02, 0.2, &mut HashMap::new());
        let latest = latest_parabolic_sar(&all_bars, 0.02, 0.2, &IndicatorArena::from_outputs(previous_outputs.clone()));

        assert_eq!(
            latest.0,
            output_at_vec(&full_outputs, "value", all_bars.len() - 1)
        );
        assert_eq!(latest.1, output_at_vec(&full_outputs, "ep", all_bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&full_outputs, "af", all_bars.len() - 1));
        assert_eq!(
            latest.3,
            output_at_vec(&full_outputs, "trend", all_bars.len() - 1)
        );
    }

    #[test]
    fn ichimoku_returns_five_lines() {
        let bars = bars(&(1..=60).map(|value| value as f64).collect::<Vec<_>>());
        let outputs = ichimoku(&bars, 9, 26, 52, &mut HashMap::new());
        let latest = latest_ichimoku(&bars, 9, 26, 52);

        assert_eq!(
            outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec!["tenkan", "kijun", "senkou_a", "senkou_b", "chikou"]
        );
        assert_eq!(latest.0, output_at_vec(&outputs, "tenkan", bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&outputs, "kijun", bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&outputs, "senkou_a", bars.len() - 1));
        assert_eq!(latest.3, output_at_vec(&outputs, "senkou_b", bars.len() - 1));
        assert_eq!(latest.4, output_at_vec(&outputs, "chikou", bars.len() - 1));
    }

    #[test]
    fn pivot_points_match_latest_values() {
        let bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 7.0, 10.0), (12.0, 6.0, 11.0)]);
        let outputs = pivot_points(&bars, &mut HashMap::new());
        let latest = latest_pivot_points(&bars);

        assert_eq!(latest.0, output_at_vec(&outputs, "pp", bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&outputs, "r1", bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&outputs, "s1", bars.len() - 1));
        assert_eq!(latest.3, output_at_vec(&outputs, "r2", bars.len() - 1));
        assert_eq!(latest.4, output_at_vec(&outputs, "s2", bars.len() - 1));
    }

    #[test]
    fn roc_matches_latest_value() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(
            latest_roc(&bars, 3),
            roc(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn aroon_matches_latest_values() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 7.0, 10.0),
            (12.0, 6.0, 11.0),
            (13.0, 5.0, 12.0),
        ]);
        let outputs = aroon(&bars, 3, &mut HashMap::new());
        let latest = latest_aroon(&bars, 3);

        assert_eq!(latest.0, output_at_vec(&outputs, "up", bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&outputs, "down", bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&outputs, "oscillator", bars.len() - 1));
    }

    #[test]
    fn cmf_matches_latest_value() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 7.0, 10.0),
            (12.0, 6.0, 11.0),
            (13.0, 5.0, 12.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0]) {
            bar.volume = volume;
        }
        assert_eq!(
            latest_cmf(&bars, 3),
            cmf(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn adl_matches_latest_value() {
        let mut previous_bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 7.0, 10.0)]);
        previous_bars[0].volume = 1.0;
        previous_bars[1].volume = 2.0;
        let mut all_bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 7.0, 10.0), (12.0, 6.0, 11.0)]);
        all_bars[0].volume = 1.0;
        all_bars[1].volume = 2.0;
        all_bars[2].volume = 3.0;
        let output = IndicatorOutput {
            name: "value".to_string(),
            values: adl(&previous_bars),
        };

        assert_eq!(
            latest_adl(&all_bars, Some(&output.values[..])),
            adl(&all_bars).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn vwma_matches_latest_value() {
        let mut bars = bars(&(1..=10).map(|value| value as f64).collect::<Vec<_>>());
        for (index, bar) in bars.iter_mut().enumerate() {
            bar.volume = (index + 1) as f64;
        }
        assert_eq!(
            latest_vwma(&bars, 5),
            vwma(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn williams_ad_matches_latest_value() {
        let previous_bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 7.0, 10.0)]);
        let all_bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 7.0, 10.0), (12.0, 6.0, 11.0)]);
        let output = IndicatorOutput {
            name: "value".to_string(),
            values: williams_ad(&previous_bars),
        };

        assert_eq!(
            latest_williams_ad(&all_bars, Some(&output.values[..])),
            williams_ad(&all_bars).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn chaikin_volatility_matches_latest_value() {
        let bars = ohlc(
            &(1..=25)
                .map(|value| {
                    let value = value as f64;
                    (value + 2.0, value - 1.0, value)
                })
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            latest_chaikin_volatility(&bars, 10),
            chaikin_volatility(&bars, 10).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn price_channel_matches_latest_values() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 7.0, 10.0),
            (12.0, 6.0, 11.0),
            (13.0, 5.0, 12.0),
        ]);
        let outputs = price_channel(&bars, 3, &mut HashMap::new());
        let latest = latest_price_channel(&bars, 3);

        assert_eq!(latest.0, outputs[0].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.1, outputs[1].values.last().copied().and_then(nan_to_none));
        assert_eq!(latest.2, outputs[2].values.last().copied().and_then(nan_to_none));
    }

    #[test]
    fn starc_matches_latest_values() {
        let bars = ohlc(&[
            (10.0, 9.0, 9.5),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (14.0, 10.0, 13.0),
            (15.0, 12.0, 14.0),
        ]);
        let outputs = starc(&bars, 3, 2.0, &mut HashMap::new());
        let latest = latest_starc(&bars, 3, 2.0);

        assert_eq!(latest.0, output_at_vec(&outputs, "upper", bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&outputs, "middle", bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&outputs, "lower", bars.len() - 1));
    }

    #[test]
    fn wma_matches_latest_value() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            latest_wma(&bars, 3),
            wma(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn hma_matches_latest_value() {
        let bars = bars(&(1..=10).map(|value| value as f64).collect::<Vec<_>>());
        let outputs = hma(&bars, 4, &mut HashMap::new());
        assert_eq!(latest_hma(&bars, 4), outputs.last().copied().and_then(nan_to_none));
    }

    #[test]
    fn linear_regression_matches_latest_value() {
        let bars = bars(&(1..=10).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_linear_regression(&bars, 5),
            linear_regression(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn dema_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_dema(&bars, 5),
            dema(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn tema_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_tema(&bars, 5),
            tema(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn trima_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_trima(&bars, 5),
            trima(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn stddev_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_stddev(&bars, 5),
            stddev(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn envelope_matches_latest_values() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        let outputs = envelope(&bars, 5, 2.0, &mut HashMap::new());
        let latest = latest_envelope(&bars, 5, 2.0);

        assert_eq!(latest.0, output_at_vec(&outputs, "upper", bars.len() - 1));
        assert_eq!(latest.1, output_at_vec(&outputs, "middle", bars.len() - 1));
        assert_eq!(latest.2, output_at_vec(&outputs, "lower", bars.len() - 1));
    }

    #[test]
    fn trix_matches_latest_value() {
        let bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_trix(&bars, 5),
            trix(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn tsi_matches_latest_value() {
        let bars = bars(&(1..=40).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_tsi(&bars, 25, 13),
            tsi(&bars, 25, 13).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn kst_matches_latest_value() {
        let bars = bars(&(1..=60).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(latest_kst(&bars), kst(&bars).last().copied().and_then(nan_to_none));
    }

    #[test]
    fn bop_matches_latest_value() {
        let bars = ohlc(&[(10.0, 8.0, 9.0), (11.0, 9.0, 10.0), (12.0, 10.0, 11.0)]);
        assert_eq!(latest_bop(&bars), bop(&bars).last().copied().and_then(nan_to_none));
    }

    #[test]
    fn dpo_matches_latest_value() {
        let bars = bars(&(1..=40).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_dpo(&bars, 20),
            dpo(&bars, 20).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn momentum_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_momentum(&bars, 10),
            momentum(&bars, 10).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn ultimate_oscillator_matches_latest_value() {
        let bars = ohlc(
            &(1..=40)
                .map(|value| {
                    let value = value as f64;
                    (value + 1.0, value - 1.0, value)
                })
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            latest_ultimate_oscillator(&bars, 7, 14, 28),
            ultimate_oscillator(&bars, 7, 14, 28)
                .last()
                .copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn chaikin_oscillator_matches_latest_value() {
        let mut bars = ohlc(
            &(1..=20)
                .map(|value| {
                    let value = value as f64;
                    (value + 1.0, value - 1.0, value)
                })
                .collect::<Vec<_>>(),
        );
        for (bar, volume) in bars.iter_mut().zip(1..=20) {
            bar.volume = volume as f64;
        }
        let params = MacdParams {
            fast: 3,
            slow: 10,
            signal: 9,
        };
        assert_eq!(
            latest_chaikin_oscillator(&bars, params),
            chaikin_oscillator(&bars, params).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn force_index_matches_latest_value() {
        let mut bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        for (bar, volume) in bars.iter_mut().zip(1..=20) {
            bar.volume = volume as f64;
        }
        assert_eq!(
            latest_force_index(&bars, 13),
            force_index(&bars, 13).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn ppo_matches_latest_values() {
        let bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        let params = MacdParams {
            fast: 12,
            slow: 26,
            signal: 9,
        };
        let outputs = ppo(&bars, params, &mut HashMap::new());
        let latest = latest_ppo(&bars, params);
        let index = bars.len() - 1;
        assert_eq!(latest.0, output_at_vec(&outputs, "ppo", index));
        assert_eq!(latest.1, output_at_vec(&outputs, "signal", index));
        assert_eq!(latest.2, output_at_vec(&outputs, "histogram", index));
    }

    #[test]
    fn remove_indicator_reports_if_it_removed_one() {
        let mut engine = ChartEngine::new();
        let id = engine
            .add_indicator_from_config(IndicatorConfig {
                kind: "SMA".to_string(),
                period: Some(2),
                stoch_period: None,
                smooth: None,
                fast: None,
                slow: None,
                signal: None,
                multiplier: None,
                tenkan_period: None,
                kijun_period: None,
                senkou_b_period: None,
                psar_step: None,
                psar_max_step: None,
            })
            .unwrap();
        assert!(engine.remove_indicator(id));
        assert!(!engine.remove_indicator(id));
    }

    #[test]
    fn vwap_does_not_require_period() {
        let mut engine = ChartEngine::new();
        assert!(engine
            .add_indicator_from_config(IndicatorConfig {
                kind: "VWAP".to_string(),
                period: None,
                stoch_period: None,
                smooth: None,
                fast: None,
                slow: None,
                signal: None,
                multiplier: None,
                tenkan_period: None,
                kijun_period: None,
                senkou_b_period: None,
                psar_step: None,
                psar_max_step: None,
            })
            .is_ok());
    }

    #[test]
    fn upsert_bar_replaces_latest_or_appends_next() {
        let mut bars = bars(&[1.0, 2.0]);
        upsert_bar(
            &mut bars,
            Bar {
                time: 1,
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 3.0,
                volume: 1.0,
            },
        );
        upsert_bar(
            &mut bars,
            Bar {
                time: 2,
                open: 4.0,
                high: 4.0,
                low: 4.0,
                close: 4.0,
                volume: 1.0,
            },
        );

        assert_eq!(bars.len(), 3);
        assert_eq!(bars[1].close, 3.0);
        assert_eq!(bars[2].close, 4.0);
    }
}
