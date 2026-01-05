"use client";

export type DateCellProps = {
  value?: string;
  className?: string;
};

export function formatDateShort(value?: string) {
  if (!value) {
    return "-";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "-";
  }
  const now = new Date();
  const sameYear = date.getFullYear() === now.getFullYear();
  const options: Intl.DateTimeFormatOptions = sameYear
    ? { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" }
    : {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      };
  return new Intl.DateTimeFormat("ja-JP", options).format(date);
}

export default function DateCell({ value, className }: DateCellProps) {
  return <span className={className}>{formatDateShort(value)}</span>;
}
