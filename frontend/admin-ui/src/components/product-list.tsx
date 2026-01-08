"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Image as ImageIcon } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { listProductsAdmin } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { buildProductPreviewUrl } from "@/lib/storefront";
import type { ProductAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { AdminTableToolbar } from "@/components/admin-table";
import ProductInventoryQuickEdit from "@/components/product-inventory-quick-edit";

export default function ProductList() {
  const [query, setQuery] = useState("");
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<ProductAdmin[]>(
    async () => {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const data = await listProductsAdmin();
      return data.products ?? [];
    },
    []
  );

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load products");
    }
  }, [error, notifyError]);

  const products = data ?? [];

  const filtered = query.trim()
    ? products.filter((product) => {
        const haystack = [
          product.title,
          product.id,
          product.description ?? "",
        ]
          .join(" ")
          .toLowerCase();
        return haystack.includes(query.trim().toLowerCase());
      })
    : products;

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Products</CardTitle>
        <CardDescription className="text-neutral-500">
          Recently created products in this store.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <AdminTableToolbar
          left={`${filtered.length} products`}
          right={
            <>
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search by title, id, description"
                className="h-9 w-full min-w-[220px] max-w-[320px]"
              />
              <Button variant="outline" onClick={reload} disabled={loading} size="sm">
                {loading ? "Loading..." : "Refresh"}
              </Button>
            </>
          }
        />
        {filtered.length === 0 ? (
          <div className="text-sm text-neutral-600">No products yet.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {filtered.map((product) => (
              <div key={product.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex items-start gap-4">
                  <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-md border border-dashed border-neutral-200 bg-neutral-50 text-neutral-400">
                    <div className="flex flex-col items-center gap-0.5 text-[9px] font-semibold leading-none">
                      <ImageIcon className="h-4 w-4" aria-hidden />
                      <span>No image</span>
                    </div>
                  </div>
                  <div className="flex flex-1 items-start justify-between gap-3">
                    <div>
                      <div className="font-medium text-neutral-900">{product.title}</div>
                      <div className="text-xs text-neutral-500">
                        id: {product.id} / status: {product.status}
                      </div>
                    </div>
                    <div className="flex flex-col gap-2 text-xs">
                      <Link
                        className="rounded-md border border-neutral-200 px-3 py-1 text-center font-medium text-neutral-700 hover:bg-neutral-50"
                        href={`/admin/products/${product.id}`}
                      >
                        Details
                      </Link>
                      <a
                        className="rounded-md border border-neutral-200 px-3 py-1 text-center font-medium text-neutral-700 hover:bg-neutral-50"
                        href={buildProductPreviewUrl(product.id)}
                        target="_blank"
                        rel="noreferrer"
                      >
                        Preview
                      </a>
                    </div>
                  </div>
                </div>
                <div className="mt-3">
                  <ProductInventoryQuickEdit productId={product.id} />
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
