"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useToast } from "@/components/ui/toast";
import { consumeAuthFlashMessage, getActiveAccessToken } from "@/lib/auth";
import { identitySignIn } from "@/lib/identity";
import { useApiCall } from "@/lib/use-api-call";

export default function LoginPage() {
  const router = useRouter();
  const [storeCode, setStoreCode] = useState("");
  const [email, setEmail] = useState("");
  const [staffIdentifier, setStaffIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  useEffect(() => {
    if (getActiveAccessToken()) {
      router.replace("/admin");
      return;
    }
    const saved = sessionStorage.getItem("store_code");
    if (saved) {
      setStoreCode(saved);
    }
    const flash = consumeAuthFlashMessage();
    if (flash) {
      push(flash);
    }
  }, [push]);

  async function handleAdminSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      const data = await identitySignIn({
        storeCode,
        email,
        password,
      });
      sessionStorage.setItem("store_code", storeCode);
      router.push("/admin");
    } catch (err) {
      notifyError(err, "Sign in failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleStaffSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      const trimmed = staffIdentifier.trim();
      const isEmail = trimmed.includes("@");
      const data = await identitySignIn({
        storeCode,
        email: isEmail ? trimmed : undefined,
        loginId: !isEmail ? trimmed : undefined,
        password,
      });
      sessionStorage.setItem("store_code", storeCode);
      router.push("/admin");
    } catch (err) {
      notifyError(err, "Sign in failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto flex max-w-lg flex-col gap-6 px-6 py-16">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Admin / Staff Login</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Sign in to manage products, orders, promotions, and store settings.
          </p>
        </div>

        <Alert>
          <AlertTitle>Security Notice</AlertTitle>
          <AlertDescription>
            Access tokens are kept in memory. Avoid sharing credentials and use trusted devices only.
          </AlertDescription>
        </Alert>

        <Card>
          <CardHeader>
            <CardTitle>Sign in</CardTitle>
            <CardDescription>Choose role and provide credentials.</CardDescription>
          </CardHeader>
          <CardContent>
            <Tabs defaultValue="admin">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="admin">Admin</TabsTrigger>
                <TabsTrigger value="staff">Staff</TabsTrigger>
              </TabsList>
              <TabsContent value="admin" className="mt-6 space-y-4">
                <LoginForm
                  roleLabel="Admin"
                  storeCode={storeCode}
                  email={email}
                  password={password}
                  isSubmitting={isSubmitting}
                  onStoreCodeChange={setStoreCode}
                  onEmailChange={setEmail}
                  onPasswordChange={setPassword}
                  onSubmit={handleAdminSubmit}
                />
              </TabsContent>
              <TabsContent value="staff" className="mt-6 space-y-4">
                <StaffLoginForm
                  storeCode={storeCode}
                  staffIdentifier={staffIdentifier}
                  password={password}
                  isSubmitting={isSubmitting}
                  onStoreCodeChange={setStoreCode}
                  onStaffIdentifierChange={setStaffIdentifier}
                  onPasswordChange={setPassword}
                  onSubmit={handleStaffSubmit}
                />
              </TabsContent>
            </Tabs>
            <div className="mt-6 text-sm text-neutral-500">
              Vendor login? <a className="font-medium text-neutral-900" href="/vendor/login">Go here</a>.
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function LoginForm({
  roleLabel,
  storeCode,
  email,
  password,
  isSubmitting,
  onStoreCodeChange,
  onEmailChange,
  onPasswordChange,
  onSubmit,
}: {
  roleLabel: string;
  storeCode: string;
  email: string;
  password: string;
  isSubmitting: boolean;
  onStoreCodeChange: (value: string) => void;
  onEmailChange: (value: string) => void;
  onPasswordChange: (value: string) => void;
  onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <form className="space-y-4" onSubmit={onSubmit}>
      <div className="space-y-2">
        <Label htmlFor="storeCode">Store Code</Label>
        <Input
          id="storeCode"
          placeholder="example-store"
          autoComplete="organization"
          value={storeCode}
          onChange={(e) => onStoreCodeChange(e.target.value)}
          required
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="email">Email</Label>
        <Input
          id="email"
          type="email"
          placeholder="admin@example.com"
          autoComplete="username"
          value={email}
          onChange={(e) => onEmailChange(e.target.value)}
          required
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="password">Password</Label>
        <Input
          id="password"
          type="password"
          placeholder={`${roleLabel} password`}
          autoComplete="current-password"
          value={password}
          onChange={(e) => onPasswordChange(e.target.value)}
          required
        />
      </div>
      <Button className="w-full" type="submit" disabled={isSubmitting}>
        {isSubmitting ? "Signing in..." : `Sign in as ${roleLabel}`}
      </Button>
    </form>
  );
}

function StaffLoginForm({
  storeCode,
  staffIdentifier,
  password,
  isSubmitting,
  onStoreCodeChange,
  onStaffIdentifierChange,
  onPasswordChange,
  onSubmit,
}: {
  storeCode: string;
  staffIdentifier: string;
  password: string;
  isSubmitting: boolean;
  onStoreCodeChange: (value: string) => void;
  onStaffIdentifierChange: (value: string) => void;
  onPasswordChange: (value: string) => void;
  onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <form className="space-y-4" onSubmit={onSubmit}>
      <div className="space-y-2">
        <Label htmlFor="storeCodeStaff">Store Code</Label>
        <Input
          id="storeCodeStaff"
          placeholder="example-store"
          autoComplete="organization"
          value={storeCode}
          onChange={(e) => onStoreCodeChange(e.target.value)}
          required
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="staffIdentifier">Email or Staff ID</Label>
        <Input
          id="staffIdentifier"
          placeholder="staff@example.com or STAFF-001"
          autoComplete="username"
          value={staffIdentifier}
          onChange={(e) => onStaffIdentifierChange(e.target.value)}
          required
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="passwordStaff">Password</Label>
        <Input
          id="passwordStaff"
          type="password"
          placeholder="Staff password"
          autoComplete="current-password"
          value={password}
          onChange={(e) => onPasswordChange(e.target.value)}
          required
        />
      </div>
      <Button className="w-full" type="submit" disabled={isSubmitting}>
        {isSubmitting ? "Signing in..." : "Sign in as Staff"}
      </Button>
    </form>
  );
}
