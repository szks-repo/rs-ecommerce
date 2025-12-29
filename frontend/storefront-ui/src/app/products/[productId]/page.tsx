"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { getProduct, getTenantId } from "@/lib/storefront";
import type { Product } from "@/gen/ecommerce/v1/storefront_pb";

const TENANT_STORAGE_KEY = "storefront_tenant_id";

export default function ProductDetailPage() {
  const params = useParams();
  const productId = useMemo(() => {
    if (!params?.productId) {
      return "";
    }
    return Array.isArray(params.productId) ? params.productId[0] : params.productId;
  }, [params]);

  const [tenantId, setTenantId] = useState(getTenantId());
  const [product, setProduct] = useState<Product | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const saved = window.localStorage.getItem(TENANT_STORAGE_KEY);
    if (saved) {
      setTenantId(saved);
    }
  }, []);

  useEffect(() => {
    if (!tenantId || !productId) {
      return;
    }
    getProduct(tenantId, productId)
      .then((data) => {
        setProduct(data.product ?? null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Failed to load product");
      });
  }, [tenantId, productId]);

  return (
    <main className="shell">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <div className="eyebrow">Product detail</div>
          <h1 className="headline">{product?.title || "Product"}</h1>
        </div>
        <Link className="button secondary" href="/">
          Back
        </Link>
      </div>

      {error ? <div className="warning">{error}</div> : null}

      <div className="card" style={{ marginTop: "24px" }}>
        {product ? (
          <>
            <div className="pill">{product.status || "draft"}</div>
            <p className="subhead" style={{ marginTop: "12px" }}>
              {product.description || "No description yet."}
            </p>
            <div className="meta" style={{ marginTop: "12px" }}>
              product id: {product.id}
            </div>
          </>
        ) : (
          <p className="subhead">Loading product detail...</p>
        )}
      </div>

      {product?.variants?.length ? (
        <div style={{ marginTop: "32px" }}>
          <h2 style={{ fontSize: "22px", marginBottom: "12px" }}>Variants</h2>
          <div className="grid">
            {product.variants.map((variant) => (
              <div className="card" key={variant.id}>
                <div className="pill">{variant.status || "active"}</div>
                <h3 style={{ marginTop: "10px" }}>{variant.sku}</h3>
                {variant.price ? (
                  <div className="meta">
                    {variant.price.amount} {variant.price.currency}
                  </div>
                ) : null}
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </main>
  );
}
