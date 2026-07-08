import {
  CandlestickSeries,
  createChart,
  HistogramSeries,
  LineSeries,
  LineStyle,
  type IPrimitivePaneRenderer,
  type IPrimitivePaneView,
  type ISeriesApi,
  type ISeriesPrimitive,
  type SeriesAttachedParameter,
  type Time,
} from "lightweight-charts";
import type { CanvasRenderingTarget2D } from "fancy-canvas";
import {
  type Bar,
  type CandleColumns,
  type DagDebug,
  type IndicatorConfig,
  type IndicatorDescriptor,
  type IndicatorKind,
  type IndicatorOutputPoint,
  type OutputDescriptor,
  type ParamDescriptor,
  RapidChartEngine,
  initEngine,
  indicatorOutputShift,
  seriesSpacingSeconds,
  shiftedOutputTime,
} from "./index";
import { fixtureColumns } from "./fixtures";
import "./style.css";

type IndicatorPoint = {
  time: number;
  value: number | null;
};

type IndicatorSeries = {
  output: string;
  series: ISeriesApi<"Line"> | ISeriesApi<"Histogram">;
  lastPoint?: IndicatorPoint;
};

type IndicatorCloud = {
  hostSeries: ISeriesApi<"Line">;
  primitive: IchimokuCloudPrimitive;
};

type Indicator = {
  kind: IndicatorKind;
  config: IndicatorConfig;
  engineId?: number;
  series: IndicatorSeries[];
  cloud?: IndicatorCloud;
};

type LayoutState = {
  name: string;
  symbol: string;
  interval: string;
  indicators: IndicatorConfig[];
};

type IndicatorPreset = {
  name: string;
  indicators: IndicatorConfig[];
};

type PerfSample = {
  engineMs: number;
  candlesMs: number;
  volumeMs: number;
  indicatorsMs: number;
  totalMs: number;
};

type LoadBreakdown = {
  source: "cache" | "fixture" | "network";
  fetchMs: number;
  parseMs: number;
  ingestMs: number;
  candleRenderMs: number;
  volumeRenderMs: number;
  indicatorRenderMs: number;
  dagMs: number;
  fitMs: number;
  streamMs: number;
  totalMs: number;
};

type PresetBreakdown = {
  preset: string;
  clearMs: number;
  attachMs: number;
  listMs: number;
  dagMs: number;
  totalMs: number;
};

const symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];
const intervals = ["1m", "5m", "15m", "1h", "4h", "1d"];
const colors = ["#2563eb", "#dc2626", "#059669", "#9333ea", "#ea580c"];
const ichimokuBullCloud = "rgba(5, 150, 105, 0.18)";
const ichimokuBearCloud = "rgba(220, 38, 38, 0.18)";
const layoutStorageKey = "rapidchart.layouts";
const candleCacheKey = "rapidchart.candles";
const presets: IndicatorPreset[] = [
  {
    name: "Trend Stack",
    indicators: [
      { kind: "EMA", period: 20 },
      { kind: "EMA", period: 50 },
      { kind: "ADX", period: 14 },
      { kind: "SUPERTREND", period: 10, multiplier: 3 },
    ],
  },
  {
    name: "Momentum Stack",
    indicators: [
      { kind: "MACD", fast: 12, slow: 26, signal: 9 },
      { kind: "RSI", period: 14 },
      { kind: "ROC", period: 14 },
      { kind: "VWMA", period: 20 },
    ],
  },
  {
    name: "Mean Reversion",
    indicators: [
      { kind: "BB", period: 20, multiplier: 2 },
      { kind: "STARC", period: 15, multiplier: 2 },
      { kind: "RSI", period: 14 },
    ],
  },
];

type CloudPoint = {
  time: number;
  a: number;
  b: number;
};

class IchimokuCloudRenderer implements IPrimitivePaneRenderer {
  constructor(
    private readonly hostSeries: ISeriesApi<"Line">,
    private readonly points: readonly CloudPoint[],
  ) {}

  drawBackground(target: CanvasRenderingTarget2D): void {
    if (this.points.length < 2) return;
    target.useMediaCoordinateSpace(({ context }) => {
      context.save();
      for (let index = 1; index < this.points.length; index += 1) {
        const left = this.points[index - 1];
        const right = this.points[index];
        const x1 = chart.timeScale().timeToCoordinate(left.time as Time);
        const x2 = chart.timeScale().timeToCoordinate(right.time as Time);
        const yA1 = this.hostSeries.priceToCoordinate(left.a);
        const yA2 = this.hostSeries.priceToCoordinate(right.a);
        const yB1 = this.hostSeries.priceToCoordinate(left.b);
        const yB2 = this.hostSeries.priceToCoordinate(right.b);
        if ([x1, x2, yA1, yA2, yB1, yB2].some((value) => value === null)) continue;
        fillCloudSegment(
          context,
          { x: x1!, upper: yA1!, lower: yB1!, delta: left.a - left.b },
          { x: x2!, upper: yA2!, lower: yB2!, delta: right.a - right.b },
        );
      }
      context.restore();
    });
  }

  draw(): void {}
}

class IchimokuCloudPaneView implements IPrimitivePaneView {
  constructor(private readonly primitive: IchimokuCloudPrimitive) {}

  zOrder() {
    return "bottom" as const;
  }

  renderer(): IPrimitivePaneRenderer | null {
    return this.primitive.renderer();
  }
}

class IchimokuCloudPrimitive implements ISeriesPrimitive<Time> {
  private attachedParams?: SeriesAttachedParameter<Time>;
  private points: CloudPoint[] = [];
  private readonly byTime = new Map<number, { a?: number; b?: number }>();
  private readonly paneView = new IchimokuCloudPaneView(this);

  attached(params: SeriesAttachedParameter<Time>): void {
    this.attachedParams = params;
  }

  detached(): void {
    this.attachedParams = undefined;
  }

  paneViews(): readonly IPrimitivePaneView[] {
    return [this.paneView];
  }

  updateAllViews(): void {}

  setData(a: IndicatorPoint[] = [], b: IndicatorPoint[] = []): void {
    this.byTime.clear();
    for (const point of a) {
      if (point.value === null) continue;
      this.byTime.set(point.time, { ...(this.byTime.get(point.time) ?? {}), a: point.value });
    }
    for (const point of b) {
      if (point.value === null) continue;
      this.byTime.set(point.time, { ...(this.byTime.get(point.time) ?? {}), b: point.value });
    }
    this.rebuildPoints();
    this.attachedParams?.requestUpdate();
  }

