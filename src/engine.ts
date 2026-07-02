import initWasm, {
  ChartEngine as WasmChartEngine,
  type InitInput,
} from "../pkg/rapidchart_engine";

export type Bar = {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
};

export type IndicatorPoint = {
  time: number;
  value: number | null;
};

export type IndicatorOutputPoint = IndicatorPoint & {
  output: string;
};

export type IndicatorOutputSeries = {
  output: string;
  points: IndicatorPoint[];
};

type RawIndicatorOutput = {
  name: string;
  values: Array<number | null>;
};

type RawIndicatorLatestValue = {
  output: string;
  value: number | null;
};

export type IndicatorKind =
  | "SMA"
  | "EMA"
  | "RSI"
  | "CCI"
  | "MFI"
  | "WILLIAMS_R"
  | "ATR"
  | "ADX"
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
  | "STOCHASTIC"
  | "STOCH_RSI"
  | "MACD"
  | "PPO"
  | "BB"
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
  | "OBV"
  | "VWAP";

export type IndicatorConfig = {
  kind: IndicatorKind;
  period?: number;
  stoch_period?: number;
  smooth?: number;
  fast?: number;
  slow?: number;
  signal?: number;
  multiplier?: number;
  tenkan_period?: number;
  kijun_period?: number;
  senkou_b_period?: number;
  psar_step?: number;
  psar_max_step?: number;
};

export type ParamDescriptor = {
  name: string;
  label: string;
  default: number;
  min: number;
  step: string;
};

export type OutputDescriptor = {
  name: string;
  renderer: "line" | "histogram";
  pane: "overlay" | "separate";
  color: string;
};

export type IndicatorDescriptor = {
  kind: IndicatorKind;
  name: string;
  pane: "overlay" | "separate";
  params: ParamDescriptor[];
  outputs: OutputDescriptor[];
};

export type DagDebug = {
  nodes: string[];
  edges: { from: string; to: string }[];
};

export async function initEngine(
  moduleOrPath?: InitInput | Promise<InitInput>,
): Promise<void> {
  await initWasm(moduleOrPath);
}

export class RapidChartEngine {
  readonly #engine: WasmChartEngine;
  #configs = new Map<number, IndicatorConfig>();

  constructor() {
    this.#engine = new WasmChartEngine();
  }

  ingestBars(bars: Bar[]): void {
    this.#engine.ingest_bars(bars);
  }

  upsertBar(bar: Bar): void {
    this.#engine.upsert_bar(bar);
  }

  candles(): Bar[] {
    return this.#engine.candles() as Bar[];
  }

  addIndicator(config: IndicatorConfig): number {
    const id = this.#engine.add_indicator_config(config);
    this.#configs.set(id, { ...config });
    return id;
  }

  removeIndicator(id: number): boolean {
    this.#configs.delete(id);
    return this.#engine.remove_indicator(id);
  }

  indicatorDescriptors(): IndicatorDescriptor[] {
    return this.#engine.indicator_descriptors() as IndicatorDescriptor[];
  }

  // Render mapping stays in TS: Rust owns raw candle/output data, TS pairs values with times.
  indicatorSeries(id: number): IndicatorOutputSeries[] {
    const outputs = this.#engine.indicator_outputs_all(id) as RawIndicatorOutput[];
    const bars = this.candles();
    const spacing = seriesSpacingSeconds(bars);
    const config = this.#configs.get(id);
    return outputs.map((output) => ({
      output: output.name,
      points: bars.map((bar, index) => ({
        time: shiftedOutputTime(bar.time, spacing, indicatorOutputShift(config, output.name)),
        value: output.values[index] ?? null,
      })),
    }));
  }

  latestIndicatorPoints(id: number): IndicatorOutputPoint[] {
    const bars = this.candles();
    const bar = bars.at(-1);
    if (!bar) return [];
    const outputs = this.#engine.latest_indicator_values(id) as RawIndicatorLatestValue[];
    const spacing = seriesSpacingSeconds(bars);
    const config = this.#configs.get(id);
    return outputs.map((output) => ({
      output: output.output,
      time: shiftedOutputTime(bar.time, spacing, indicatorOutputShift(config, output.output)),
      value: output.value,
    }));
  }

  dagDebug(): DagDebug {
    return this.#engine.dag_debug() as DagDebug;
  }
}

function seriesSpacingSeconds(bars: Bar[]) {
  return bars
    .slice(1)
    .map((bar, index) => bar.time - bars[index].time)
    .filter((value) => value > 0)
    .reduce((min, value) => Math.min(min, value), 60);
}

function indicatorOutputShift(config: IndicatorConfig | undefined, output: string) {
  if (config?.kind !== "ICHIMOKU") return 0;
  const shift = config.kijun_period ?? 26;
  if (output === "senkou_a" || output === "senkou_b") return shift;
  if (output === "chikou") return -shift;
  return 0;
}

function shiftedOutputTime(baseTime: number, spacing: number, shift: number) {
  const delta = spacing * Math.abs(shift);
  return shift >= 0 ? baseTime + delta : baseTime - delta;
}
