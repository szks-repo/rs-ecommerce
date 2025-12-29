"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { updateVariant } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";

export default function VariantUpdateForm() {
  const [variantId, setVariantId] = useState("");
  const [priceAmount, setPriceAmount] = useState("0");
  const [compareAtAmount, setCompareAtAmount] = useState("");
  const [status, setStatus] = useState("active");
  const [fulfillmentType, setFulfillmentType] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
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
      setMessage(`Updated variant: ${data.variant.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
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
          <div className="space-y-2">
            <Label htmlFor="updateVariantId">Variant ID</Label>
            <Input
              id="updateVariantId"
              value={variantId}
              onChange={(e) => setVariantId(e.target.value)}
              required
            />
          </div>
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
            <Input
              id="updateVariantStatus"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateVariantFulfillment">Fulfillment Type (optional)</Label>
            <Input
              id="updateVariantFulfillment"
              value={fulfillmentType}
              onChange={(e) => setFulfillmentType(e.target.value)}
              placeholder="physical or digital"
            />
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Updating..." : "Update Variant"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