  updatePoints(a?: IndicatorOutputPoint, b?: IndicatorOutputPoint): void {
    if (a && a.value !== null) this.upsertPoint(a.time, "a", a.value);
    if (b && b.value !== null) this.upsertPoint(b.time, "b", b.value);
    this.rebuildPoints();
    this.attachedParams?.requestUpdate();
  }

  renderer(): IPrimitivePaneRenderer | null {
    return this.points.length > 1 && this.attachedParams
      ? new IchimokuCloudRenderer(this.attachedParams.series as ISeriesApi<"Line">, this.points)
      : null;
  }

  private upsertPoint(time: number, side: "a" | "b", value: number): void {
    this.byTime.set(time, { ...(this.byTime.get(time) ?? {}), [side]: value });
  }

  private rebuildPoints(): void {
    this.points = [...this.byTime.entries()]
      .map(([time, values]) =>
        values.a === undefined || values.b === undefined
          ? undefined
          : { time, a: values.a, b: values.b }
      )
      .filter((point): point is CloudPoint => point !== undefined)
      .sort((left, right) => left.time - right.time);
  }
}

function fillCloudSegment(
  context: CanvasRenderingContext2D,
  left: { x: number; upper: number; lower: number; delta: number },
  right: { x: number; upper: number; lower: number; delta: number },
) {
  if (left.x === right.x) return;
  if (left.delta === 0 && right.delta === 0) return;
  if (Math.sign(left.delta) === Math.sign(right.delta) || left.delta === 0 || right.delta === 0) {
    fillPolygon(
      context,
      [
        [left.x, left.upper],
        [right.x, right.upper],
        [right.x, right.lower],
        [left.x, left.lower],
      ],
      cloudColor(left.delta || right.delta),
    );
    return;
  }
  const ratio = left.delta / (left.delta - right.delta);
  const crossX = left.x + (right.x - left.x) * ratio;
  const crossY = left.upper + (right.upper - left.upper) * ratio;
  fillPolygon(
    context,
    [
      [left.x, left.upper],
      [crossX, crossY],
      [crossX, crossY],
      [left.x, left.lower],
    ],
    cloudColor(left.delta),
  );
  fillPolygon(
    context,
    [
      [crossX, crossY],
      [right.x, right.upper],
      [right.x, right.lower],
      [crossX, crossY],
    ],
    cloudColor(right.delta),
  );
}

function fillPolygon(
  context: CanvasRenderingContext2D,
  points: [number, number][],
  fillStyle: string,
) {
  context.beginPath();
  context.moveTo(points[0][0], points[0][1]);
  for (let index = 1; index < points.length; index += 1) {
    context.lineTo(points[index][0], points[index][1]);
  }
  context.closePath();
  context.fillStyle = fillStyle;
  context.fill();
}

function cloudColor(delta: number) {
  return delta >= 0 ? ichimokuBullCloud : ichimokuBearCloud;
}

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <main>
    <form id="toolbar">
      <label>Symbol <select id="symbol">${symbols.map((s) => `<option>${s}</option>`).join("")}</select></label>
      <label>Timeframe <select id="interval">${intervals.map((i) => `<option>${i}</option>`).join("")}</select></label>
      <label>Indicator <select id="kind"></select></label>
      <fieldset id="params"></fieldset>
      <button type="submit">Add</button>
    </form>
    <section id="layouts">
      <label>Layout <select id="layout-select"></select></label>
      <label>Name <input id="layout-name" type="text" maxlength="40" placeholder="BTC trend" /></label>
      <button id="layout-save" type="button">Save</button>
      <button id="layout-load" type="button">Load</button>
      <button id="layout-delete" type="button">Delete</button>
      <label>Preset <select id="preset-select"></select></label>
      <button id="preset-apply" type="button">Apply Preset</button>
    </section>
    <section id="dag"></section>
    <section id="chart"></section>
    <section id="indicators"></section>
    <footer>
      <span id="status"></span>
      <span id="perf"></span>
      <a href="https://www.tradingview.com/" target="_blank" rel="noreferrer">Charts by TradingView</a>
    </footer>
  </main>
