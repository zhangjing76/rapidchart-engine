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
}

#[derive(Clone, Serialize)]
pub(crate) struct IndicatorOutput {
    pub name: String,
    pub values: Vec<f64>,
}

/// Packed contiguous storage for all outputs of a single indicator.
/// All output slots share the same length (= bars count). The backing
/// Vec stores them sequentially: [slot_0: len values][slot_1: len values]...
pub(crate) struct IndicatorArena {
    data: Vec<f64>,
    /// Metadata per slot: (name, slot_index) — slot_index is used to compute offset as slot_index * slot_len.
    slots: Vec<String>,
    /// Number of f64 values per slot (= bars count for this indicator).
    slot_len: usize,
}

impl IndicatorArena {
    /// Create from a Vec<IndicatorOutput>, packing all outputs into one allocation.
    pub(crate) fn from_outputs(outputs: Vec<IndicatorOutput>) -> Self {
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
    pub(crate) fn get(&self, name: &str) -> Option<&[f64]> {
        let idx = self.slots.iter().position(|s| s == name)?;
        let start = idx * self.slot_len;
        Some(&self.data[start..start + self.slot_len])
    }

    /// Get value at a specific index for a named output.
    pub(crate) fn value_at(&self, name: &str, index: usize) -> Option<f64> {
        let idx = self.slots.iter().position(|s| s == name)?;
        let start = idx * self.slot_len;
        self.data.get(start + index).copied().and_then(|v| if v.is_nan() { None } else { Some(v) })
    }

    /// Resize all slots to a new length (for incremental append).
    /// Fills new slots with NaN. If the slot doesn't exist, creates it.
    pub(crate) fn resize_all(&mut self, new_len: usize) {
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
    pub(crate) fn upsert_last(&mut self, name: &str, target_len: usize, value: f64) {
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
    pub(crate) fn iter_slots(&self) -> impl Iterator<Item = (&str, &[f64])> {
        self.slots.iter().enumerate().map(move |(i, name)| {
            let start = i * self.slot_len;
            (name.as_str(), &self.data[start..start + self.slot_len])
        })
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
