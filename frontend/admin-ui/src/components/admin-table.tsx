import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function AdminTable({
  children,
  maxHeight = 520,
  className,
}: {
  children: ReactNode;
  maxHeight?: number;
  className?: string;
}) {
  return (
    <div className={cn("overflow-hidden rounded-lg border border-neutral-200 bg-white", className)}>
      <div className="overflow-auto" style={{ maxHeight }}>
        <table className="min-w-full text-sm">{children}</table>
      </div>
    </div>
  );
}

export function AdminTableHeaderCell({
  children,
  align = "left",
  className,
}: {
  children: ReactNode;
  align?: "left" | "right" | "center";
  className?: string;
}) {
  return (
    <th
      className={cn(
        "px-3 py-2 text-xs font-medium uppercase tracking-wide text-neutral-500",
        align === "right" && "text-right",
        align === "center" && "text-center",
        className
      )}
    >
      {children}
    </th>
  );
}

export function AdminTableCell({
  children,
  align = "left",
  className,
  truncate,
}: {
  children: ReactNode;
  align?: "left" | "right" | "center";
  className?: string;
  truncate?: boolean;
}) {
  return (
    <td
      className={cn(
        "px-3 py-1.5 text-[11px] text-neutral-600 align-top",
        align === "right" && "text-right",
        align === "center" && "text-center",
        className
      )}
    >
      {truncate ? <div className="truncate">{children}</div> : children}
    </td>
  );
}

export function AdminTableToolbar({
  left,
  right,
  className,
}: {
  left?: ReactNode;
  right?: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("flex flex-wrap items-center justify-between gap-2", className)}>
      <div className="text-sm text-neutral-500">{left}</div>
      {right ? <div className="flex flex-wrap items-center gap-2">{right}</div> : null}
    </div>
  );
}

export function AdminTablePagination({
  label,
  onPrev,
  onNext,
  canPrev,
  canNext,
}: {
  label: ReactNode;
  onPrev: () => void;
  onNext: () => void;
  canPrev: boolean;
  canNext: boolean;
}) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-2 border-t border-neutral-200 pt-3 text-sm">
      <div className="text-neutral-500">{label}</div>
      <div className="flex items-center gap-2">
        <Button type="button" variant="outline" size="sm" onClick={onPrev} disabled={!canPrev}>
          Prev
        </Button>
        <Button type="button" variant="outline" size="sm" onClick={onNext} disabled={!canNext}>
          Next
        </Button>
      </div>
    </div>
  );
}
