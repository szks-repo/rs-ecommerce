import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

type AdminPageHeaderProps = {
  title: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
};

export default function AdminPageHeader({
  title,
  description,
  actions,
  className,
}: AdminPageHeaderProps) {
  return (
    <div className={cn("flex flex-wrap items-center justify-between gap-4", className)}>
      <div>
        <h1 className="text-lg font-semibold text-neutral-900">{title}</h1>
        {description ? (
          <p className="mt-2 text-sm text-neutral-600">{description}</p>
        ) : null}
      </div>
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </div>
  );
}
