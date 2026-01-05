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
        <div className="flex w-full flex-col gap-2 md:w-auto md:flex-row md:items-center">
          <Select
            value={status}
            onValueChange={(value) => {
              setStatus(value);
              setPageToken("");
            }}
          >
            <SelectTrigger className="bg-white">
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
            <SelectTrigger className="bg-white">
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
          <Button variant="outline" onClick={() => loadAuctions()} disabled={isLoading}>
            {isLoading ? "Loading..." : "Refresh"}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {auctions.length === 0 ? (
          <div className="text-sm text-neutral-600">No auctions found.</div>
        ) : (
          <div className="overflow-hidden rounded-lg border border-neutral-200 bg-white">
            <div className="max-h-[520px] overflow-auto">
              <table className="min-w-full text-sm">
                <thead className="sticky top-0 bg-neutral-50 text-xs uppercase text-neutral-500">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">Auction</th>
                    <th className="px-3 py-2 text-left font-medium">Status</th>
                    <th className="px-3 py-2 text-left font-medium">Schedule</th>
                    <th className="px-3 py-2 text-left font-medium">Price</th>
                    <th className="px-3 py-2 text-left font-medium">Created</th>
                    <th className="px-3 py-2 text-right font-medium">Detail</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-neutral-200">
                  {auctions.map((auction, index) => (
                    <tr key={auction.id || `${auction.skuId}-${index}`} className="align-top">
                      <td className="px-3 py-2">
                        <div className="text-sm font-medium text-neutral-900">
                          {auction.title || "(Untitled auction)"}
                        </div>
                        <div className="text-[11px] text-neutral-500">
                          id: {auction.id || "-"} / sku: {auction.skuId || "-"}
                        </div>
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-600">
                        <div>{auction.status}</div>
                        <div className="text-neutral-400">{auction.auctionType}</div>
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-600">
                        <div>
                          <DateCell value={toIsoString(auction.startAt)} />
                        </div>
                        <div>
                          <DateCell value={toIsoString(auction.endAt)} />
                        </div>
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-600">
                        <div>Start: {formatMoney(auction.startPrice)}</div>
                        <div>Current: {formatMoney(auction.currentPrice)}</div>
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-500">
                        <DateCell value={toIsoString(auction.createdAt)} />
                      </td>
                      <td className="px-3 py-2 text-right">
                        {auction.id ? (
                          <Button asChild type="button" size="sm" variant="outline">
                            <Link href={`/admin/auctions/${auction.id}`}>Open</Link>
                          </Button>
                        ) : (
                          <Button type="button" size="sm" variant="outline" disabled>
                            Missing id
                          </Button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
        <div className="mt-4 flex flex-wrap items-center justify-between gap-2 text-sm">
          <div className="text-neutral-500">
            Showing {auctions.length} auctions
          </div>
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => {
                const offset = Number.parseInt(pageToken || "0", 10);
                const prev = Math.max(0, offset - pageSize);
                setPageToken(prev > 0 ? String(prev) : "");
              }}
              disabled={!pageToken || Number.parseInt(pageToken || "0", 10) <= 0}
            >
              Prev
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setPageToken(nextPageToken)}
              disabled={!nextPageToken}
            >
              Next
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
