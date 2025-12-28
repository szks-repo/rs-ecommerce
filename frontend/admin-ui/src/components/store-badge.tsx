"use client";

import { useEffect, useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

export default function StoreBadge() {
  const [storeId, setStoreId] = useState<string | null>(null);
  const [tenantId, setTenantId] = useState<string | null>(null);

  useEffect(() => {
    setStoreId(sessionStorage.getItem("store_id"));
    setTenantId(sessionStorage.getItem("tenant_id"));
  }, []);

  return (
    <Alert className="border-neutral-800 bg-neutral-900 text-neutral-200">
      <AlertTitle>Store Context</AlertTitle>
      <AlertDescription>
        store_id: {storeId || "-"} / tenant_id: {tenantId || "-"}
      </AlertDescription>
    </Alert>
  );
}
