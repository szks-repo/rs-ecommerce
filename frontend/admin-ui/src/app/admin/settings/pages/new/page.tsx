"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import Link from "next/link";
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
import { createPage } from "@/lib/pages";
import AdminPageHeader from "@/components/admin-page-header";
import { dateInputToTimestamp } from "@/lib/time";

const STATUS_OPTIONS = ["draft", "published"] as const;

export default function PageCreatePage() {
  const router = useRouter();
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const [title, setTitle] = useState("");
  const [slug, setSlug] = useState("");
  const [body, setBody] = useState("");
  const [status, setStatus] = useState<(typeof STATUS_OPTIONS)[number]>("draft");
  const [publishStartDate, setPublishStartDate] = useState("");
  const [publishEndDate, setPublishEndDate] = useState("");
  const [seoTitle, setSeoTitle] = useState("");
  const [seoDescription, setSeoDescription] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    try {
      const resp = await createPage({
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
        title: "Page created",
        description: "Page has been created successfully.",
      });
      if (resp.page?.id) {
        router.push(`/admin/settings/pages/${resp.page.id}`);
      } else {
        router.push("/admin/settings/pages");
      }
    } catch (err) {
      notifyError(err, "Create failed", "Failed to create page");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="New Page"
        description="Draft a new free page. Markdown is supported for the body."
        actions={
          <Button asChild variant="outline">
            <Link href="/admin/settings/pages">Back</Link>
          </Button>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Page Details</CardTitle>
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
            <div className="md:col-span-2 flex items-center justify-end gap-2">
              <Button type="submit" disabled={isSubmitting}>
                Create Page
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