`;

const toolbar = document.querySelector<HTMLFormElement>("#toolbar")!;
const symbolInput = document.querySelector<HTMLSelectElement>("#symbol")!;
const intervalInput = document.querySelector<HTMLSelectElement>("#interval")!;
const kindInput = document.querySelector<HTMLSelectElement>("#kind")!;
const layoutSelect = document.querySelector<HTMLSelectElement>("#layout-select")!;
const layoutNameInput = document.querySelector<HTMLInputElement>("#layout-name")!;
const saveLayoutButton = document.querySelector<HTMLButtonElement>("#layout-save")!;
const loadLayoutButton = document.querySelector<HTMLButtonElement>("#layout-load")!;
const deleteLayoutButton = document.querySelector<HTMLButtonElement>("#layout-delete")!;
const presetSelect = document.querySelector<HTMLSelectElement>("#preset-select")!;
const applyPresetButton = document.querySelector<HTMLButtonElement>("#preset-apply")!;
const params = document.querySelector<HTMLFieldSetElement>("#params")!;
const status = document.querySelector<HTMLParagraphElement>("#status")!;
const perf = document.querySelector<HTMLParagraphElement>("#perf")!;
const indicatorList = document.querySelector<HTMLElement>("#indicators")!;
const dag = document.querySelector<HTMLElement>("#dag")!;
const perfSamples: PerfSample[] = [];
const perfSampleLimit = 60;

declare global {
  interface Window {
    __rapidChartLoadBreakdown?: LoadBreakdown;
    __rapidChartBenchmarkTick?: () => boolean;
    __rapidChartPresetBreakdown?: PresetBreakdown;
  }
}

const chart = createChart(document.querySelector<HTMLElement>("#chart")!, {
  autoSize: true,
  layout: { attributionLogo: false, background: { color: "#0f172a" }, textColor: "#dbe5f0" },
  grid: { vertLines: { color: "#223047" }, horzLines: { color: "#223047" } },
  rightPriceScale: { borderVisible: false },
  timeScale: { borderVisible: false },
});
const candles = chart.addSeries(CandlestickSeries);
const volume = chart.addSeries(HistogramSeries, {
  color: "#475569",
  priceFormat: { type: "volume" },
  priceScaleId: "",
});
chart.priceScale("").applyOptions({
  scaleMargins: { top: 0.8, bottom: 0 },
});
let engine: RapidChartEngine;
let indicators: Indicator[] = [];
let stream: WebSocket | undefined;
let descriptors: IndicatorDescriptor[] = [];
let currentColumns: CandleColumns | undefined;
let currentSpacing = 60;
const dataMode = readDataMode();
const fixtureName = readFixtureName();

await initEngine();
engine = new RapidChartEngine();
descriptors = engine.indicatorDescriptors();
window.__rapidChartBenchmarkTick = benchmarkTick;
renderIndicatorPicker();
renderLayoutPicker();
renderPresetPicker();
await load();

toolbar.addEventListener("submit", (event) => {
  event.preventDefault();
  addIndicator(kindInput.value as IndicatorKind);
});
symbolInput.addEventListener("change", load);
intervalInput.addEventListener("change", load);
kindInput.addEventListener("change", syncIndicatorForm);
indicatorList.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-id]");
  if (button) removeIndicator(Number(button.dataset.id));
});
layoutSelect.addEventListener("change", syncLayoutName);
saveLayoutButton.addEventListener("click", saveLayout);
loadLayoutButton.addEventListener("click", () => void loadSelectedLayout());
deleteLayoutButton.addEventListener("click", deleteSelectedLayout);
applyPresetButton.addEventListener("click", applySelectedPreset);

async function load() {
  setStatus("Loading...");
  closeStream();
  detachIndicatorSeries();
  const loadStart = performance.now();
  try {
    engine = new RapidChartEngine();
    const { columns, source, fetchMs, parseMs } = await fetchBars(symbolInput.value, intervalInput.value);
    const ingestStart = performance.now();
    engine.ingestColumnsFast(columns);
    const ingestMs = performance.now() - ingestStart;
    currentColumns = engine.candleColumns();
    currentSpacing = seriesSpacingSeconds(currentColumns.time);
    const candleRenderStart = performance.now();
    candles.setData(chartBarsFromColumns(currentColumns));
    const candleRenderMs = performance.now() - candleRenderStart;
    const volumeRenderStart = performance.now();
    volume.setData(volumePointsFromColumns(currentColumns));
    const volumeRenderMs = performance.now() - volumeRenderStart;
    const indicatorRenderStart = performance.now();
    attachIndicators();
    const indicatorRenderMs = performance.now() - indicatorRenderStart;
    const dagStart = performance.now();
    renderDag();
    const dagMs = performance.now() - dagStart;
    const fitStart = performance.now();
    chart.timeScale().fitContent();
    const fitMs = performance.now() - fitStart;
    const streamStart = performance.now();
    if (dataMode === "live") openStream(symbolInput.value, intervalInput.value);
    const streamMs = performance.now() - streamStart;
    window.__rapidChartLoadBreakdown = {
      source,
      fetchMs,
      parseMs,
      ingestMs,
      candleRenderMs,
      volumeRenderMs,
      indicatorRenderMs,
      dagMs,
      fitMs,
      streamMs,
      totalMs: performance.now() - loadStart,
    };
    setStatus(
      `${symbolInput.value} ${intervalInput.value} ${
        dataMode === "live" ? "live" : dataMode === "fixture" ? `fixture:${fixtureName}` : "cached"
      }`,
    );
  } catch (error) {
    setStatus(error instanceof Error ? error.message : "Failed to load data", "error");
  }
}

async function fetchBars(
  symbol: string,
  interval: string,
): Promise<{ columns: CandleColumns; source: "cache" | "fixture" | "network"; fetchMs: number; parseMs: number }> {
  if (dataMode === "fixture") {
    const columns = fixtureColumns(fixtureName);
    if (!columns) throw new Error(`Unknown fixture ${fixtureName}`);
    return { columns, source: "fixture", fetchMs: 0, parseMs: 0 };
  }
  const cached = readCachedColumns(symbol, interval);
  if (cached && dataMode === "cached") {
    return { columns: cached, source: "cache", fetchMs: 0, parseMs: 0 };
  }
  const url = new URL("https://api.binance.com/api/v3/klines");
  url.search = new URLSearchParams({ symbol, interval, limit: "500" }).toString();
  const fetchStart = performance.now();
  const response = await fetch(url);
  const fetchMs = performance.now() - fetchStart;
  if (!response.ok) throw new Error(`Binance returned ${response.status}`);
  const parseStart = performance.now();
  const rows = await response.json();
  if (!Array.isArray(rows)) throw new Error("Unexpected Binance response");
  const validRows = rows.filter(isFiniteRestRow);
  if (validRows.length === 0) throw new Error("No valid bars returned");
  const columns = {
    time: Uint32Array.from(validRows, (row) => Math.floor(Number(row[0]) / 1000)),
    open: Float64Array.from(validRows, (row) => Number(row[1])),
    high: Float64Array.from(validRows, (row) => Number(row[2])),
    low: Float64Array.from(validRows, (row) => Number(row[3])),
    close: Float64Array.from(validRows, (row) => Number(row[4])),
    volume: Float64Array.from(validRows, (row) => Number(row[5])),
  };
  writeCachedColumns(symbol, interval, columns);
  return {
    columns,
    source: "network",
    fetchMs,
    parseMs: performance.now() - parseStart,
  };
}

function readDataMode(): "live" | "cached" | "fixture" {
  const mode = new URLSearchParams(window.location.search).get("data");
  if (mode === "fixture") return "fixture";
  return mode === "cached" ? "cached" : "live";
}

function readFixtureName() {
  return new URLSearchParams(window.location.search).get("fixture") ?? "btcusdt-1m";
}

function readCachedColumns(symbol: string, interval: string): CandleColumns | undefined {
  try {
    const raw = localStorage.getItem(candleCacheKey);
    if (!raw) return undefined;
    const cache = JSON.parse(raw) as Record<string, number[][] | undefined>;
    const entry = cache[`${symbol}:${interval}`];
    if (!Array.isArray(entry) || entry.length !== 6) return undefined;
    const [time, open, high, low, close, volume] = entry;
    if (
      !Array.isArray(time) || !Array.isArray(open) || !Array.isArray(high)
      || !Array.isArray(low) || !Array.isArray(close) || !Array.isArray(volume)
    ) return undefined;
    return {
      time: Uint32Array.from(time),
      open: Float64Array.from(open),
      high: Float64Array.from(high),
      low: Float64Array.from(low),
      close: Float64Array.from(close),
      volume: Float64Array.from(volume),
    };
  } catch {
    return undefined;
  }
}

function writeCachedColumns(symbol: string, interval: string, columns: CandleColumns) {
  try {
    const raw = localStorage.getItem(candleCacheKey);
    const cache = raw ? JSON.parse(raw) as Record<string, number[][]> : {};
    cache[`${symbol}:${interval}`] = [
      Array.from(columns.time),
      Array.from(columns.open),
      Array.from(columns.high),
      Array.from(columns.low),
      Array.from(columns.close),
      Array.from(columns.volume),
    ];
    localStorage.setItem(candleCacheKey, JSON.stringify(cache));
  } catch {
    // ponytail: cache is best-effort for repeatable testing; ignore quota/parse failures.
  }
}

function isFiniteRestRow(row: unknown): row is unknown[] {
  return Array.isArray(row)
    && Number.isFinite(Math.floor(Number(row[0]) / 1000))
    && Number.isFinite(Number(row[1]))
    && Number.isFinite(Number(row[2]))
    && Number.isFinite(Number(row[3]))
    && Number.isFinite(Number(row[4]))
    && Number.isFinite(Number(row[5]));
}

function addIndicator(kind: IndicatorKind) {
  const config = readIndicatorConfig(kind);
  if (!config) return;
  const indicator: Indicator = {
    kind,
    config,
    series: [],
  };
  indicators.push(indicator);
  attachIndicator(indicator, indicators.length - 1);
  renderIndicatorList();
  renderDag();
}

function indicatorFromConfig(config: IndicatorConfig): Indicator {
  return {
    kind: config.kind,
    config: { ...config },
    series: [],
  };
}

function attachIndicator(indicator: Indicator, index: number) {
  indicator.engineId = engine.addIndicator(indicatorConfig(indicator));
  currentColumns = engine.candleColumns();
  attachIndicatorSeries(indicator, index);
  renderIndicator(indicator);
}

function attachIndicatorSeries(indicator: Indicator, index: number) {
  indicator.series = descriptorFor(indicator.kind).outputs.map((output, outputIndex) =>
    addOutputSeries(output, colors[(index + outputIndex) % colors.length]),
  );
  if (indicator.kind === "ICHIMOKU") attachIchimokuCloud(indicator);
  if (indicator.kind === "RSI") addRsiGuides(indicator.series[0].series as ISeriesApi<"Line">);
}

function attachIchimokuCloud(indicator: Indicator) {
  const hostSeries = indicator.series.find((item) => item.output === "senkou_a")?.series;
  if (!hostSeries || hostSeries.seriesType() !== "Line") return;
  const primitive = new IchimokuCloudPrimitive();
  (hostSeries as ISeriesApi<"Line">).attachPrimitive(primitive);
  indicator.cloud = {
    hostSeries: hostSeries as ISeriesApi<"Line">,
    primitive,
  };
}

function addOutputSeries(output: OutputDescriptor, fallbackColor: string): IndicatorSeries {
  const paneIndex = output.pane === "separate" ? 1 : 0;
  const color = output.color || fallbackColor;
  return output.renderer === "histogram"
    ? {
        output: output.name,
        series: chart.addSeries(HistogramSeries, {
          color,
          priceFormat: { type: "price" },
          priceLineVisible: false,
        }, paneIndex),
      }
    : {
        output: output.name,
        series: chart.addSeries(LineSeries, {
          color,
          lineWidth: 2,
          priceLineVisible: false,
        }, paneIndex),
      };
}

function addRsiGuides(series: ISeriesApi<"Line">) {
  for (const price of [70, 30]) {
    series.createPriceLine({
      price,
      color: "#94a3b8",
      lineWidth: 1,
      lineStyle: LineStyle.Dashed,
      axisLabelVisible: true,
      title: String(price),
    });
  }
}

function attachIndicators() {
  const configs = indicators.map((indicator) => indicatorConfig(indicator));
  const ids = engine.addIndicators(configs);
  currentColumns = engine.candleColumns();
  indicators.forEach((indicator, index) => {
    indicator.engineId = ids[index];
    attachIndicatorSeries(indicator, index);
  });
  renderIndicators(indicators);
}

function openStream(symbol: string, interval: string) {
  const socket = new WebSocket(`wss://stream.binance.com:9443/ws/${symbol.toLowerCase()}@kline_${interval}`);
  stream = socket;
  socket.addEventListener("message", (event) => {
    const bar = barFromStream(JSON.parse(event.data));
    if (!bar) return;
    try {
      applyBarUpdate(bar);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Live update failed", "error");
      if (stream === socket) stream = undefined;
      socket.close();
    }
  });
  socket.addEventListener("error", () => setStatus(`${symbol} ${interval} stream failed`, "error"));
  socket.addEventListener("close", () => {
    if (stream === socket) setStatus(`${symbol} ${interval} stream closed`);
  });
}

