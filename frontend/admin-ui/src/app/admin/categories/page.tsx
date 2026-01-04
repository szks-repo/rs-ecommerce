"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import {
  createCategory,
  deleteCategory,
  listCategoriesAdmin,
  reorderCategories,
} from "@/lib/category";
import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";
import { categoryLabel, flattenCategories } from "@/lib/category-utils";

const STATUS_OPTIONS = ["active", "inactive"] as const;

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

  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugTouched, setSlugTouched] = useState(false);
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<(typeof STATUS_OPTIONS)[number]>("active");
  const [parentId, setParentId] = useState("__none__");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isReordering, setIsReordering] = useState(false);

  function slugify(value: string) {
    return value
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, "")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-");
  }

  async function handleCreate(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    try {
      if (!name.trim()) {
        throw new Error("Category name is required.");
      }
      if (!slug.trim()) {
        throw new Error("Slug is required.");
      }
      await createCategory({
        name: name.trim(),
        slug: slug.trim(),
        description: description.trim() || undefined,
        status,
        parentId: parentId === "__none__" ? undefined : parentId,
      });
      push({
        variant: "success",
        title: "Category created",
        description: `${name.trim()} has been created.`,
      });
      setName("");
      setSlug("");
      setSlugTouched(false);
      setDescription("");
      setStatus("active");
      setParentId("__none__");
      reload();
    } catch (err) {
      notifyError(err, "Create failed", "Failed to create category");
    } finally {
      setIsSubmitting(false);
    }
  }

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
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Catalog</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Categories</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Manage hierarchical product categories and ordering.
        </p>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Create Category</CardTitle>
          <CardDescription className="text-neutral-500">
            Add a new category and optionally assign a parent.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4 md:grid-cols-2" onSubmit={handleCreate}>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="categoryName">Name</Label>
              <Input
                id="categoryName"
                value={name}
                onChange={(e) => {
                  const value = e.target.value;
                  setName(value);
                  if (!slugTouched) {
                    setSlug(slugify(value));
                  }
                }}
                required
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="categorySlug">Slug</Label>
              <Input
                id="categorySlug"
                value={slug}
                onChange={(e) => {
                  setSlug(e.target.value);
                  setSlugTouched(true);
                }}
                placeholder="e.g. apparel"
                required
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="categoryDescription">Description</Label>
              <Input
                id="categoryDescription"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="categoryStatus">Status</Label>
              <Select value={status} onValueChange={setStatus}>
                <SelectTrigger id="categoryStatus" className="bg-white">
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  {STATUS_OPTIONS.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="categoryParent">Parent Category</Label>
              <Select value={parentId} onValueChange={setParentId}>
                <SelectTrigger id="categoryParent" className="bg-white">
                  <SelectValue placeholder="No parent" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__none__">No parent</SelectItem>
                  {flattened.map((category) => (
                    <SelectItem key={category.id} value={category.id}>
                      {categoryLabel(category)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="md:col-span-2">
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? "Saving..." : "Create Category"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Category List</CardTitle>
          <CardDescription className="text-neutral-500">
            Click a category to edit its details or reorder within its parent.
          </CardDescription>
        </CardHeader>
        <CardContent>
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
