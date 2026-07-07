import initWasm, {
  ChartEngine as WasmChartEngine,
  type InitInput,
  type InitOutput,
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

export type CandleColumns = {
  time: Uint32Array;
  open: Float64Array;
  high: Float64Array;
  low: Float64Array;
  close: Float64Array;
  volume: Float64Array;
};

export type IndicatorValueSeries = {
  output: string;
  values: Float64Array;
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
  | "ADL"
  | "ADX"
  | "ALLIGATOR"
  | "ANCHORED_VWAP"
  | "AROON"
  | "ATR"
  | "ATR_BANDS"
  | "AWESOME_OSCILLATOR"
  | "BB"
  | "BETA"
  | "BOLLINGER_BANDWIDTH"
  | "BOLLINGER_PCT_B"
  | "BOP"
  | "CCI"
  | "CENTER_OF_GRAVITY"
  | "CHAIKIN_OSCILLATOR"
  | "CHAIKIN_VOLATILITY"
  | "CHANDE_FORECAST"
  | "CHANDE_MOMENTUM"
  | "CHOPPINESS_INDEX"
  | "CMF"
  | "COPPOCK_CURVE"
  | "CORRELATION_COEFFICIENT"
  | "DARVAS_BOX"
  | "DEMA"
  | "DISPARITY_INDEX"
  | "DONCHIAN"
  | "DONCHIAN_WIDTH"
  | "DPO"
  | "EASE_OF_MOVEMENT"
  | "EHLER_FISHER"
  | "ELDER_IMPULSE"
  | "ELDER_RAY"
  | "EMA"
  | "ENVELOPE"
  | "FORCE_INDEX"
  | "FRACTAL_CHAOS_BANDS"
  | "FRACTAL_CHAOS_OSCILLATOR"
  | "GATOR_OSCILLATOR"
  | "GMMA"
  | "GONOGO_TREND"
  | "GOPALAKRISHNAN_RANGE"
  | "HIGH_LOW_BANDS"
  | "HIGH_MINUS_LOW"
  | "HIGHEST_HIGH"
  | "HISTORICAL_VOLATILITY"
  | "HMA"
  | "ICHIMOKU"
  | "INTRADAY_MOMENTUM"
  | "KELTNER"
  | "KLINGER_VOLUME"
  | "KST"
  | "LINEAR_REG_FORECAST"
  | "LINEAR_REG_INTERCEPT"
  | "LINEAR_REG_SLOPE"
  | "LINEAR_REG_R2"
  | "LINEAR_REGRESSION"
  | "LOWEST_LOW"
  | "MA_CROSS"
  | "MA_DEVIATION"
  | "MACD"
  | "MARKET_FACILITATION"
  | "MASS_INDEX"
  | "MEDIAN_PRICE"
  | "MFI"
  | "MOMENTUM"
  | "NEGATIVE_VOLUME_INDEX"
  | "OBV"
  | "PARABOLIC_SAR"
  | "PERFORMANCE_INDEX"
  | "PIVOT_POINTS"
  | "POSITIVE_VOLUME_INDEX"
  | "PPO"
  | "PRETTY_GOOD_OSCILLATOR"
  | "PRICE_CHANNEL"
  | "PRICE_MOMENTUM_OSCILLATOR"
  | "PRICE_OSCILLATOR"
  | "PRICE_RELATIVE"
  | "PRICE_VOLUME_TREND"
  | "PRIME_NUMBER_BANDS"
  | "PRIME_NUMBER_OSCILLATOR"
  | "PROJECTED_AGGREGATE_VOLUME"
  | "PROJECTED_VOLUME_AT_TIME"
  | "PSYCHOLOGICAL_LINE"
  | "QSTICK"
  | "RAINBOW_MA"
  | "RAINBOW_OSCILLATOR"
  | "RANDOM_WALK_INDEX"
  | "RAVI"
  | "RELATIVE_VIGOR"
  | "RELATIVE_VOLATILITY"
  | "ROC"
  | "RSI"
  | "SCHAFF_TREND_CYCLE"
  | "SHINOHARA_INTENSITY"
  | "SMA"
  | "STARC"
  | "STDDEV"
  | "STOCH_RSI"
  | "STOCHASTIC"
  | "STOCHASTIC_MOMENTUM"
  | "SUPERTREND"
  | "SWING_INDEX"
  | "TEMA"
  | "TIME_SERIES_FORECAST"
  | "TRADE_VOLUME_INDEX"
  | "TREND_INTENSITY"
  | "TRIMA"
  | "TRIX"
  | "TRUE_RANGE"
  | "TSI"
  | "TWIGGS_MONEY_FLOW"
  | "TYPICAL_PRICE"
  | "ULCER_INDEX"
  | "ULTIMATE_OSCILLATOR"
  | "VALUATION_LINES"
  | "VERTICAL_HORIZONTAL_FILTER"
  | "VOLUME_CHART"
  | "VOLUME_OSCILLATOR"
  | "VOLUME_PROFILE"
  | "VOLUME_ROC"
  | "VOLUME_UNDERLAY"
  | "VORTEX_INDICATOR"
  | "VWAP"
  | "VWMA"
  | "WEIGHTED_CLOSE"
  | "WILLIAMS_AD"
  | "WILLIAMS_R"
  | "WMA"
  | "ZIGZAG";

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
  anchor?: number;
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
  category: string;
  pane: "overlay" | "separate";
  params: ParamDescriptor[];
  outputs: OutputDescriptor[];
};

export type DagDebug = {
  nodes: string[];
  edges: { from: string; to: string }[];
};

let wasmMemory: WebAssembly.Memory | undefined;

export async function initEngine(
  moduleOrPath?: InitInput | Promise<InitInput>,
): Promise<void> {
  const output = await initWasm(moduleOrPath);
  wasmMemory = output.memory;
}

export function getWasmMemory(): WebAssembly.Memory {
  if (!wasmMemory) throw new Error("Engine not initialized. Call initEngine() first.");
  return wasmMemory;
}

export class RapidChartEngine {
  readonly #engine: WasmChartEngine;
  #configs = new Map<number, IndicatorConfig>();
  #lastBarTime: number | undefined;
  #seriesSpacing = 60;

  constructor() {
    this.#engine = new WasmChartEngine();
  }

  ingestBars(bars: Bar[]): void {
    this.#engine.ingest_bars(bars);
    this.#lastBarTime = bars.at(-1)?.time;
    this.#seriesSpacing = seriesSpacingFromBars(bars);
  }

  ingestColumns(columns: CandleColumns): void {
    this.#engine.ingest_columns(columns);
    this.#lastBarTime = columns.time.at(-1);
    this.#seriesSpacing = seriesSpacingSeconds(columns.time);
  }

  ingestColumnsFast(columns: CandleColumns): void {
    this.#engine.ingest_columns_fast(
      columns.time,
      columns.open,
      columns.high,
      columns.low,
      columns.close,
      columns.volume,
    );
    this.#lastBarTime = columns.time.at(-1);
    this.#seriesSpacing = seriesSpacingSeconds(columns.time);
  }

  /**
   * Zero-copy ingest: writes candle data directly into WASM linear memory,
   * avoiding the TypedArray-to-Vec copy that ingestColumnsFast performs.
   */
  ingestColumnsZeroCopy(columns: CandleColumns): void {
    const len = columns.time.length;
    const memory = getWasmMemory();
    const ptrs = this.#engine.alloc_candle_buffer(len) as {
      time_ptr: number;
      open_ptr: number;
      high_ptr: number;
      low_ptr: number;
      close_ptr: number;
      volume_ptr: number;
      len: number;
    };

    // Write directly into WASM linear memory — one copy from JS heap to WASM heap
    new Uint32Array(memory.buffer, ptrs.time_ptr, len).set(columns.time);
    new Float64Array(memory.buffer, ptrs.open_ptr, len).set(columns.open);
    new Float64Array(memory.buffer, ptrs.high_ptr, len).set(columns.high);
    new Float64Array(memory.buffer, ptrs.low_ptr, len).set(columns.low);
    new Float64Array(memory.buffer, ptrs.close_ptr, len).set(columns.close);
    new Float64Array(memory.buffer, ptrs.volume_ptr, len).set(columns.volume);

    this.#engine.finalize_candle_buffer();
    this.#lastBarTime = columns.time.at(-1);
    this.#seriesSpacing = seriesSpacingSeconds(columns.time);
  }

  upsertBar(bar: Bar): void {
    this.#engine.upsert_bar(bar);
    this.#updateSpacingForBar(bar.time);
  }

  upsertBarFast(bar: Bar): void {
    this.#engine.upsert_bar_fast(
      bar.time,
      bar.open,
      bar.high,
      bar.low,
      bar.close,
      bar.volume,
    );
    this.#updateSpacingForBar(bar.time);
  }

  candles(): Bar[] {
    return this.#engine.candles() as Bar[];
  }

  candleColumns(): CandleColumns {
    return this.#engine.candle_columns_fast() as CandleColumns;
  }

  addIndicator(config: IndicatorConfig): number {
    const id = this.#engine.add_indicator_config(config);
    this.#configs.set(id, { ...config });
    return id;
  }

  addIndicators(configs: IndicatorConfig[]): number[] {
    const ids = this.#engine.add_indicator_configs(configs) as number[];
    ids.forEach((id, index) => {
      const config = configs[index];
      if (config) this.#configs.set(id, { ...config });
    });
    return ids;
  }

  removeIndicator(id: number): boolean {
    this.#configs.delete(id);
    return this.#engine.remove_indicator(id);
  }

  indicatorDescriptors(): IndicatorDescriptor[] {
    return this.#engine.indicator_descriptors() as IndicatorDescriptor[];
  }

  indicatorValueSeries(id: number): IndicatorValueSeries[] {
    return (
      this.#engine.indicator_output_values_fast(id) as Array<{
        name: string;
        values: Float64Array;
      }>
    ).map((output) => ({
      output: output.name,
      values: output.values,
    }));
  }

  latestIndicatorValues(id: number): Float64Array {
    return this.#engine.latest_indicator_values_fast(id) as Float64Array;
  }

  // Render mapping stays in TS: Rust owns raw candle/output data, TS pairs values with times.
  indicatorSeries(id: number): IndicatorOutputSeries[] {
    const outputs = this.indicatorValueSeries(id);
    const candles = this.candleColumns();
    const spacing = seriesSpacingSeconds(candles.time);
    const config = this.#configs.get(id);
    return outputs.map((output) => ({
      output: output.output,
      points: Array.from(candles.time, (time, index) => ({
        time: shiftedOutputTime(time, spacing, indicatorOutputShift(config, output.output)),
        value: Number.isNaN(output.values[index]!) ? null : output.values[index]!,
      })),
    }));
  }

  latestIndicatorPoints(id: number): IndicatorOutputPoint[] {
    const latestTime = this.#lastBarTime;
    if (latestTime === undefined) return [];
    const outputs = this.#engine.latest_indicator_values(id) as RawIndicatorLatestValue[];
    const config = this.#configs.get(id);
    return outputs.map((output) => ({
      output: output.output,
      time: shiftedOutputTime(
        latestTime,
        this.#seriesSpacing,
        indicatorOutputShift(config, output.output),
      ),
      value: output.value,
    }));
  }

  latestIndicatorTime(id: number, output: string): number | undefined {
    const latestTime = this.#lastBarTime;
    if (latestTime === undefined) return undefined;
    return shiftedOutputTime(
      latestTime,
      this.#seriesSpacing,
      indicatorOutputShift(this.#configs.get(id), output),
    );
  }

  dagDebug(): DagDebug {
    return this.#engine.dag_debug() as DagDebug;
  }

  #updateSpacingForBar(time: number) {
    const previousTime = this.#lastBarTime;
    this.#lastBarTime = time;
    if (previousTime === undefined) return;
    const spacing = time - previousTime;
    if (spacing > 0) this.#seriesSpacing = spacing;
  }
}

export function seriesSpacingSeconds(times: Uint32Array) {
  let spacing = 60;
  for (let index = 1; index < times.length; index += 1) {
    const value = times[index]! - times[index - 1]!;
    if (value > 0) spacing = Math.min(spacing, value);
  }
  return spacing;
}

export function seriesSpacingFromBars(bars: Bar[]) {
  let spacing = 60;
  for (let index = 1; index < bars.length; index += 1) {
    const value = bars[index]!.time - bars[index - 1]!.time;
    if (value > 0) spacing = Math.min(spacing, value);
  }
  return spacing;
}

export function indicatorOutputShift(config: IndicatorConfig | undefined, output: string) {
  if (config?.kind !== "ICHIMOKU") return 0;
  const shift = config.kijun_period ?? 26;
  if (output === "senkou_a" || output === "senkou_b") return shift;
  if (output === "chikou") return -shift;
  return 0;
}

export function shiftedOutputTime(baseTime: number, spacing: number, shift: number) {
  const delta = spacing * Math.abs(shift);
  return shift >= 0 ? baseTime + delta : baseTime - delta;
}
