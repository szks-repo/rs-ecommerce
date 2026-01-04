"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identityForceSignOutStaff, identityListStaffSessions } from "@/lib/identity";
import { formatTimestampWithStoreTz } from "@/lib/time";
import type { IdentityStaffSession } from "@/gen/ecommerce/v1/identity_pb";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";

export default function StaffSessionList() {
  const [isSubmitting, setIsSubmitting] = useState<string | null>(null);
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<IdentityStaffSession[]>(
    async () => {
      const res = await identityListStaffSessions();
      return res.sessions ?? [];
    },
    []
  );

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load staff sessions");
    }
  }, [error, notifyError]);

  async function handleForceSignOut(staffId: string) {
    setIsSubmitting(staffId);
    try {
      await identityForceSignOutStaff({ staffId });
      push({
        variant: "success",
        title: "Signed out",
        description: "All active sessions were revoked.",
      });
      await reload();
    } catch (err) {
      notifyError(err, "Action failed", "Failed to revoke sessions");
    } finally {
      setIsSubmitting(null);
    }
  }

  const sessions = data ?? [];

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Active Staff Sessions</CardTitle>
        <CardDescription className="text-neutral-500">
          Review signed-in staff and force sign-out if needed.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="text-sm text-neutral-600">Loading...</div>
        ) : sessions.length === 0 ? (
          <div className="text-sm text-neutral-600">No active sessions.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {sessions.map((session) => (
              <div key={session.sessionId} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <div className="font-medium text-neutral-900">
                      {session.displayName || session.email || session.staffId}
                    </div>
                    <div className="text-xs text-neutral-500">
                      {session.roleKey || "-"} / {session.status}
                    </div>
                    <div className="text-xs text-neutral-500">staff_id: {session.staffId}</div>
                    <div className="text-xs text-neutral-500">session_id: {session.sessionId}</div>
                    <div className="text-xs text-neutral-600">
                      last seen: {formatTimestampWithStoreTz(session.lastSeenAt?.seconds, session.lastSeenAt?.nanos)}
                    </div>
                    <div className="text-xs text-neutral-600">
                      ip: {session.ipAddress || "-"}
                    </div>
                    <div className="text-xs text-neutral-600">
                      ua: {session.userAgent || "-"}
                    </div>
                  </div>
                  <div className="flex flex-col gap-2 text-xs">
                    <Button
                      variant="outline"
                      onClick={() => handleForceSignOut(session.staffId)}
                      disabled={isSubmitting === session.staffId}
                    >
                      {isSubmitting === session.staffId ? "Revoking..." : "Force logout"}
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
