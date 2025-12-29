"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { rpcFetch } from "@/lib/api";
import { identityListRoles } from "@/lib/identity";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";

export default function RoleAssignForm() {
  const [staffId, setStaffId] = useState("");
  const [roles, setRoles] = useState<Array<{ id: string; key: string; name: string }>>([]);
  const [roleId, setRoleId] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoadingRoles, setIsLoadingRoles] = useState(false);

  useEffect(() => {
    const storeId = sessionStorage.getItem("store_id");
    if (!storeId) {
      return;
    }
    let cancelled = false;
    setIsLoadingRoles(true);
    identityListRoles({ storeId })
      .then((data) => {
        if (!cancelled) {
          setRoles(data.roles ?? []);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load roles");
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
      if (!roleId) {
        throw new Error("role_id is missing. Please select a role.");
      }
      const data = await rpcFetch<{ assigned: boolean }>(
        "/rpc/ecommerce.v1.IdentityService/AssignRoleToStaff",
        {
          store: { storeId },
          staffId,
          roleId,
        }
      );
      setMessage(data.assigned ? "Role assigned" : "Role not assigned");
      setStaffId("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Assign Role</CardTitle>
        <CardDescription className="text-neutral-500">
          Attach a role to a staff member.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Assign failed</AlertTitle>
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
            <Label htmlFor="staffId">Staff ID</Label>
            <Input id="staffId" value={staffId} onChange={(e) => setStaffId(e.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="roleId">Role ID</Label>
            <Select value={roleId} onValueChange={setRoleId}>
            <SelectTrigger id="roleId" className="bg-white">
              <SelectValue placeholder={isLoadingRoles ? "Loading roles..." : "Select role"} />
            </SelectTrigger>
              <SelectContent>
                {roles.length === 0 && !isLoadingRoles ? (
                  <SelectItem value="none" disabled>
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
          <div className="md:col-span-2">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Assigning..." : "Assign Role"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
