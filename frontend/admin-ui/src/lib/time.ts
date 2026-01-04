export function getStoreTimeZone(): string {
  if (typeof window === "undefined") {
    return "Asia/Tokyo";
  }
  return window.sessionStorage.getItem("store_time_zone") || "Asia/Tokyo";
}

export function formatTimestampWithStoreTz(seconds?: string | number | bigint, nanos?: number): string {
  if (seconds == null) {
    return "-";
  }
  const sec = typeof seconds === "bigint" ? Number(seconds) : Number(seconds);
  if (!Number.isFinite(sec)) {
    return "-";
  }
  const date = new Date(sec * 1000);
  return date.toLocaleString("ja-JP", { timeZone: getStoreTimeZone() });
}

export function formatDateWithStoreTz(date: Date): string {
  return date.toLocaleString("ja-JP", { timeZone: getStoreTimeZone() });
}

export type TimestampLike = {
  seconds: bigint | number | string;
  nanos: number;
};

export function timestampToDateInput(ts?: TimestampLike): string {
  if (!ts) {
    return "";
  }
  const sec = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
  if (!Number.isFinite(sec)) {
    return "";
  }
  const date = new Date(sec * 1000);
  const formatter = new Intl.DateTimeFormat("en-CA", {
    timeZone: getStoreTimeZone(),
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
  return formatter.format(date);
}

export function dateInputToTimestamp(
  dateInput: string,
  endOfDay: boolean
): TimestampLike | undefined {
  const date = toUtcDateFromStoreDateInput(dateInput, endOfDay);
  if (!date) {
    return undefined;
  }
  const seconds = Math.floor(date.getTime() / 1000);
  return { seconds: BigInt(seconds), nanos: 0 };
}

function getTimeZoneOffsetMinutes(date: Date, timeZone: string): number {
  const formatter = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
  const parts = formatter.formatToParts(date);
  const values: Record<string, string> = {};
  parts.forEach((part) => {
    values[part.type] = part.value;
  });
  const asUtc = Date.UTC(
    Number(values.year),
    Number(values.month) - 1,
    Number(values.day),
    Number(values.hour),
    Number(values.minute),
    Number(values.second)
  );
  return (asUtc - date.getTime()) / 60000;
}

export function toUtcDateFromStoreDateInput(dateInput: string, endOfDay: boolean): Date | null {
  if (!dateInput) {
    return null;
  }
  const [yearStr, monthStr, dayStr] = dateInput.split("-");
  const year = Number(yearStr);
  const month = Number(monthStr);
  const day = Number(dayStr);
  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
    return null;
  }
  const hour = endOfDay ? 23 : 0;
  const minute = endOfDay ? 59 : 0;
  const second = endOfDay ? 59 : 0;
  const timeZone = getStoreTimeZone();
  const utcDate = new Date(Date.UTC(year, month - 1, day, hour, minute, second));
  const offsetMinutes = getTimeZoneOffsetMinutes(utcDate, timeZone);
  return new Date(utcDate.getTime() - offsetMinutes * 60000);
}
