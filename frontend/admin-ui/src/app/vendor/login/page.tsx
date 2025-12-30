"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identitySignIn } from "@/lib/identity";
import { formatConnectError } from "@/lib/handle-error";

export default function VendorLoginPage() {
  const router = useRouter();
  const [storeCode, setStoreCode] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  useEffect(() => {
    const saved = sessionStorage.getItem("store_code");
    if (saved) {
      setStoreCode(saved);
    }
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      const data = await identitySignIn({
        storeCode,
        email,
        password,
      });
      sessionStorage.setItem("store_code", storeCode);
      router.push("/vendor");
    } catch (err) {
      const uiError = formatConnectError(err, "Sign in failed", "Unknown error");
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
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto flex max-w-lg flex-col gap-6 px-6 py-16">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Vendor Login</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Manage your store inside the mall.
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Sign in</CardTitle>
            <CardDescription>Enter vendor credentials.</CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="storeCode">Store Code</Label>
                <Input
                  id="storeCode"
                  placeholder="example-store"
                  autoComplete="organization"
                  value={storeCode}
                  onChange={(e) => setStoreCode(e.target.value)}
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
