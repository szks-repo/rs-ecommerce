"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { setInventory } from "@/lib/product";
import { listStoreLocations } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { StoreLocation } from "@/gen/ecommerce/v1/store_settings_pb";
import { useApiCall } from "@/lib/use-api-call";

export default function InventorySetForm() {
  const [skuId, setSkuId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [isLoadingLocations, setIsLoadingLocations] = useState(false);
  const [onHand, setOnHand] = useState("0");
  const [reserved, setReserved] = useState("0");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    let cancelled = false;
    setIsLoadingLocations(true);
    listStoreLocations()
      .then((data) => {
        if (!cancelled) {
          setLocations(data.locations ?? []);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          notifyError(err, "Load failed", "Failed to load locations");
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
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      if (!locationId) {
        throw new Error("location_id is missing. Please select a location.");
      }
      const onHandValue = Number(onHand);
      const reservedValue = Number(reserved);
      if (!Number.isFinite(onHandValue) || !Number.isFinite(reservedValue)) {
        throw new Error("on_hand/reserved must be numbers.");
      }
      const data = await setInventory({
        skuId,
        locationId,
        onHand: onHandValue,
        reserved: reservedValue,
      });
      push({
        variant: "success",
        title: "Inventory updated",
        description: `Inventory updated for SKU: ${data.inventory.skuId}`,
      });
      setSkuId("");
      setLocationId("");
      setOnHand("0");
      setReserved("0");
    } catch (err) {
      notifyError(err, "Update failed", "Unknown error");
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
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="inventorySkuId">SKU ID</Label>
            <Input
              id="inventorySkuId"
              value={skuId}
              onChange={(e) => setSkuId(e.target.value)}
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
              <Label htmlFor="inventoryOnHand">On-hand</Label>
              <Input
                id="inventoryOnHand"
                value={onHand}
                onChange={(e) => setOnHand(e.target.value)}
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
