"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { listProductsAdmin } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";

type ProductAdmin = {
  id: string;
  storeId: string;
  vendorId?: string;
  title: string;
  description: string;
  status: string;
};

export default function ProductList() {
  const [products, setProducts] = useState<ProductAdmin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  async function loadProducts() {
    if (!getActiveAccessToken()) {
      setError("access_token is missing. Please sign in first.");
      return;
    }
    setError(null);
    setIsLoading(true);
    try {
      const data = await listProductsAdmin();
      setProducts(data.products ?? []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load products");
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadProducts();
  }, []);

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader className="flex flex-row items-center justify-between gap-4">
        <div>
          <CardTitle>Products</CardTitle>
          <CardDescription className="text-neutral-500">
            Recently created products in this store.
          </CardDescription>
        </div>
        <Button variant="outline" onClick={loadProducts} disabled={isLoading}>
          {isLoading ? "Loading..." : "Refresh"}
        </Button>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Load failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {products.length === 0 ? (
          <div className="text-sm text-neutral-600">No products yet.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {products.map((product) => (
              <div key={product.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="font-medium text-neutral-900">{product.title}</div>
                <div className="text-xs text-neutral-500">
                  id: {product.id} / status: {product.status}
                </div>
                {product.description ? (
                  <div className="mt-2 text-sm text-neutral-600">
                    {product.description}
                  </div>
                ) : null}
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
