"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { initializeStore } from "@/lib/setup";

export default function InitPage() {
  const router = useRouter();
  const [storeName, setStoreName] = useState("");
  const [ownerEmail, setOwnerEmail] = useState("");
  const [ownerPassword, setOwnerPassword] = useState("");
  const [ownerLoginId, setOwnerLoginId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);
    try {
      const data = await initializeStore({
        storeName,
        ownerEmail,
        ownerPassword,
        ownerLoginId: ownerLoginId || undefined,
      });
      sessionStorage.setItem("store_id", data.storeId);
      sessionStorage.setItem("tenant_id", data.tenantId);
      router.push("/login");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto max-w-3xl px-6 py-12">
        <div className="mb-8">
          <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Init Wizard</h1>
          <p className="mt-2 text-sm text-neutral-600">
            One-time setup to create tenant, store, and owner account. Settings can be configured later.
          </p>
        </div>

        {error && (
          <Alert>
            <AlertTitle>Init failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <form className="space-y-6" onSubmit={handleSubmit}>
          <Card>
            <CardHeader>
              <CardTitle>Store Basics</CardTitle>
              <CardDescription>Required information to create the store.</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <Label htmlFor="storeName">Store Name</Label>
                <Input
                  id="storeName"
                  placeholder="Example Store"
                  value={storeName}
                  onChange={(e) => setStoreName(e.target.value)}
                  required
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Owner Account</CardTitle>
              <CardDescription>Owner credentials for initial login.</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="ownerEmail">Owner Email</Label>
                <Input
                  id="ownerEmail"
                  type="email"
                  placeholder="owner@example.com"
                  value={ownerEmail}
                  onChange={(e) => setOwnerEmail(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="ownerPassword">Owner Password</Label>
                <Input
                  id="ownerPassword"
                  type="password"
                  placeholder="At least 8 characters"
                  value={ownerPassword}
                  onChange={(e) => setOwnerPassword(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="ownerLoginId">Owner Login ID (optional)</Label>
                <Input
                  id="ownerLoginId"
                  placeholder="owner-001"
                  value={ownerLoginId}
                  onChange={(e) => setOwnerLoginId(e.target.value)}
                />
              </div>
            </CardContent>
          </Card>

          <div className="flex items-center justify-end gap-3">
            <Button variant="outline" type="button" disabled>
              Save Draft
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Initializing..." : "Initialize Store"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
