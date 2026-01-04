"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { useApiCall } from "@/lib/use-api-call";
import { identityCreateRole } from "@/lib/identity";
import {
  DEFAULT_PERMISSION_KEYS,
  PERMISSION_GROUPS,
  type PermissionKeyLiteral,
} from "@/lib/permissions";

export default function RoleCreateForm() {
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [permissionKeys, setPermissionKeys] = useState<Set<PermissionKeyLiteral>>(
    () => new Set(DEFAULT_PERMISSION_KEYS)
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { call } = useApiCall();

  function handleSelectAll() {
    setPermissionKeys(new Set(DEFAULT_PERMISSION_KEYS));
  }

  function handleClearAll() {
    setPermissionKeys(new Set());
  }

  function togglePermission(permissionKey: PermissionKeyLiteral) {
    setPermissionKeys((prev) => {
      const next = new Set(prev);
      if (next.has(permissionKey)) {
        next.delete(permissionKey);
      } else {
        next.add(permissionKey);
      }
      return next;
    });
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    const keys = Array.from(permissionKeys);
    const data = await call(
      () =>
        identityCreateRole({
          key,
          name,
          description,
          permissionKeys: keys,
        }),
      {
        success: {
          title: "Role created",
          description: `Created role: ${key || ""}`,
        },
        errorTitle: "Create failed",
        errorDescription: "Failed to create role",
      }
    );
    if (data) {
      setKey("");
      setName("");
      setDescription("");
      setPermissionKeys(new Set(DEFAULT_PERMISSION_KEYS));
    }
    setIsSubmitting(false);
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Role</CardTitle>
        <CardDescription className="text-neutral-500">
          Define a role and its permissions.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-6 md:grid-cols-2" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="roleKey">Role Key</Label>
            <Input id="roleKey" value={key} onChange={(e) => setKey(e.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="roleName">Role Name</Label>
            <Input id="roleName" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="roleDescription">Description</Label>
            <Input id="roleDescription" value={description} onChange={(e) => setDescription(e.target.value)} />
          </div>
          <div className="space-y-3 md:col-span-2">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <Label>Permissions</Label>
              <div className="flex flex-wrap items-center gap-2 text-xs text-neutral-500">
                <span>Selected: {permissionKeys.size}</span>
                <Button type="button" variant="outline" size="sm" onClick={handleSelectAll}>
                  Select all
                </Button>
                <Button type="button" variant="outline" size="sm" onClick={handleClearAll}>
                  Clear
                </Button>
              </div>
            </div>
            <div className="grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
              {PERMISSION_GROUPS.map((group) => (
                <div key={group.label} className="rounded-lg border border-neutral-200 bg-neutral-50/70 p-4">
                  <div className="text-sm font-semibold text-neutral-900">{group.label}</div>
                  <div className="text-xs text-neutral-500">{group.description}</div>
                  <div className="mt-3 grid gap-2">
                    {group.permissions.map((permission) => {
                      const checked = permissionKeys.has(permission.key);
                      return (
                        <label
                          key={permission.key}
                          className="flex cursor-pointer items-center gap-2 rounded-md border border-transparent px-2 py-1 text-sm text-neutral-700 transition hover:border-neutral-200 hover:bg-white"
                        >
                          <Checkbox
                            checked={checked}
                            onCheckedChange={() => togglePermission(permission.key)}
                          />
                          <span>{permission.label}</span>
                        </label>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>
          <div className="md:col-span-2">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Role"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
