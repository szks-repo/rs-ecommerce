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
