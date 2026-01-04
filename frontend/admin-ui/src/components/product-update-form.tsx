"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
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
import { useApiCall } from "@/lib/use-api-call";
import { dateInputToTimestamp, timestampToDateInput } from "@/lib/time";
import { Switch } from "@/components/ui/switch";

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
  const [saleStartDate, setSaleStartDate] = useState(
    product?.saleStartAt ? timestampToDateInput(product.saleStartAt) : ""
  );
  const [saleEndDate, setSaleEndDate] = useState(
    product?.saleEndAt ? timestampToDateInput(product.saleEndAt) : ""
  );
  const [applyTaxRuleToSkus, setApplyTaxRuleToSkus] = useState(false);
  const [taxRules, setTaxRules] = useState<TaxRule[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive", "draft"] as const;
  const canSubmit = productId.trim().length > 0;
  const { push } = useToast();
  const { notifyError } = useApiCall();

  useEffect(() => {
    if (!product) {
      return;
    }
    setProductId(product.id);
    setTitle(product.title);
    setDescription(product.description);
    setStatus(product.status || "active");
    setTaxRuleId(product.taxRuleId || "__default__");
    setSaleStartDate(product.saleStartAt ? timestampToDateInput(product.saleStartAt) : "");
    setSaleEndDate(product.saleEndAt ? timestampToDateInput(product.saleEndAt) : "");
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
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const saleStartAt = dateInputToTimestamp(saleStartDate, false);
      const saleEndAt = dateInputToTimestamp(saleEndDate, true);
      const hasSaleStart = Boolean(saleStartAt);
      const hasSaleEnd = Boolean(saleEndAt);
      if (hasSaleStart !== hasSaleEnd) {
        throw new Error("sale_start_at and sale_end_at must both be set or both be empty.");
      }
      if (saleStartAt && saleEndAt && saleStartAt.seconds > saleEndAt.seconds) {
        throw new Error("sale_end_at must be later than sale_start_at.");
      }
      const data = await updateProduct({
        productId,
        title,
        description,
        status,
        taxRuleId: taxRuleId === "__default__" ? undefined : taxRuleId || undefined,
        saleStartAt,
        saleEndAt,
        applyTaxRuleToVariants: applyTaxRuleToSkus,
      });
      push({
        variant: "success",
        title: "Product updated",
        description: `Updated product: ${data.product.id}`,
      });
      setApplyTaxRuleToSkus(false);
      onUpdated?.();
    } catch (err) {
      notifyError(err, "Update failed", "Unknown error");
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
            <div className="flex items-center justify-between rounded-md border border-neutral-200 bg-neutral-50 px-3 py-2 text-xs text-neutral-600">
              <div>
                Apply this tax rule to all SKUs
              </div>
              <Switch
                checked={applyTaxRuleToSkus}
                onCheckedChange={setApplyTaxRuleToSkus}
              />
            </div>
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="updateProductSaleStart">Sale start date (optional)</Label>
              <Input
                id="updateProductSaleStart"
                type="date"
                value={saleStartDate}
                onChange={(e) => setSaleStartDate(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="updateProductSaleEnd">Sale end date (optional)</Label>
              <Input
                id="updateProductSaleEnd"
                type="date"
                value={saleEndDate}
                onChange={(e) => setSaleEndDate(e.target.value)}
              />
            </div>
            <div className="text-xs text-neutral-500 md:col-span-2">
              Both dates must be set together. Leave both empty to keep it always purchasable.
            </div>
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
