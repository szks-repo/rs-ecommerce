"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import DateCell from "@/components/date-cell";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { getPage, listPages, updatePage } from "@/lib/pages";
import { toNumber } from "@/lib/number";
import type { PageSummary } from "@/gen/ecommerce/v1/backoffice_pb";
import { Checkbox } from "@/components/ui/checkbox";
import { useToast } from "@/components/ui/toast";
import AdminPageHeader from "@/components/admin-page-header";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTableToolbar,
} from "@/components/admin-table";

function toIsoString(ts?: { seconds?: string | number | bigint; nanos?: number }) {
  if (!ts || ts.seconds == null) {
    return "";
  }
  const sec = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
  if (!Number.isFinite(sec)) {
    return "";
  }
  return new Date(sec * 1000).toISOString();
}

export default function PagesListPage() {
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<PageSummary[]>(
    async () => {
      const resp = await listPages();
      return resp.pages ?? [];
    },
    []
  );
  const pages = data ?? [];
  const sorted = useMemo(
    () =>
      [...pages].sort((a, b) => {
        const bSec = toNumber(b.updatedAt?.seconds);
        const aSec = toNumber(a.updatedAt?.seconds);
        return bSec - aSec;
      }),
    [pages]
  );
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const { push } = useToast();

  const allSelected = sorted.length > 0 && sorted.every((page) => selected.has(page.id));

  function toggleAll() {
    if (allSelected) {
      setSelected(new Set());
      return;
    }
    setSelected(new Set(sorted.map((page) => page.id)));
  }

  function toggleOne(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  async function updateSelectedStatus(nextStatus: "draft" | "published") {
    if (selected.size === 0) {
      push({
        variant: "error",
        title: "No pages selected",
        description: "Select at least one page to update.",
      });
      return;
    }
    try {
      const ids = Array.from(selected);
      for (const id of ids) {
        const resp = await getPage({ pageId: id });
        const page = resp.page;
        if (!page) {
          continue;
        }
        await updatePage({
          pageId: id,
          page: {
            title: page.title ?? "",
            slug: page.slug ?? "",
            body: page.body ?? "",
            bodyFormat: page.bodyFormat ?? "markdown",
            status: nextStatus,
            publishStartAt: page.publishStartAt,
            publishEndAt: page.publishEndAt,
            seoTitle: page.seoTitle ?? "",
            seoDescription: page.seoDescription ?? "",
          },
        });
      }
      push({
        variant: "success",
        title: "Pages updated",
        description: `Set ${selected.size} pages to ${nextStatus}.`,
      });
      setSelected(new Set());
      await reload();
    } catch (err) {
      notifyError(err, "Bulk update failed", "Failed to update selected pages");
    }
  }

  async function handleRefresh() {
    try {
      await reload();
    } catch (err) {
      notifyError(err, "Reload failed", "Failed to refresh pages");
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Pages"
        description="Manage free pages for your storefront. Markdown is supported."
        actions={
          <>
            <Button asChild>
              <Link href="/admin/settings/pages/new">New Page</Link>
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={selected.size === 0}
              onClick={() => updateSelectedStatus("published")}
            >
              Publish Selected
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={selected.size === 0}
              onClick={() => updateSelectedStatus("draft")}
            >
              Unpublish Selected
            </Button>
            <Button variant="outline" onClick={handleRefresh} disabled={loading}>
              Refresh
            </Button>
          </>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Page List</CardTitle>
          <CardDescription className="text-neutral-500">
            Click a page to edit content, slug, and publishing window.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar left={`${sorted.length} pages`} />
          {loading ? (
            <div className="text-sm text-neutral-500">Loading pages…</div>
          ) : error ? (
            <div className="text-sm text-red-600">Failed to load pages.</div>
          ) : sorted.length === 0 ? (
            <div className="text-sm text-neutral-500">
              No pages yet. Create your first page to get started.
            </div>
          ) : (
            <AdminTable>
              <thead className="sticky top-0 bg-neutral-50">
                <tr>
                  <AdminTableHeaderCell>
                    <div className="flex items-center gap-2">
                      <Checkbox checked={allSelected} onCheckedChange={toggleAll} />
                      Page
                    </div>
                  </AdminTableHeaderCell>
                  <AdminTableHeaderCell>Slug</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Status</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Updated</AdminTableHeaderCell>
                  <AdminTableHeaderCell align="right">Detail</AdminTableHeaderCell>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-200">
                {sorted.map((page) => (
                  <tr key={page.id}>
                    <AdminTableCell>
                      <div className="flex items-center gap-2">
                        <Checkbox
                          checked={selected.has(page.id)}
                          onCheckedChange={() => toggleOne(page.id)}
                        />
                        <div>
                          <div className="text-sm font-medium text-neutral-900">{page.title}</div>
                          <div className="text-[11px] text-neutral-500">id: {page.id}</div>
                        </div>
                      </div>
                    </AdminTableCell>
                    <AdminTableCell>/{page.slug}</AdminTableCell>
                    <AdminTableCell>{page.status}</AdminTableCell>
                    <AdminTableCell className="text-neutral-500">
                      <DateCell value={toIsoString(page.updatedAt)} />
                    </AdminTableCell>
                    <AdminTableCell align="right">
                      <Button asChild type="button" size="sm" variant="outline">
                        <Link href={`/admin/settings/pages/${page.id}`}>Open</Link>
                      </Button>
                    </AdminTableCell>
                  </tr>
                ))}
              </tbody>
            </AdminTable>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
