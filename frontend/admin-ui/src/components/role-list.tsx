"use client";

import { useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { identityListRolesWithPermissions } from "@/lib/identity";
import { AdminTableToolbar } from "@/components/admin-table";

type RoleRow = {
  id: string;
  key: string;
  name: string;
  description: string;
  permissionKeys: string[];
};

export default function RoleList() {
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<RoleRow[]>(
    async () => {
      const data = await identityListRolesWithPermissions();
      return (data.roles ?? []).map((role) => ({
        id: role.id,
        key: role.key,
        name: role.name,
        description: role.description ?? "",
        permissionKeys: role.permissionKeys ?? [],
      }));
    },
    []
  );

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load roles");
    }
  }, [error, notifyError]);

  const roles = data ?? [];

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Roles</CardTitle>
        <CardDescription className="text-neutral-500">
          Manage roles and permissions.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <AdminTableToolbar
          left={`${roles.length} roles`}
          right={
            <Button type="button" variant="outline" size="sm" onClick={reload} disabled={loading}>
              Refresh
            </Button>
          }
        />
        {roles.length === 0 ? (
          <div className="text-sm text-neutral-600">No roles found.</div>
        ) : (
          <div className="grid gap-3">
            {roles.map((role) => (
              <a
                key={role.id}
                href={`/admin/identity/roles/${role.id}`}
                className="rounded-lg border border-neutral-200 bg-neutral-50/60 p-4 transition hover:border-neutral-300 hover:bg-white"
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="text-sm font-semibold text-neutral-900">{role.name}</div>
                  <div className="text-xs text-neutral-500">{role.permissionKeys.length} permissions</div>
                </div>
                <div className="mt-1 text-xs text-neutral-500">key: {role.key}</div>
                {role.description ? (
                  <div className="mt-2 text-sm text-neutral-600">{role.description}</div>
                ) : null}
              </a>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
