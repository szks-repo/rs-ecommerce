"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";
import { approveAuction, closeAuction, getAuction, listAutoBids, listBids, updateAuction } from "@/lib/auction";
import { formatMoney } from "@/lib/money";
import { formatTimestampWithStoreTz } from "@/lib/time";
import type { Auction, AuctionAutoBid, AuctionBid } from "@/gen/ecommerce/v1/auction_pb";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export default function AuctionDetail({ auctionId }: { auctionId: string }) {
  const [auction, setAuction] = useState<Auction | null>(null);
  const [bids, setBids] = useState<AuctionBid[]>([]);
  const [autoBids, setAutoBids] = useState<AuctionAutoBid[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isActionLoading, setIsActionLoading] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editDescription, setEditDescription] = useState("");
  const [editAuctionType, setEditAuctionType] = useState("open");
  const [editStartAt, setEditStartAt] = useState("");
  const [editEndAt, setEditEndAt] = useState("");
  const [editStartPrice, setEditStartPrice] = useState("");
  const [editReservePrice, setEditReservePrice] = useState("");
  const [editBuyoutPrice, setEditBuyoutPrice] = useState("");
  const [editBidIncrement, setEditBidIncrement] = useState("");
  const { push } = useToast();
  const { call } = useApiCall();
  const isValidId = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(
    auctionId
  );

  async function loadAuction() {
    if (!isValidId) {
      setAuction(null);
      setBids([]);
      setAutoBids([]);
      push({
        variant: "error",
        title: "Invalid auction ID",
        description: `Auction id "${auctionId}" is not a valid UUID.`,
      });
      return;
    }
    setIsLoading(true);
    const result = await call(
      async () => {
        const [auctionRes, bidsRes, autoRes] = await Promise.all([
          getAuction({ auctionId }),
          listBids({ auctionId }),
          listAutoBids({ auctionId }),
        ]);
        return {
          auction: auctionRes.auction ?? null,
          bids: bidsRes.bids ?? [],
          autoBids: autoRes.autoBids ?? [],
        };
      },
      { errorTitle: "Load failed", errorDescription: "Failed to load auction" }
    );
    if (result) {
      setAuction(result.auction);
      setBids(result.bids);
      setAutoBids(result.autoBids);
      if (result.auction?.status === "draft") {
        const start = result.auction.startAt?.seconds
          ? new Date(Number(result.auction.startAt.seconds) * 1000 + Number(result.auction.startAt.nanos || 0) / 1e6)
              .toISOString()
              .slice(0, 16)
          : "";
        const end = result.auction.endAt?.seconds
          ? new Date(Number(result.auction.endAt.seconds) * 1000 + Number(result.auction.endAt.nanos || 0) / 1e6)
              .toISOString()
              .slice(0, 16)
          : "";
        setEditTitle(result.auction.title || "");
        setEditDescription(result.auction.description || "");
        setEditAuctionType(result.auction.auctionType || "open");
        setEditStartAt(start);
        setEditEndAt(end);
        setEditStartPrice(result.auction.startPrice?.amount?.toString() || "");
        setEditReservePrice(result.auction.reservePrice?.amount?.toString() || "");
        setEditBuyoutPrice(result.auction.buyoutPrice?.amount?.toString() || "");
        setEditBidIncrement(result.auction.bidIncrement?.amount?.toString() || "");
      }
    }
    setIsLoading(false);
  }

  useEffect(() => {
    void loadAuction();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auctionId]);

  async function handleClose() {
    setIsActionLoading(true);
    const res = await call(
      () => closeAuction({ auctionId }),
      {
        success: {
          title: "Auction closed",
          description: "Auction has been closed and waiting for approval if eligible.",
        },
        errorTitle: "Close failed",
        errorDescription: "Failed to close auction",
      }
    );
    if (res?.auction) {
      setAuction(res.auction ?? null);
      await loadAuction();
    }
    setIsActionLoading(false);
  }

  async function handleApprove() {
    setIsActionLoading(true);
    const res = await call(
      () => approveAuction({ auctionId }),
      {
        success: {
          title: "Auction approved",
          description: "Winning bid has been approved.",
        },
        errorTitle: "Approve failed",
        errorDescription: "Failed to approve auction",
      }
    );
    if (res?.auction) {
      setAuction(res.auction ?? null);
      await loadAuction();
    }
    setIsActionLoading(false);
  }

  const canApprove = auction?.status === "awaiting_approval";
  const canClose = auction?.status === "running" || auction?.status === "scheduled";
  const canEdit = auction?.status === "draft";

  async function handleUpdate(nextStatus: "draft" | "scheduled") {
    if (!auction) {
      return;
    }
    setIsUpdating(true);
    try {
      const start = new Date(editStartAt);
      const end = new Date(editEndAt);
      if (!Number.isFinite(start.getTime()) || !Number.isFinite(end.getTime())) {
        throw new Error("Start/End time is required.");
      }
      const startAmount = Number(editStartPrice);
      const bidIncrementAmount = Number(editBidIncrement);
      const reserveAmount =
        editReservePrice.trim().length > 0 ? Number(editReservePrice) : undefined;
      const buyoutAmount =
        editBuyoutPrice.trim().length > 0 ? Number(editBuyoutPrice) : undefined;
      const resp = await updateAuction({
        auctionId: auction.id,
        skuId: auction.skuId,
        title: editTitle,
        description: editDescription,
        auctionType: editAuctionType,
        status: nextStatus,
        startAt: start,
        endAt: end,
        startPriceAmount: startAmount,
        reservePriceAmount: reserveAmount,
        buyoutPriceAmount: buyoutAmount,
        bidIncrementAmount: bidIncrementAmount,
        currency: auction.startPrice?.currency || "JPY",
      });
      push({
        variant: "success",
        title: nextStatus === "draft" ? "Draft updated" : "Auction published",
        description:
          nextStatus === "draft"
            ? "Draft auction has been updated."
            : "Auction has been scheduled.",
      });
      if (resp.auction) {
        setAuction(resp.auction);
      }
      await loadAuction();
    } catch (err) {
      push({
        variant: "error",
        title: "Update failed",
        description: err instanceof Error ? err.message : "Failed to update auction",
      });
    } finally {
      setIsUpdating(false);
    }
  }

  async function handlePublish() {
    if (typeof window !== "undefined") {
      const confirmed = window.confirm("Publish this auction now?");
      if (!confirmed) {
        return;
      }
    }
    await handleUpdate("scheduled");
  }

  return (
    <div className="space-y-6">
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Auction Details</CardTitle>
          <CardDescription className="text-neutral-500">
            Review auction status and approval workflow.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {!isValidId ? (
            <div className="text-sm text-neutral-600">Invalid auction id.</div>
          ) : isLoading ? (
            <div className="text-sm text-neutral-600">Loading...</div>
          ) : auction ? (
            <div className="space-y-3 text-sm text-neutral-700">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <div className="text-base font-semibold text-neutral-900">
                    {auction.title || auction.id}
                  </div>
                  {auction.description && (
                    <div className="text-xs text-neutral-500">{auction.description}</div>
                  )}
                  <div className="text-xs text-neutral-500">
                    product: {auction.productId || "-"} / sku: {auction.skuId || "-"}
                  </div>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button variant="outline" onClick={handleClose} disabled={!canClose || isActionLoading}>
                    {isActionLoading ? "Processing..." : "Close Auction"}
                  </Button>
                  <Button onClick={handleApprove} disabled={!canApprove || isActionLoading}>
                    {isActionLoading ? "Processing..." : "Approve"}
                  </Button>
                </div>
              </div>
              <div className="grid gap-2 md:grid-cols-2">
                <div>
                  <div className="text-xs text-neutral-500">Type / Status</div>
                  <div>{auction.auctionType} / {auction.status}</div>
                </div>
                <div>
                  <div className="text-xs text-neutral-500">Schedule</div>
                  <div>
                    {formatTimestampWithStoreTz(auction.startAt?.seconds, auction.startAt?.nanos)}
                    {" - "}
                    {formatTimestampWithStoreTz(auction.endAt?.seconds, auction.endAt?.nanos)}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-neutral-500">Start / Current</div>
                  <div>
                    {formatMoney(auction.startPrice)} / {formatMoney(auction.currentPrice)}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-neutral-500">Reserve / Buyout</div>
                  <div>
                    {formatMoney(auction.reservePrice)} / {formatMoney(auction.buyoutPrice)}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-neutral-500">Winning Bid</div>
                  <div>
                    {auction.winningBidId || "-"} / {formatMoney(auction.winningPrice)}
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="text-sm text-neutral-600">Auction not found.</div>
          )}
        </CardContent>
      </Card>

      {canEdit && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Edit Draft</CardTitle>
            <CardDescription className="text-neutral-500">
              Update draft details before publishing.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="editTitle">Title</Label>
                <Input id="editTitle" value={editTitle} onChange={(e) => setEditTitle(e.target.value)} />
              </div>
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="editDescription">Description</Label>
                <Textarea id="editDescription" value={editDescription} onChange={(e) => setEditDescription(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label>Auction Type</Label>
                <Select value={editAuctionType} onValueChange={setEditAuctionType}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="open">open</SelectItem>
                    <SelectItem value="sealed">sealed</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>SKU</Label>
                <Input value={auction?.skuId || ""} disabled />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editStartAt">Start At</Label>
                <Input id="editStartAt" type="datetime-local" value={editStartAt} onChange={(e) => setEditStartAt(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editEndAt">End At</Label>
                <Input id="editEndAt" type="datetime-local" value={editEndAt} onChange={(e) => setEditEndAt(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editStartPrice">Start Price</Label>
                <Input id="editStartPrice" type="number" value={editStartPrice} onChange={(e) => setEditStartPrice(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editBidIncrement">Bid Increment</Label>
                <Input id="editBidIncrement" type="number" value={editBidIncrement} onChange={(e) => setEditBidIncrement(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editReservePrice">Reserve Price (optional)</Label>
                <Input id="editReservePrice" type="number" value={editReservePrice} onChange={(e) => setEditReservePrice(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="editBuyoutPrice">Buyout Price (optional)</Label>
                <Input id="editBuyoutPrice" type="number" value={editBuyoutPrice} onChange={(e) => setEditBuyoutPrice(e.target.value)} />
              </div>
            </div>
            <div className="mt-4 flex flex-wrap gap-2">
              <Button type="button" variant="outline" onClick={() => handleUpdate("draft")} disabled={isUpdating}>
                {isUpdating ? "Saving..." : "Save Draft"}
              </Button>
              <Button type="button" onClick={handlePublish} disabled={isUpdating}>
                {isUpdating ? "Publishing..." : "Publish"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Bids</CardTitle>
          <CardDescription className="text-neutral-500">Bidding history for this auction.</CardDescription>
        </CardHeader>
        <CardContent>
          {bids.length === 0 ? (
            <div className="text-sm text-neutral-600">No bids yet.</div>
          ) : (
            <div className="space-y-2 text-sm text-neutral-700">
              {bids.map((bid) => (
                <div key={bid.id} className="rounded-lg border border-neutral-200 p-3">
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div>
                      <div className="font-medium text-neutral-900">{formatMoney(bid.amount)}</div>
                      <div className="text-xs text-neutral-500">customer: {bid.customerId || "-"}</div>
                    </div>
                    <div className="text-xs text-neutral-500">
                      {formatTimestampWithStoreTz(bid.createdAt?.seconds, bid.createdAt?.nanos)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Auto-bid settings (read-only)</CardTitle>
          <CardDescription className="text-neutral-500">
            Auto-bid configuration is managed by end users. This view is for auditing only.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {autoBids.length === 0 ? (
            <div className="text-sm text-neutral-600">No auto-bid rules registered.</div>
          ) : (
            <div className="space-y-2 text-sm text-neutral-700">
              {autoBids.map((autoBid) => (
                <div key={autoBid.id} className="rounded-lg border border-neutral-200 p-3">
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div>
                      <div className="font-medium text-neutral-900">
                        max: {formatMoney(autoBid.maxAmount)}
                      </div>
                      <div className="text-xs text-neutral-500">customer: {autoBid.customerId || "-"}</div>
                      <div className="text-xs text-neutral-500">status: {autoBid.status || "-"}</div>
                    </div>
                    <div className="text-xs text-neutral-500">
                      updated: {formatTimestampWithStoreTz(autoBid.updatedAt?.seconds, autoBid.updatedAt?.nanos)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
