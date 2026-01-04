"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useApiCall } from "@/lib/use-api-call";
import { identityInviteStaff, identityListRoles } from "@/lib/identity";
import { formatDateWithStoreTz } from "@/lib/time";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export default function StaffInviteForm() {
  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [roles, setRoles] = useState<Array<{ id: string; key: string; name: string }>>([]);
  const [roleId, setRoleId] = useState("");
  const [inviteToken, setInviteToken] = useState("");
  const [expiresAt, setExpiresAt] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoadingRoles, setIsLoadingRoles] = useState(false);
  const { call } = useApiCall();

  useEffect(() => {
    let cancelled = false;
    setIsLoadingRoles(true);
    call(() => identityListRoles(), {
      errorTitle: "Load failed",
      errorDescription: "Failed to load roles",
    })
      .then((data) => {
        if (cancelled || !data) return;
        const list = data.roles ?? [];
        setRoles(list);
        if (!roleId && list.length > 0) {
          setRoleId(list[0].id || "");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoadingRoles(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleInvite(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    if (!roleId) {
      await call(
        async () => {
          throw new Error("role is required");
        },
        { errorTitle: "Invite failed", errorDescription: "Role is required" }
      );
      setIsSubmitting(false);
      return;
    }
    const resp = await call(
      () =>
        identityInviteStaff({
          email,
          roleId,
          displayName,
        }),
      {
        success: {
          title: "Invite created",
          description: `Invite for ${email}`,
        },
        errorTitle: "Invite failed",
        errorDescription: "Failed to invite staff",
      }
    );
    if (resp) {
      setInviteToken(resp.inviteToken || "");
      setExpiresAt(
        resp.expiresAt?.seconds
          ? formatDateWithStoreTz(new Date(Number(resp.expiresAt.seconds) * 1000))
          : ""
      );
      setEmail("");
      setDisplayName("");
      setRoleId(roles[0]?.id ?? "");
    }
    setIsSubmitting(false);
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Invite Staff</CardTitle>
        <CardDescription className="text-neutral-500">
          Invite staff by email. Owner only.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4 md:grid-cols-2" onSubmit={handleInvite}>
          <div className="space-y-2">
            <Label htmlFor="inviteEmail">Email</Label>
            <Input
              id="inviteEmail"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="inviteDisplayName">Display Name</Label>
            <Input
              id="inviteDisplayName"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
            />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="inviteRole">Role</Label>
            <Select value={roleId} onValueChange={setRoleId}>
              <SelectTrigger id="inviteRole" className="bg-white">
                <SelectValue placeholder={isLoadingRoles ? "Loading roles..." : "Select role"} />
              </SelectTrigger>
              <SelectContent>
                {roles.length === 0 && !isLoadingRoles ? (
                  <SelectItem value="__none__" disabled>
                    No roles found
                  </SelectItem>
                ) : (
                  roles.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.name} {item.key ? `(${item.key})` : ""}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
          <div className="md:col-span-2">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Inviting..." : "Create Invite"}
            </Button>
          </div>
        </form>
        {inviteToken ? (
          <div className="mt-4 rounded-lg border border-emerald-200 bg-emerald-50 p-4 text-sm text-emerald-900">
            <div className="font-medium">Invite token</div>
            <div className="mt-1 break-all font-mono text-xs">{inviteToken}</div>
            {expiresAt ? <div className="mt-2 text-xs text-emerald-700">Expires: {expiresAt}</div> : null}
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