function closeStream() {
  const socket = stream;
  stream = undefined;
  socket?.close();
}

function benchmarkTick() {
  const columns = currentColumns;
  if (!columns || columns.time.length === 0) return false;
  const lastIndex = columns.time.length - 1;
  const lastTime = columns.time[lastIndex]!;
  const open = columns.close[lastIndex]!;
  const close = open + Math.sin(lastTime / 600) * 12 + Math.cos(lastTime / 180) * 4;
  const high = Math.max(open, close) + 6;
  const low = Math.min(open, close) - 6;
  const volumeValue = columns.volume[lastIndex]! + 5;
  applyBarUpdate({
    time: lastTime + Math.max(currentSpacing, 60),
    open,
    high,
    low,
    close,
    volume: volumeValue,
  });
  return true;
}

function applyBarUpdate(bar: Bar) {
  const totalStart = performance.now();
  const engineStart = performance.now();
  engine.upsertBarFast(bar);
  currentColumns = engine.candleColumns();
  const engineMs = performance.now() - engineStart;
  const candleStart = performance.now();
  candles.update({ ...bar, time: bar.time as Time });
  const candlesMs = performance.now() - candleStart;
  const volumeStart = performance.now();
  volume.update(volumePoint(bar));
  const volumeMs = performance.now() - volumeStart;
  const indicatorsStart = performance.now();
  for (const indicator of indicators) updateIndicator(indicator);
  const indicatorsMs = performance.now() - indicatorsStart;
  pushPerfSample({
    engineMs,
    candlesMs,
    volumeMs,
    indicatorsMs,
    totalMs: performance.now() - totalStart,
  });
}

