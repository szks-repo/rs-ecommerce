"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
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
  const [password, setPassword] = useState("");
  const [roles, setRoles] = useState<Array<{ id: string; key: string; name: string }>>([]);
  const [role, setRole] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isLoadingRoles, setIsLoadingRoles] = useState(false);
  const { push } = useToast();

  useEffect(() => {
    let cancelled = false;
    setIsLoadingRoles(true);
    identityListRoles()
      .then((data) => {
        if (cancelled) return;
        const list = data.roles ?? [];
        setRoles(list);
        if (!role && list.length > 0) {
          setRole(list[0].key || "");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          push({
            variant: "error",
            title: "Load failed",
            description: err instanceof Error ? err.message : "Failed to load roles",
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
      if (!role) {
        throw new Error("role is required");
      }
      const data = await identityCreateStaff({
        email,
        loginId,
        phone,
        password,
        role,
      });
      push({
        variant: "success",
        title: "Staff created",
        description: `Created staff: ${data.staffId}`,
      });
      setEmail("");
      setLoginId("");
      setPhone("");
      setPassword("");
      setRole(roles[0]?.key ?? "");
    } catch (err) {
      push({
        variant: "error",
        title: "Create failed",
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      setIsSubmitting(false);
    }
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
            <Label htmlFor="password">Password</Label>
            <Input id="password" type="password" value={password} onChange={(e) => setPassword(e.target.value)} required />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="role">Role</Label>
            <Select value={role} onValueChange={setRole}>
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
                    <SelectItem key={item.id} value={item.key}>
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
