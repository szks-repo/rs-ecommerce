"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identityAssignRole, identityListRoles } from "@/lib/identity";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { formatConnectError } from "@/lib/handle-error";

export default function RoleAssignForm() {
  const [staffId, setStaffId] = useState("");
  const [roles, setRoles] = useState<Array<{ id: string; key: string; name: string }>>([]);
  const [roleId, setRoleId] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoadingRoles, setIsLoadingRoles] = useState(false);
  const { push } = useToast();

  useEffect(() => {
    let cancelled = false;
    setIsLoadingRoles(true);
    identityListRoles()
      .then((data) => {
        if (!cancelled) {
          setRoles(data.roles ?? []);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          const uiError = formatConnectError(err, "Load failed", "Failed to load roles");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
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
    setIsSubmitting(true);
    try {
      if (!roleId) {
        throw new Error("role_id is missing. Please select a role.");
      }
      const data = await identityAssignRole({ staffId, roleId });
      push({
        variant: data.assigned ? "success" : "error",
        title: data.assigned ? "Role assigned" : "Assign failed",
        description: data.assigned ? "Role assigned." : "Role not assigned.",
      });
      setStaffId("");
    } catch (err) {
      const uiError = formatConnectError(err, "Assign failed", "Unknown error");
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
        <CardTitle>Assign Role</CardTitle>
        <CardDescription className="text-neutral-500">
          Attach a role to a staff member.
        </CardDescription>
      </CardHeader>
      <CardContent>
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
