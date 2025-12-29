"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { getActiveAccessToken } from "@/lib/auth";

export default function RequireAuth({
  children,
  redirectTo = "/login",
}: {
  children: React.ReactNode;
  redirectTo?: string;
}) {
  const router = useRouter();
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    const token = getActiveAccessToken();
    if (!token) {
      router.replace(redirectTo);
      return;
    }
    setChecked(true);
  }, [router, redirectTo]);

  if (!checked) {
    return null;
  }

  return <>{children}</>;
}
