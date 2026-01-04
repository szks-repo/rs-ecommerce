"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { updateVariant } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import type { VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { useApiCall } from "@/lib/use-api-call";

type VariantUpdateFormProps = {
  variant?: VariantAdmin | null;
  onUpdated?: () => void;
};

export default function VariantUpdateForm({ variant, onUpdated }: VariantUpdateFormProps) {
  const [variantId, setVariantId] = useState(variant?.id ?? "");
  const [priceAmount, setPriceAmount] = useState(
    variant?.price?.amount != null ? String(variant.price.amount) : "0"
  );
  const [compareAtAmount, setCompareAtAmount] = useState(
    variant?.compareAt?.amount != null ? String(variant.compareAt.amount) : ""
  );
  const [status, setStatus] = useState(variant?.status ?? "active");
  const [fulfillmentType, setFulfillmentType] = useState(variant?.fulfillmentType ?? "");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;
  const canSubmit = variantId.trim().length > 0;
  const { push } = useToast();
  const { notifyError } = useApiCall();

  useEffect(() => {
    if (!variant) {
      return;
    }
    setVariantId(variant.id);
    setPriceAmount(variant.price?.amount != null ? String(variant.price.amount) : "0");
    setCompareAtAmount(variant.compareAt?.amount != null ? String(variant.compareAt.amount) : "");
    setStatus(variant.status || "active");
    setFulfillmentType(variant.fulfillmentType || "");
  }, [variant]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const price = Number(priceAmount);
      if (!Number.isFinite(price)) {
        throw new Error("price_amount must be a number.");
      }
      const compareAt = compareAtAmount.trim().length > 0 ? Number(compareAtAmount) : undefined;
      if (typeof compareAt === "number" && !Number.isFinite(compareAt)) {
        throw new Error("compare_at_amount must be a number.");
      }
      const data = await updateVariant({
        variantId,
        priceAmount: price,
        compareAtAmount: compareAt,
        currency: "JPY",
        status,
        fulfillmentType: fulfillmentType || undefined,
      });
      push({
        variant: "success",
        title: "Variant updated",
        description: `Updated variant: ${data.variant.id}`,
      });
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
        <CardTitle>Update Variant</CardTitle>
        <CardDescription className="text-neutral-500">
          Update price, status, or fulfillment type.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          {variant ? (
            <div className="rounded-md border border-neutral-200 bg-neutral-50 px-3 py-2 text-sm text-neutral-700">
              Variant ID: {variantId}
            </div>
          ) : (
            <div className="text-sm text-neutral-500">
              Select a variant from the list to edit.
            </div>
          )}
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="updateVariantPrice">Price Amount (JPY)</Label>
              <Input
                id="updateVariantPrice"
                value={priceAmount}
                onChange={(e) => setPriceAmount(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="updateVariantCompareAt">Compare-at Amount (JPY)</Label>
              <Input
                id="updateVariantCompareAt"
                value={compareAtAmount}
                onChange={(e) => setCompareAtAmount(e.target.value)}
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateVariantStatus">Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger id="updateVariantStatus" className="bg-white">
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
            <Label htmlFor="updateVariantFulfillment">Fulfillment Type (optional)</Label>
            <Select value={fulfillmentType} onValueChange={setFulfillmentType}>
              <SelectTrigger id="updateVariantFulfillment" className="bg-white">
                <SelectValue placeholder="Select fulfillment type" />
              </SelectTrigger>
              <SelectContent>
                {fulfillmentOptions.map((option) => (
                  <SelectItem key={option} value={option}>
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting || !canSubmit}>
              {isSubmitting ? "Updating..." : "Update Variant"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
