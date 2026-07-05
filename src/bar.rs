use serde::{Deserialize, Serialize};

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
pub(crate) struct CandleStore {
    pub time: Vec<u32>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
}

impl CandleStore {
    pub(crate) fn from_bars(bars: Vec<Bar>) -> Self {
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

    pub(crate) fn from_columns(columns: CandleColumnsInput) -> Result<Self, &'static str> {
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

    pub(crate) fn from_raw_columns(
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
        Self {
            time,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.time.len()
    }

    pub(crate) fn push(&mut self, bar: Bar) {
        self.time.push(bar.time);
        self.open.push(bar.open);
        self.high.push(bar.high);
        self.low.push(bar.low);
        self.close.push(bar.close);
        self.volume.push(bar.volume);
    }

    pub(crate) fn set_last(&mut self, bar: Bar) {
        let index = self.len() - 1;
        self.time[index] = bar.time;
        self.open[index] = bar.open;
        self.high[index] = bar.high;
        self.low[index] = bar.low;
        self.close[index] = bar.close;
        self.volume[index] = bar.volume;
    }

    pub(crate) fn last_time(&self) -> Option<u32> {
        self.time.last().copied()
    }

    pub(crate) fn last_close(&self) -> Option<f64> {
        self.close.last().copied()
    }

    pub(crate) fn to_bars(&self) -> Vec<Bar> {
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

#[derive(Deserialize)]
pub(crate) struct CandleColumnsInput {
    pub time: Vec<u32>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
}
