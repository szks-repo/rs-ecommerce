"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { setInventory } from "@/lib/product";
import { rpcFetch } from "@/lib/api";
import { getActiveAccessToken } from "@/lib/auth";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

type StoreLocation = {
  id: string;
  code: string;
  name: string;
  status: string;
};

export default function InventorySetForm() {
  const [variantId, setVariantId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [isLoadingLocations, setIsLoadingLocations] = useState(false);
  const [stock, setStock] = useState("0");
  const [reserved, setReserved] = useState("0");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    let cancelled = false;
    setIsLoadingLocations(true);
    rpcFetch<{ locations: StoreLocation[] }>(
      "/rpc/ecommerce.v1.StoreSettingsService/ListStoreLocations",
      {}
    )
      .then((data) => {
        if (!cancelled) {
          setLocations(data.locations ?? []);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load locations");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoadingLocations(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      if (!locationId) {
        throw new Error("location_id is missing. Please select a location.");
      }
      const stockValue = Number(stock);
      const reservedValue = Number(reserved);
      if (!Number.isFinite(stockValue) || !Number.isFinite(reservedValue)) {
        throw new Error("stock/reserved must be numbers.");
      }
      const data = await setInventory({
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
            <Label htmlFor="inventoryLocationId">Location</Label>
            <Select value={locationId} onValueChange={setLocationId}>
              <SelectTrigger id="inventoryLocationId" className="bg-white">
                <SelectValue
                  placeholder={isLoadingLocations ? "Loading locations..." : "Select location"}
                />
              </SelectTrigger>
              <SelectContent>
                {locations.length === 0 && !isLoadingLocations ? (
                  <SelectItem value="none" disabled>
                    No locations found
                  </SelectItem>
                ) : (
                  locations.map((loc) => (
                    <SelectItem key={loc.id} value={loc.id}>
                      {loc.code} — {loc.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
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
