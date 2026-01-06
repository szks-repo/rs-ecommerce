"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import {
  FileArchive,
  FileAudio,
  FileImage,
  FileText,
  FileVideo,
  Trash2,
} from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";
import {
  createMediaAsset,
  createMediaUploadUrl,
  deleteMediaAsset,
  listMediaAssets,
  updateMediaAssetTags,
} from "@/lib/product";
import type { MediaAsset } from "@/gen/ecommerce/v1/backoffice_pb";
import { toNumber } from "@/lib/number";
import AdminPageHeader from "@/components/admin-page-header";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTableToolbar,
} from "@/components/admin-table";

function formatBytes(value?: string | number | bigint) {
  const size = toNumber(value);
  if (!size) {
    return "-";
  }
  if (size < 1024) {
    return `${size} B`;
  }
  const kb = size / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(1)} KB`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(1)} MB`;
}

export default function FilesPage() {
  const [assets, setAssets] = useState<MediaAsset[]>([]);
  const [query, setQuery] = useState("");
  const [publicUrl, setPublicUrl] = useState("");
  const [objectKey, setObjectKey] = useState("");
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [tagEdits, setTagEdits] = useState<Record<string, string>>({});
  const [savingTags, setSavingTags] = useState<Record<string, boolean>>({});
  const [deleting, setDeleting] = useState<Record<string, boolean>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  const filteredAssets = useMemo(() => {
    if (!query.trim()) {
      return assets;
    }
    const needle = query.trim().toLowerCase();
    return assets.filter((asset) => {
      const hay = `${asset.publicUrl ?? ""} ${asset.objectKey ?? ""} ${asset.contentType ?? ""}`.toLowerCase();
      return hay.includes(needle);
    });
  }, [assets, query]);

  async function loadAssets() {
    setIsLoading(true);
    try {
      const resp = await listMediaAssets({ query: "" });
      setAssets(resp.assets ?? []);
    } catch (err) {
      notifyError(err, "Load failed", "Failed to load assets");
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadAssets();
  }, []);

  function normalizeTags(input: string) {
    return input
      .split(",")
      .map((value) => value.trim())
      .filter((value) => value.length > 0);
  }

  function getIconForAsset(asset: MediaAsset) {
    const type = (asset.contentType ?? "").toLowerCase();
    if (type.startsWith("image/")) {
      return FileImage;
    }
    if (type.startsWith("video/")) {
      return FileVideo;
    }
    if (type.startsWith("audio/")) {
      return FileAudio;
    }
    if (type.includes("zip") || type.includes("compressed")) {
      return FileArchive;
    }
    if (type.includes("pdf") || type.includes("text")) {
      return FileText;
    }
    return FileText;
  }

  async function handleSaveTags(asset: MediaAsset) {
    if (!asset.id) {
      return;
    }
    const raw = tagEdits[asset.id] ?? asset.tags?.join(", ") ?? "";
    const tags = normalizeTags(raw);
    setSavingTags((prev) => ({ ...prev, [asset.id]: true }));
    try {
      const resp = await updateMediaAssetTags({ assetId: asset.id, tags });
      if (resp.asset) {
        setAssets((prev) => prev.map((item) => (item.id === asset.id ? resp.asset! : item)));
      }
      setTagEdits((prev) => ({ ...prev, [asset.id]: tags.join(", ") }));
      push({
        variant: "success",
        title: "Tags updated",
        description: "The asset tags have been saved.",
      });
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update tags");
    } finally {
      setSavingTags((prev) => ({ ...prev, [asset.id]: false }));
    }
  }

  async function handleDeleteAsset(asset: MediaAsset) {
    if (!asset.id) {
      return;
    }
    if (!window.confirm("Delete this asset? This cannot be undone.")) {
      return;
    }
    setDeleting((prev) => ({ ...prev, [asset.id]: true }));
    try {
      const resp = await deleteMediaAsset({ assetId: asset.id });
      if (resp.deleted) {
        setAssets((prev) => prev.filter((item) => item.id !== asset.id));
      }
      push({
        variant: "success",
        title: "Asset deleted",
        description: "The asset has been removed from the library.",
      });
    } catch (err) {
      notifyError(err, "Delete failed", "Failed to delete asset");
    } finally {
      setDeleting((prev) => ({ ...prev, [asset.id]: false }));
    }
  }

  async function handleCreateAsset() {
    if (!publicUrl.trim()) {
      push({
        variant: "error",
        title: "Invalid input",
        description: "Public URL is required.",
      });
      return;
    }
    setIsCreating(true);
    try {
      const resp = await createMediaAsset({
        publicUrl: publicUrl.trim(),
        objectKey: objectKey.trim(),
      });
      if (resp.asset) {
        setAssets((prev) => [resp.asset!, ...prev]);
      }
      setPublicUrl("");
      setObjectKey("");
      push({
        variant: "success",
        title: "File added",
        description: "The asset has been saved to your library.",
      });
    } catch (err) {
      notifyError(err, "Create failed", "Failed to add asset");
    } finally {
      setIsCreating(false);
    }
  }

  async function handleUploadFile() {
    if (!uploadFile) {
      return;
    }
    setIsUploading(true);
    try {
      const upload = await createMediaUploadUrl({
        filename: uploadFile.name,
        contentType: uploadFile.type || "",
        sizeBytes: uploadFile.size,
      });
      const headers = new Headers();
      Object.entries(upload.headers ?? {}).forEach(([key, value]) => {
        headers.set(key, value);
      });
      if (!headers.has("Content-Type") && uploadFile.type) {
        headers.set("Content-Type", uploadFile.type);
      }
      const putResp = await fetch(upload.uploadUrl, {
        method: "PUT",
        headers,
        body: uploadFile,
      });
      if (!putResp.ok) {
        throw new Error(`upload failed (${putResp.status})`);
      }
      const assetResp = await createMediaAsset({
        publicUrl: upload.publicUrl,
        provider: upload.provider,
        bucket: upload.bucket,
        objectKey: upload.objectKey,
        contentType: uploadFile.type || "",
        sizeBytes: uploadFile.size,
      });
      if (assetResp.asset) {
        setAssets((prev) => [assetResp.asset!, ...prev]);
      }
      setUploadFile(null);
      push({
        variant: "success",
        title: "Upload complete",
        description: "The file is now available in the library.",
      });
    } catch (err) {
      notifyError(err, "Upload failed", "Failed to upload file");
    } finally {
      setIsUploading(false);
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Files"
        description="Manage uploaded assets. Images uploaded here can be reused in product detail pages."
        actions={
          <Button asChild variant="outline">
            <Link href="/admin/settings">Back to settings</Link>
          </Button>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Upload</CardTitle>
          <CardDescription className="text-neutral-500">
            Upload a file or register an existing public URL.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="mediaPublicUrl">Register existing URL</Label>
            <div className="flex flex-col gap-2 md:flex-row">
              <Input
                id="mediaPublicUrl"
                value={publicUrl}
                onChange={(e) => setPublicUrl(e.target.value)}
                placeholder="https://cdn.example.com/assets/hero.jpg"
              />
              <Input
                value={objectKey}
                onChange={(e) => setObjectKey(e.target.value)}
                placeholder="optional object key"
              />
              <Button type="button" onClick={handleCreateAsset} disabled={isCreating}>
                {isCreating ? "Adding..." : "Add"}
              </Button>
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="mediaUpload">Upload file</Label>
            <div className="flex flex-col gap-2 md:flex-row md:items-center">
              <Input
                id="mediaUpload"
                type="file"
                onChange={(e) => setUploadFile(e.target.files?.[0] ?? null)}
              />
              <Button type="button" onClick={handleUploadFile} disabled={!uploadFile || isUploading}>
                {isUploading ? "Uploading..." : "Upload"}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Asset Library</CardTitle>
          <CardDescription className="text-neutral-500">
            Browse all uploaded assets and search by filename or URL.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar
            left={`${filteredAssets.length} assets`}
            right={
              <>
                <Input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Search assets"
                  className="h-9 w-full min-w-[220px] max-w-[320px] bg-white"
                />
                <Button type="button" variant="outline" onClick={loadAssets} disabled={isLoading} size="sm">
                  Refresh
                </Button>
              </>
            }
          />

          {isLoading ? (
            <div className="text-sm text-neutral-500">Loading assets…</div>
          ) : filteredAssets.length === 0 ? (
            <div className="text-sm text-neutral-500">No assets found.</div>
          ) : (
            <AdminTable>
              <thead className="sticky top-0 bg-neutral-50">
                <tr>
                  <AdminTableHeaderCell>Preview</AdminTableHeaderCell>
                  <AdminTableHeaderCell>URL</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Type</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Size</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Tags</AdminTableHeaderCell>
                  <AdminTableHeaderCell>Actions</AdminTableHeaderCell>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-200">
                {filteredAssets.map((asset) => (
                  <tr key={asset.id}>
                    <AdminTableCell>
                      {asset.publicUrl && (asset.contentType ?? "").startsWith("image/") ? (
                        <img
                          src={asset.publicUrl}
                          alt="asset preview"
                          className="h-12 w-12 rounded-md object-cover"
                        />
                      ) : (
                        <div className="flex h-12 w-12 items-center justify-center rounded-md border border-dashed border-neutral-200 bg-neutral-50 text-neutral-400">
                          {(() => {
                            const Icon = getIconForAsset(asset);
                            return <Icon className="h-5 w-5" />;
                          })()}
                        </div>
                      )}
                    </AdminTableCell>
                    <AdminTableCell>
                      <div className="max-w-[360px] truncate">{asset.publicUrl || "-"}</div>
                      <div className="text-[10px] text-neutral-400">{asset.objectKey || ""}</div>
                    </AdminTableCell>
                    <AdminTableCell>{asset.contentType || "-"}</AdminTableCell>
                    <AdminTableCell>{formatBytes(asset.sizeBytes)}</AdminTableCell>
                    <AdminTableCell>
                      <div className="flex flex-col gap-2">
                        <Input
                          value={tagEdits[asset.id] ?? asset.tags?.join(", ") ?? ""}
                          onChange={(e) =>
                            setTagEdits((prev) => ({ ...prev, [asset.id]: e.target.value }))
                          }
                          placeholder="tag1, tag2"
                          className="h-8 text-[11px]"
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          className="h-7 w-fit text-xs"
                          onClick={() => handleSaveTags(asset)}
                          disabled={savingTags[asset.id]}
                        >
                          {savingTags[asset.id] ? "Saving..." : "Save tags"}
                        </Button>
                      </div>
                    </AdminTableCell>
                    <AdminTableCell>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="h-7 gap-1 text-xs text-red-600 hover:text-red-700"
                        onClick={() => handleDeleteAsset(asset)}
                        disabled={deleting[asset.id]}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                        {deleting[asset.id] ? "Deleting..." : "Delete"}
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
