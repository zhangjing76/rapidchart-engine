import { readFile } from "node:fs/promises";
import path from "node:path";

const files = process.argv.slice(2);
if (files.length < 2) {
  throw new Error("usage: node scripts/bench-compare.mjs <run-a.json> <run-b.json> [run-c.json]");
}

const runs = await Promise.all(
  files.map(async (file) => ({
    file,
    data: JSON.parse(await readFile(file, "utf8")),
  })),
);

const scenarioNames = Array.from(
  new Set(runs.flatMap((run) => run.data.scenarios.map((scenario) => scenario.name))),
).sort(compareScenarioNames);

const labels = runs.map((run) => path.basename(run.file, ".json"));
const firstWidth = Math.max(
  "scenario".length,
  ...scenarioNames.map((name) => name.length),
);
const colWidth = Math.max(
  12,
  ...labels.map((label) => label.length),
);

console.log(
  `${pad("scenario", firstWidth)} ${labels.map((label) => pad(label, colWidth)).join(" ")}`,
);

for (const name of scenarioNames) {
  const cols = runs.map((run) => {
    const scenario = run.data.scenarios.find((item) => item.name === name);
    return pad(scenario ? `${scenario.medianMs.toFixed(3)}ms` : "-", colWidth);
  });
  console.log(`${pad(name, firstWidth)} ${cols.join(" ")}`);
}

function pad(value, width) {
  return String(value).padEnd(width);
}

function compareScenarioNames(a, b) {
  const [aPrefix, aSize] = splitScenario(a);
  const [bPrefix, bSize] = splitScenario(b);
  if (aSize !== bSize) return aSize - bSize;
  return aPrefix.localeCompare(bPrefix);
}

function splitScenario(name) {
  const match = name.match(/^(.*?):(\d+)/);
  if (!match) return [name, Number.MAX_SAFE_INTEGER];
  return [match[1], Number.parseInt(match[2], 10)];
}
