#![allow(clippy::collapsible_else_if)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::manual_memcpy)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::type_complexity)]

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
    latest_values_scratch: Vec<f64>,
}

impl Default for ChartEngine {
    fn default() -> Self {
        Self {
            bars: CandleStore::default(),
            indicators: Vec::new(),
            next_indicator_id: 1,
            dag: DagDebug::default(),
            latest_values_scratch: Vec::new(),
        }
    }
}

#[wasm_bindgen]
impl ChartEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
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

    pub fn upsert_bar_fast(
        &mut self,
        time: u32,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<(), JsValue> {
        if upsert_candle_store(
            &mut self.bars,
            Bar {
                time,
                open,
                high,
                low,
                close,
                volume,
            },
        ) && !self.update_indicators_incremental()
        {
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
        let kind_name = config.kind.to_uppercase();
        let kind = kind_name.parse::<IndicatorKind>().map_err(|_| {
            JsValue::from_str(&format!("unsupported indicator kind: {}", kind_name))
        })?;
        if !is_valid_kind(&kind_name) {
            return Err(JsValue::from_str(&format!(
                "unsupported indicator kind: {}",
                kind_name
            )));
        }
        let macd = if kind.uses_macd_params() {
            Some(MacdParams {
                fast: config
                    .fast
                    .unwrap_or(if kind == IndicatorKind::CHAIKIN_OSCILLATOR {
                        3
                    } else if kind == IndicatorKind::MA_CROSS {
                        10
                    } else if kind == IndicatorKind::VOLUME_OSCILLATOR {
                        5
                    } else {
                        12
                    }),
                slow: config
                    .slow
                    .unwrap_or(if kind == IndicatorKind::CHAIKIN_OSCILLATOR {
                        10
                    } else if kind == IndicatorKind::MA_CROSS {
                        20
                    } else if kind == IndicatorKind::VOLUME_OSCILLATOR {
                        10
                    } else {
                        26
                    }),
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
        let anchor = config.anchor.unwrap_or(0);
        validate_indicator(
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
            anchor,
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
        let mut nodes = HashMap::new();
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
                indicator.kind,
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
                indicator.anchor,
                &mut nodes,
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
        // Add derived series (hl2, hlc3) as DAG nodes if any indicator used them.
        for (key, sources) in [
            ("hl2", &["high", "low"][..]),
            ("hlc3", &["high", "low", "close"][..]),
        ] {
            if nodes.contains_key(key) && !dag.nodes.contains(&key.to_string()) {
                dag.nodes.push(key.to_string());
                for src in sources {
                    dag.edges.push(DagEdge {
                        from: src.to_string(),
                        to: key.to_string(),
                    });
                }
            }
        }
        self.dag = dag;
    }

    fn update_indicators_incremental(&mut self) -> bool {
        let target_len = self.bars.len();
        for indicator in &mut self.indicators {
            update_indicator_incremental(&self.bars, indicator, target_len);
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
}
