"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { useToast } from "@/components/ui/toast";
import { identityListRoles, identityListStaff, identityUpdateStaff } from "@/lib/identity";
import { formatConnectError } from "@/lib/handle-error";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

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

export default function StaffListForm() {
  const [staff, setStaff] = useState<StaffRow[]>([]);
  const [roles, setRoles] = useState<RoleRow[]>([]);
  const [pending, setPending] = useState<Record<string, string>>({});
  const [pendingName, setPendingName] = useState<Record<string, string>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState<string | null>(null);
  const { push } = useToast();

  const roleLabelMap = useMemo(() => {
    const map = new Map<string, string>();
    roles.forEach((role) => {
      map.set(role.key, role.name || role.key);
    });
    return map;
  }, [roles]);

  async function loadAll() {
    setIsLoading(true);
    try {
      const [roleResp, staffResp] = await Promise.all([
        identityListRoles(),
        identityListStaff(),
      ]);
      setRoles(roleResp.roles ?? []);
      const list = (staffResp.staff ?? []).map((item) => ({
        staffId: item.staffId,
        email: item.email ?? "",
        loginId: item.loginId ?? "",
        phone: item.phone ?? "",
        roleId: item.roleId ?? "",
        roleKey: item.roleKey ?? "",
        status: item.status ?? "",
        displayName: item.displayName ?? "",
      }));
      setStaff(list);
      const initial: Record<string, string> = {};
      const initialNames: Record<string, string> = {};
      list.forEach((row) => {
        initial[row.staffId] = row.roleId;
        initialNames[row.staffId] = row.displayName;
      });
      setPending(initial);
      setPendingName(initialNames);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load staff");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadAll();
  }, []);

  function formatStaffLabel(row: StaffRow) {
    const primary = row.displayName || row.email || row.loginId || row.phone || row.staffId;
    return primary;
  }

  async function handleSave(staffId: string) {
    const roleId = pending[staffId] ?? "";
    const displayName = pendingName[staffId] ?? "";
    if (!roleId) {
      push({
        variant: "error",
        title: "Role required",
        description: "Please select a role before saving.",
      });
      return;
    }
    setIsSaving(staffId);
    try {
      const resp = await identityUpdateStaff({ staffId, roleId, displayName });
      if (!resp.updated) {
        throw new Error("Update failed");
      }
      setStaff((prev) =>
        prev.map((row) =>
          row.staffId === staffId ? { ...row, roleId, displayName } : row
        )
      );
      push({
        variant: "success",
        title: "Updated",
        description: "Staff updated.",
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Update failed", "Failed to update staff");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsSaving(null);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Staff List</CardTitle>
        <CardDescription className="text-neutral-500">
          Update staff roles directly from this list.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between gap-2 text-sm text-neutral-500">
          <div>{staff.length} staff members</div>
          <Button type="button" variant="outline" size="sm" onClick={loadAll} disabled={isLoading}>
            Refresh
          </Button>
        </div>
        <div className="space-y-3">
          {staff.length === 0 ? (
            <div className="text-sm text-neutral-600">No staff found.</div>
          ) : (
            staff.map((row) => {
              const isOwner = row.roleKey === "owner";
              return (
                <div
                  key={row.staffId}
                  className="grid gap-3 rounded-lg border border-neutral-200 bg-neutral-50/60 p-3 md:grid-cols-[1.6fr_1fr_auto]"
                >
                  <div className="space-y-3">
                    <div>
                      <div className="text-sm font-medium text-neutral-900">{formatStaffLabel(row)}</div>
                      <div className="text-xs text-neutral-500">
                        role: {(roleLabelMap.get(row.roleKey) ?? row.roleKey) || "-"}
                      </div>
                      {isOwner ? (
                        <div className="mt-1 text-xs text-emerald-600">Owner account (locked)</div>
                      ) : null}
                    </div>
                    <div className="space-y-1">
                      <Label className="text-xs text-neutral-500">Display name</Label>
                      <Input
                        value={pendingName[row.staffId] ?? ""}
                        onChange={(e) =>
                          setPendingName((prev) => ({ ...prev, [row.staffId]: e.target.value }))
                        }
                        disabled={isOwner}
                      />
                    </div>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs text-neutral-500">Role</Label>
                    <Select
                      value={pending[row.staffId] ?? ""}
                      onValueChange={(value) =>
                        setPending((prev) => ({ ...prev, [row.staffId]: value }))
                      }
                      disabled={isOwner}
                    >
                      <SelectTrigger className="bg-white">
                        <SelectValue placeholder="Select role" />
                      </SelectTrigger>
                      <SelectContent>
                        {roles.length === 0 ? (
                          <SelectItem value="__none__" disabled>
                            No roles found
                          </SelectItem>
                        ) : (
                          roles.map((role) => (
                            <SelectItem key={role.id} value={role.id}>
                              {role.name} {role.key ? `(${role.key})` : ""}
                            </SelectItem>
                          ))
                        )}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex items-center justify-end">
                    <Button
                      type="button"
                      onClick={() => handleSave(row.staffId)}
                      disabled={isOwner || isSaving === row.staffId}
                    >
                      {isSaving === row.staffId ? "Saving..." : "Save"}
                    </Button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </CardContent>
    </Card>
  );
}
