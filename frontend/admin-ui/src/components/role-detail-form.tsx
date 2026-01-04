"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { useApiCall } from "@/lib/use-api-call";
import { identityDeleteRole, identityListRolesWithPermissions, identityUpdateRole } from "@/lib/identity";
import { PERMISSION_GROUPS, type PermissionKeyLiteral } from "@/lib/permissions";

type RoleRow = {
  id: string;
  key: string;
  name: string;
  description: string;
  permissionKeys: PermissionKeyLiteral[];
};

export default function RoleDetailForm({ roleId }: { roleId: string }) {
  const [role, setRole] = useState<RoleRow | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const { call } = useApiCall();

  const rolePermissions = useMemo(() => new Set(role?.permissionKeys ?? []), [role]);

  async function loadRole() {
    setIsLoading(true);
    const data = await call(
      () => identityListRolesWithPermissions(),
      { errorTitle: "Load failed", errorDescription: "Failed to load role" }
    );
    if (data) {
      const found = (data.roles ?? []).find((item) => item.id === roleId);
      if (!found) {
        setRole(null);
      } else {
        setRole({
          id: found.id,
          key: found.key,
          name: found.name,
          description: found.description ?? "",
          permissionKeys: (found.permissionKeys ?? []) as PermissionKeyLiteral[],
        });
      }
    }
    setIsLoading(false);
  }

  useEffect(() => {
    void loadRole();
  }, [roleId]);

  function togglePermission(permissionKey: PermissionKeyLiteral) {
    if (!role) return;
    const next = new Set(role.permissionKeys);
    if (next.has(permissionKey)) {
      next.delete(permissionKey);
    } else {
      next.add(permissionKey);
    }
    setRole({ ...role, permissionKeys: Array.from(next) });
  }

  async function handleSave() {
    if (!role) return;
    setIsSaving(true);
    const resp = await call(
      () =>
        identityUpdateRole({
          roleId: role.id,
          name: role.name,
          description: role.description,
          permissionKeys: role.permissionKeys,
        }),
      {
        success: {
          title: "Role updated",
          description: `Updated ${role.name}`,
        },
        errorTitle: "Update failed",
        errorDescription: "Failed to update role",
      }
    );
    if (resp && !resp.updated) {
      await call(
        async () => {
          throw new Error("Update failed");
        },
        { errorTitle: "Update failed", errorDescription: "Failed to update role" }
      );
    }
    setIsSaving(false);
  }

  async function handleDelete() {
    if (!role) return;
    const confirmed = window.confirm(`Delete role "${role.name}"?`);
    if (!confirmed) return;
    setIsSaving(true);
    const resp = await call(
      () => identityDeleteRole({ roleId: role.id }),
      {
        success: {
          title: "Role deleted",
          description: `Deleted ${role.name}`,
        },
        errorTitle: "Delete failed",
        errorDescription: "Failed to delete role",
      }
    );
    if (resp?.deleted) {
      window.location.href = "/admin/identity/roles";
    }
    setIsSaving(false);
  }

  if (isLoading) {
    return (
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardContent className="py-6 text-sm text-neutral-600">Loading...</CardContent>
      </Card>
    );
  }

  if (!role) {
    return (
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardContent className="py-6 text-sm text-neutral-600">Role not found.</CardContent>
      </Card>
    );
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Role Details</CardTitle>
        <CardDescription className="text-neutral-500">
          Edit role name, description, and permissions.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid gap-4 md:grid-cols-[1.4fr_1fr_auto] md:items-end">
          <div className="space-y-2">
            <Label className="text-xs text-neutral-500">Name</Label>
            <Input value={role.name} onChange={(e) => setRole({ ...role, name: e.target.value })} />
          </div>
          <div className="space-y-2">
            <Label className="text-xs text-neutral-500">Description</Label>
            <Input
              value={role.description}
              onChange={(e) => setRole({ ...role, description: e.target.value })}
            />
          </div>
          <div className="flex items-center justify-end gap-2">
            <Button type="button" onClick={handleSave} disabled={isSaving}>
              {isSaving ? "Saving..." : "Save"}
            </Button>
            <div className="relative group">
              <Button
                type="button"
                variant="outline"
                onClick={handleDelete}
                disabled
                title="Cannot delete roles with staff attached"
              >
                Delete
              </Button>
              <div className="pointer-events-none absolute right-0 top-[-2.4rem] hidden whitespace-nowrap rounded-md border border-neutral-200 bg-white px-2 py-1 text-xs text-neutral-700 shadow-sm group-hover:block">
                Detach all staff before deleting this role.
              </div>
            </div>
          </div>
        </div>
        <div className="text-xs text-neutral-500">Key: {role.key}</div>
        <div className="grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
          {PERMISSION_GROUPS.map((group) => (
            <div key={group.label} className="rounded-lg border border-neutral-200 bg-neutral-50/60 p-4">
              <div className="text-sm font-semibold text-neutral-900">{group.label}</div>
              <div className="text-xs text-neutral-500">{group.description}</div>
              <div className="mt-3 grid gap-2">
                {group.permissions.map((permission) => (
                  <label
                    key={permission.key}
                    className="flex cursor-pointer items-center gap-2 rounded-md border border-transparent px-2 py-1 text-sm text-neutral-700 transition hover:border-neutral-200 hover:bg-white"
                  >
                    <Checkbox
                      checked={rolePermissions.has(permission.key)}
                      onCheckedChange={() => togglePermission(permission.key)}
                    />
                    <span>{permission.label}</span>
                  </label>
                ))}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
