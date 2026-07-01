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
  type DagDebug,
  type IndicatorConfig,
  type IndicatorDescriptor,
  type IndicatorKind,
  type IndicatorOutputPoint,
  type IndicatorOutputSeries,
  type OutputDescriptor,
  type ParamDescriptor,
  RapidChartEngine,
  initEngine,
} from "./engine";
import "./style.css";

type IndicatorPoint = {
  time: number;
  value: number | null;
};

type IndicatorSeries = {
  output: string;
  series: ISeriesApi<"Line"> | ISeriesApi<"Histogram">;
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

const symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];
const intervals = ["1m", "5m", "15m", "1h", "4h", "1d"];
const colors = ["#2563eb", "#dc2626", "#059669", "#9333ea", "#ea580c"];
const ichimokuBullCloud = "rgba(5, 150, 105, 0.18)";
const ichimokuBearCloud = "rgba(220, 38, 38, 0.18)";

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
    <section id="dag"></section>
    <section id="chart"></section>
    <section id="indicators"></section>
    <footer>
      <span id="status"></span>
      <a href="https://www.tradingview.com/" target="_blank" rel="noreferrer">Charts by TradingView</a>
    </footer>
  </main>
`;

const toolbar = document.querySelector<HTMLFormElement>("#toolbar")!;
const symbolInput = document.querySelector<HTMLSelectElement>("#symbol")!;
const intervalInput = document.querySelector<HTMLSelectElement>("#interval")!;
const kindInput = document.querySelector<HTMLSelectElement>("#kind")!;
const params = document.querySelector<HTMLFieldSetElement>("#params")!;
const status = document.querySelector<HTMLParagraphElement>("#status")!;
const indicatorList = document.querySelector<HTMLElement>("#indicators")!;
const dag = document.querySelector<HTMLElement>("#dag")!;

const chart = createChart(document.querySelector<HTMLElement>("#chart")!, {
  autoSize: true,
  layout: { attributionLogo: false, background: { color: "#ffffff" }, textColor: "#111827" },
  grid: { vertLines: { color: "#e5e7eb" }, horzLines: { color: "#e5e7eb" } },
  rightPriceScale: { borderVisible: false },
  timeScale: { borderVisible: false },
});
const candles = chart.addSeries(CandlestickSeries);
const volume = chart.addSeries(HistogramSeries, {
  color: "#94a3b8",
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

await initEngine();
engine = new RapidChartEngine(symbolInput.value, intervalInput.value);
descriptors = engine.indicatorDescriptors();
renderIndicatorPicker();
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

async function load() {
  setStatus("Loading...");
  closeStream();
  detachIndicatorSeries();
  try {
    engine = new RapidChartEngine(symbolInput.value, intervalInput.value);
    const bars = await fetchBars(symbolInput.value, intervalInput.value);
    engine.ingestBars(bars);
    candles.setData(engine.candles().map((bar: Bar) => ({ ...bar, time: bar.time as Time })));
    volume.setData(bars.map(volumePoint));
    attachIndicators();
    renderDag();
    chart.timeScale().fitContent();
    openStream(symbolInput.value, intervalInput.value);
    setStatus(`${symbolInput.value} ${intervalInput.value} live`);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : "Failed to load data", "error");
  }
}

async function fetchBars(symbol: string, interval: string): Promise<Bar[]> {
  const url = new URL("https://api.binance.com/api/v3/klines");
  url.search = new URLSearchParams({ symbol, interval, limit: "500" }).toString();
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Binance returned ${response.status}`);
  const rows = await response.json();
  if (!Array.isArray(rows)) throw new Error("Unexpected Binance response");
  const bars = rows
    .map(barFromRestRow)
    .filter((bar): bar is Bar => bar !== undefined);
  if (bars.length === 0) throw new Error("No valid bars returned");
  return bars;
}

