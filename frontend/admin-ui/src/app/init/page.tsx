"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { initializeStore, validateStoreCode } from "@/lib/setup";
import { formatConnectError } from "@/lib/handle-error";

export default function InitPage() {
  const router = useRouter();
  const [storeName, setStoreName] = useState("");
  const [storeCode, setStoreCode] = useState("");
  const [ownerEmail, setOwnerEmail] = useState("");
  const [ownerPassword, setOwnerPassword] = useState("");
  const [codeStatus, setCodeStatus] = useState<"idle" | "checking" | "available" | "unavailable">("idle");
  const [codeMessage, setCodeMessage] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  useEffect(() => {
    if (!storeCode.trim()) {
      setCodeStatus("idle");
      setCodeMessage("");
      return;
    }
    setCodeStatus("checking");
    const handle = setTimeout(() => {
      validateStoreCode({ storeCode: storeCode.trim() })
        .then((resp) => {
          if (resp.available) {
            setCodeStatus("available");
            setCodeMessage(resp.message || "store_code is available");
          } else {
            setCodeStatus("unavailable");
            setCodeMessage(resp.message || "store_code is not available");
          }
        })
        .catch((err) => {
          const uiError = formatConnectError(err, "Validation failed", "Failed to validate store_code");
          setCodeStatus("unavailable");
          setCodeMessage(uiError.description);
        });
    }, 400);
    return () => clearTimeout(handle);
  }, [storeCode]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!storeCode.trim()) {
        throw new Error("store_code is required");
      }
      if (codeStatus !== "available") {
        const resp = await validateStoreCode({ storeCode: storeCode.trim() });
        if (!resp.available) {
          setCodeStatus("unavailable");
          setCodeMessage(resp.message || "store_code is not available");
          throw new Error(resp.message || "store_code is not available");
        }
        setCodeStatus("available");
        setCodeMessage(resp.message || "store_code is available");
      }
      const data = await initializeStore({
        storeName,
        storeCode,
        ownerEmail,
        ownerPassword,
      });
      sessionStorage.setItem("store_id", data.storeId);
      sessionStorage.setItem("tenant_id", data.tenantId);
      sessionStorage.setItem("store_code", data.storeCode);
      router.push("/login");
    } catch (err) {
      const uiError = formatConnectError(err, "Init failed", "Unknown error");
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
      <div className="mx-auto max-w-3xl px-6 py-12">
        <div className="mb-8">
          <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Init Wizard</h1>
          <p className="mt-2 text-sm text-neutral-600">
            One-time setup to create tenant, store, and owner account. Settings can be configured later.
          </p>
        </div>

        <form className="space-y-6" onSubmit={handleSubmit}>
          <Card>
            <CardHeader>
              <CardTitle>Store Basics</CardTitle>
              <CardDescription>Required information to create the store.</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
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
                <div className="space-y-2">
                  <Label htmlFor="storeCode">Store Code</Label>
                  <Input
                    id="storeCode"
                    placeholder="example-store"
                    value={storeCode}
                    onChange={(e) => setStoreCode(e.target.value)}
                    required
                  />
                  {codeStatus === "checking" ? (
                    <p className="text-xs text-neutral-500">Checking availability...</p>
                  ) : null}
                  {codeStatus === "available" ? (
                    <p className="text-xs text-emerald-600">{codeMessage || "Available"}</p>
                  ) : null}
                  {codeStatus === "unavailable" ? (
                    <p className="text-xs text-red-600">{codeMessage || "Not available"}</p>
                  ) : null}
                </div>
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
