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
import { createVariant } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { useApiCall } from "@/lib/use-api-call";
import { validateSkuCode } from "@/lib/sku-code";

type VariantCreateFormProps = {
  productId?: string;
  variantAxes?: { name: string; position?: number }[];
  onCreated?: () => void;
};

export default function VariantCreateForm({
  productId: initialProductId,
  variantAxes = [],
  onCreated,
}: VariantCreateFormProps) {
  const [productId, setProductId] = useState(initialProductId ?? "");
  const [sku, setSku] = useState("");
  const [janCode, setJanCode] = useState("");
  const [fulfillmentType, setFulfillmentType] = useState("physical");
  const [priceAmount, setPriceAmount] = useState("0");
  const [compareAtAmount, setCompareAtAmount] = useState("");
  const [status, setStatus] = useState("active");
  const [axisValues, setAxisValues] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const axes = variantAxes
    .slice()
    .sort((a, b) => (a.position ?? 0) - (b.position ?? 0));

  useEffect(() => {
    if (initialProductId) {
      setProductId(initialProductId);
    }
  }, [initialProductId]);

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
      const skuError = validateSkuCode(sku.trim());
      if (skuError) {
        throw new Error(skuError);
      }
      const axisValuesPayload = axes.map((axis) => {
        const value = axisValues[axis.name]?.trim() ?? "";
        if (!value) {
          throw new Error(`${axis.name} is required.`);
        }
        return { name: axis.name, value };
      });
      const data = await createVariant({
        productId,
        sku,
        janCode: janCode.trim() || undefined,
        fulfillmentType,
        priceAmount: price,
        compareAtAmount: compareAt,
        currency: "JPY",
        status,
        axisValues: axisValuesPayload,
      });
      push({
        variant: "success",
        title: "Variant created",
        description: `Created variant: ${data.variant.id}`,
      });
      setProductId(initialProductId ?? "");
      setSku("");
      setJanCode("");
      setFulfillmentType("physical");
      setPriceAmount("0");
      setCompareAtAmount("");
      setStatus("active");
      setAxisValues({});
      onCreated?.();
    } catch (err) {
      notifyError(err, "Create failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Variant</CardTitle>
        <CardDescription className="text-neutral-500">
          Register SKU and pricing.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          {initialProductId ? null : (
            <div className="space-y-2">
              <Label htmlFor="variantProductId">Product ID</Label>
              <Input
                id="variantProductId"
                value={productId}
                onChange={(e) => setProductId(e.target.value)}
                required
              />
            </div>
          )}
          <div className="space-y-2">
            <Label htmlFor="variantSku">SKU</Label>
            <Input id="variantSku" value={sku} onChange={(e) => setSku(e.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="variantJanCode">JAN Code (optional)</Label>
            <Input
              id="variantJanCode"
              value={janCode}
              onChange={(e) => setJanCode(e.target.value)}
            />
          </div>
          {axes.length > 0 ? (
            <div className="grid gap-4 md:grid-cols-2">
              {axes.map((axis) => (
                <div key={axis.name} className="space-y-2">
                  <Label htmlFor={`variantAxis-${axis.name}`}>{axis.name}</Label>
                  <Input
                    id={`variantAxis-${axis.name}`}
                    value={axisValues[axis.name] ?? ""}
                    onChange={(e) =>
                      setAxisValues((prev) => ({ ...prev, [axis.name]: e.target.value }))
                    }
                    placeholder={`${axis.name} value`}
                    required
                  />
                </div>
              ))}
            </div>
          ) : null}
          <div className="space-y-2">
            <Label htmlFor="variantFulfillment">Fulfillment Type</Label>
            <Select value={fulfillmentType} onValueChange={setFulfillmentType}>
              <SelectTrigger id="variantFulfillment" className="bg-white">
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
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="variantPrice">Price Amount (JPY)</Label>
              <Input
                id="variantPrice"
                value={priceAmount}
                onChange={(e) => setPriceAmount(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="variantCompareAt">Compare-at Amount (JPY)</Label>
              <Input
                id="variantCompareAt"
                value={compareAtAmount}
                onChange={(e) => setCompareAtAmount(e.target.value)}
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="variantStatus">Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger id="variantStatus" className="bg-white">
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
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Variant"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
