use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct IndicatorDescriptor {
    pub kind: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub pane: &'static str,
    pub params: Vec<ParamDescriptor>,
    pub outputs: Vec<OutputDescriptor>,
}

#[derive(Serialize)]
pub(crate) struct ParamDescriptor {
    pub name: &'static str,
    pub label: &'static str,
    pub default: f64,
    pub min: f64,
    pub step: &'static str,
}

#[derive(Serialize)]
pub(crate) struct OutputDescriptor {
    pub name: &'static str,
    pub renderer: &'static str,
    pub pane: &'static str,
    pub color: &'static str,
}

pub(crate) fn period_descriptor(
    kind: &'static str,
    name: &'static str,
    category: &'static str,
    pane: &'static str,
    default: usize,
) -> IndicatorDescriptor {
    IndicatorDescriptor {
        kind,
        name,
        category,
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

pub(crate) fn output_descriptor(
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
