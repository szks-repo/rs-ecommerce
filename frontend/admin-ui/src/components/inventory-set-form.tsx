"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { setInventory } from "@/lib/product";

export default function InventorySetForm() {
  const [variantId, setVariantId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [stock, setStock] = useState("0");
  const [reserved, setReserved] = useState("0");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      const storeId = sessionStorage.getItem("store_id");
      if (!storeId) {
        throw new Error("store_id is missing. Please sign in first.");
      }
      const stockValue = Number(stock);
      const reservedValue = Number(reserved);
      if (!Number.isFinite(stockValue) || !Number.isFinite(reservedValue)) {
        throw new Error("stock/reserved must be numbers.");
      }
      const data = await setInventory({
        storeId,
        variantId,
        locationId,
        stock: stockValue,
        reserved: reservedValue,
      });
      setMessage(`Inventory updated for variant: ${data.inventory.variantId}`);
      setVariantId("");
      setLocationId("");
      setStock("0");
      setReserved("0");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Set Inventory</CardTitle>
        <CardDescription className="text-neutral-500">
          Update stock per location.
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
            <Label htmlFor="inventoryVariantId">Variant ID</Label>
            <Input
              id="inventoryVariantId"
              value={variantId}
              onChange={(e) => setVariantId(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="inventoryLocationId">Location ID</Label>
            <Input
              id="inventoryLocationId"
              value={locationId}
              onChange={(e) => setLocationId(e.target.value)}
              required
            />
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="inventoryStock">Stock</Label>
              <Input
                id="inventoryStock"
                value={stock}
                onChange={(e) => setStock(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="inventoryReserved">Reserved</Label>
              <Input
                id="inventoryReserved"
                value={reserved}
                onChange={(e) => setReserved(e.target.value)}
              />
            </div>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Updating..." : "Set Inventory"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
