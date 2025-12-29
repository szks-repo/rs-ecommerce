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
import { updateProduct } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { listTaxRules } from "@/lib/store_settings";
import type { ProductAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import type { TaxRule } from "@/gen/ecommerce/v1/store_settings_pb";

type ProductUpdateFormProps = {
  product?: ProductAdmin | null;
  onUpdated?: () => void;
};

export default function ProductUpdateForm({ product, onUpdated }: ProductUpdateFormProps) {
  const [productId, setProductId] = useState(product?.id ?? "");
  const [title, setTitle] = useState(product?.title ?? "");
  const [description, setDescription] = useState(product?.description ?? "");
  const [status, setStatus] = useState(product?.status ?? "active");
  const [taxRuleId, setTaxRuleId] = useState(product?.taxRuleId ?? "__default__");
  const [taxRules, setTaxRules] = useState<TaxRule[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive", "draft"] as const;
  const canSubmit = productId.trim().length > 0;

  useEffect(() => {
    if (!product) {
      return;
    }
    setProductId(product.id);
    setTitle(product.title);
    setDescription(product.description);
    setStatus(product.status || "active");
    setTaxRuleId(product.taxRuleId || "__default__");
  }, [product]);

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
      const data = await updateProduct({
        productId,
        title,
        description,
        status,
        taxRuleId: taxRuleId === "__default__" ? undefined : taxRuleId || undefined,
      });
      setMessage(`Updated product: ${data.product.id}`);
      onUpdated?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Update Product</CardTitle>
        <CardDescription className="text-neutral-500">
          Update title, description, or status.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Update failed</AlertTitle>
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
          {product ? (
            <div className="rounded-md border border-neutral-200 bg-neutral-50 px-3 py-2 text-sm text-neutral-700">
              Product ID: {productId}
            </div>
          ) : (
            <div className="text-sm text-neutral-500">
              Select a product from the details page to edit.
            </div>
          )}
          <div className="space-y-2">
            <Label htmlFor="updateProductTitle">Title</Label>
            <Input
              id="updateProductTitle"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateProductDescription">Description</Label>
            <Textarea
              id="updateProductDescription"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateProductStatus">Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger id="updateProductStatus" className="bg-white">
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
            <Label htmlFor="updateProductTaxRule">Tax Rule</Label>
            <Select value={taxRuleId} onValueChange={setTaxRuleId}>
              <SelectTrigger id="updateProductTaxRule" className="bg-white">
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
            <Button type="submit" disabled={isSubmitting || !canSubmit}>
              {isSubmitting ? "Updating..." : "Update Product"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
