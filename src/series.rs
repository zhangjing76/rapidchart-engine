use std::collections::HashMap;
use std::rc::Rc;

/// Alias for a computed indicator series.
pub(crate) type Series = Vec<f64>;

/// Reference-counted indicator series, shared via NodeCache.
pub(crate) type RcSeries = Rc<Series>;

/// Cache for intermediate computation results, keyed by descriptive string.
pub(crate) type NodeCache = HashMap<String, RcSeries>;

/// Unwrap an Rc<Vec<f64>> into owned Vec<f64>. Zero-cost if sole owner, clones if shared.
#[inline]
pub(crate) fn rc_into_owned(rc: RcSeries) -> Series {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}

#[inline(always)]
pub(crate) fn nan_to_none(v: f64) -> Option<f64> {
    if v.is_nan() {
        None
    } else {
        Some(v)
    }
}