function pushPerfSample(sample: PerfSample) {
  perfSamples.push(sample);
  if (perfSamples.length > perfSampleLimit) perfSamples.shift();
  renderPerf();
}

function renderPerf() {
  if (!perfSamples.length) {
    perf.textContent = "";
    return;
  }
  const latest = perfSamples[perfSamples.length - 1]!;
  const average = averagePerfSample(perfSamples);
  perf.textContent =
    `tick ${formatPerf(latest.totalMs)}/${formatPerf(average.totalMs)} ` +
    `engine ${formatPerf(latest.engineMs)}/${formatPerf(average.engineMs)} ` +
    `candle ${formatPerf(latest.candlesMs)}/${formatPerf(average.candlesMs)} ` +
    `volume ${formatPerf(latest.volumeMs)}/${formatPerf(average.volumeMs)} ` +
    `ind ${formatPerf(latest.indicatorsMs)}/${formatPerf(average.indicatorsMs)}`;
}

function averagePerfSample(samples: PerfSample[]): PerfSample {
  const total = samples.reduce(
    (sum, sample) => ({
      engineMs: sum.engineMs + sample.engineMs,
      candlesMs: sum.candlesMs + sample.candlesMs,
      volumeMs: sum.volumeMs + sample.volumeMs,
      indicatorsMs: sum.indicatorsMs + sample.indicatorsMs,
      totalMs: sum.totalMs + sample.totalMs,
    }),
    { engineMs: 0, candlesMs: 0, volumeMs: 0, indicatorsMs: 0, totalMs: 0 },
  );
  const count = samples.length;
  return {
    engineMs: total.engineMs / count,
    candlesMs: total.candlesMs / count,
    volumeMs: total.volumeMs / count,
    indicatorsMs: total.indicatorsMs / count,
    totalMs: total.totalMs / count,
  };
}

function formatPerf(value: number) {
  return `${value.toFixed(value >= 10 ? 1 : 2)}ms`;
}

function barFromStream(message: unknown): Bar | undefined {
  if (!message || typeof message !== "object" || !("k" in message)) return undefined;
  const kline = (message as { k: Record<string, unknown> }).k;
  if (!kline || typeof kline !== "object") return undefined;
  const bar = {
    time: Math.floor(Number(kline.t) / 1000),
    open: Number(kline.o),
    high: Number(kline.h),
    low: Number(kline.l),
    close: Number(kline.c),
    volume: Number(kline.v),
  };
  return isFiniteBar(bar) ? bar : undefined;
}

function isFiniteBar(bar: Bar) {
  return Object.values(bar).every(Number.isFinite);
}

function volumePoint(bar: Bar) {
  return {
    time: bar.time as Time,
    value: bar.volume,
    color: bar.close >= bar.open ? "#86efac" : "#fca5a5",
  };
}

function chartBarsFromColumns(columns: CandleColumns) {
  const bars = new Array(columns.time.length);
  for (let index = 0; index < columns.time.length; index += 1) {
    bars[index] = {
      time: columns.time[index]! as Time,
      open: columns.open[index]!,
      high: columns.high[index]!,
      low: columns.low[index]!,
      close: columns.close[index]!,
    };
  }
  return bars;
}

function volumePointsFromColumns(columns: CandleColumns) {
  return Array.from(columns.time, (time, index) => ({
    time: time as Time,
    value: columns.volume[index]!,
    color: columns.close[index]! >= columns.open[index]! ? "#86efac" : "#fca5a5",
  }));
}

function shiftedTimes(shift: number, cache: Map<number, Time[]>) {
  if (shift === 0 || !currentColumns) return undefined;
  const cached = cache.get(shift);
  if (cached) return cached;
  const times = new Array(currentColumns.time.length);
  for (let index = 0; index < currentColumns.time.length; index += 1) {
    times[index] = shiftedOutputTime(currentColumns.time[index]!, currentSpacing, shift);
  }
  cache.set(shift, times);
  return times;
}

function isSimpleLineIndicator(indicator: Indicator) {
  return !indicator.cloud && indicator.series.every((item) =>
    item.series.seriesType() === "Line" && indicatorOutputShift(indicator.config, item.output) === 0
  );
}

function renderSimpleLineIndicator(indicator: Indicator, outputs: ReturnType<RapidChartEngine["indicatorValueSeries"]>) {
  if (!currentColumns) return;
  for (let outputIndex = 0; outputIndex < indicator.series.length; outputIndex += 1) {
    const item = indicator.series[outputIndex]!;
    const output = outputs[outputIndex];
    if (!output || output.output !== item.output) continue;
    const renderedPoints = [];
    item.lastPoint = undefined;
    for (let index = 0; index < currentColumns.time.length; index += 1) {
      const value = output.values[index];
      if (value === undefined || Number.isNaN(value)) continue;
      const point = { time: currentColumns.time[index]!, value };
      renderedPoints.push({ time: point.time as Time, value: point.value });
      item.lastPoint = point;
    }
    item.series.setData(renderedPoints);
  }
}

function renderIndicator(indicator: Indicator, shiftedTimesCache = new Map<number, Time[]>()) {
  if (!indicator.engineId || !currentColumns) return;
  const outputs = engine.indicatorValueSeries(indicator.engineId);
  if (isSimpleLineIndicator(indicator)) {
    renderSimpleLineIndicator(indicator, outputs);
    return;
  }
  const cloudA: IndicatorPoint[] = [];
  const cloudB: IndicatorPoint[] = [];
  for (let outputIndex = 0; outputIndex < indicator.series.length; outputIndex += 1) {
    const item = indicator.series[outputIndex]!;
    const output = outputs[outputIndex];
    if (!output || output.output !== item.output) continue;
    const renderedPoints = [];
    item.lastPoint = undefined;
    const shift = indicatorOutputShift(indicator.config, item.output);
    const times = shiftedTimes(shift, shiftedTimesCache);
    for (let index = 0; index < currentColumns.time.length; index += 1) {
      const value = output.values[index];
      if (value === undefined || Number.isNaN(value)) continue;
      const point = {
        time: times ? times[index]! as number : currentColumns.time[index]!,
        value,
      };
      renderedPoints.push(indicatorPoint(item.output, point));
      if (item.output === "senkou_a") cloudA.push(point);
      if (item.output === "senkou_b") cloudB.push(point);
      if (item.lastPoint === undefined || point.time >= item.lastPoint.time) {
        item.lastPoint = point;
      }
    }
    item.series.setData(renderedPoints);
  }
  indicator.cloud?.primitive.setData(cloudA, cloudB);
}

