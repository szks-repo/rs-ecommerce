"use client";

import { useCallback, useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTableToolbar,
  AdminTablePagination,
} from "@/components/admin-table";
import { listInventoryStocks } from "@/lib/product";
import { useApiCall } from "@/lib/use-api-call";
import { formatTimestampWithStoreTz } from "@/lib/time";
import type { InventoryAdmin } from "@/gen/ecommerce/v1/backoffice_pb";

export default function InventoryList() {
  const { call } = useApiCall();
  const [items, setItems] = useState<InventoryAdmin[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [skuQuery, setSkuQuery] = useState("");
  const [locationQuery, setLocationQuery] = useState("");
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");

  const load = useCallback(
    async (token: string) => {
      setIsLoading(true);
      const data = await call(() =>
        listInventoryStocks({
          skuId: skuQuery.trim() || undefined,
          locationId: locationQuery.trim() || undefined,
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
    [call, skuQuery, locationQuery]
  );

  useEffect(() => {
    load("");
  }, [load]);

  return (
    <div className="space-y-4">
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
            <Button onClick={() => load("")} disabled={isLoading}>
              {isLoading ? "Loading..." : "Search"}
            </Button>
          </div>
        }
      />
      <AdminTable>
        <thead>
          <tr>
            <AdminTableHeaderCell>SKU</AdminTableHeaderCell>
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
              <AdminTableCell colSpan={6}>
                {isLoading ? "Loading..." : "No inventory records found."}
              </AdminTableCell>
            </tr>
          ) : (
            items.map((row) => (
              <tr key={`${row.skuId}-${row.locationId}`}>
                <AdminTableCell>{row.skuId}</AdminTableCell>
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
        onNext={nextPageToken ? () => load(nextPageToken) : undefined}
        onPrev={pageToken ? () => load("") : undefined}
        nextDisabled={!nextPageToken}
        prevDisabled={!pageToken}
      />
    </div>
  );
}
