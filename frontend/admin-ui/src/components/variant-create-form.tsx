"use client";

import { useState } from "react";
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
import { formatConnectError } from "@/lib/handle-error";

export default function VariantCreateForm() {
  const [productId, setProductId] = useState("");
  const [sku, setSku] = useState("");
  const [fulfillmentType, setFulfillmentType] = useState("physical");
  const [priceAmount, setPriceAmount] = useState("0");
  const [compareAtAmount, setCompareAtAmount] = useState("");
  const [status, setStatus] = useState("active");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;
  const { push } = useToast();

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
      const data = await createVariant({
        productId,
        sku,
        fulfillmentType,
        priceAmount: price,
        compareAtAmount: compareAt,
        currency: "JPY",
        status,
      });
      push({
        variant: "success",
        title: "Variant created",
        description: `Created variant: ${data.variant.id}`,
      });
      setProductId("");
      setSku("");
      setFulfillmentType("physical");
      setPriceAmount("0");
      setCompareAtAmount("");
      setStatus("active");
    } catch (err) {
      const uiError = formatConnectError(err, "Create failed", "Unknown error");
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
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Variant</CardTitle>
        <CardDescription className="text-neutral-500">
          Register SKU and pricing.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="variantProductId">Product ID</Label>
            <Input
              id="variantProductId"
              value={productId}
              onChange={(e) => setProductId(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="variantSku">SKU</Label>
            <Input id="variantSku" value={sku} onChange={(e) => setSku(e.target.value)} required />
          </div>
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
