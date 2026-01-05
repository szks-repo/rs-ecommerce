"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { identityListRoles, identityListStaff, identityUpdateStaff } from "@/lib/identity";

type StaffRow = {
  staffId: string;
  email: string;
  loginId: string;
  phone: string;
  roleId: string;
  roleKey: string;
  status: string;
  displayName: string;
};

type RoleRow = {
  id: string;
  key: string;
  name: string;
};

export default function StaffDetailForm({ staffId }: { staffId: string }) {
  const [staff, setStaff] = useState<StaffRow | null>(null);
  const [roles, setRoles] = useState<RoleRow[]>([]);
  const [pendingName, setPendingName] = useState("");
  const [pendingRole, setPendingRole] = useState("");
  const [saving, setSaving] = useState(false);
  const { notifyError } = useApiCall();
  const { push } = useToast();

  const { data, loading, error, reload } = useAsyncResource<{
    roles: RoleRow[];
    staff: StaffRow | null;
  }>(async () => {
    const [roleResp, staffResp] = await Promise.all([
      identityListRoles(),
      identityListStaff({ page: 1, pageSize: 1, query: staffId }),
    ]);
    const roleRows = roleResp.roles ?? [];
    const row = (staffResp.staff ?? []).find((item) => item.staffId === staffId);
    const mapped = row
      ? {
          staffId: row.staffId,
          email: row.email ?? "",
          loginId: row.loginId ?? "",
          phone: row.phone ?? "",
          roleId: row.roleId ?? "",
          roleKey: row.roleKey ?? "",
          status: row.status ?? "",
          displayName: row.displayName ?? "",
        }
      : null;
    return { roles: roleRows, staff: mapped };
  }, [staffId]);

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load staff");
      return;
    }
    if (!data) {
      return;
    }
    setRoles(data.roles);
    setStaff(data.staff);
    if (data.staff) {
      setPendingName(data.staff.displayName || "");
      setPendingRole(data.staff.roleId || "");
    }
  }, [data, error, notifyError]);

  const roleLabelMap = useMemo(() => {
    const map = new Map<string, string>();
    roles.forEach((role) => {
      map.set(role.id, role.name || role.key || role.id);
    });
    return map;
  }, [roles]);

  async function handleSave() {
    if (!staff) {
      return;
    }
    if (!pendingRole) {
      push({
        variant: "error",
        title: "Role required",
        description: "Please select a role before saving.",
      });
      return;
    }
    setSaving(true);
    try {
      const resp = await identityUpdateStaff({
        staffId: staff.staffId,
        roleId: pendingRole,
        displayName: pendingName,
      });
      if (!resp.updated) {
        throw new Error("Update failed");
      }
      setStaff((prev) =>
        prev ? { ...prev, roleId: pendingRole, displayName: pendingName } : prev
      );
      push({
        variant: "success",
        title: "Updated",
        description: "Staff updated.",
      });
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update staff");
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return (
      <Card className="border-neutral-200 bg-white">
        <CardHeader>
          <CardTitle>Staff Detail</CardTitle>
          <CardDescription className="text-neutral-500">Loading staff...</CardDescription>
        </CardHeader>
      </Card>
    );
  }

  if (!staff) {
    return (
      <Card className="border-neutral-200 bg-white">
        <CardHeader>
          <CardTitle>Staff Detail</CardTitle>
          <CardDescription className="text-neutral-500">
            Staff not found.{" "}
            <Button variant="link" size="sm" asChild className="px-0">
              <Link href="/admin/identity">Back to list</Link>
            </Button>
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  const isOwner = staff.roleKey === "owner";

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <div className="flex items-center justify-between gap-2">
          <div>
            <CardTitle>Staff Detail</CardTitle>
            <CardDescription className="text-neutral-500">
              View and update staff profile.
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={reload}>
              Refresh
            </Button>
            <Button variant="outline" size="sm" asChild>
              <Link href="/admin/identity">Back to list</Link>
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <div className="text-sm font-medium text-neutral-900">Primary</div>
            <div className="text-sm text-neutral-600">
              {staff.displayName || staff.email || staff.loginId || staff.phone || staff.staffId}
            </div>
            <div className="text-xs text-neutral-500">staff_id: {staff.staffId}</div>
            <div className="text-xs text-neutral-500">status: {staff.status}</div>
          </div>
          <div className="space-y-1 text-xs text-neutral-500">
            {staff.email ? <div>email: {staff.email}</div> : null}
            {staff.loginId ? <div>login_id: {staff.loginId}</div> : null}
            {!staff.loginId && staff.phone ? <div>phone: {staff.phone}</div> : null}
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label className="text-xs text-neutral-500">Display name</Label>
            {isOwner ? (
              <div className="text-sm text-neutral-500">Owner account (locked)</div>
            ) : (
              <Input
                value={pendingName}
                onChange={(e) => setPendingName(e.target.value)}
                placeholder="Display name"
              />
            )}
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-neutral-500">Role</Label>
            {isOwner ? (
              <div className="text-sm text-neutral-500">
                {roleLabelMap.get(staff.roleId) ?? staff.roleKey}
              </div>
            ) : (
              <Select value={pendingRole} onValueChange={setPendingRole}>
                <SelectTrigger className="bg-white">
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  {roles.map((role) => (
                    <SelectItem key={role.id} value={role.id}>
                      {role.name} {role.key ? `(${role.key})` : ""}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>
        </div>

        {!isOwner ? (
          <div className="flex justify-end">
            <Button type="button" onClick={handleSave} disabled={saving}>
              {saving ? "Saving..." : "Save changes"}
            </Button>
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
