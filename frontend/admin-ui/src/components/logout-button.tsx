"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identitySignOut } from "@/lib/identity";
import { clearActiveStoreSession, getActiveStoreId, getActiveTenantId } from "@/lib/auth";
import { formatConnectError } from "@/lib/handle-error";

export default function LogoutButton() {
  const router = useRouter();
  const { push } = useToast();
  const [isSigningOut, setIsSigningOut] = useState(false);

  async function handleSignOut() {
    if (isSigningOut) {
      return;
    }
    setIsSigningOut(true);
    try {
      await identitySignOut({
        storeId: getActiveStoreId() || undefined,
        tenantId: getActiveTenantId() || undefined,
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Sign out failed", "Failed to sign out");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      clearActiveStoreSession();
      router.push("/login");
      setIsSigningOut(false);
    }
  }

  return (
    <Button variant="outline" size="sm" onClick={handleSignOut} disabled={isSigningOut}>
      {isSigningOut ? "Signing out..." : "Sign out"}
    </Button>
  );
}
