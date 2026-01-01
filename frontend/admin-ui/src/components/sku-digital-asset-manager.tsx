"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/toast";
import { formatConnectError } from "@/lib/handle-error";
import {
  createDigitalAsset,
  createDigitalDownloadUrl,
  createDigitalUploadUrl,
  listDigitalAssets,
} from "@/lib/product";
import type { DigitalAsset } from "@/gen/ecommerce/v1/backoffice_pb";

export default function SkuDigitalAssetManager({ skuId }: { skuId: string }) {
  const [assets, setAssets] = useState<DigitalAsset[]>([]);
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  const hasAssets = useMemo(() => assets.length > 0, [assets.length]);

  async function loadAssets() {
    setIsLoading(true);
    try {
      const res = await listDigitalAssets({ skuId });
      setAssets(res.assets ?? []);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load digital assets");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadAssets();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [skuId]);

  async function handleUpload() {
    if (!uploadFile) {
      return;
    }
    setIsUploading(true);
    try {
      const upload = await createDigitalUploadUrl({
        skuId,
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
      const assetResp = await createDigitalAsset({
        skuId,
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
        description: "Digital asset uploaded.",
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Upload failed", "Failed to upload digital asset");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsUploading(false);
    }
  }

  async function handleDownload(assetId: string) {
    try {
      const resp = await createDigitalDownloadUrl({ assetId });
      if (resp.downloadUrl) {
        window.open(resp.downloadUrl, "_blank", "noopener,noreferrer");
      }
    } catch (err) {
      const uiError = formatConnectError(err, "Download failed", "Failed to create download URL");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Digital Asset</CardTitle>
        <CardDescription className="text-neutral-500">
          Upload files for digital fulfillment. Downloads are generated from private storage.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="digitalUpload">Upload file</Label>
          <div className="flex flex-col gap-2 md:flex-row md:items-center">
            <Input
              id="digitalUpload"
              type="file"
              onChange={(e) => setUploadFile(e.target.files?.[0] ?? null)}
            />
            <Button type="button" onClick={handleUpload} disabled={!uploadFile || isUploading}>
              {isUploading ? "Uploading..." : "Upload"}
            </Button>
          </div>
        </div>
        <div className="flex items-center justify-between text-sm text-neutral-600">
          <div>{hasAssets ? `${assets.length} asset(s)` : "No assets yet."}</div>
          <Button type="button" variant="outline" onClick={loadAssets} disabled={isLoading}>
            {isLoading ? "Loading..." : "Refresh"}
          </Button>
        </div>
        <div className="space-y-2">
          {assets.map((asset) => (
            <div
              key={asset.id}
              className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-neutral-200 px-3 py-2"
            >
              <div className="text-sm text-neutral-800">{asset.objectKey || asset.id}</div>
              <Button type="button" variant="outline" onClick={() => handleDownload(asset.id)}>
                Download URL
              </Button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
