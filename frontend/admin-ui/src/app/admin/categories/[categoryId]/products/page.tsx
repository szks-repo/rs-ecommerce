"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import AdminPageHeader from "@/components/admin-page-header";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { listCategoriesAdmin, listCategoryProducts, reorderCategoryProducts } from "@/lib/category";
import type { Category, CategoryProductAdmin } from "@/gen/ecommerce/v1/backoffice_pb";

export default function CategoryProductsPage() {
  const params = useParams();
  const categoryId = useMemo(() => {
    if (!params?.categoryId) {
      return "";
    }
    return Array.isArray(params.categoryId) ? params.categoryId[0] : params.categoryId;
  }, [params]);
  const { notifyError } = useApiCall();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [products, setProducts] = useState<CategoryProductAdmin[]>([]);

  const { data: categories, loading: categoriesLoading } = useAsyncResource<Category[]>(
    async () => {
      const resp = await listCategoriesAdmin();
      return resp.categories ?? [];
    },
    []
  );
  const { data, loading, error, reload } = useAsyncResource<CategoryProductAdmin[]>(
    async () => {
      if (!categoryId) {
        return [];
      }
      const resp = await listCategoryProducts({ categoryId });
      return resp.products ?? [];
    },
    [categoryId]
  );

  useEffect(() => {
    if (data) {
      setProducts(data);
    }
  }, [data]);

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load category products");
    }
  }, [error, notifyError]);

  const category = categories?.find((item) => item.id === categoryId);

  const moveProduct = useCallback(
    async (index: number, direction: "up" | "down") => {
      if (isSubmitting) {
        return;
      }
      const next = [...products];
      const swapWith = direction === "up" ? index - 1 : index + 1;
      if (swapWith < 0 || swapWith >= next.length) {
        return;
      }
      [next[index], next[swapWith]] = [next[swapWith], next[index]];
      setProducts(next);
      setIsSubmitting(true);
      try {
        const resp = await reorderCategoryProducts({
          categoryId,
          orderedProductIds: next.map((item) => item.productId),
        });
        setProducts(resp.products ?? []);
      } catch (err) {
        notifyError(err, "Reorder failed", "Failed to reorder products");
        reload();
      } finally {
        setIsSubmitting(false);
      }
    },
    [categoryId, isSubmitting, notifyError, products, reload]
  );

  if (!categoryId) {
    return <div className="text-sm text-neutral-600">Category ID is missing.</div>;
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Category Products"
        description={
          category
            ? `Reorder products in ${category.name}.`
            : "Reorder products inside this category."
        }
        actions={
          <>
            <Button asChild variant="outline">
              <Link href={`/admin/categories/${categoryId}`}>Back</Link>
            </Button>
            <Button variant="outline" onClick={reload} disabled={loading || categoriesLoading}>
              Refresh
            </Button>
          </>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Products</CardTitle>
          <CardDescription className="text-neutral-500">
            Move products up or down to change their order inside this category.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-sm text-neutral-500">Loading products…</div>
          ) : products.length === 0 ? (
            <div className="text-sm text-neutral-600">No products linked to this category.</div>
          ) : (
            <div className="divide-y divide-neutral-100 rounded-md border border-neutral-200">
              {products.map((product, index) => (
                <div
                  key={product.productId}
                  className="flex flex-wrap items-center justify-between gap-2 px-4 py-3"
                >
                  <div className="flex-1">
                    <div className="text-sm font-medium text-neutral-900">{product.title}</div>
                    <div className="text-xs text-neutral-500">
                      id: {product.productId} · status: {product.status}
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => moveProduct(index, "up")}
                      disabled={isSubmitting || index === 0}
                    >
                      Up
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => moveProduct(index, "down")}
                      disabled={isSubmitting || index === products.length - 1}
                    >
                      Down
                    </Button>
                    <Button asChild variant="ghost" size="sm">
                      <Link href={`/admin/products/${product.productId}`}>View</Link>
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
