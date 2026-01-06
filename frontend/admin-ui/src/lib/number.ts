export function toNumber(value: unknown): number {
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "string") {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  return 0;
}
