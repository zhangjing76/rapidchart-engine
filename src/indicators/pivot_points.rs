use crate::NodeCache;
use crate::CandleStore;
use std::rc::Rc;

type PivotPointsResult = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

pub fn pivot_points_store(store: &CandleStore, nodes: &mut NodeCache) -> Vec<crate::NamedSeries> {
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
        crate::named_series("pp", pp,),
        crate::named_series("r1", r1,),
        crate::named_series("s1", s1,),
        crate::named_series("r2", r2,),
        crate::named_series("s2", s2,),
    ]
}
pub fn latest_pivot_points_store(store: &CandleStore) -> PivotPointsResult {
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
