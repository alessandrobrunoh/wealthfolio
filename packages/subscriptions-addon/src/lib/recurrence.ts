export const MAX_INTERVAL_MONTHS = 12;

function daysInMonth(year: number, month: number): number {
  const isLeapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, isLeapYear ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return days[month - 1] ?? 0;
}

export function isCalendarDate(value: string): boolean {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return false;

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  return month >= 1 && month <= 12 && day >= 1 && day <= daysInMonth(year, month);
}

export function getCalendarDay(value: string): number {
  if (!isCalendarDate(value)) {
    throw new RangeError("A valid YYYY-MM-DD billing date is required");
  }
  return Number(value.slice(-2));
}

export function advanceBillingDate(
  currentDate: string,
  intervalMonths: number,
  billingDay = getCalendarDay(currentDate),
): string {
  if (!isCalendarDate(currentDate)) {
    throw new RangeError("A valid YYYY-MM-DD billing date is required");
  }
  if (
    !Number.isInteger(intervalMonths) ||
    intervalMonths < 1 ||
    intervalMonths > MAX_INTERVAL_MONTHS
  ) {
    throw new RangeError(`Billing interval must be between 1 and ${MAX_INTERVAL_MONTHS} months`);
  }
  if (!Number.isInteger(billingDay) || billingDay < 1 || billingDay > 31) {
    throw new RangeError("Billing day must be between 1 and 31");
  }

  const [year, month] = currentDate.split("-").map(Number);
  const targetMonthIndex = year * 12 + month - 1 + intervalMonths;
  const targetYear = Math.floor(targetMonthIndex / 12);
  const targetMonth = (targetMonthIndex % 12) + 1;
  const lastDayOfMonth = daysInMonth(targetYear, targetMonth);
  const targetDay = Math.min(billingDay, lastDayOfMonth);

  return `${targetYear}-${String(targetMonth).padStart(2, "0")}-${String(targetDay).padStart(2, "0")}`;
}
