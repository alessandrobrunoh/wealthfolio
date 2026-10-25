import assert from "node:assert/strict";
import test from "node:test";
import { advanceBillingDate, getCalendarDay, isCalendarDate } from "./recurrence.ts";

test("validates calendar dates without accepting rollover dates", () => {
  assert.equal(isCalendarDate("2024-02-29"), true);
  assert.equal(isCalendarDate("2023-02-29"), false);
  assert.equal(isCalendarDate("2024-13-01"), false);
  assert.equal(isCalendarDate("2024-1-01"), false);
});

test("advances monthly dates while preserving the original billing day", () => {
  assert.equal(advanceBillingDate("2024-01-31", 1, 31), "2024-02-29");
  assert.equal(advanceBillingDate("2024-02-29", 1, 31), "2024-03-31");
});

test("advances multi-month billing intervals", () => {
  assert.equal(advanceBillingDate("2024-01-31", 2, 31), "2024-03-31");
  assert.equal(advanceBillingDate("2024-11-30", 3, 30), "2025-02-28");
});

test("rejects invalid intervals and dates", () => {
  assert.throws(() => advanceBillingDate("2024-02-30", 1), RangeError);
  assert.throws(() => advanceBillingDate("2024-01-31", 0), RangeError);
  assert.throws(() => advanceBillingDate("2024-01-31", 13), RangeError);
  assert.equal(getCalendarDay("2024-01-31"), 31);
  assert.throws(() => getCalendarDay("not-a-date"), RangeError);
});
