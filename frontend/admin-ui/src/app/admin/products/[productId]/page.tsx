"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useToast } from "@/components/ui/toast";
import { Button } from "@/components/ui/button";
import ProductUpdateForm from "@/components/product-update-form";
import VariantUpdateForm from "@/components/variant-update-form";
import SkuImageManager from "@/components/sku-image-manager";
import { listProductsAdmin, listVariantsAdmin } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { buildProductPreviewUrl } from "@/lib/storefront";
import type { ProductAdmin, VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { formatConnectError } from "@/lib/handle-error";

export default function ProductDetailPage() {
  const params = useParams();
  const productId = useMemo(() => {
    if (!params?.productId) {
      return "";
    }
    return Array.isArray(params.productId) ? params.productId[0] : params.productId;
  }, [params]);

  const [product, setProduct] = useState<ProductAdmin | null>(null);
  const [variants, setVariants] = useState<VariantAdmin[]>([]);
  const [selectedVariant, setSelectedVariant] = useState<VariantAdmin | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  async function loadData() {
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
        description: "product_id is missing.",
      });
      return;
    }
    setIsLoading(true);
    try {
      const productResp = await listProductsAdmin();
      const found = (productResp.products ?? []).find((p) => p.id === productId) ?? null;
      setProduct(found);
      const variantsResp = await listVariantsAdmin({ productId });
      setVariants(variantsResp.variants ?? []);
      if (variantsResp.variants && variantsResp.variants.length > 0) {
        const next = selectedVariant
          ? variantsResp.variants.find((v) => v.id === selectedVariant.id) ?? variantsResp.variants[0]
          : variantsResp.variants[0];
        setSelectedVariant(next ?? null);
      } else {
        setSelectedVariant(null);
      }
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load product");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [productId]);

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Products</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">
            {product?.title || "Product details"}
          </h1>
          <p className="mt-2 text-sm text-neutral-600">
            Review the product and manage its variants.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <a
            className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
            href={buildProductPreviewUrl(productId)}
            target="_blank"
            rel="noreferrer"
          >
            Preview
          </a>
          <Link
            className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
            href="/admin/products"
          >
            Back
          </Link>
          <Button variant="outline" onClick={loadData} disabled={isLoading}>
            {isLoading ? "Loading..." : "Refresh"}
          </Button>
        </div>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Product</CardTitle>
          <CardDescription className="text-neutral-500">
            {product ? `status: ${product.status}` : "Select a product to review."}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-neutral-700">
          {product ? (
            <>
              <div className="text-xs text-neutral-500">id: {product.id}</div>
              <div className="text-xs text-neutral-500">
                tax rule: {product.taxRuleId || "default"}
              </div>
              {product.description ? <div>{product.description}</div> : <div>No description.</div>}
            </>
          ) : (
            <div className="text-sm text-neutral-600">Product not found in recent list.</div>
          )}
        </CardContent>
      </Card>

      <div className="grid gap-6 md:grid-cols-2">
        <ProductUpdateForm product={product} onUpdated={loadData} />
        <VariantUpdateForm variant={selectedVariant} onUpdated={loadData} />
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Variants</CardTitle>
          <CardDescription className="text-neutral-500">
            Select a variant to edit.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-neutral-700">
          {variants.length === 0 ? (
            <div className="text-sm text-neutral-600">No variants yet.</div>
          ) : (
            variants.map((variant) => (
              <button
                key={variant.id}
                className={`w-full rounded-lg border px-3 py-2 text-left transition ${
                  selectedVariant?.id === variant.id
                    ? "border-neutral-900 bg-neutral-50"
                    : "border-neutral-200 hover:bg-neutral-50"
                }`}
                type="button"
                onClick={() => setSelectedVariant(variant)}
              >
                <div className="font-medium text-neutral-900">{variant.sku}</div>
                <div className="text-xs text-neutral-500">
                  id: {variant.id} / status: {variant.status} / type: {variant.fulfillmentType}
                </div>
              </button>
            ))
          )}
        </CardContent>
      </Card>

      {selectedVariant ? (
        <SkuImageManager skuId={selectedVariant.id} />
      ) : (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>SKU Images</CardTitle>
            <CardDescription className="text-neutral-500">
              Select a variant to manage its images.
            </CardDescription>
          </CardHeader>
        </Card>
      )}
    </div>
  );
}
