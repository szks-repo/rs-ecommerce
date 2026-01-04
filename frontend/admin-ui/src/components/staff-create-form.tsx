"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useApiCall } from "@/lib/use-api-call";
import { identityCreateStaff, identityListRoles } from "@/lib/identity";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export default function StaffCreateForm() {
  const [email, setEmail] = useState("");
  const [loginId, setLoginId] = useState("");
  const [phone, setPhone] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [roles, setRoles] = useState<Array<{ id: string; key: string; name: string }>>([]);
  const [roleId, setRoleId] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoadingRoles, setIsLoadingRoles] = useState(false);
  const { call } = useApiCall();

  useEffect(() => {
    let cancelled = false;
    setIsLoadingRoles(true);
    call(() => identityListRoles(), {
      errorTitle: "Load failed",
      errorDescription: "Failed to load roles",
    })
      .then((data) => {
        if (cancelled || !data) return;
        const list = data.roles ?? [];
        setRoles(list);
        if (!roleId && list.length > 0) {
          setRoleId(list[0].id || "");
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
    if (!roleId) {
      await call(
        async () => {
          throw new Error("role is required");
        },
        {
          errorTitle: "Create failed",
          errorDescription: "Role is required",
        }
      );
      setIsSubmitting(false);
      return;
    }
    const data = await call(
      () =>
        identityCreateStaff({
          email,
          loginId,
          phone,
          password,
          roleId,
          displayName,
        }),
      {
        success: {
          title: "Staff created",
          description: "Created staff successfully.",
        },
        errorTitle: "Create failed",
        errorDescription: "Failed to create staff",
      }
    );
    if (data) {
      setEmail("");
      setLoginId("");
      setPhone("");
      setPassword("");
      setRoleId(roles[0]?.id ?? "");
      setDisplayName("");
    }
    setIsSubmitting(false);
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Staff</CardTitle>
        <CardDescription className="text-neutral-500">
          Add staff with email/login_id/phone.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4 md:grid-cols-2" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input id="email" value={email} onChange={(e) => setEmail(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="loginId">Login ID</Label>
            <Input id="loginId" value={loginId} onChange={(e) => setLoginId(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="phone">Phone</Label>
            <Input id="phone" value={phone} onChange={(e) => setPhone(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="displayName">Display Name</Label>
            <Input id="displayName" value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">Password</Label>
            <Input id="password" type="password" value={password} onChange={(e) => setPassword(e.target.value)} required />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="role">Role</Label>
            <Select value={roleId} onValueChange={setRoleId}>
              <SelectTrigger id="role" className="bg-white">
                <SelectValue placeholder={isLoadingRoles ? "Loading roles..." : "Select role"} />
              </SelectTrigger>
              <SelectContent>
                {roles.length === 0 && !isLoadingRoles ? (
                  <SelectItem value="__none__" disabled>
                    No roles found
                  </SelectItem>
                ) : (
                  roles.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.name} {item.key ? `(${item.key})` : ""}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
          <div className="md:col-span-2">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Staff"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
