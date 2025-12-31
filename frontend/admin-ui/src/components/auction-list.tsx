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
import { useToast } from "@/components/ui/toast";
import { listAuctions } from "@/lib/auction";
import { formatConnectError } from "@/lib/handle-error";
import { formatTimestampWithStoreTz } from "@/lib/time";
import type { Auction } from "@/gen/ecommerce/v1/auction_pb";

const statusOptions = [
  "all",
  "draft",
  "scheduled",
  "running",
  "ended",
  "awaiting_approval",
  "approved",
] as const;

function formatMoney(money?: { amount?: string | number | bigint; currency?: string }) {
  if (!money || money.amount == null) {
    return "-";
  }
  const amount = typeof money.amount === "bigint" ? Number(money.amount) : Number(money.amount);
  if (!Number.isFinite(amount)) {
    return "-";
  }
  return `${amount.toLocaleString("ja-JP")} ${money.currency || ""}`.trim();
}

export default function AuctionList() {
  const [status, setStatus] = useState<string>("all");
  const [auctions, setAuctions] = useState<Auction[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  async function loadAuctions(nextStatus?: string) {
    setIsLoading(true);
    try {
      const selected = (nextStatus ?? status) === "all" ? "" : nextStatus ?? status;
      const data = await listAuctions({ status: selected });
      setAuctions(data.auctions ?? []);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load auctions");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadAuctions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
          <Select value={status} onValueChange={(value) => setStatus(value)}>
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
          <Button variant="outline" onClick={() => loadAuctions()} disabled={isLoading}>
            {isLoading ? "Loading..." : "Apply"}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {auctions.length === 0 ? (
          <div className="text-sm text-neutral-600">No auctions found.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {auctions.map((auction, index) => (
              <div key={auction.id || `${auction.skuId}-${index}`} className="rounded-lg border border-neutral-200 p-4">
                <div className="flex flex-wrap items-start justify-between gap-4">
                  <div className="space-y-1">
                    <div className="text-base font-semibold text-neutral-900">
                      {auction.title || (auction.productId ? `Product ${auction.productId}` : "(No product)")}
                    </div>
                    {auction.description && (
                      <div className="text-xs text-neutral-500">{auction.description}</div>
                    )}
                    <div className="text-xs text-neutral-500">
                      id: {auction.id || "-"} / sku: {auction.skuId || "-"}
                    </div>
                    <div className="text-xs text-neutral-500">
                      {auction.auctionType} / {auction.status}
                    </div>
                    <div className="text-xs text-neutral-600">
                      {formatTimestampWithStoreTz(auction.startAt?.seconds, auction.startAt?.nanos)}
                      {" - "}
                      {formatTimestampWithStoreTz(auction.endAt?.seconds, auction.endAt?.nanos)}
                    </div>
                    <div className="text-xs text-neutral-600">
                      Start: {formatMoney(auction.startPrice)} / Current:{" "}
                      {formatMoney(auction.currentPrice)}
                    </div>
                  </div>
                  <div className="flex flex-col gap-2 text-xs">
                    {auction.id ? (
                      <Link
                        className="rounded-md border border-neutral-200 px-3 py-1 text-center font-medium text-neutral-700 hover:bg-neutral-50"
                        href={`/admin/auctions/${auction.id}`}
                      >
                        Details
                      </Link>
                    ) : (
                      <div className="rounded-md border border-neutral-200 px-3 py-1 text-center font-medium text-neutral-400">
                        Missing id
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
