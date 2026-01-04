"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
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
import { useToast } from "@/components/ui/toast";
import { createAuction } from "@/lib/auction";
import { listSkusAdmin } from "@/lib/product";
import type { SkuAdmin } from "@/gen/ecommerce/v1/backoffice_pb";
import { Textarea } from "@/components/ui/textarea";
import { useApiCall } from "@/lib/use-api-call";

const auctionTypes = ["open", "sealed"] as const;

function parseNumber(value: string) {
  if (value.trim() === "") {
    return undefined;
  }
  const num = Number(value);
  if (!Number.isFinite(num)) {
    return undefined;
  }
  return num;
}

export default function AuctionCreateForm() {
  const router = useRouter();
  const [skuId, setSkuId] = useState("");
  const [skuQuery, setSkuQuery] = useState("");
  const [skuResults, setSkuResults] = useState<SkuAdmin[]>([]);
  const [selectedSku, setSelectedSku] = useState<SkuAdmin | null>(null);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [auctionType, setAuctionType] = useState<(typeof auctionTypes)[number]>("open");
  const [startAt, setStartAt] = useState("");
  const [endAt, setEndAt] = useState("");
  const [startPriceAmount, setStartPriceAmount] = useState("");
  const [reservePriceAmount, setReservePriceAmount] = useState("");
  const [buyoutPriceAmount, setBuyoutPriceAmount] = useState("");
  const [bidIncrementAmount, setBidIncrementAmount] = useState("");
  const [startTouched, setStartTouched] = useState(false);
  const [reserveTouched, setReserveTouched] = useState(false);
  const [buyoutTouched, setBuyoutTouched] = useState(false);
  const [incrementTouched, setIncrementTouched] = useState(false);
  const [submitAttempted, setSubmitAttempted] = useState(false);
  const [currency, setCurrency] = useState("JPY");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSearching, setIsSearching] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  const startAmount = parseNumber(startPriceAmount);
  const reserveAmount = parseNumber(reservePriceAmount);
  const buyoutAmount = parseNumber(buyoutPriceAmount);
  const incrementAmount = parseNumber(bidIncrementAmount);
  const startError =
    startPriceAmount.trim().length === 0
      ? "Start price is required."
      : startAmount == null
        ? "Start price must be a number."
        : startAmount < 0
          ? "Start price must be zero or positive."
          : null;
  const reserveError =
    reservePriceAmount.trim().length === 0
      ? null
      : reserveAmount == null
        ? "Reserve price must be a number."
        : reserveAmount < 0
          ? "Reserve price must be zero or positive."
          : null;
  const buyoutError =
    buyoutPriceAmount.trim().length === 0
      ? null
      : buyoutAmount == null
        ? "Buyout price must be a number."
        : buyoutAmount < 0
          ? "Buyout price must be zero or positive."
          : null;
  const incrementError =
    bidIncrementAmount.trim().length === 0
      ? null
      : incrementAmount == null
        ? "Bid increment must be a number."
        : incrementAmount < 0
          ? "Bid increment must be zero or positive."
          : null;

  async function handleSkuSearch() {
    setIsSearching(true);
    try {
      const res = await listSkusAdmin({ query: skuQuery });
      setSkuResults(res.skus ?? []);
    } catch (err) {
      notifyError(err, "Search failed", "Failed to search SKUs");
    } finally {
      setIsSearching(false);
    }
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setSubmitAttempted(true);
    setIsSubmitting(true);
    try {
      if (!skuId) {
        throw new Error("SKU selection is required.");
      }
      const start = new Date(startAt);
      const end = new Date(endAt);
      if (!Number.isFinite(start.getTime()) || !Number.isFinite(end.getTime())) {
        throw new Error("Start/End time is required.");
      }
      if (startError || reserveError || buyoutError || incrementError) {
        throw new Error("Please fix validation errors.");
      }
      if (startAmount == null) {
        throw new Error("Start price is required.");
      }
      const resp = await createAuction({
        skuId,
        title,
        description,
        auctionType,
        status: "draft",
        startAt: start,
        endAt: end,
        startPriceAmount: startAmount,
        reservePriceAmount: reserveAmount ?? undefined,
        buyoutPriceAmount: buyoutAmount ?? undefined,
        bidIncrementAmount: incrementAmount ?? undefined,
        currency,
      });
      push({
        variant: "success",
        title: "Auction created",
        description: "Auction has been created successfully.",
      });
      const auctionId = resp.auction?.id;
      if (auctionId) {
        router.push(`/admin/auctions/${auctionId}`);
        return;
      }
      setSkuId("");
      setSkuQuery("");
      setSkuResults([]);
      setSelectedSku(null);
      setTitle("");
      setDescription("");
      setAuctionType("open");
      setStartAt("");
      setEndAt("");
      setStartPriceAmount("");
      setReservePriceAmount("");
      setBuyoutPriceAmount("");
      setBidIncrementAmount("");
    } catch (err) {
      notifyError(err, "Create failed", "Failed to create auction");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Auction</CardTitle>
        <CardDescription className="text-neutral-500">
          Configure auction rules and schedule bidding windows.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-3">
            <div className="text-sm font-semibold text-neutral-800">SKU</div>
            <div className="grid gap-4 md:grid-cols-[1fr_auto]">
              <div className="space-y-2">
                <Label htmlFor="skuSearch">Search SKU (store scope)</Label>
                <Input
                  id="skuSearch"
                  value={skuQuery}
                  onChange={(e) => setSkuQuery(e.target.value)}
                  placeholder="Search by SKU or product title"
                />
              </div>
              <div className="flex items-end">
                <Button type="button" variant="outline" onClick={handleSkuSearch} disabled={isSearching}>
                  {isSearching ? "Searching..." : "Search"}
                </Button>
              </div>
            </div>
            {skuId ? (
              <div className="rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm text-neutral-700">
                <div className="text-xs uppercase tracking-widest text-neutral-500">Selected</div>
                <div className="mt-1 font-medium text-neutral-900">
                  {selectedSku?.sku || "SKU"}
                </div>
                <div className="text-xs text-neutral-600">
                  {selectedSku?.productTitle || "-"} / {selectedSku?.fulfillmentType || "-"} /{" "}
                  {selectedSku?.status || "-"}
                </div>
                <div className="mt-1 text-xs text-neutral-500">{skuId}</div>
              </div>
            ) : (
              <div className="text-xs text-neutral-500">Select a SKU from the search results.</div>
            )}
          </div>

          {skuResults.length > 0 && (
            <div className="rounded-lg border border-neutral-200 bg-neutral-50 p-3 text-sm text-neutral-700">
              <div className="mb-2 text-xs uppercase tracking-widest text-neutral-500">Results</div>
              <div className="space-y-2">
                {skuResults.map((result) => (
                  <button
                    key={result.id}
                    type="button"
                    className="flex w-full items-center justify-between rounded-md border border-neutral-200 bg-white px-3 py-2 text-left transition hover:border-neutral-300"
                    onClick={() => {
                      setSkuId(result.id);
                      setSelectedSku(result);
                    }}
                  >
                    <div>
                      <div className="text-sm font-medium text-neutral-900">{result.sku}</div>
                      <div className="text-xs text-neutral-500">
                        {result.productTitle} / {result.fulfillmentType} / {result.status}
                      </div>
                    </div>
                    <span className="text-xs text-neutral-500">{result.id}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="auctionTitle">Title</Label>
              <Input
                id="auctionTitle"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="auctionDescription">Description</Label>
              <Textarea
                id="auctionDescription"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional"
              />
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-3">
            <div className="space-y-2">
              <Label>Auction Type</Label>
              <Select value={auctionType} onValueChange={(value) => setAuctionType(value as typeof auctionType)}>
                <SelectTrigger className="bg-white">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  {auctionTypes.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="currency">Currency</Label>
              <Input id="currency" value={currency} onChange={(e) => setCurrency(e.target.value)} />
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="startAt">Start At</Label>
              <Input id="startAt" type="datetime-local" value={startAt} onChange={(e) => setStartAt(e.target.value)} required />
            </div>
            <div className="space-y-2">
              <Label htmlFor="endAt">End At</Label>
              <Input id="endAt" type="datetime-local" value={endAt} onChange={(e) => setEndAt(e.target.value)} required />
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="startPrice">Start Price</Label>
              <Input
                id="startPrice"
                type="number"
                value={startPriceAmount}
                onChange={(e) => setStartPriceAmount(e.target.value)}
                onBlur={() => setStartTouched(true)}
                required
              />
              {(startTouched || submitAttempted) && startError ? (
                <p className="text-xs text-red-600">{startError}</p>
              ) : null}
            </div>
            <div className="space-y-2">
              <Label htmlFor="bidIncrement">Bid Increment (optional)</Label>
              <Input
                id="bidIncrement"
                type="number"
                value={bidIncrementAmount}
                onChange={(e) => setBidIncrementAmount(e.target.value)}
                onBlur={() => setIncrementTouched(true)}
              />
              {(incrementTouched || submitAttempted) && incrementError ? (
                <p className="text-xs text-red-600">{incrementError}</p>
              ) : null}
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="reservePrice">Reserve Price (optional)</Label>
              <Input
                id="reservePrice"
                type="number"
                value={reservePriceAmount}
                onChange={(e) => setReservePriceAmount(e.target.value)}
                onBlur={() => setReserveTouched(true)}
              />
              {(reserveTouched || submitAttempted) && reserveError ? (
                <p className="text-xs text-red-600">{reserveError}</p>
              ) : null}
            </div>
            <div className="space-y-2">
              <Label htmlFor="buyoutPrice">Buyout Price (optional)</Label>
              <Input
                id="buyoutPrice"
                type="number"
                value={buyoutPriceAmount}
                onChange={(e) => setBuyoutPriceAmount(e.target.value)}
                onBlur={() => setBuyoutTouched(true)}
              />
              {(buyoutTouched || submitAttempted) && buyoutError ? (
                <p className="text-xs text-red-600">{buyoutError}</p>
              ) : null}
            </div>
          </div>

          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Auction"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
