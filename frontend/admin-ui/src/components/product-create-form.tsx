"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { createProduct } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { listTaxRules } from "@/lib/store_settings";
import type { TaxRule } from "@/gen/ecommerce/v1/store_settings_pb";

export default function ProductCreateForm() {
  const [title, setTitle] = useState("");
  const [vendorId, setVendorId] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("active");
  const [taxRuleId, setTaxRuleId] = useState("__default__");
  const [taxRules, setTaxRules] = useState<TaxRule[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive", "draft"] as const;

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    listTaxRules()
      .then((data) => {
        setTaxRules(data.rules ?? []);
      })
      .catch(() => {
        setTaxRules([]);
      });
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const data = await createProduct({
        vendorId: vendorId.trim() || undefined,
        title,
        description,
        status,
        taxRuleId: taxRuleId === "__default__" ? undefined : taxRuleId || undefined,
      });
      setMessage(`Created product: ${data.product.id}`);
      setTitle("");
      setVendorId("");
      setDescription("");
      setStatus("active");
      setTaxRuleId("__default__");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Product</CardTitle>
        <CardDescription className="text-neutral-500">
          Register product master data.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Create failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {message && (
          <Alert className="mb-4">
            <AlertTitle>Success</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        )}
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="productTitle">Title</Label>
            <Input
              id="productTitle"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productVendor">Vendor ID (optional)</Label>
            <Input
              id="productVendor"
              value={vendorId}
              onChange={(e) => setVendorId(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productDescription">Description</Label>
            <Textarea
              id="productDescription"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productStatus">Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger id="productStatus" className="bg-white">
                <SelectValue placeholder="Select status" />
              </SelectTrigger>
              <SelectContent>
                {statusOptions.map((option) => (
                  <SelectItem key={option} value={option}>
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="productTaxRule">Tax Rule</Label>
            <Select value={taxRuleId} onValueChange={setTaxRuleId}>
              <SelectTrigger id="productTaxRule" className="bg-white">
                <SelectValue placeholder="Default (store setting)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__default__">Default (store setting)</SelectItem>
                {taxRules.map((rule) => (
                  <SelectItem key={rule.id} value={rule.id}>
                    {rule.name} ({(rule.rate * 100).toFixed(1)}%)
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Product"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
