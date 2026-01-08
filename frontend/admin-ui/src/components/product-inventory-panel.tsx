"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTableToolbar,
  AdminTablePagination,
} from "@/components/admin-table";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";
import { listInventoryStocks, setInventory } from "@/lib/product";
import { listStoreLocations } from "@/lib/store_settings";
import { formatTimestampWithStoreTz } from "@/lib/time";
import InventoryMovementList from "@/components/inventory-movement-list";
import type { InventoryAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import type { StoreLocation } from "@/gen/ecommerce/v1/store_settings_pb";

type ProductInventoryPanelProps = {
  skuId: string;
  skuLabel?: string;
};

export default function ProductInventoryPanel({ skuId, skuLabel }: ProductInventoryPanelProps) {
  const { push } = useToast();
  const { call, notifyError } = useApiCall();
  const [items, setItems] = useState<InventoryAdmin[]>([]);
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [locationId, setLocationId] = useState("");
  const [onHand, setOnHand] = useState("0");
  const [reserved, setReserved] = useState("0");
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");

  const title = useMemo(() => (skuLabel ? `Inventory — ${skuLabel}` : "Inventory"), [skuLabel]);

  const loadStocks = useCallback(
    async (token: string) => {
      setIsLoading(true);
      const data = await call(() =>
        listInventoryStocks({
          skuId,
          pageToken: token,
          pageSize: 50,
        })
      );
      if (data) {
        setItems(data.inventories ?? []);
        setNextPageToken(data.page?.nextPageToken || "");
        setPageToken(token);
      }
      setIsLoading(false);
    },
    [call, skuId]
  );

  useEffect(() => {
    loadStocks("");
  }, [loadStocks]);

  useEffect(() => {
    let cancelled = false;
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
      });
    return () => {
      cancelled = true;
    };
  }, [notifyError]);

  useEffect(() => {
    if (!locationId) {
      setOnHand("0");
      setReserved("0");
      return;
    }
    const existing = items.find((item) => item.locationId === locationId);
    if (existing) {
      setOnHand(String(existing.onHand));
      setReserved(String(existing.reserved));
    } else {
      setOnHand("0");
      setReserved("0");
    }
  }, [locationId, items]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
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
      await loadStocks("");
    } catch (err) {
      notifyError(err, "Update failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="space-y-6">
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>{title}</CardTitle>
          <CardDescription className="text-neutral-500">
            Adjust stock per location for this SKU.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <form className="grid gap-4 md:grid-cols-3" onSubmit={handleSubmit}>
            <div className="space-y-2 md:col-span-1">
              <Label htmlFor="productInventoryLocation">Location</Label>
              <Select value={locationId} onValueChange={setLocationId}>
                <SelectTrigger id="productInventoryLocation" className="bg-white">
                  <SelectValue placeholder="Select location" />
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
            <div className="space-y-2">
              <Label htmlFor="productInventoryOnHand">On-hand</Label>
              <Input
                id="productInventoryOnHand"
                value={onHand}
                onChange={(e) => setOnHand(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="productInventoryReserved">Reserved</Label>
              <Input
                id="productInventoryReserved"
                value={reserved}
                onChange={(e) => setReserved(e.target.value)}
              />
            </div>
            <div className="md:col-span-3">
              <Button type="submit" disabled={isSubmitting || !locationId}>
                {isSubmitting ? "Updating..." : "Set inventory"}
              </Button>
            </div>
          </form>

          <AdminTableToolbar
            left={<div className="text-sm font-medium text-neutral-700">Current stock</div>}
          />
          <AdminTable>
            <thead>
              <tr>
                <AdminTableHeaderCell>Location</AdminTableHeaderCell>
                <AdminTableHeaderCell align="right">On-hand</AdminTableHeaderCell>
                <AdminTableHeaderCell align="right">Reserved</AdminTableHeaderCell>
                <AdminTableHeaderCell align="right">Available</AdminTableHeaderCell>
                <AdminTableHeaderCell>Updated</AdminTableHeaderCell>
              </tr>
            </thead>
            <tbody>
              {items.length === 0 ? (
                <tr>
                  <AdminTableCell colSpan={5}>
                    {isLoading ? "Loading..." : "No inventory records found."}
                  </AdminTableCell>
                </tr>
              ) : (
                items.map((row) => (
                  <tr key={`${row.skuId}-${row.locationId}`}>
                    <AdminTableCell>{row.locationId}</AdminTableCell>
                    <AdminTableCell align="right">{row.onHand}</AdminTableCell>
                    <AdminTableCell align="right">{row.reserved}</AdminTableCell>
                    <AdminTableCell align="right">{row.available}</AdminTableCell>
                    <AdminTableCell>
                      {formatTimestampWithStoreTz(row.updatedAt?.seconds, row.updatedAt?.nanos)}
                    </AdminTableCell>
                  </tr>
                ))
              )}
            </tbody>
          </AdminTable>
          <AdminTablePagination
            onNext={nextPageToken ? () => loadStocks(nextPageToken) : undefined}
            onPrev={pageToken ? () => loadStocks("") : undefined}
            nextDisabled={!nextPageToken}
            prevDisabled={!pageToken}
          />
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Inventory movements</CardTitle>
          <CardDescription className="text-neutral-500">
            History for this SKU (latest first).
          </CardDescription>
        </CardHeader>
        <CardContent>
          <InventoryMovementList skuId={skuId} hideFilters />
        </CardContent>
      </Card>
    </div>
  );
}