function renderIndicators(items: Indicator[]) {
  const shiftedTimesCache = new Map<number, Time[]>();
  for (const indicator of items) renderIndicator(indicator, shiftedTimesCache);
}

function updateIndicator(indicator: Indicator) {
  if (!indicator.engineId) return;
  const values = engine.latestIndicatorValues(indicator.engineId);
  let senkouA: IndicatorOutputPoint | undefined;
  let senkouB: IndicatorOutputPoint | undefined;
  for (let index = 0; index < indicator.series.length; index += 1) {
    const item = indicator.series[index]!;
    const value = values[index];
    if (value === undefined || Number.isNaN(value)) continue;
    const time = engine.latestIndicatorTime(indicator.engineId, item.output);
    if (time === undefined) continue;
    const point = { output: item.output, time, value };
    if (item.lastPoint?.time === point.time && item.lastPoint.value === point.value) {
      if (item.output === "senkou_a") senkouA = point;
      if (item.output === "senkou_b") senkouB = point;
      continue;
    }
    item.series.update(indicatorPoint(item.output, point));
    item.lastPoint = point;
    if (item.output === "senkou_a") senkouA = point;
    if (item.output === "senkou_b") senkouB = point;
  }
  indicator.cloud?.primitive.updatePoints(senkouA, senkouB);
}

function indicatorPoint(output: string, point: IndicatorPoint) {
  return {
    time: point.time as Time,
    value: point.value!,
    color: output === "histogram" && point.value! < 0 ? "#fca5a5" : "#86efac",
  };
}

function removeIndicator(index: number) {
  const indicator = indicators[index];
  if (!indicator) return;
  if (indicator.engineId) engine.removeIndicator(indicator.engineId);
  indicator.cloud?.hostSeries.detachPrimitive(indicator.cloud.primitive);
  for (const item of indicator.series) chart.removeSeries(item.series);
  indicators.splice(index, 1);
  renderIndicatorList();
  renderDag();
}

function detachIndicatorSeries() {
  for (const indicator of indicators) {
    indicator.cloud?.hostSeries.detachPrimitive(indicator.cloud.primitive);
    for (const item of indicator.series) chart.removeSeries(item.series);
    indicator.engineId = undefined;
    indicator.series = [];
    indicator.cloud = undefined;
  }
}

function clearIndicators() {
  for (const indicator of indicators) {
    if (indicator.engineId) engine.removeIndicator(indicator.engineId);
  }
  detachIndicatorSeries();
  indicators = [];
  renderIndicatorList();
  renderDag();
}

function renderIndicatorList() {
  indicatorList.replaceChildren(
    ...indicators.map((indicator, index) => {
      const label = indicatorLabel(indicator);
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.id = String(index);
      button.setAttribute("aria-label", `Remove ${label}`);
      button.textContent = `${label} ×`;
      return button;
    }),
  );
}

function renderDag() {
  const graph = engine.dagDebug();
  const sources = graph.nodes.filter((node) => node === "close" || node === "high" || node === "low" || node === "volume");
  const derived = graph.nodes.filter((node) => node === "hl2" || node === "hlc3" || node === "ohlc4");
  const computed = graph.nodes.filter((node) => node.includes(":"));
  const indicatorNodes = graph.nodes.filter((node) => node.includes("#"));
  const outDegree = new Map<string, number>();
  for (const edge of graph.edges) outDegree.set(edge.from, (outDegree.get(edge.from) ?? 0) + 1);
  dag.innerHTML = `
    <header><strong>DAG</strong><span>${graph.nodes.length} nodes</span><span>${graph.edges.length} edges</span></header>
    <div class="dag-grid">
      ${dagLayer("Source", sources, outDegree)}
      ${derived.length ? dagLayer("Derived", derived, outDegree) : ""}
      ${dagLayer("Computed", computed, outDegree)}
      ${dagLayer("Indicators", indicatorNodes, outDegree)}
    </div>
    <div class="dag-edges">${graph.edges.map((edge) => `<span>${formatDagNode(edge.from)} -> ${formatDagNode(edge.to)}</span>`).join("") || "<span>none</span>"}</div>
  `;
}

function dagLayer(title: string, nodes: string[], outDegree: Map<string, number>) {
  return `
    <section class="dag-layer">
      <b>${title}</b>
      ${(nodes.length ? nodes : ["none"]).map((node) => {
        const shared = (outDegree.get(node) ?? 0) > 1 ? " shared" : "";
        return `<span class="dag-node${shared}">${formatDagNode(node)}</span>`;
      }).join("")}
    </section>
  `;
}

function formatDagNode(node: string) {
  if (node === "none") return node;
  if (node.includes("#")) {
    const [kind, id] = node.split("#");
    return `${formatDagToken(kind)} #${id}`;
  }
  if (!node.includes(":")) return formatDagToken(node);
  const [kind, ...args] = node.split(":");
  return `${formatDagToken(kind)}(${args.map(formatDagToken).join(", ")})`;
}

function formatDagToken(token: string) {
  return token.replaceAll("_", " ");
}

