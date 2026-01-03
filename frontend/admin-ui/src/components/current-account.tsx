"use client";

import { useEffect, useState } from "react";
import { getActiveActorInfo } from "@/lib/auth";

type AccountInfo = {
  staffId: string;
  role: string;
  storeId: string;
  storeCode?: string | null;
};

export default function CurrentAccount() {
  const [info, setInfo] = useState<AccountInfo | null>(null);

  useEffect(() => {
    const actor = getActiveActorInfo();
    if (!actor) {
      setInfo(null);
      return;
    }
    const storeCode =
      window.sessionStorage.getItem("store_code") ||
      window.localStorage.getItem("store_code");
    setInfo({
      staffId: actor.staffId,
      role: actor.role,
      storeId: actor.storeId,
      storeCode,
    });
  }, []);

  if (!info) {
    return (
      <div className="text-xs text-neutral-500">
        Signed in
      </div>
    );
  }

  return (
    <div className="min-w-0 max-w-full space-y-1 text-xs text-neutral-500">
      <div className="font-medium text-neutral-700">Signed in as</div>
      <div className="truncate">
        {info.storeCode ? `store: ${info.storeCode}` : `store_id: ${info.storeId}`}
      </div>
      <div className="truncate">{`role: ${info.role}`}</div>
      <div className="truncate">{`staff_id: ${info.staffId}`}</div>
    </div>
  );
}
