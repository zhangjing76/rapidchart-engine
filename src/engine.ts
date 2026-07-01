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

  constructor(symbol: string, timeframe: string) {
    this.#engine = new WasmChartEngine(symbol, timeframe);
  }

  symbol(): string {
    return this.#engine.symbol();
  }

  timeframe(): string {
    return this.#engine.timeframe();
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
    return this.#engine.add_indicator_config(config);
  }

  removeIndicator(id: number): boolean {
    return this.#engine.remove_indicator(id);
  }

  indicatorDescriptors(): IndicatorDescriptor[] {
    return this.#engine.indicator_descriptors() as IndicatorDescriptor[];
  }

  indicatorSeries(id: number): IndicatorOutputSeries[] {
    return this.#engine.indicator_series_all(id) as IndicatorOutputSeries[];
  }

  latestIndicatorPoints(id: number): IndicatorOutputPoint[] {
    return this.#engine.latest_indicator_points(id) as IndicatorOutputPoint[];
  }

  dagDebug(): DagDebug {
    return this.#engine.dag_debug() as DagDebug;
  }
}
