"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { identityCreateRole } from "@/lib/identity";

const DEFAULT_PERMISSIONS = [
  "catalog.read",
  "catalog.write",
  "orders.read",
  "orders.write",
  "promotions.read",
  "promotions.write",
  "settings.read",
  "settings.write",
  "staff.manage",
];

export default function RoleCreateForm() {
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [permissionKeys, setPermissionKeys] = useState(DEFAULT_PERMISSIONS.join(", "));
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      const storeId = sessionStorage.getItem("store_id");
      if (!storeId) {
        throw new Error("store_id is missing. Please sign in first.");
      }
      const keys = permissionKeys
        .split(",")
        .map((v) => v.trim())
        .filter((v) => v.length > 0);
      const data = await identityCreateRole({
        storeId,
        key,
        name,
        description,
        permissionKeys: keys,
      });
      setMessage(`Created role: ${data.role?.id ?? ""}`);
      setKey("");
      setName("");
      setDescription("");
      setPermissionKeys(DEFAULT_PERMISSIONS.join(", "));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
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
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Create failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {message && (
          <Alert className="mb-4">
            <AlertTitle>Success</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        )}
        <form className="grid gap-4 md:grid-cols-2" onSubmit={handleSubmit}>
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
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="permissionKeys">Permission Keys (comma-separated)</Label>
            <Input
              id="permissionKeys"
              value={permissionKeys}
              onChange={(e) => setPermissionKeys(e.target.value)}
            />
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
