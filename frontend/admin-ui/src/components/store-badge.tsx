"use client";

import { useEffect, useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

export default function StoreBadge() {
  const [storeCode, setStoreCode] = useState<string | null>(null);
  const [tenantId, setTenantId] = useState<string | null>(null);

  useEffect(() => {
    setStoreCode(sessionStorage.getItem("store_code"));
    setTenantId(sessionStorage.getItem("tenant_id"));
  }, []);

  return (
    <Alert className="border-neutral-200 bg-white text-neutral-700">
      <AlertTitle>Store Context</AlertTitle>
      <AlertDescription>
        store_code: {storeCode || "-"} / tenant_id: {tenantId || "-"}
      </AlertDescription>
    </Alert>
  );
}