function indicatorLabel(indicator: Indicator) {
  if (indicator.kind === "MACD" || indicator.kind === "PPO") {
    const label = indicator.kind === "MACD" ? "MACD" : "PPO";
    return `${label} ${indicator.config.fast}/${indicator.config.slow}/${indicator.config.signal}`;
  }
  if (indicator.kind === "CHAIKIN_OSCILLATOR") {
    return `CHAIKIN OSCILLATOR ${indicator.config.fast}/${indicator.config.slow}`;
  }
  if (indicator.kind === "ULTIMATE_OSCILLATOR") {
    return `ULTIMATE OSCILLATOR ${indicator.config.period}/${indicator.config.stoch_period}/${indicator.config.smooth}`;
  }
  if (indicator.kind === "TRIX") {
    return `TRIX ${indicator.config.period}`;
  }
  if (indicator.kind === "DEMA") {
    return `DEMA ${indicator.config.period}`;
  }
  if (indicator.kind === "TEMA") {
    return `TEMA ${indicator.config.period}`;
  }
  if (indicator.kind === "TRIMA") {
    return `TRIMA ${indicator.config.period}`;
  }
  if (indicator.kind === "STDDEV") {
    return `STDDEV ${indicator.config.period}`;
  }
  if (indicator.kind === "ENVELOPE") {
    return `ENVELOPE ${indicator.config.period}/${indicator.config.multiplier}`;
  }
  if (indicator.kind === "TSI") {
    return `TSI ${indicator.config.period}/${indicator.config.stoch_period}`;
  }
  if (indicator.kind === "KST") {
    return "KST";
  }
  if (indicator.kind === "BOP") {
    return "BOP";
  }
  if (indicator.kind === "DPO") {
    return `DPO ${indicator.config.period}`;
  }
  if (indicator.kind === "MOMENTUM") {
    return `MOMENTUM ${indicator.config.period}`;
  }
  if (indicator.kind === "FORCE_INDEX") {
    return `FORCE INDEX ${indicator.config.period}`;
  }
  if (indicator.kind === "VWMA") {
    return `VWMA ${indicator.config.period}`;
  }
  if (indicator.kind === "WILLIAMS_AD") {
    return "WILLIAMS A/D";
  }
  if (indicator.kind === "CHAIKIN_VOLATILITY") {
    return `CHAIKIN VOLATILITY ${indicator.config.period}`;
  }
  if (indicator.kind === "PRICE_CHANNEL") {
    return `PRICE CHANNEL ${indicator.config.period}`;
  }
  if (indicator.kind === "STARC") {
    return `STARC ${indicator.config.period}/${indicator.config.multiplier}`;
  }
  if (indicator.kind === "BB") {
    return `BOLLINGER ${indicator.config.period}/${indicator.config.multiplier}`;
  }
  if (indicator.kind === "KELTNER") {
    return `KELTNER ${indicator.config.period}/${indicator.config.multiplier}`;
  }
  if (indicator.kind === "DONCHIAN") {
    return `DONCHIAN ${indicator.config.period}`;
  }
  if (indicator.kind === "PARABOLIC_SAR") {
    return `PARABOLIC SAR ${indicator.config.psar_step}/${indicator.config.psar_max_step}`;
  }
  if (indicator.kind === "ICHIMOKU") {
    return `ICHIMOKU ${indicator.config.tenkan_period}/${indicator.config.kijun_period}/${indicator.config.senkou_b_period}`;
  }
  if (indicator.kind === "PIVOT_POINTS") {
    return "PIVOT POINTS";
  }
  if (indicator.kind === "ROC") {
    return `ROC ${indicator.config.period}`;
  }
  if (indicator.kind === "AROON") {
    return `AROON ${indicator.config.period}`;
  }
  if (indicator.kind === "CMF") {
    return `CMF ${indicator.config.period}`;
  }
  if (indicator.kind === "ADL") {
    return "ADL";
  }
  if (indicator.kind === "WMA") {
    return `WMA ${indicator.config.period}`;
  }
  if (indicator.kind === "HMA") {
    return `HMA ${indicator.config.period}`;
  }
  if (indicator.kind === "LINEAR_REGRESSION") {
    return `LINEAR REGRESSION ${indicator.config.period}`;
  }
  if (indicator.kind === "ADX") {
    return `ADX ${indicator.config.period}`;
  }
  if (indicator.kind === "SUPERTREND") {
    return `SUPERTREND ${indicator.config.period}/${indicator.config.multiplier}`;
  }
  if (indicator.kind === "CCI") {
    return `CCI ${indicator.config.period}`;
  }
  if (indicator.kind === "MFI") {
    return `MFI ${indicator.config.period}`;
  }
  if (indicator.kind === "WILLIAMS_R") {
    return `WILLIAMS %R ${indicator.config.period}`;
  }
  if (indicator.kind === "STOCH_RSI") {
    return `STOCH RSI ${indicator.config.period}/${indicator.config.stoch_period}/${indicator.config.smooth}/${indicator.config.signal}`;
  }
  if (indicator.kind === "STOCHASTIC") {
    return `STOCHASTIC ${indicator.config.period}/${indicator.config.smooth}`;
  }
  return indicator.config.period ? `${indicator.kind} ${indicator.config.period}` : indicator.kind;
}

function syncIndicatorForm() {
  const descriptor = descriptorFor(kindInput.value);
  params.replaceChildren(
    ...descriptor.params.map((param) => {
      const label = document.createElement("label");
      const input = document.createElement("input");
      input.name = param.name;
      input.type = "number";
      input.min = String(param.min);
      input.step = param.step;
      input.value = String(param.default);
      label.append(param.label, " ", input);
      return label;
    }),
  );
}

function renderIndicatorPicker() {
  const categories = new Map<string, IndicatorDescriptor[]>();
  for (const descriptor of descriptors) {
    const cat = descriptor.category || "Other";
    if (!categories.has(cat)) categories.set(cat, []);
    categories.get(cat)!.push(descriptor);
  }
  const groups: (HTMLOptGroupElement | HTMLOptionElement)[] = [];
  for (const [category, items] of categories) {
    const group = document.createElement("optgroup");
    group.label = category;
    for (const descriptor of items) {
      const option = document.createElement("option");
      option.value = descriptor.kind;
      option.textContent = descriptor.name;
      group.appendChild(option);
    }
    groups.push(group);
  }
  kindInput.replaceChildren(...groups);
  syncIndicatorForm();
}

function renderLayoutPicker() {
  const layouts = readLayouts();
  layoutSelect.replaceChildren(
    ...[
      { value: "", label: "Saved layouts" },
      ...layouts.map((layout) => ({ value: layout.name, label: layout.name })),
    ].map((item) => {
      const option = document.createElement("option");
      option.value = item.value;
      option.textContent = item.label;
      return option;
    }),
  );
  syncLayoutName();
}

function renderPresetPicker() {
  presetSelect.replaceChildren(
    ...presets.map((preset) => {
      const option = document.createElement("option");
      option.value = preset.name;
      option.textContent = preset.name;
      return option;
    }),
  );
}

function syncLayoutName() {
  layoutNameInput.value = layoutSelect.value;
}

function readLayouts(): LayoutState[] {
  try {
    const raw = localStorage.getItem(layoutStorageKey);
    if (!raw) return [];
    const layouts = JSON.parse(raw);
    if (!Array.isArray(layouts)) return [];
    return layouts.filter(isLayoutState);
  } catch {
    return [];
  }
}

function writeLayouts(layouts: LayoutState[]) {
  localStorage.setItem(layoutStorageKey, JSON.stringify(layouts));
}

function isLayoutState(value: unknown): value is LayoutState {
  if (!value || typeof value !== "object") return false;
  const layout = value as Record<string, unknown>;
  return typeof layout.name === "string"
    && typeof layout.symbol === "string"
    && typeof layout.interval === "string"
    && Array.isArray(layout.indicators);
}

