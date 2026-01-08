"use client";

import { useEffect, useMemo, useState } from "react";
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
import { listVariantsAdmin, listInventoryStocks, setInventory } from "@/lib/product";
import { listStoreLocations } from "@/lib/store_settings";
import { useApiCall } from "@/lib/use-api-call";
import { useToast } from "@/components/ui/toast";
import type { VariantAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import type { StoreLocation } from "@/gen/ecommerce/v1/store_settings_pb";

type ProductInventoryQuickEditProps = {
  productId: string;
};

export default function ProductInventoryQuickEdit({ productId }: ProductInventoryQuickEditProps) {
  const { notifyError } = useApiCall();
  const { push } = useToast();
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [variants, setVariants] = useState<VariantAdmin[]>([]);
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [skuId, setSkuId] = useState("");
  const [locationId, setLocationId] = useState("");
  const [onHand, setOnHand] = useState("0");
  const [reserved, setReserved] = useState("0");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    Promise.all([listVariantsAdmin({ productId }), listStoreLocations()])
      .then(([variantResp, locationResp]) => {
        if (cancelled) {
          return;
        }
        setVariants(variantResp.variants ?? []);
        setLocations(locationResp.locations ?? []);
      })
      .catch((err) => {
        if (!cancelled) {
          notifyError(err, "Load failed", "Failed to load inventory data");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [isOpen, productId, notifyError]);

  useEffect(() => {
    if (!skuId || !locationId) {
      return;
    }
    let cancelled = false;
    listInventoryStocks({ skuId, locationId })
      .then((data) => {
        if (cancelled) {
          return;
        }
        const record = (data.inventories ?? [])[0];
        if (record) {
          setOnHand(String(record.onHand));
          setReserved(String(record.reserved));
        } else {
          setOnHand("0");
          setReserved("0");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          notifyError(err, "Load failed", "Failed to load inventory stock");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [skuId, locationId, notifyError]);

  const selectedVariant = useMemo(
    () => variants.find((variant) => variant.id === skuId) ?? null,
    [variants, skuId]
  );

  const variantLabel = selectedVariant
    ? [selectedVariant.sku, ...selectedVariant.axisValues.map((axis) => `${axis.name}: ${axis.value}`)]
        .filter(Boolean)
        .join(" / ")
    : "";

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!skuId) {
        throw new Error("sku_id is required.");
      }
      if (!locationId) {
        throw new Error("location_id is required.");
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
    } catch (err) {
      notifyError(err, "Update failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="space-y-2">
      <Button size="sm" variant="outline" type="button" onClick={() => setIsOpen(!isOpen)}>
        {isOpen ? "Close inventory" : "Inventory"}
      </Button>
      {isOpen ? (
        <form className="grid gap-3 rounded-lg border border-neutral-200 bg-white p-3" onSubmit={handleSubmit}>
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor={`inventorySku-${productId}`}>SKU</Label>
              <Select value={skuId} onValueChange={setSkuId} disabled={isLoading}>
                <SelectTrigger id={`inventorySku-${productId}`} className="bg-white">
                  <SelectValue placeholder={isLoading ? "Loading..." : "Select SKU"} />
                </SelectTrigger>
                <SelectContent>
                  {variants.length === 0 ? (
                    <SelectItem value="none" disabled>
                      No SKUs available
                    </SelectItem>
                  ) : (
                    variants.map((variant) => (
                      <SelectItem key={variant.id} value={variant.id}>
                        {variant.sku}
                      </SelectItem>
                    ))
                  )}
                </SelectContent>
              </Select>
              {variantLabel ? (
                <div className="text-xs text-neutral-500">Selected: {variantLabel}</div>
              ) : null}
            </div>
            <div className="space-y-2">
              <Label htmlFor={`inventoryLocation-${productId}`}>Location</Label>
              <Select value={locationId} onValueChange={setLocationId} disabled={isLoading}>
                <SelectTrigger id={`inventoryLocation-${productId}`} className="bg-white">
                  <SelectValue placeholder={isLoading ? "Loading..." : "Select location"} />
                </SelectTrigger>
                <SelectContent>
                  {locations.length === 0 ? (
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
          </div>
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor={`inventoryOnHand-${productId}`}>On-hand</Label>
              <Input
                id={`inventoryOnHand-${productId}`}
                value={onHand}
                onChange={(e) => setOnHand(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor={`inventoryReserved-${productId}`}>Reserved</Label>
              <Input
                id={`inventoryReserved-${productId}`}
                value={reserved}
                onChange={(e) => setReserved(e.target.value)}
              />
            </div>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting || !skuId || !locationId}>
              {isSubmitting ? "Updating..." : "Set inventory"}
            </Button>
          </div>
        </form>
      ) : null}
    </div>
  );
}
