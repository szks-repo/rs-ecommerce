"use client";

import { useEffect, useState } from "react";
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
import { listShippingZones, listShippingRates, upsertShippingRate } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import { useToast } from "@/components/ui/toast";
import type { ShippingRate, ShippingZone } from "@/gen/ecommerce/v1/store_settings_pb";

export default function ShippingRatesForm() {
  const [zones, setZones] = useState<ShippingZone[]>([]);
  const [rates, setRates] = useState<ShippingRate[]>([]);
  const [zoneId, setZoneId] = useState("");
  const [selectedRateId, setSelectedRateId] = useState("");
  const [name, setName] = useState("");
  const [minSubtotal, setMinSubtotal] = useState("");
  const [maxSubtotal, setMaxSubtotal] = useState("");
  const [feeAmount, setFeeAmount] = useState("");
  const [currency, setCurrency] = useState("JPY");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  async function loadZones() {
    if (!getActiveAccessToken()) {
      return;
    }
    try {
      const data = await listShippingZones();
      setZones(data.zones ?? []);
    } catch (err) {
      push({
        variant: "error",
        title: "Load failed",
        description: err instanceof Error ? err.message : "Failed to load zones",
      });
    }
  }

  async function loadRates(targetZoneId: string) {
    if (!getActiveAccessToken()) {
      return;
    }
    if (!targetZoneId) {
      setRates([]);
      return;
    }
    try {
      const data = await listShippingRates({ zoneId: targetZoneId });
      setRates(data.rates ?? []);
    } catch (err) {
      push({
        variant: "error",
        title: "Load failed",
        description: err instanceof Error ? err.message : "Failed to load shipping rates",
      });
    }
  }

  useEffect(() => {
    void loadZones();
  }, []);

  useEffect(() => {
    void loadRates(zoneId);
  }, [zoneId]);

  useEffect(() => {
    const found = rates.find((rate) => rate.id === selectedRateId);
    if (!found) {
      return;
    }
    setName(found.name);
    setMinSubtotal(found.minSubtotal?.amount ? found.minSubtotal.amount.toString() : "");
    setMaxSubtotal(found.maxSubtotal?.amount ? found.maxSubtotal.amount.toString() : "");
    setFeeAmount(found.fee?.amount ? found.fee.amount.toString() : "");
    setCurrency(found.fee?.currency || "JPY");
  }, [rates, selectedRateId]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      if (!zoneId) {
        throw new Error("zone is required.");
      }
      if (!/^-?\\d+$/.test(feeAmount.trim())) {
        throw new Error("fee amount must be an integer.");
      }
      if (minSubtotal && !/^-?\\d+$/.test(minSubtotal.trim())) {
        throw new Error("min subtotal must be an integer.");
      }
      if (maxSubtotal && !/^-?\\d+$/.test(maxSubtotal.trim())) {
        throw new Error("max subtotal must be an integer.");
      }
      await upsertShippingRate({
        id: selectedRateId || undefined,
        zoneId,
        name,
        minSubtotal: minSubtotal.trim() || undefined,
        maxSubtotal: maxSubtotal.trim() || undefined,
        feeAmount: feeAmount.trim(),
        currency,
      });
      push({
        variant: "success",
        title: "Shipping rate saved",
        description: "Shipping rate has been saved.",
      });
      setSelectedRateId("");
      setName("");
      setMinSubtotal("");
      setMaxSubtotal("");
      setFeeAmount("");
      await loadRates(zoneId);
    } catch (err) {
      push({
        variant: "error",
        title: "Save failed",
        description: err instanceof Error ? err.message : "Unknown error",
      });
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Shipping Rates</CardTitle>
        <CardDescription className="text-neutral-500">
          Configure rates per shipping zone.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="rateZoneSelect">Zone</Label>
            <Select value={zoneId} onValueChange={setZoneId}>
              <SelectTrigger id="rateZoneSelect" className="bg-white">
                <SelectValue placeholder="Select zone" />
              </SelectTrigger>
              <SelectContent>
                {zones.length === 0 ? (
                  <SelectItem value="none" disabled>
                    No zones yet
                  </SelectItem>
                ) : (
                  zones.map((zone) => (
                    <SelectItem key={zone.id} value={zone.id}>
                      {zone.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="rateSelect">Edit Existing Rate</Label>
            <Select value={selectedRateId} onValueChange={setSelectedRateId}>
              <SelectTrigger id="rateSelect" className="bg-white">
                <SelectValue placeholder="Create new rate" />
              </SelectTrigger>
              <SelectContent>
                {rates.length === 0 ? (
                  <SelectItem value="none" disabled>
                    No rates yet
                  </SelectItem>
                ) : (
                  rates.map((rate) => (
                    <SelectItem key={rate.id} value={rate.id}>
                      {rate.name}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
          </div>
        </div>
        <form className="grid gap-4 md:grid-cols-3" onSubmit={handleSubmit}>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="rateName">Name</Label>
            <Input id="rateName" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="rateCurrency">Currency</Label>
            <Select value={currency} onValueChange={setCurrency}>
              <SelectTrigger id="rateCurrency" className="bg-white">
                <SelectValue placeholder="Currency" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="JPY">JPY</SelectItem>
                <SelectItem value="USD">USD</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="minSubtotal">Min Subtotal</Label>
            <Input id="minSubtotal" value={minSubtotal} onChange={(e) => setMinSubtotal(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="maxSubtotal">Max Subtotal</Label>
            <Input id="maxSubtotal" value={maxSubtotal} onChange={(e) => setMaxSubtotal(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="feeAmount">Fee Amount</Label>
            <Input id="feeAmount" value={feeAmount} onChange={(e) => setFeeAmount(e.target.value)} required />
          </div>
          <div className="md:col-span-3">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Saving..." : "Save Rate"}
            </Button>
          </div>
        </form>
        <div className="space-y-2 text-sm text-neutral-600">
          {rates.length === 0 ? (
            <div>No rates yet.</div>
          ) : (
            rates.map((rate) => (
              <div key={rate.id} className="rounded-lg border border-neutral-200 px-3 py-2">
                <div className="text-neutral-900">{rate.name}</div>
                <div className="text-xs text-neutral-500">
                  fee: {rate.fee?.amount?.toString() ?? "-"} {rate.fee?.currency || ""}
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