function currentLayout(name: string): LayoutState {
  return {
    name,
    symbol: symbolInput.value,
    interval: intervalInput.value,
    indicators: indicators.map((indicator) => ({ ...indicator.config })),
  };
}

function saveLayout() {
  const name = layoutNameInput.value.trim();
  if (!name) {
    setStatus("Layout name is required", "error");
    return;
  }
  const layouts = readLayouts().filter((layout) => layout.name !== name);
  layouts.push(currentLayout(name));
  layouts.sort((left, right) => left.name.localeCompare(right.name));
  writeLayouts(layouts);
  renderLayoutPicker();
  layoutSelect.value = name;
  syncLayoutName();
  setStatus(`Saved layout ${name}`);
}

async function loadSelectedLayout() {
  const layout = readLayouts().find((item) => item.name === layoutSelect.value);
  if (!layout) {
    setStatus("Select a saved layout", "error");
    return;
  }
  await applyLayout(layout);
  setStatus(`Loaded layout ${layout.name}`);
}

function deleteSelectedLayout() {
  const name = layoutSelect.value;
  if (!name) {
    setStatus("Select a saved layout", "error");
    return;
  }
  writeLayouts(readLayouts().filter((layout) => layout.name !== name));
  renderLayoutPicker();
  setStatus(`Deleted layout ${name}`);
}

async function applyLayout(layout: LayoutState) {
  closeStream();
  clearIndicators();
  symbolInput.value = layout.symbol;
  intervalInput.value = layout.interval;
  indicators = layout.indicators.map(indicatorFromConfig);
  renderIndicatorList();
  await load();
}

function applySelectedPreset() {
  const preset = presets.find((item) => item.name === presetSelect.value);
  if (!preset) return;
  const totalStart = performance.now();
  const clearStart = performance.now();
  clearIndicators();
  const clearMs = performance.now() - clearStart;
  indicators = preset.indicators.map(indicatorFromConfig);
  const attachStart = performance.now();
  attachIndicators();
  // Refresh cached column views — engine reallocation during indicator
  // computation can invalidate old TypedArray views into WASM memory.
  currentColumns = engine.candleColumns();
  const attachMs = performance.now() - attachStart;
  const listStart = performance.now();
  renderIndicatorList();
  const listMs = performance.now() - listStart;
  const dagStart = performance.now();
  renderDag();
  const dagMs = performance.now() - dagStart;
  window.__rapidChartPresetBreakdown = {
    preset: preset.name,
    clearMs,
    attachMs,
    listMs,
    dagMs,
    totalMs: performance.now() - totalStart,
  };
  setStatus(`Applied preset ${preset.name}`);
}

function readIndicatorConfig(kind: IndicatorKind) {
  const descriptor = descriptorFor(kind);
  const config: IndicatorConfig = { kind };
  for (const param of descriptor.params) {
    const input = params.querySelector<HTMLInputElement>(`input[name="${param.name}"]`)!;
    const value = Number(input.value);
    if (!Number.isFinite(value) || value < param.min || (param.step === "1" && !Number.isInteger(value))) {
      setStatus(`${param.label} must be ${param.step === "1" ? "an integer" : "a number"} >= ${param.min}`, "error");
      return undefined;
    }
    setConfigParam(config, param.name, value);
  }
  if (
    (kind === "MACD" || kind === "PPO" || kind === "CHAIKIN_OSCILLATOR") &&
    Number(config.slow) <= Number(config.fast)
  ) {
    setStatus(
      kind === "CHAIKIN_OSCILLATOR"
        ? "CHAIKIN OSCILLATOR requires slow > fast"
        : `${kind} requires slow > fast and positive fast/signal`,
      "error",
    );
    return undefined;
  }
  if (
    kind === "PARABOLIC_SAR" &&
    (Number(config.psar_step) <= 0 || Number(config.psar_max_step) < Number(config.psar_step))
  ) {
    setStatus("PARABOLIC SAR requires max_step >= step > 0", "error");
    return undefined;
  }
  if (
    kind === "ICHIMOKU" &&
    (Number(config.tenkan_period) < 1 ||
      Number(config.kijun_period) < Number(config.tenkan_period) ||
      Number(config.senkou_b_period) < Number(config.kijun_period))
  ) {
    setStatus("ICHIMOKU requires kijun >= tenkan and senkou_b >= kijun", "error");
    return undefined;
  }
  if ((kind === "STOCHASTIC" || kind === "STOCH_RSI") && Number(config.smooth) < 1) {
    setStatus(`${kind} smooth must be at least 1`, "error");
    return undefined;
  }
  if (kind === "STOCH_RSI" && (Number(config.stoch_period) < 1 || Number(config.signal) < 1)) {
    setStatus("STOCH_RSI stoch period and %D must be at least 1", "error");
    return undefined;
  }
  if (kind === "TSI" && Number(config.stoch_period) < 1) {
    setStatus("TSI short period must be at least 1", "error");
    return undefined;
  }
  if ((kind === "ENVELOPE" || kind === "STARC") && Number(config.multiplier) <= 0) {
    setStatus(`${kind} multiplier must be greater than zero`, "error");
    return undefined;
  }
  if (
    kind === "ULTIMATE_OSCILLATOR" &&
    (Number(config.stoch_period) < Number(config.period) ||
      Number(config.smooth) < Number(config.stoch_period))
  ) {
    setStatus("ULTIMATE OSCILLATOR requires short <= medium <= long", "error");
    return undefined;
  }
  return config;
}

function indicatorConfig(indicator: Indicator) {
  return indicator.config;
}

function setConfigParam(config: IndicatorConfig, name: string, value: number) {
  if (
    name === "period" ||
    name === "stoch_period" ||
    name === "smooth" ||
    name === "fast" ||
    name === "slow" ||
    name === "signal" ||
    name === "multiplier" ||
    name === "tenkan_period" ||
    name === "kijun_period" ||
    name === "senkou_b_period" ||
    name === "psar_step" ||
    name === "psar_max_step" ||
    name === "anchor"
  ) {
    config[name] = value;
  }
}

function descriptorFor(kind: string) {
  return descriptors.find((descriptor) => descriptor.kind === kind)!;
}

function setStatus(message: string, state: "info" | "error" = "info") {
  status.textContent = message;
  status.classList.toggle("status-error", state === "error");
}
