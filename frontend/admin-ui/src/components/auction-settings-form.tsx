"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { getAuctionSettings, updateAuctionSettings } from "@/lib/auction";
import { formatConnectError } from "@/lib/handle-error";

export default function AuctionSettingsForm() {
  const [bidIncrementAmount, setBidIncrementAmount] = useState("100");
  const [feeRatePercent, setFeeRatePercent] = useState("0");
  const [currency, setCurrency] = useState("JPY");
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  async function loadSettings() {
    setIsLoading(true);
    try {
      const res = await getAuctionSettings();
      const settings = res.settings;
      if (settings?.bidIncrement?.amount != null) {
        const amount = typeof settings.bidIncrement.amount === "bigint"
          ? Number(settings.bidIncrement.amount)
          : Number(settings.bidIncrement.amount);
        if (Number.isFinite(amount)) {
          setBidIncrementAmount(String(amount));
        }
        if (settings.bidIncrement.currency) {
          setCurrency(settings.bidIncrement.currency);
        }
      }
      if (typeof settings?.feeRatePercent === "number") {
        setFeeRatePercent(String(settings.feeRatePercent));
      }
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load auction settings");
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
    void loadSettings();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      const bidAmount = Number(bidIncrementAmount);
      const fee = Number(feeRatePercent);
      if (!Number.isFinite(bidAmount)) {
        throw new Error("Bid increment amount must be a number.");
      }
      if (!Number.isFinite(fee)) {
        throw new Error("Fee rate must be a number.");
      }
      await updateAuctionSettings({
        bidIncrementAmount: bidAmount,
        currency,
        feeRatePercent: fee,
      });
      push({
        variant: "success",
        title: "Settings saved",
        description: "Auction settings updated.",
      });
    } catch (err) {
      const uiError = formatConnectError(err, "Save failed", "Failed to update auction settings");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Auction Settings</CardTitle>
        <CardDescription className="text-neutral-500">
          Configure bid increment and fee rate for this store.
        </CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="text-sm text-neutral-600">Loading...</div>
        ) : (
          <form className="grid gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-4 md:grid-cols-3">
              <div className="space-y-2">
                <Label htmlFor="bidIncrement">Bid Increment</Label>
                <Input
                  id="bidIncrement"
                  type="number"
                  value={bidIncrementAmount}
                  onChange={(e) => setBidIncrementAmount(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="currency">Currency</Label>
                <Input
                  id="currency"
                  value={currency}
                  onChange={(e) => setCurrency(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="feeRate">Fee Rate (%)</Label>
                <Input
                  id="feeRate"
                  type="number"
                  value={feeRatePercent}
                  onChange={(e) => setFeeRatePercent(e.target.value)}
                />
              </div>
            </div>
            <div>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? "Saving..." : "Save Settings"}
              </Button>
            </div>
          </form>
        )}
      </CardContent>
    </Card>
  );
}
