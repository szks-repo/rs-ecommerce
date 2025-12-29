"use client";

import { useEffect, useState } from "react";

export default function StoreBadge() {
  const [storeCode, setStoreCode] = useState<string | null>(null);
  const [tenantId, setTenantId] = useState<string | null>(null);

  useEffect(() => {
    setStoreCode(sessionStorage.getItem("store_code"));
    setTenantId(sessionStorage.getItem("tenant_id"));
  }, []);

  return (
    <div className="rounded-lg border border-neutral-200 bg-white px-4 py-3 text-sm text-neutral-700">
      <div className="text-xs font-semibold uppercase tracking-[0.2em] text-neutral-400">
        Store Context
      </div>
      <div className="mt-1">
        store_code: <span className="font-medium text-neutral-900">{storeCode || "-"}</span>
      </div>
      <div className="text-xs text-neutral-500">
        tenant_id: {tenantId || "-"}
      </div>
    </div>
  );
}
