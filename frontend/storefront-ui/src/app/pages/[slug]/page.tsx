"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { getPageBySlug, getTenantId } from "@/lib/storefront";
import type { StorefrontPage } from "@/gen/ecommerce/v1/storefront_pb";

const TENANT_STORAGE_KEY = "storefront_tenant_id";

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function renderMarkdown(body: string) {
  const lines = body.split("\n");
  const html = lines
    .map((line) => {
      if (line.startsWith("### ")) {
        return `<h3>${escapeHtml(line.slice(4))}</h3>`;
      }
      if (line.startsWith("## ")) {
        return `<h2>${escapeHtml(line.slice(3))}</h2>`;
      }
      if (line.startsWith("# ")) {
        return `<h1>${escapeHtml(line.slice(2))}</h1>`;
      }
      if (line.trim().length === 0) {
        return `<br />`;
      }
      return `<p>${escapeHtml(line)}</p>`;
    })
    .join("");
  return html;
}

export default function StorefrontPageBySlug() {
  const params = useParams();
  const slug = useMemo(() => {
    if (!params?.slug) {
      return "";
    }
    return Array.isArray(params.slug) ? params.slug[0] : params.slug;
  }, [params]);

  const [tenantId, setTenantId] = useState(getTenantId());
  const [page, setPage] = useState<StorefrontPage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    const saved = window.localStorage.getItem(TENANT_STORAGE_KEY);
    if (saved) {
      setTenantId(saved);
    }
  }, []);

  useEffect(() => {
    if (!tenantId || !slug) {
      return;
    }
    setIsLoading(true);
    getPageBySlug(tenantId, slug)
      .then((data) => {
        setPage(data.page ?? null);
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Failed to load page");
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [tenantId, slug]);

  return (
    <main className="shell">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <div className="eyebrow">Storefront page</div>
          <h1 className="headline">{page?.title || "Page"}</h1>
        </div>
        <Link className="button secondary" href="/">
          Back
        </Link>
      </div>

      {error ? <div className="warning">{error}</div> : null}

      <div className="card" style={{ marginTop: "24px" }}>
        {isLoading ? (
          <p className="subhead">Loading page...</p>
        ) : page ? (
          <div
            className="markdown"
            dangerouslySetInnerHTML={{ __html: renderMarkdown(page.body ?? "") }}
          />
        ) : (
          <p className="subhead">Page not found.</p>
        )}
      </div>
    </main>
  );
}
