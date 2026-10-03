// pkg/csv/internal/address.csv (TSV) -> src/regcodes.json ([code, name] 배열, 폐지되지 않은 항목만)
import { readFileSync, writeFileSync } from "node:fs";

const src = new URL("../../pkg/csv/internal/address.csv", import.meta.url);
const out = new URL("../src/regcodes.json", import.meta.url);

const rows = readFileSync(src, "utf8")
  .split(/\r?\n/)
  .filter(Boolean)
  .map((l) => l.split("\t"))
  .filter((r) => r[2] === "존재")
  .map((r) => [r[0], r[1]]);

writeFileSync(out, JSON.stringify(rows));
console.log(`wrote ${rows.length} regcodes`);
