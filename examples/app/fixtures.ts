import type { CandleColumns } from "../../src/index";

export type FixtureName = "btcusdt-1m";

export function fixtureColumns(name: string): CandleColumns | undefined {
  if (name !== "btcusdt-1m") return undefined;
  return generateBtcUsdt1mFixture();
}

function generateBtcUsdt1mFixture(): CandleColumns {
  const count = 500;
  const time = new Uint32Array(count);
  const open = new Float64Array(count);
  const high = new Float64Array(count);
  const low = new Float64Array(count);
  const close = new Float64Array(count);
  const volume = new Float64Array(count);

  let lastClose = 42000;
  let timestamp = 1704067200;
  for (let index = 0; index < count; index += 1) {
    const drift = Math.sin(index / 17) * 120 + Math.cos(index / 29) * 55 + index * 0.8;
    const nextClose = Math.max(1000, lastClose + drift * 0.08);
    time[index] = timestamp;
    open[index] = round2(lastClose);
    close[index] = round2(nextClose);
    high[index] = round2(Math.max(lastClose, nextClose) + 18 + (index % 7) * 3);
    low[index] = round2(Math.min(lastClose, nextClose) - 16 - (index % 5) * 2);
    volume[index] = round2(800 + (index % 13) * 37 + Math.abs(Math.sin(index / 11)) * 220);
    lastClose = nextClose;
    timestamp += 60;
  }

  return { time, open, high, low, close, volume };
}

function round2(value: number) {
  return Math.round(value * 100) / 100;
}
