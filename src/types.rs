use serde::{Deserialize, Serialize};

pub(crate) struct Indicator {
    pub id: u32,
    pub kind: String,
    pub period: usize,
    pub stoch_period: usize,
    pub smooth: usize,
    pub signal: usize,
    pub tenkan_period: usize,
    pub kijun_period: usize,
    pub senkou_b_period: usize,
    pub macd: Option<MacdParams>,
    pub multiplier: f64,
    pub psar_step: f64,
    pub psar_max_step: f64,
    pub anchor: usize,
    pub outputs: IndicatorArena,
}

#[derive(Clone, Copy)]
pub(crate) struct MacdParams {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

#[derive(Deserialize)]
pub(crate) struct IndicatorConfig {
    pub kind: String,
    pub period: Option<usize>,
    pub stoch_period: Option<usize>,
    pub smooth: Option<usize>,
    pub fast: Option<usize>,
    pub slow: Option<usize>,
    pub signal: Option<usize>,
    pub multiplier: Option<f64>,
    pub tenkan_period: Option<usize>,
    pub kijun_period: Option<usize>,
    pub senkou_b_period: Option<usize>,
    pub psar_step: Option<f64>,
    pub psar_max_step: Option<f64>,
    pub anchor: Option<usize>,
}

#[derive(Clone, Serialize)]
pub(crate) struct IndicatorOutput {
    pub name: String,
    pub values: Vec<f64>,
}

/// Per-slot Vec storage for indicator outputs. Each slot is an independent
/// Vec<f64> that can be appended to without copying other slots.
pub(crate) struct IndicatorArena {
    names: Vec<String>,
    values: Vec<Vec<f64>>,
}

impl IndicatorArena {
    /// Create from a Vec<IndicatorOutput>.
    pub(crate) fn from_outputs(outputs: Vec<IndicatorOutput>) -> Self {
        let mut names = Vec::with_capacity(outputs.len());
        let mut values = Vec::with_capacity(outputs.len());
        for output in outputs {
            names.push(output.name);
            values.push(output.values);
        }
        Self { names, values }
    }

    /// Get the slice for a named output.
    pub(crate) fn get(&self, name: &str) -> Option<&[f64]> {
        let idx = self.names.iter().position(|s| s == name)?;
        Some(&self.values[idx])
    }

    /// Get value at a specific index for a named output.
    pub(crate) fn value_at(&self, name: &str, index: usize) -> Option<f64> {
        let idx = self.names.iter().position(|s| s == name)?;
        self.values[idx]
            .get(index)
            .copied()
            .and_then(|v| if v.is_nan() { None } else { Some(v) })
    }

    /// Ensure all slots have at least `target_len` values (pad with NaN).
    #[inline]
    pub(crate) fn ensure_len(&mut self, target_len: usize) {
        for slot in &mut self.values {
            if slot.len() < target_len {
                slot.resize(target_len, f64::NAN);
            }
        }
    }

    /// Set the last value for a named output. If the slot doesn't exist, create it.
    #[inline]
    pub(crate) fn upsert_last(&mut self, name: &str, target_len: usize, value: f64) {
        if let Some(idx) = self.names.iter().position(|s| s == name) {
            let slot = &mut self.values[idx];
            if slot.len() < target_len {
                slot.resize(target_len, f64::NAN);
            }
            let last = slot.len() - 1;
            slot[last] = value;
        } else {
            // New slot — create NaN-filled Vec, set last value
            self.names.push(name.to_string());
            let mut slot = vec![f64::NAN; target_len];
            if target_len > 0 {
                slot[target_len - 1] = value;
            }
            self.values.push(slot);
        }
    }

    /// Resolve a slot name to its index. Returns None if the slot doesn't exist.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn slot_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|s| s == name)
    }

    /// Set the last value for a slot by its pre-resolved index. No string lookup.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn upsert_last_at(&mut self, slot_idx: usize, target_len: usize, value: f64) {
        let slot = &mut self.values[slot_idx];
        if slot.len() < target_len {
            slot.resize(target_len, f64::NAN);
        }
        let last = slot.len() - 1;
        slot[last] = value;
    }

    /// Iterate over (name, slice) for all slots.
    pub(crate) fn iter_slots(&self) -> impl Iterator<Item = (&str, &[f64])> {
        self.names
            .iter()
            .zip(self.values.iter())
            .map(|(name, values)| (name.as_str(), values.as_slice()))
    }
}

#[derive(Serialize)]
pub(crate) struct IndicatorLatestValue {
    pub output: String,
    pub value: Option<f64>,
}

#[derive(Default, Serialize)]
pub(crate) struct DagDebug {
    pub nodes: Vec<String>,
    pub edges: Vec<DagEdge>,
}

#[derive(Serialize)]
pub(crate) struct DagEdge {
    pub from: String,
    pub to: String,
}
