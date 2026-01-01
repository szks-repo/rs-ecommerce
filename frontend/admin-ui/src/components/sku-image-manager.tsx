"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/toast";
import { formatConnectError } from "@/lib/handle-error";
import { createMediaAsset, createMediaUploadUrl, listMediaAssets, listSkuImages, setSkuImages } from "@/lib/product";
import type { MediaAsset } from "@/gen/ecommerce/v1/backoffice_pb";

type SelectedImage = {
  asset: MediaAsset;
};

export default function SkuImageManager({ skuId }: { skuId: string }) {
  const [assets, setAssets] = useState<MediaAsset[]>([]);
  const [selectedImages, setSelectedImages] = useState<SelectedImage[]>([]);
  const [query, setQuery] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isCreatingAsset, setIsCreatingAsset] = useState(false);
  const [publicUrl, setPublicUrl] = useState("");
  const [objectKey, setObjectKey] = useState("");
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const { push } = useToast();

  const selectedAssetIds = useMemo(
    () => new Set(selectedImages.map((img) => img.asset.id)),
    [selectedImages]
  );

  async function loadAssets(currentQuery?: string) {
    setIsLoading(true);
    try {
      const res = await listMediaAssets({ query: currentQuery ?? query });
      setAssets(res.assets ?? []);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load assets");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsLoading(false);
    }
  }

  async function loadSkuImages() {
    try {
      const res = await listSkuImages({ skuId });
      const images = (res.images ?? [])
        .filter((img) => img.asset)
        .map((img) => ({
          asset: img.asset!,
        }));
      setSelectedImages(images);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load SKU images");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    }
  }

  useEffect(() => {
    void loadAssets("");
    void loadSkuImages();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [skuId]);

  function addToSelection(asset: MediaAsset) {
    if (selectedAssetIds.has(asset.id)) {
      return;
    }
    setSelectedImages((prev) => [...prev, { asset }]);
  }

  function moveImage(index: number, direction: "up" | "down") {
    setSelectedImages((prev) => {
      const next = [...prev];
      const targetIndex = direction === "up" ? index - 1 : index + 1;
      if (targetIndex < 0 || targetIndex >= next.length) {
        return prev;
      }
      const temp = next[index];
      next[index] = next[targetIndex];
      next[targetIndex] = temp;
      return next;
    });
  }

  function removeImage(index: number) {
    setSelectedImages((prev) => prev.filter((_, idx) => idx !== index));
  }

  async function saveImages() {
    if (!skuId) {
      return;
    }
    setIsSaving(true);
    try {
      const payload = selectedImages.map((img, index) => ({
        assetId: img.asset.id,
        position: index + 1,
      }));
      await setSkuImages({ skuId, images: payload });
      push({
        variant: "success",
        title: "Images updated",
        description: "SKU images have been updated.",
      });
      await loadSkuImages();
    } catch (err) {
      const uiError = formatConnectError(err, "Save failed", "Failed to update images");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsSaving(false);
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
    setIsCreatingAsset(true);
    try {
      const res = await createMediaAsset({
        publicUrl: publicUrl.trim(),
        objectKey: objectKey.trim(),
      });
      if (res.asset) {
        setAssets((prev) => [res.asset!, ...prev]);
        setPublicUrl("");
        setObjectKey("");
      }
      push({
        variant: "success",
        title: "Image imported",
        description: "The image has been copied into your storage and added to the library.",
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Create failed", "Failed to add asset");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsCreatingAsset(false);
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
        addToSelection(assetResp.asset!);
      }
      setUploadFile(null);
      push({
        variant: "success",
        title: "Upload complete",
        description: "The image has been uploaded and added to the library.",
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Upload failed", "Failed to upload image");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsUploading(false);
    }
  }

  return (
    <div className="space-y-6">
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>SKU Images</CardTitle>
          <CardDescription className="text-neutral-500">
            Manage images for this SKU. Select from the asset library or add a new URL.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="mediaPublicUrl">Register existing image URL</Label>
            <div className="flex flex-col gap-2 md:flex-row">
              <Input
                id="mediaPublicUrl"
                value={publicUrl}
                onChange={(e) => setPublicUrl(e.target.value)}
                placeholder="https://cdn.example.com/products/sku-001.jpg"
              />
              <Input
                value={objectKey}
                onChange={(e) => setObjectKey(e.target.value)}
                placeholder="optional object key"
              />
              <Button type="button" onClick={handleCreateAsset} disabled={isCreatingAsset}>
                {isCreatingAsset ? "Adding..." : "Add"}
              </Button>
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="mediaUpload">Upload new image</Label>
            <div className="flex flex-col gap-2 md:flex-row md:items-center">
              <Input
                id="mediaUpload"
                type="file"
                accept="image/*"
                onChange={(e) => setUploadFile(e.target.files?.[0] ?? null)}
              />
              <Button type="button" onClick={handleUploadFile} disabled={!uploadFile || isUploading}>
                {isUploading ? "Uploading..." : "Upload"}
              </Button>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search assets"
              className="max-w-xs"
            />
            <Button type="button" variant="outline" onClick={() => loadAssets(query)} disabled={isLoading}>
              {isLoading ? "Loading..." : "Search"}
            </Button>
            <Button type="button" variant="outline" onClick={() => loadAssets("")} disabled={isLoading}>
              Refresh
            </Button>
          </div>
          <div className="grid gap-3 md:grid-cols-3">
            {assets.length === 0 ? (
              <div className="text-sm text-neutral-600">No assets found.</div>
            ) : (
              assets.map((asset) => (
                <div key={asset.id} className="rounded-lg border border-neutral-200 p-3">
                  <div className="aspect-square w-full overflow-hidden rounded-md bg-neutral-100">
                    {asset.publicUrl ? (
                      <img
                        src={asset.publicUrl}
                        alt={asset.objectKey || asset.publicUrl}
                        className="h-full w-full object-cover"
                      />
                    ) : null}
                  </div>
                  <div className="mt-2 text-xs text-neutral-500 line-clamp-2">
                    {asset.objectKey || asset.publicUrl}
                  </div>
                  <Button
                    type="button"
                    className="mt-2 w-full"
                    variant={selectedAssetIds.has(asset.id) ? "outline" : "default"}
                    onClick={() => addToSelection(asset)}
                    disabled={selectedAssetIds.has(asset.id)}
                  >
                    {selectedAssetIds.has(asset.id) ? "Selected" : "Use"}
                  </Button>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Selected Images</CardTitle>
          <CardDescription className="text-neutral-500">
            Arrange the display order for this SKU.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {selectedImages.length === 0 ? (
            <div className="text-sm text-neutral-600">No images selected.</div>
          ) : (
            selectedImages.map((image, index) => (
              <div
                key={`${image.asset.id}-${index}`}
                className="flex flex-col gap-3 rounded-lg border border-neutral-200 p-3 md:flex-row md:items-center"
              >
                <div className="h-20 w-20 overflow-hidden rounded-md bg-neutral-100">
                  {image.asset.publicUrl ? (
                    <img
                      src={image.asset.publicUrl}
                      alt={image.asset.objectKey || image.asset.publicUrl}
                      className="h-full w-full object-cover"
                    />
                  ) : null}
                </div>
                <div className="flex-1 text-sm text-neutral-700">
                  <div className="font-medium text-neutral-900">#{index + 1}</div>
                  <div className="text-xs text-neutral-500 line-clamp-2">
                    {image.asset.objectKey || image.asset.publicUrl}
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button type="button" variant="outline" size="sm" onClick={() => moveImage(index, "up")}>
                    Up
                  </Button>
                  <Button type="button" variant="outline" size="sm" onClick={() => moveImage(index, "down")}>
                    Down
                  </Button>
                  <Button type="button" variant="destructive" size="sm" onClick={() => removeImage(index)}>
                    Remove
                  </Button>
                </div>
              </div>
            ))
          )}
          <div className="flex justify-end">
            <Button type="button" onClick={saveImages} disabled={isSaving}>
              {isSaving ? "Saving..." : "Save Images"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
