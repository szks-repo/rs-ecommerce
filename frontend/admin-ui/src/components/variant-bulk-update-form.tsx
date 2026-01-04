"use client";

import { useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { updateVariant } from "@/lib/product";
import type { VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { useApiCall } from "@/lib/use-api-call";
import { useToast } from "@/components/ui/toast";

type VariantBulkUpdateFormProps = {
  variants: VariantAdmin[];
  onUpdated?: () => void;
};

export default function VariantBulkUpdateForm({ variants, onUpdated }: VariantBulkUpdateFormProps) {
  const [status, setStatus] = useState("__keep__");
  const [fulfillmentType, setFulfillmentType] = useState("__keep__");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { notifyError } = useApiCall();
  const { push } = useToast();

  const hasVariants = variants.length > 0;
  const statusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;
  const canSubmit =
    hasVariants && (status !== "__keep__" || fulfillmentType !== "__keep__");

  const affectedCount = useMemo(() => variants.length, [variants.length]);

  async function handleApply() {
    if (!canSubmit) {
      return;
    }
    setIsSubmitting(true);
    try {
      await Promise.all(
        variants.map(async (variant) => {
          const priceAmount = Number(variant.price?.amount ?? 0);
          if (!Number.isFinite(priceAmount)) {
            throw new Error(`invalid price for variant ${variant.id}`);
          }
          const compareAtAmount =
            variant.compareAt?.amount != null
              ? Number(variant.compareAt.amount)
              : undefined;
          if (typeof compareAtAmount === "number" && !Number.isFinite(compareAtAmount)) {
            throw new Error(`invalid compare_at for variant ${variant.id}`);
          }
          const nextStatus =
            status === "__keep__" ? variant.status : status;
          const nextFulfillment =
            fulfillmentType === "__keep__" ? undefined : fulfillmentType;
          await updateVariant({
            variantId: variant.id,
            priceAmount,
            compareAtAmount,
            currency: variant.price?.currency || "JPY",
            status: nextStatus,
            fulfillmentType: nextFulfillment,
          });
        })
      );
      push({
        variant: "success",
        title: "Bulk update complete",
        description: `Updated ${affectedCount} SKU(s).`,
      });
      onUpdated?.();
    } catch (err) {
      notifyError(err, "Bulk update failed", "Failed to update SKUs");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Bulk Update SKUs</CardTitle>
        <CardDescription className="text-neutral-500">
          Apply status or fulfillment type to every SKU in this product.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {!hasVariants ? (
          <div className="text-sm text-neutral-600">No SKUs to update.</div>
        ) : (
          <>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="bulkVariantStatus">Status (optional)</Label>
                <Select value={status} onValueChange={setStatus}>
                  <SelectTrigger id="bulkVariantStatus" className="bg-white">
                    <SelectValue placeholder="Keep current status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__keep__">Keep current</SelectItem>
                    {statusOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="bulkVariantFulfillment">Fulfillment Type (optional)</Label>
                <Select value={fulfillmentType} onValueChange={setFulfillmentType}>
                  <SelectTrigger id="bulkVariantFulfillment" className="bg-white">
                    <SelectValue placeholder="Keep current fulfillment" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__keep__">Keep current</SelectItem>
                    {fulfillmentOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="flex items-center justify-between text-sm text-neutral-600">
              <div>Targets: {affectedCount} SKU(s)</div>
              <Button type="button" onClick={handleApply} disabled={!canSubmit || isSubmitting}>
                {isSubmitting ? "Updating..." : "Apply to all SKUs"}
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
