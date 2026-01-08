"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTableToolbar,
  AdminTablePagination,
} from "@/components/admin-table";
import { listInventoryMovements } from "@/lib/product";
import { useApiCall } from "@/lib/use-api-call";
import { formatTimestampWithStoreTz } from "@/lib/time";
import type { InventoryMovement } from "@/gen/ecommerce/v1/backoffice_pb";

type InventoryMovementListProps = {
  skuId?: string;
  locationId?: string;
  hideFilters?: boolean;
};

const MOVEMENT_TYPE_HINTS = [
  "adjust",
  "transfer_out",
  "transfer_in",
  "reserve",
  "release",
  "consume",
];

export default function InventoryMovementList({
  skuId,
  locationId,
  hideFilters,
}: InventoryMovementListProps) {
  const { call } = useApiCall();
  const [items, setItems] = useState<InventoryMovement[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [skuQuery, setSkuQuery] = useState(skuId ?? "");
  const [locationQuery, setLocationQuery] = useState(locationId ?? "");
  const [movementType, setMovementType] = useState("");
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");

  const movementTypeHint = useMemo(() => MOVEMENT_TYPE_HINTS.join(", "), []);

  const load = useCallback(
    async (token: string) => {
      setIsLoading(true);
      const data = await call(() =>
        listInventoryMovements({
          skuId: skuQuery.trim() || undefined,
          locationId: locationQuery.trim() || undefined,
          movementType: movementType.trim() || undefined,
          pageToken: token,
          pageSize: 50,
        })
      );
      if (data) {
        setItems(data.movements ?? []);
        setNextPageToken(data.page?.nextPageToken || "");
        setPageToken(token);
      }
      setIsLoading(false);
    },
    [call, skuQuery, locationQuery, movementType]
  );

  useEffect(() => {
    setSkuQuery(skuId ?? "");
  }, [skuId]);

  useEffect(() => {
    setLocationQuery(locationId ?? "");
  }, [locationId]);

  useEffect(() => {
    load("");
  }, [load]);

  return (
    <div className="space-y-4">
      {!hideFilters ? (
        <AdminTableToolbar
          left={
            <div className="flex flex-wrap items-center gap-2">
              <Input
                value={skuQuery}
                onChange={(e) => setSkuQuery(e.target.value)}
                placeholder="SKU ID"
                className="h-9 w-56"
              />
              <Input
                value={locationQuery}
                onChange={(e) => setLocationQuery(e.target.value)}
                placeholder="Location ID"
                className="h-9 w-56"
              />
              <Input
                value={movementType}
                onChange={(e) => setMovementType(e.target.value)}
                placeholder={`Type (${movementTypeHint})`}
                className="h-9 w-64"
              />
              <Button onClick={() => load("")} disabled={isLoading}>
                {isLoading ? "Loading..." : "Search"}
              </Button>
            </div>
          }
        />
      ) : null}
      <AdminTable>
        <thead>
          <tr>
            <AdminTableHeaderCell>Time</AdminTableHeaderCell>
            <AdminTableHeaderCell>Type</AdminTableHeaderCell>
            <AdminTableHeaderCell>SKU</AdminTableHeaderCell>
            <AdminTableHeaderCell>Location</AdminTableHeaderCell>
            <AdminTableHeaderCell align="right">Qty</AdminTableHeaderCell>
            <AdminTableHeaderCell align="right">On-hand</AdminTableHeaderCell>
            <AdminTableHeaderCell align="right">Reserved</AdminTableHeaderCell>
            <AdminTableHeaderCell>Source</AdminTableHeaderCell>
          </tr>
        </thead>
        <tbody>
          {items.length === 0 ? (
            <tr>
              <AdminTableCell colSpan={8}>
                {isLoading ? "Loading..." : "No inventory movements found."}
              </AdminTableCell>
            </tr>
          ) : (
            items.map((row) => (
              <tr key={row.id}>
                <AdminTableCell>
                  {formatTimestampWithStoreTz(row.occurredAt?.seconds, row.occurredAt?.nanos)}
                </AdminTableCell>
                <AdminTableCell>{row.movementType}</AdminTableCell>
                <AdminTableCell>{row.skuId}</AdminTableCell>
                <AdminTableCell>{row.locationId}</AdminTableCell>
                <AdminTableCell align="right">{row.quantity}</AdminTableCell>
                <AdminTableCell align="right">
                  {row.beforeOnHand} → {row.afterOnHand}
                </AdminTableCell>
                <AdminTableCell align="right">
                  {row.beforeReserved} → {row.afterReserved}
                </AdminTableCell>
                <AdminTableCell className="max-w-[260px] truncate">
                  {[row.sourceType, row.sourceId].filter(Boolean).join(": ") || "-"}
                </AdminTableCell>
              </tr>
            ))
          )}
        </tbody>
      </AdminTable>
      <AdminTablePagination
        onNext={nextPageToken ? () => load(nextPageToken) : undefined}
        onPrev={pageToken ? () => load("") : undefined}
        nextDisabled={!nextPageToken}
        prevDisabled={!pageToken}
      />
    </div>
  );
}