function barFromRestRow(row: unknown): Bar | undefined {
  if (!Array.isArray(row)) return undefined;
  const bar = {
    time: Math.floor(Number(row[0]) / 1000),
    open: Number(row[1]),
    high: Number(row[2]),
    low: Number(row[3]),
    close: Number(row[4]),
    volume: Number(row[5]),
  };
  return isFiniteBar(bar) ? bar : undefined;
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

function attachIndicator(indicator: Indicator, index: number) {
  indicator.engineId = engine.addIndicator(indicatorConfig(indicator));
  indicator.series = descriptorFor(indicator.kind).outputs.map((output, outputIndex) =>
    addOutputSeries(output, colors[(index + outputIndex) % colors.length]),
  );
  if (indicator.kind === "ICHIMOKU") attachIchimokuCloud(indicator);
  if (indicator.kind === "RSI") addRsiGuides(indicator.series[0].series as ISeriesApi<"Line">);
  renderIndicator(indicator);
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
  indicators.forEach((indicator, index) => attachIndicator(indicator, index));
}

function openStream(symbol: string, interval: string) {
  const socket = new WebSocket(`wss://stream.binance.com:9443/ws/${symbol.toLowerCase()}@kline_${interval}`);
  stream = socket;
  socket.addEventListener("message", (event) => {
    const bar = barFromStream(JSON.parse(event.data));
    if (!bar) return;
    try {
      engine.upsertBar(bar);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Live update failed", "error");
      if (stream === socket) stream = undefined;
      socket.close();
      return;
    }
    candles.update({ ...bar, time: bar.time as Time });
    volume.update(volumePoint(bar));
    for (const indicator of indicators) updateIndicator(indicator);
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

function renderIndicator(indicator: Indicator) {
  if (!indicator.engineId) return;
  const outputSeries = engine.indicatorSeries(indicator.engineId);
  const series = new Map(outputSeries.map((item) => [item.output, item.points]));
  for (const item of indicator.series) {
    const points = series.get(item.output);
    if (!points) continue;
    item.series.setData(
      points
        .filter((point) => point.value !== null)
        .map((point) => indicatorPoint(item.output, point)),
    );
  }
  indicator.cloud?.primitive.setData(series.get("senkou_a"), series.get("senkou_b"));
}

function updateIndicator(indicator: Indicator) {
  if (!indicator.engineId) return;
  const points = new Map(
    engine.latestIndicatorPoints(indicator.engineId).map((point) => [
      point.output,
      point,
    ]),
  );
  for (const item of indicator.series) {
    const point = points.get(item.output);
    if (!point) continue;
    if (point.value !== null) item.series.update(indicatorPoint(item.output, point));
  }
  indicator.cloud?.primitive.updatePoints(points.get("senkou_a"), points.get("senkou_b"));
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
  const computed = graph.nodes.filter((node) => node.includes(":"));
  const indicatorNodes = graph.nodes.filter((node) => node.includes("#"));
  const outDegree = new Map<string, number>();
  for (const edge of graph.edges) outDegree.set(edge.from, (outDegree.get(edge.from) ?? 0) + 1);
  dag.innerHTML = `
    <header><strong>DAG</strong><span>${graph.nodes.length} nodes</span><span>${graph.edges.length} edges</span></header>
    <div class="dag-grid">
      ${dagLayer("Source", sources, outDegree)}
      ${dagLayer("Computed", computed, outDegree)}
      ${dagLayer("Indicators", indicatorNodes, outDegree)}
    </div>
    <div class="dag-edges">${graph.edges.map((edge) => `<span>${edge.from} -> ${edge.to}</span>`).join("") || "<span>none</span>"}</div>
  `;
}

function dagLayer(title: string, nodes: string[], outDegree: Map<string, number>) {
  return `
    <section class="dag-layer">
      <b>${title}</b>
      ${(nodes.length ? nodes : ["none"]).map((node) => {
        const shared = (outDegree.get(node) ?? 0) > 1 ? " shared" : "";
        return `<span class="dag-node${shared}">${node}</span>`;
      }).join("")}
    </section>
  `;
}

function indicatorLabel(indicator: Indicator) {
  if (indicator.kind === "MACD") {
    return `MACD ${indicator.config.fast}/${indicator.config.slow}/${indicator.config.signal}`;
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
  kindInput.replaceChildren(
    ...descriptors.map((descriptor) => {
      const option = document.createElement("option");
      option.value = descriptor.kind;
      option.textContent = descriptor.name;
      return option;
    }),
  );
  syncIndicatorForm();
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
  if (kind === "MACD" && Number(config.slow) <= Number(config.fast)) {
    setStatus("MACD requires slow > fast and positive fast/signal", "error");
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
    name === "psar_max_step"
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
