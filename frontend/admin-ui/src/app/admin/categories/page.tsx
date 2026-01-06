"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import AdminPageHeader from "@/components/admin-page-header";
import { AdminTableToolbar } from "@/components/admin-table";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { deleteCategory, listCategoriesAdmin, reorderCategories } from "@/lib/category";
import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";
import { categoryLabel, flattenCategories } from "@/lib/category-utils";

export default function CategoriesPage() {
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<Category[]>(
    async () => {
      const resp = await listCategoriesAdmin();
      return resp.categories ?? [];
    },
    []
  );
  const categories = data ?? [];
  const flattened = useMemo(() => flattenCategories(categories), [categories]);
  const siblingsMap = useMemo(() => {
    const map = new Map<string, Category[]>();
    for (const category of categories) {
      const key = category.parentId || "";
      const list = map.get(key);
      if (list) {
        list.push(category);
      } else {
        map.set(key, [category]);
      }
    }
    for (const list of map.values()) {
      list.sort((a, b) => (a.position ?? 0) - (b.position ?? 0));
    }
    return map;
  }, [categories]);

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isReordering, setIsReordering] = useState(false);

  async function handleMove(category: Category, direction: "up" | "down") {
    const key = category.parentId || "";
    const siblings = siblingsMap.get(key) ?? [];
    const index = siblings.findIndex((item) => item.id === category.id);
    if (index < 0) {
      return;
    }
    const next = [...siblings];
    const swapWith = direction === "up" ? index - 1 : index + 1;
    if (swapWith < 0 || swapWith >= next.length) {
      return;
    }
    [next[index], next[swapWith]] = [next[swapWith], next[index]];
    setIsReordering(true);
    try {
      await reorderCategories({
        parentId: key ? key : undefined,
        orderedIds: next.map((item) => item.id),
      });
      await reload();
    } catch (err) {
      notifyError(err, "Reorder failed", "Failed to reorder categories");
    } finally {
      setIsReordering(false);
    }
  }

  async function handleDelete(category: Category) {
    setIsSubmitting(true);
    try {
      await deleteCategory({ categoryId: category.id });
      push({
        variant: "success",
        title: "Category deleted",
        description: `${category.name} has been deleted.`,
      });
      reload();
    } catch (err) {
      notifyError(err, "Delete failed", "Failed to delete category");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Categories"
        description="Manage hierarchical product categories and ordering."
        actions={
          <Button asChild>
            <Link href="/admin/categories/new">New Category</Link>
          </Button>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Category List</CardTitle>
          <CardDescription className="text-neutral-500">
            Click a category to edit its details or reorder within its parent.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar
            left={`${flattened.length} categories`}
            right={
              <Button variant="outline" size="sm" onClick={reload} disabled={loading}>
                Refresh
              </Button>
            }
          />
          {loading ? (
            <div className="text-sm text-neutral-500">Loading categories…</div>
          ) : error ? (
            <div className="text-sm text-red-600">Failed to load categories.</div>
          ) : flattened.length === 0 ? (
            <div className="text-sm text-neutral-500">No categories yet.</div>
          ) : (
            <div className="divide-y divide-neutral-100 rounded-md border border-neutral-200">
              {flattened.map((category) => {
                const key = category.parentId || "";
                const siblings = siblingsMap.get(key) ?? [];
                const index = siblings.findIndex((item) => item.id === category.id);
                const canMoveUp = index > 0;
                const canMoveDown = index >= 0 && index < siblings.length - 1;
                return (
                  <div
                    key={category.id}
                    className="flex flex-wrap items-center justify-between gap-2 px-4 py-3"
                  >
                    <Link
                      href={`/admin/categories/${category.id}`}
                      className="flex-1 text-sm text-neutral-800 hover:underline"
                      style={{ paddingLeft: `${category.depth * 12}px` }}
                    >
                      <div className="font-medium">{category.name}</div>
                      <div className="text-xs text-neutral-500">
                        {category.slug} · {category.status}
                      </div>
                    </Link>
                    <div className="flex items-center gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!canMoveUp || isReordering}
                        onClick={() => handleMove(category, "up")}
                      >
                        ↑
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!canMoveDown || isReordering}
                        onClick={() => handleMove(category, "down")}
                      >
                        ↓
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => handleDelete(category)}
                        disabled={isSubmitting}
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
