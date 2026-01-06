"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useApiCall } from "@/lib/use-api-call";
import { listAuctions } from "@/lib/auction";
import { formatMoney } from "@/lib/money";
import type { Auction } from "@/gen/ecommerce/v1/auction_pb";
import DateCell from "@/components/date-cell";
import {
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTablePagination,
  AdminTableToolbar,
} from "@/components/admin-table";

const statusOptions = [
  "all",
  "draft",
  "scheduled",
  "running",
  "ended",
  "awaiting_approval",
  "approved",
] as const;

function toIsoString(ts?: { seconds?: string | number | bigint; nanos?: number }) {
  if (!ts?.seconds) {
    return "";
  }
  const seconds = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
  if (!Number.isFinite(seconds)) {
    return "";
  }
  const date = new Date(seconds * 1000);
  return Number.isNaN(date.getTime()) ? "" : date.toISOString();
}

export default function AuctionList() {
  const [status, setStatus] = useState<string>("all");
  const [auctions, setAuctions] = useState<Auction[]>([]);
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");
  const [pageSize, setPageSize] = useState(50);
  const [isLoading, setIsLoading] = useState(false);
  const { call } = useApiCall();

  async function loadAuctions(nextStatus?: string) {
    setIsLoading(true);
    const result = await call(async () => {
      const selected = (nextStatus ?? status) === "all" ? "" : nextStatus ?? status;
      const data = await listAuctions({
        status: selected,
        pageSize,
        pageToken,
      });
      return {
        auctions: data.auctions ?? [],
        nextPageToken: data.page?.nextPageToken ?? "",
      };
    }, {
      errorTitle: "Load failed",
      errorDescription: "Failed to load auctions",
    });
    if (result) {
      setAuctions(result.auctions);
      setNextPageToken(result.nextPageToken);
    }
    setIsLoading(false);
  }

  useEffect(() => {
    void loadAuctions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, pageSize, pageToken]);

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <CardTitle>Auctions</CardTitle>
          <CardDescription className="text-neutral-500">
            Manage store auctions and review bidding status.
          </CardDescription>
        </div>
      </CardHeader>
      <CardContent>
        <AdminTableToolbar
          left={`Showing ${auctions.length} auctions`}
          right={
            <>
              <Select
                value={status}
                onValueChange={(value) => {
                  setStatus(value);
                  setPageToken("");
                }}
              >
                <SelectTrigger className="h-9 w-[160px] bg-white">
                  <SelectValue placeholder="Filter status" />
                </SelectTrigger>
                <SelectContent>
                  {statusOptions.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Select
                value={String(pageSize)}
                onValueChange={(value) => {
                  setPageSize(Number(value));
                  setPageToken("");
                }}
              >
                <SelectTrigger className="h-9 w-[120px] bg-white">
                  <SelectValue placeholder="Rows" />
                </SelectTrigger>
                <SelectContent>
                  {[25, 50, 100].map((size) => (
                    <SelectItem key={size} value={String(size)}>
                      {size} / page
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button variant="outline" onClick={() => loadAuctions()} disabled={isLoading} size="sm">
                {isLoading ? "Loading..." : "Refresh"}
              </Button>
            </>
          }
        />
        {auctions.length === 0 ? (
          <div className="text-sm text-neutral-600">No auctions found.</div>
        ) : (
          <AdminTable>
            <thead className="sticky top-0 bg-neutral-50">
              <tr>
                <AdminTableHeaderCell>Auction</AdminTableHeaderCell>
                <AdminTableHeaderCell>Status</AdminTableHeaderCell>
                <AdminTableHeaderCell>Schedule</AdminTableHeaderCell>
                <AdminTableHeaderCell>Price</AdminTableHeaderCell>
                <AdminTableHeaderCell>Created</AdminTableHeaderCell>
                <AdminTableHeaderCell align="right">Detail</AdminTableHeaderCell>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-200">
              {auctions.map((auction, index) => (
                <tr key={auction.id || `${auction.skuId}-${index}`}>
                  <AdminTableCell>
                    <div className="text-sm font-medium text-neutral-900">
                      {auction.title || "(Untitled auction)"}
                    </div>
                    <div className="text-[11px] text-neutral-500">
                      id: {auction.id || "-"} / sku: {auction.skuId || "-"}
                    </div>
                  </AdminTableCell>
                  <AdminTableCell>
                    <div>{auction.status}</div>
                    <div className="text-neutral-400">{auction.auctionType}</div>
                  </AdminTableCell>
                  <AdminTableCell>
                    <div>
                      <DateCell value={toIsoString(auction.startAt)} />
                    </div>
                    <div>
                      <DateCell value={toIsoString(auction.endAt)} />
                    </div>
                  </AdminTableCell>
                  <AdminTableCell>
                    <div>Start: {formatMoney(auction.startPrice)}</div>
                    <div>Current: {formatMoney(auction.currentPrice)}</div>
                  </AdminTableCell>
                  <AdminTableCell className="text-neutral-500">
                    <DateCell value={toIsoString(auction.createdAt)} />
                  </AdminTableCell>
                  <AdminTableCell align="right">
                    {auction.id ? (
                      <Button asChild type="button" size="sm" variant="outline">
                        <Link href={`/admin/auctions/${auction.id}`}>Open</Link>
                      </Button>
                    ) : (
                      <Button type="button" size="sm" variant="outline" disabled>
                        Missing id
                      </Button>
                    )}
                  </AdminTableCell>
                </tr>
              ))}
            </tbody>
          </AdminTable>
        )}
        <AdminTablePagination
          label={`Showing ${auctions.length} auctions`}
          onPrev={() => {
            const offset = Number.parseInt(pageToken || "0", 10);
            const prev = Math.max(0, offset - pageSize);
            setPageToken(prev > 0 ? String(prev) : "");
          }}
          onNext={() => setPageToken(nextPageToken)}
          canPrev={!!pageToken && Number.parseInt(pageToken || "0", 10) > 0}
          canNext={!!nextPageToken}
        />
      </CardContent>
    </Card>
  );
}
