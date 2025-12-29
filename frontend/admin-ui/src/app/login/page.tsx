"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { identitySignIn } from "@/lib/identity";

export default function LoginPage() {
  const router = useRouter();
  const [storeCode, setStoreCode] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    const saved = sessionStorage.getItem("store_code");
    if (saved) {
      setStoreCode(saved);
    }
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
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
      setError(err instanceof Error ? err.message : "Unknown error");
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
                {error && (
                  <Alert>
                    <AlertTitle>Sign in failed</AlertTitle>
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                )}
                <LoginForm
                  roleLabel="Admin"
                  storeCode={storeCode}
                  email={email}
                  password={password}
                  isSubmitting={isSubmitting}
                  onStoreCodeChange={setStoreCode}
                  onEmailChange={setEmail}
                  onPasswordChange={setPassword}
                  onSubmit={handleSubmit}
                />
              </TabsContent>
              <TabsContent value="staff" className="mt-6 space-y-4">
                {error && (
                  <Alert>
                    <AlertTitle>Sign in failed</AlertTitle>
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                )}
                <LoginForm
                  roleLabel="Staff"
                  storeCode={storeCode}
                  email={email}
                  password={password}
                  isSubmitting={isSubmitting}
                  onStoreCodeChange={setStoreCode}
                  onEmailChange={setEmail}
                  onPasswordChange={setPassword}
                  onSubmit={handleSubmit}
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
