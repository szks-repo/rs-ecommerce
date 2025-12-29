"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { listVariantsAdmin } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import type { VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";

export default function VariantList() {
  const [productId, setProductId] = useState("");
  const [variants, setVariants] = useState<VariantAdmin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const saved = sessionStorage.getItem("last_product_id");
    if (saved) {
      setProductId(saved);
    }
  }, []);

  async function loadVariants() {
    if (!getActiveAccessToken()) {
      setError("access_token is missing. Please sign in first.");
      return;
    }
    if (!productId) {
      setError("product_id is required.");
      return;
    }
    setError(null);
    setIsLoading(true);
    try {
      const data = await listVariantsAdmin({ productId });
      setVariants(data.variants ?? []);
      sessionStorage.setItem("last_product_id", productId);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load variants");
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader className="flex flex-row items-center justify-between gap-4">
        <div>
          <CardTitle>Variants</CardTitle>
          <CardDescription className="text-neutral-500">
            List variants by product.
          </CardDescription>
        </div>
        <Button variant="outline" onClick={loadVariants} disabled={isLoading}>
          {isLoading ? "Loading..." : "Refresh"}
        </Button>
      </CardHeader>
      <CardContent className="space-y-4">
        {error && (
          <Alert>
            <AlertTitle>Load failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        <div className="space-y-2">
          <label className="text-sm text-neutral-600" htmlFor="variantProductId">
            Product ID
          </label>
          <input
            id="variantProductId"
            className="w-full rounded-md border border-neutral-200 px-3 py-2 text-sm"
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            placeholder="product_id"
          />
        </div>
        {variants.length === 0 ? (
          <div className="text-sm text-neutral-600">No variants yet.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {variants.map((variant) => (
              <div key={variant.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="font-medium text-neutral-900">{variant.sku}</div>
                <div className="text-xs text-neutral-500">
                  id: {variant.id} / status: {variant.status} / type:{" "}
                  {variant.fulfillmentType}
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
