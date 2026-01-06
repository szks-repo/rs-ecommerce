"use client";

import Link from "next/link";
import { useMemo, useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
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
import AdminPageHeader from "@/components/admin-page-header";
import { deletePage, getPage, updatePage } from "@/lib/pages";
import { dateInputToTimestamp, timestampToDateInput } from "@/lib/time";
import type { PageAdmin } from "@/gen/ecommerce/v1/backoffice_pb";

const STATUS_OPTIONS = ["draft", "published"] as const;

export default function PageDetailPage() {
  const params = useParams();
  const router = useRouter();
  const pageId = useMemo(() => {
    if (!params?.pageId) {
      return "";
    }
    return Array.isArray(params.pageId) ? params.pageId[0] : params.pageId;
  }, [params]);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  const { data, loading, error, reload } = useAsyncResource<PageAdmin | null>(
    async () => {
      if (!pageId) {
        return null;
      }
      const resp = await getPage({ pageId });
      return resp.page ?? null;
    },
    [pageId]
  );
  const page = data ?? null;

  const [title, setTitle] = useState("");
  const [slug, setSlug] = useState("");
  const [body, setBody] = useState("");
  const [status, setStatus] = useState<(typeof STATUS_OPTIONS)[number]>("draft");
  const [publishStartDate, setPublishStartDate] = useState("");
  const [publishEndDate, setPublishEndDate] = useState("");
  const [seoTitle, setSeoTitle] = useState("");
  const [seoDescription, setSeoDescription] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!page) {
      return;
    }
    setTitle(page.title ?? "");
    setSlug(page.slug ?? "");
    setBody(page.body ?? "");
    setStatus((page.status as (typeof STATUS_OPTIONS)[number]) ?? "draft");
    setPublishStartDate(page.publishStartAt ? timestampToDateInput(page.publishStartAt) : "");
    setPublishEndDate(page.publishEndAt ? timestampToDateInput(page.publishEndAt) : "");
    setSeoTitle(page.seoTitle ?? "");
    setSeoDescription(page.seoDescription ?? "");
  }, [page]);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!pageId) {
      return;
    }
    setIsSubmitting(true);
    try {
      await updatePage({
        pageId,
        page: {
          title: title.trim(),
          slug: slug.trim(),
          body: body.trim(),
          bodyFormat: "markdown",
          status,
          publishStartAt: dateInputToTimestamp(publishStartDate, false),
          publishEndAt: dateInputToTimestamp(publishEndDate, true),
          seoTitle: seoTitle.trim(),
          seoDescription: seoDescription.trim(),
        },
      });
      push({
        variant: "success",
        title: "Page updated",
        description: "Changes have been saved.",
      });
      reload();
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update page");
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handleDelete() {
    if (!pageId) {
      return;
    }
    if (!window.confirm("Delete this page?")) {
      return;
    }
    setIsSubmitting(true);
    try {
      await deletePage({ pageId });
      push({
        variant: "success",
        title: "Page deleted",
        description: "The page has been removed.",
      });
      router.push("/admin/settings/pages");
    } catch (err) {
      notifyError(err, "Delete failed", "Failed to delete page");
    } finally {
      setIsSubmitting(false);
    }
  }

  if (loading) {
    return <div className="text-sm text-neutral-500">Loading page…</div>;
  }

  if (error || !page) {
    return (
      <div className="space-y-4">
        <div className="text-sm text-red-600">Page not found.</div>
        <Link href="/admin/settings/pages" className="text-sm text-neutral-600 underline">
          Back to pages
        </Link>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Page Detail"
        description="Update content, slug, and publishing window. Markdown is supported."
        actions={
          <>
            <Button asChild variant="outline">
              <Link href="/admin/settings/pages">Back</Link>
            </Button>
            <Button variant="outline" onClick={reload} disabled={isSubmitting}>
              Refresh
            </Button>
          </>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>{page.title}</CardTitle>
          <CardDescription className="text-neutral-500">
            Slug can include Japanese characters. It will be used in the public URL.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4 md:grid-cols-2" onSubmit={handleSubmit}>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="pageTitle">Title</Label>
              <Input id="pageTitle" value={title} onChange={(e) => setTitle(e.target.value)} required />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="pageSlug">Slug</Label>
              <Input id="pageSlug" value={slug} onChange={(e) => setSlug(e.target.value)} required />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="pageBody">Body (Markdown)</Label>
              <Textarea
                id="pageBody"
                rows={10}
                value={body}
                onChange={(e) => setBody(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="pageStatus">Status</Label>
              <Select value={status} onValueChange={(value) => setStatus(value as typeof status)}>
                <SelectTrigger id="pageStatus">
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
            <div />
            <div className="space-y-2">
              <Label htmlFor="publishStart">Publish start</Label>
              <Input
                id="publishStart"
                type="date"
                value={publishStartDate}
                onChange={(e) => setPublishStartDate(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="publishEnd">Publish end</Label>
              <Input
                id="publishEnd"
                type="date"
                value={publishEndDate}
                onChange={(e) => setPublishEndDate(e.target.value)}
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="seoTitle">SEO title</Label>
              <Input id="seoTitle" value={seoTitle} onChange={(e) => setSeoTitle(e.target.value)} />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="seoDescription">SEO description</Label>
              <Textarea
                id="seoDescription"
                rows={3}
                value={seoDescription}
                onChange={(e) => setSeoDescription(e.target.value)}
              />
            </div>
            <div className="md:col-span-2 flex items-center justify-between gap-2">
              <Button type="button" variant="outline" onClick={handleDelete} disabled={isSubmitting}>
                Delete
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                Save Changes
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
