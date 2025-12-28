"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { identitySignIn } from "@/lib/identity";

export default function VendorLoginPage() {
  const router = useRouter();
  const [storeId, setStoreId] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);
    try {
      const data = await identitySignIn({
        storeId,
        email,
        password,
      });
      sessionStorage.setItem("access_token", data.accessToken);
      sessionStorage.setItem("store_id", data.storeId);
      sessionStorage.setItem("tenant_id", data.tenantId);
      router.push("/vendor");
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
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Vendor Login</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Manage your store inside the mall.
          </p>
        </div>

        {error && (
          <Alert>
            <AlertTitle>Sign in failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <Card>
          <CardHeader>
            <CardTitle>Sign in</CardTitle>
            <CardDescription>Enter vendor credentials.</CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="storeId">Store ID</Label>
                <Input
                  id="storeId"
                  placeholder="uuid (from Init response)"
                  autoComplete="organization"
                  value={storeId}
                  onChange={(e) => setStoreId(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="vendor@example.com"
                  autoComplete="username"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="Vendor password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                />
              </div>
              <Button className="w-full" type="submit" disabled={isSubmitting}>
                {isSubmitting ? "Signing in..." : "Sign in as Vendor"}
              </Button>
            </form>
            <div className="mt-6 text-sm text-neutral-500">
              Admin/Staff login? <a className="font-medium text-neutral-900" href="/login">Go here</a>.
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
  useEffect(() => {
    const saved = sessionStorage.getItem("store_id");
    if (saved) {
      setStoreId(saved);
    }
  }, []);
