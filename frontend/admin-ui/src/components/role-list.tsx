"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identityListRolesWithPermissions } from "@/lib/identity";
import { formatConnectError } from "@/lib/handle-error";

type RoleRow = {
  id: string;
  key: string;
  name: string;
  description: string;
  permissionKeys: string[];
};

export default function RoleList() {
  const [roles, setRoles] = useState<RoleRow[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  async function loadRoles() {
    setIsLoading(true);
    try {
      const data = await identityListRolesWithPermissions();
      const list = (data.roles ?? []).map((role) => ({
        id: role.id,
        key: role.key,
        name: role.name,
        description: role.description ?? "",
        permissionKeys: role.permissionKeys ?? [],
      }));
      setRoles(list);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load roles");
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
    void loadRoles();
  }, []);

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Roles</CardTitle>
        <CardDescription className="text-neutral-500">
          Manage roles and permissions.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between gap-2 text-sm text-neutral-500">
          <div>{roles.length} roles</div>
          <Button type="button" variant="outline" size="sm" onClick={loadRoles} disabled={isLoading}>
            Refresh
          </Button>
        </div>
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
