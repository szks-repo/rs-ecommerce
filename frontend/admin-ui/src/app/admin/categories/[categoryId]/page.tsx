"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams, useRouter } from "next/navigation";
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
  deleteCategory,
  listCategoriesAdmin,
  updateCategory,
} from "@/lib/category";
import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";
import { categoryLabel, flattenCategories } from "@/lib/category-utils";

const STATUS_OPTIONS = ["active", "inactive"] as const;

export default function CategoryDetailPage() {
  const params = useParams();
  const router = useRouter();
  const categoryId = useMemo(() => {
    if (!params?.categoryId) {
      return "";
    }
    return Array.isArray(params.categoryId) ? params.categoryId[0] : params.categoryId;
  }, [params]);
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
  const category = categories.find((item) => item.id === categoryId);

  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<(typeof STATUS_OPTIONS)[number]>("active");
  const [parentId, setParentId] = useState("__none__");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const currentCategory = category;

  useEffect(() => {
    if (!currentCategory) {
      return;
    }
    setName(currentCategory.name);
    setSlug(currentCategory.slug);
    setDescription(currentCategory.description ?? "");
    setStatus((currentCategory.status as (typeof STATUS_OPTIONS)[number]) ?? "active");
    setParentId(currentCategory.parentId || "__none__");
  }, [currentCategory]);

  const parentOptions = flattened.filter((item) => item.id !== categoryId);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!categoryId) {
      push({
        variant: "error",
        title: "Invalid category",
        description: "Category ID is missing.",
      });
      return;
    }
    setIsSubmitting(true);
    try {
      await updateCategory({
        categoryId,
        name: name.trim(),
        slug: slug.trim(),
        description: description.trim() || undefined,
        status,
        parentId: parentId === "__none__" ? undefined : parentId,
      });
      push({
        variant: "success",
        title: "Category updated",
        description: "Changes have been saved.",
      });
      reload();
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update category");
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleDelete() {
    if (!categoryId) {
      return;
    }
    setIsSubmitting(true);
    try {
      await deleteCategory({ categoryId });
      push({
        variant: "success",
        title: "Category deleted",
        description: "The category has been removed.",
      });
      router.push("/admin/categories");
    } catch (err) {
      notifyError(err, "Delete failed", "Failed to delete category");
    } finally {
      setIsSubmitting(false);
    }
  }

  if (loading) {
    return <div className="text-sm text-neutral-500">Loading category…</div>;
  }

  if (error || !currentCategory) {
    return (
      <div className="space-y-4">
        <div className="text-sm text-red-600">Category not found.</div>
        <Link href="/admin/categories" className="text-sm text-neutral-600 underline">
          Back to categories
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Catalog</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Category Detail</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Update the category attributes and hierarchy.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Link
            className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
            href="/admin/categories"
          >
            Back
          </Link>
          <Button variant="outline" onClick={reload} disabled={isSubmitting}>
            Refresh
          </Button>
        </div>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>{currentCategory.name}</CardTitle>
          <CardDescription className="text-neutral-500">
            Edit name, slug, or status. Parent can be adjusted as needed.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4 md:grid-cols-2" onSubmit={handleSubmit}>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="categoryName">Name</Label>
              <Input
                id="categoryName"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="categorySlug">Slug</Label>
              <Input
                id="categorySlug"
                value={slug}
                onChange={(e) => setSlug(e.target.value)}
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
                  {parentOptions.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {categoryLabel(item)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="md:col-span-2 flex items-center gap-2">
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? "Saving..." : "Save Changes"}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={handleDelete}
                disabled={isSubmitting}
              >
                Delete
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
