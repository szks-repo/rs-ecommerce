"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { getStoreCode, getTenantId, listProducts } from "@/lib/storefront";
import type { Product } from "@/gen/ecommerce/v1/storefront_pb";

const TENANT_STORAGE_KEY = "storefront_tenant_id";
const STORE_CODE_STORAGE_KEY = "storefront_store_code";

export default function HomePage() {
  const [tenantId, setTenantId] = useState(getTenantId());
  const [storeCode, setStoreCode] = useState(getStoreCode());
  const [products, setProducts] = useState<Product[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const saved = window.localStorage.getItem(TENANT_STORAGE_KEY);
    if (saved) {
      setTenantId(saved);
    }
    const savedStore = window.localStorage.getItem(STORE_CODE_STORAGE_KEY);
    if (savedStore) {
      setStoreCode(savedStore);
    }
  }, []);

  const hasTenant = useMemo(() => tenantId.trim().length > 0, [tenantId]);

  async function loadProducts() {
    if (!hasTenant) {
      setError("tenant_id is required. Set it below to load products.");
      return;
    }
    if (!storeCode.trim()) {
      setError("store_code is required for pages and storefront context.");
      return;
    }
    setError(null);
    setIsLoading(true);
    try {
      const data = await listProducts(tenantId.trim());
      setProducts(data.products ?? []);
      window.localStorage.setItem(TENANT_STORAGE_KEY, tenantId.trim());
      window.localStorage.setItem(STORE_CODE_STORAGE_KEY, storeCode.trim());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load products");
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    if (hasTenant) {
      void loadProducts();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasTenant]);

  return (
    <main className="shell">
      <div>
        <div className="eyebrow">rs-ecommerce storefront</div>
        <h1 className="headline">Living catalogue preview</h1>
        <p className="subhead">
          This is a foundational storefront UI. A future no-code editor will power layouts,
          but today you can preview product data streaming from the backend.
        </p>
      </div>

      <div className="warning">
        <strong>Tenant context required.</strong>{" "}
        Provide the tenant id to query products from the Storefront API.
      </div>

      <div className="card" style={{ marginTop: "24px" }}>
        <label className="meta" htmlFor="tenantIdInput">
          Tenant ID
        </label>
        <div style={{ display: "flex", gap: "12px", marginTop: "8px" }}>
          <input
            id="tenantIdInput"
            className="pill"
            style={{ flex: 1, borderRadius: "12px", padding: "10px 12px" }}
            value={tenantId}
            onChange={(e) => setTenantId(e.target.value)}
            placeholder="tenant uuid"
          />
          <button className="button" onClick={loadProducts} disabled={isLoading}>
            {isLoading ? "Loading..." : "Load"}
          </button>
        </div>
        {error ? <div className="warning">{error}</div> : null}
      </div>

      <div className="card" style={{ marginTop: "24px" }}>
        <label className="meta" htmlFor="storeCodeInput">
          Store code
        </label>
        <div style={{ display: "flex", gap: "12px", marginTop: "8px" }}>
          <input
            id="storeCodeInput"
            className="pill"
            style={{ flex: 1, borderRadius: "12px", padding: "10px 12px" }}
            value={storeCode}
            onChange={(e) => setStoreCode(e.target.value)}
            placeholder="store code"
          />
        </div>
        <p className="subhead" style={{ marginTop: "8px" }}>
          Required to resolve pages by slug.
        </p>
      </div>

      <div className="grid">
        {products.length === 0 ? (
          <div className="card">
            <h3>No products yet</h3>
            <p className="subhead">
              Create products in the admin UI, then refresh this list.
            </p>
          </div>
        ) : (
          products.map((product) => (
            <div className="card" key={product.id}>
              <div className="pill">{product.status || "draft"}</div>
              <h3 style={{ marginTop: "12px" }}>{product.title}</h3>
              <p className="subhead">{product.description || "No description yet."}</p>
              <div style={{ marginTop: "16px" }}>
                <Link className="button secondary" href={`/products/${product.id}`}>
                  View detail
                </Link>
              </div>
            </div>
          ))
        )}
      </div>
    </main>
  );
}
