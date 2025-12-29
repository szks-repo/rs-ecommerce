"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Textarea } from "@/components/ui/textarea";
import { updateProduct } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";

export default function ProductUpdateForm() {
  const [productId, setProductId] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("active");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const data = await updateProduct({
        productId,
        title,
        description,
        status,
      });
      setMessage(`Updated product: ${data.product.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Update Product</CardTitle>
        <CardDescription className="text-neutral-500">
          Update title, description, or status.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert className="mb-4">
            <AlertTitle>Update failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {message && (
          <Alert className="mb-4">
            <AlertTitle>Success</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        )}
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="updateProductId">Product ID</Label>
            <Input
              id="updateProductId"
              value={productId}
              onChange={(e) => setProductId(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateProductTitle">Title</Label>
            <Input
              id="updateProductTitle"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateProductDescription">Description</Label>
            <Textarea
              id="updateProductDescription"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="updateProductStatus">Status</Label>
            <Input
              id="updateProductStatus"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
            />
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Updating..." : "Update Product"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
