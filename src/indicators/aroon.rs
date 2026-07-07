use crate::value_at_slice;
use crate::NodeCache;
use crate::CandleStore;
use std::collections::HashMap;
use std::rc::Rc;

pub fn aroon_store(
    store: &CandleStore,
    period: usize,
    nodes: &mut NodeCache,
) -> Vec<crate::NamedSeries> {
    let key = format!("aroon:hl:{period}");
    if let Some(values) = nodes.get(&key) {
        return vec![
            crate::named_series("up", Rc::clone(values)),
            crate::named_series(
                "down",
                nodes.get(&format!("aroon:down:{period}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
            crate::named_series(
                "oscillator",
                nodes.get(&format!("aroon:oscillator:{period}"))
                    .map(Rc::clone)
                    .unwrap_or_else(|| Rc::new(Vec::new())),
            ),
        ];
    }
    let mut up = vec![f64::NAN; store.len()];
    let mut down = vec![f64::NAN; store.len()];
    let mut oscillator = vec![f64::NAN; store.len()];
    if period == 0 || store.len() < period {
        return vec![
            crate::named_series("up", up),
            crate::named_series("down", down),
            crate::named_series("oscillator", oscillator),
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
    nodes.insert(
        format!("aroon:oscillator:{period}"),
        Rc::new(oscillator.clone()),
    );
    vec![
        crate::named_series("up", up),
        crate::named_series("down", down),
        crate::named_series("oscillator", oscillator),
    ]
}
pub fn latest_aroon_store(
    store: &CandleStore,
    period: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let outputs = aroon_store(store, period, &mut HashMap::new());
    let index = store.len().saturating_sub(1);
    (
        value_at_slice(outputs[0].values.as_slice(), index),
        value_at_slice(outputs[1].values.as_slice(), index),
        value_at_slice(outputs[2].values.as_slice(), index),
    )
}
