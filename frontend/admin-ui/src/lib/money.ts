export type MoneyLike = {
  amount?: string | number | bigint;
  currency?: string;
};

export function formatMoney(
  money?: MoneyLike,
  locale: string = "ja-JP",
  emptyLabel: string = "-"
) {
  if (!money || money.amount == null) {
    return emptyLabel;
  }
  const amount = typeof money.amount === "bigint" ? Number(money.amount) : Number(money.amount);
  if (!Number.isFinite(amount)) {
    return emptyLabel;
  }
  return `${amount.toLocaleString(locale)} ${money.currency || ""}`.trim();
}
