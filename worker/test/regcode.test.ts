import { test } from "node:test";
import assert from "node:assert/strict";
import { isMatch, listRegcodes } from "../src/regcode.ts";

test("isMatch wildcard", () => {
  assert.ok(isMatch("1111000000", "1111*"));
  assert.ok(isMatch("1111000000", "*000000"));
  assert.ok(isMatch("1111000000", "11??000000"));
  assert.ok(!isMatch("1111000000", "2111*"));
});

test("서울특별시 하위 목록", () => {
  const all = listRegcodes("11*", false);
  assert.ok(all.some((r) => r.code === "1100000000"));
  assert.ok(all.every((r) => r.code.startsWith("11")));
});

test("isIgnoreZero는 1111000000을 제외", () => {
  const a = listRegcodes("1111*", false).map((r) => r.code);
  const b = listRegcodes("1111*", true).map((r) => r.code);
  assert.ok(a.includes("1111000000"));
  assert.ok(!b.includes("1111000000"));
});

test("폐지된 코드/빈 코드는 포함되지 않음", () => {
  assert.ok(listRegcodes("*", false).every((r) => r.code !== ""));
});
