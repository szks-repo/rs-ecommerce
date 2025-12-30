"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { useToast } from "@/components/ui/toast";
import { identityCreateRole } from "@/lib/identity";
import { DEFAULT_PERMISSION_KEYS, PERMISSION_GROUPS } from "@/lib/permissions";
import { formatConnectError } from "@/lib/handle-error";

export default function RoleCreateForm() {
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [permissionKeys, setPermissionKeys] = useState<Set<string>>(
    () => new Set(DEFAULT_PERMISSION_KEYS)
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  function handleSelectAll() {
    setPermissionKeys(new Set(DEFAULT_PERMISSION_KEYS));
  }

  function handleClearAll() {
    setPermissionKeys(new Set());
  }

  function togglePermission(permissionKey: string) {
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
    try {
      const keys = Array.from(permissionKeys);
      const data = await identityCreateRole({
        key,
        name,
        description,
        permissionKeys: keys,
      });
      push({
        variant: "success",
        title: "Role created",
        description: `Created role: ${data.role?.id ?? ""}`,
      });
      setKey("");
      setName("");
      setDescription("");
      setPermissionKeys(new Set(DEFAULT_PERMISSION_KEYS));
    } catch (err) {
      const uiError = formatConnectError(err, "Create failed", "Failed to create role");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsSubmitting(false);
    }
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
