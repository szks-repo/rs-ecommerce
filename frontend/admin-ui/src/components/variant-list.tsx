"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useToast } from "@/components/ui/toast";
import { Button } from "@/components/ui/button";
import { listVariantsAdmin } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import type { VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";

export default function VariantList() {
  const [productId, setProductId] = useState("");
  const [searchProductId, setSearchProductId] = useState("");
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<VariantAdmin[]>(
    async () => {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      if (!searchProductId) {
        return [];
      }
      const data = await listVariantsAdmin({ productId: searchProductId });
      return data.variants ?? [];
    },
    [searchProductId]
  );

  useEffect(() => {
    const saved = sessionStorage.getItem("last_product_id");
    if (saved) {
      setProductId(saved);
    }
  }, []);

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load variants");
    }
  }, [error, notifyError]);

  const variants = data ?? [];

  async function loadVariants() {
    if (!getActiveAccessToken()) {
      push({
        variant: "error",
        title: "Load failed",
        description: "access_token is missing. Please sign in first.",
      });
      return;
    }
    if (!productId) {
      push({
        variant: "error",
        title: "Load failed",
        description: "product_id is required.",
      });
      return;
    }
    try {
      const shouldReload = productId === searchProductId;
      setSearchProductId(productId);
      if (shouldReload) {
        await reload();
      }
      sessionStorage.setItem("last_product_id", productId);
    } catch (err) {
      notifyError(err, "Load failed", "Failed to load variants");
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
        <Button variant="outline" onClick={loadVariants} disabled={loading}>
          {loading ? "Loading..." : "Refresh"}
        </Button>
      </CardHeader>
      <CardContent className="space-y-4">
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
