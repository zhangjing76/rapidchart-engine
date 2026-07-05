use js_sys::{Array, Float64Array, Object, Uint32Array};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

// ── Module declarations ──────────────────────────────────────────────
mod bar;
mod dag;
mod descriptors;
mod dispatch;
mod helpers;
mod indicators;
mod series;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use bar::*;
pub(crate) use dag::*;
pub(crate) use descriptors::*;
pub(crate) use dispatch::*;
pub(crate) use helpers::*;
pub(crate) use indicators::*;
pub(crate) use series::*;
pub(crate) use types::*;

// ───────────────────────────────────────────
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
        js_set(
            &out,
            "time_ptr",
            JsValue::from_f64(time.as_ptr() as u32 as f64),
        )?;
        js_set(
            &out,
            "open_ptr",
            JsValue::from_f64(open.as_ptr() as u32 as f64),
        )?;
        js_set(
            &out,
            "high_ptr",
            JsValue::from_f64(high.as_ptr() as u32 as f64),
        )?;
        js_set(
            &out,
            "low_ptr",
            JsValue::from_f64(low.as_ptr() as u32 as f64),
        )?;
        js_set(
            &out,
            "close_ptr",
            JsValue::from_f64(close.as_ptr() as u32 as f64),
        )?;
        js_set(
            &out,
            "volume_ptr",
            JsValue::from_f64(volume.as_ptr() as u32 as f64),
        )?;
        js_set(&out, "len", JsValue::from_f64(len as f64))?;

        self.bars = CandleStore {
            time,
            open,
            high,
            low,
            close,
            volume,
        };
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
        js_set(&out, "close", unsafe {
            Float64Array::view(&self.bars.close)
        })?;
        js_set(&out, "volume", unsafe {
            Float64Array::view(&self.bars.volume)
        })?;
        Ok(out.into())
    }

    pub fn indicator_outputs_all(&self, id: u32) -> Result<JsValue, JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let outputs: Vec<_> = indicator
            .outputs
            .iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .map(|(name, values)| IndicatorOutput {
                name: name.to_string(),
                values: values.to_vec(),
            })
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
        for (name, values) in indicator
            .outputs
            .iter_slots()
            .filter(|(name, _)| is_visible_output(name))
        {
            let item = Object::new();
            js_set(&item, "name", JsValue::from_str(name))?;
            js_set(&item, "values", unsafe { Float64Array::view(values) })?;
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
        let points: Vec<_> = indicator
            .outputs
            .iter_slots()
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
                    let value = latest_ema_store(
                        &self.bars,
                        indicator.period,
                        indicator.outputs.get("value"),
                    );
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
                    let value = latest_atr_store(
                        &self.bars,
                        indicator.period,
                        indicator.outputs.get("value"),
                    );
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
                    let value =
                        latest_williams_ad_store(&self.bars, indicator.outputs.get("value"));
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
        self.latest_values_scratch.extend(
            indicator
                .outputs
                .iter_slots()
                .filter(|(name, _)| is_visible_output(name))
                .map(|(_, values)| values.last().copied().unwrap_or(f64::NAN)),
        );
        Ok(self.latest_values_scratch.as_slice())
    }

    fn populate_indicator_output_values_scratch(&mut self, id: u32) -> Result<(), JsValue> {
        let indicator = self
            .indicators
            .iter()
            .find(|indicator| indicator.id == id)
            .ok_or_else(|| JsValue::from_str("indicator not found"))?;
        let visible_count = indicator
            .outputs
            .iter_slots()
            .filter(|(name, _)| is_visible_output(name))
            .count();
        if self.indicator_values_scratch.len() < visible_count {
            self.indicator_values_scratch
                .resize_with(visible_count, Vec::new);
        }
        for (index, (_, values)) in indicator
            .outputs
            .iter_slots()
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
