"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
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
import AdminPageHeader from "@/components/admin-page-header";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { createCategory, listCategoriesAdmin } from "@/lib/category";
import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";
import { categoryLabel, flattenCategories } from "@/lib/category-utils";

const STATUS_OPTIONS = ["active", "inactive"] as const;

export default function CategoryNewPage() {
  const router = useRouter();
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const { data } = useAsyncResource<Category[]>(
    async () => {
      const resp = await listCategoriesAdmin();
      return resp.categories ?? [];
    },
    []
  );
  const categories = data ?? [];
  const flattened = useMemo(() => flattenCategories(categories), [categories]);

  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugTouched, setSlugTouched] = useState(false);
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<(typeof STATUS_OPTIONS)[number]>("active");
  const [parentId, setParentId] = useState("__none__");
  const [isSubmitting, setIsSubmitting] = useState(false);

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
      const resp = await createCategory({
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
      router.push(`/admin/categories/${resp.category.id}`);
    } catch (err) {
      notifyError(err, "Create failed", "Failed to create category");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="New Category"
        description="Create a new category and optionally assign a parent."
        actions={
          <Link
            className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
            href="/admin/categories"
          >
            Back
          </Link>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Category Details</CardTitle>
          <CardDescription className="text-neutral-500">
            Provide the name, slug, and hierarchy.
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
    </div>
  );
}
