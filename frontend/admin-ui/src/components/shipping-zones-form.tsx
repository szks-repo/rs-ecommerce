"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { listShippingZones, upsertShippingZone } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import { useToast } from "@/components/ui/toast";
import type { ShippingZone } from "@/gen/ecommerce/v1/store_settings_pb";
import { useApiCall } from "@/lib/use-api-call";

function parsePrefectures(input: string) {
  return input
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0)
    .map((item) => {
      const [code, name] = item.split(":").map((v) => v.trim());
      if (name) {
        return { code, name };
      }
      return { code: code, name: code };
    });
}

export default function ShippingZonesForm() {
  const [zones, setZones] = useState<ShippingZone[]>([]);
  const [selectedZoneId, setSelectedZoneId] = useState("");
  const [name, setName] = useState("");
  const [domesticOnly, setDomesticOnly] = useState(true);
  const [prefectures, setPrefectures] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  async function loadZones() {
    if (!getActiveAccessToken()) {
      return;
    }
    try {
      const data = await listShippingZones();
      setZones(data.zones ?? []);
    } catch (err) {
      notifyError(err, "Load failed", "Failed to load shipping zones");
    }
  }

  useEffect(() => {
    void loadZones();
  }, []);

  useEffect(() => {
    const found = zones.find((zone) => zone.id === selectedZoneId);
    if (!found) {
      return;
    }
    setName(found.name);
    setDomesticOnly(Boolean(found.domesticOnly));
    setPrefectures(
      (found.prefectures ?? [])
        .map((pref) => `${pref.code}:${pref.name}`)
        .join(", ")
    );
  }, [zones, selectedZoneId]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const prefs = parsePrefectures(prefectures);
      await upsertShippingZone({
        id: selectedZoneId || undefined,
        name,
        domesticOnly,
        prefectures: prefs,
      });
      push({
        variant: "success",
        title: "Shipping zone saved",
        description: "Shipping zone has been saved.",
      });
      setSelectedZoneId("");
      setName("");
      setDomesticOnly(true);
      setPrefectures("");
      await loadZones();
    } catch (err) {
      notifyError(err, "Save failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Shipping Zones</CardTitle>
        <CardDescription className="text-neutral-500">
          Define delivery areas and prefectures.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="zoneSelect">Edit Existing Zone</Label>
          <Select value={selectedZoneId} onValueChange={setSelectedZoneId}>
            <SelectTrigger id="zoneSelect" className="bg-white">
              <SelectValue placeholder="Create new zone" />
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
        <form className="grid gap-4 md:grid-cols-3" onSubmit={handleSubmit}>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="zoneName">Name</Label>
            <Input id="zoneName" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>
          <div className="flex items-center justify-between rounded-md border border-neutral-200 px-3 py-2">
            <div>
              <Label htmlFor="domesticOnly">Domestic Only</Label>
              <p className="text-xs text-neutral-500">Limit to Japan.</p>
            </div>
            <Switch id="domesticOnly" checked={domesticOnly} onCheckedChange={setDomesticOnly} />
          </div>
          <div className="space-y-2 md:col-span-3">
            <Label htmlFor="prefectures">Prefectures (code:name, comma separated)</Label>
            <Input
              id="prefectures"
              value={prefectures}
              onChange={(e) => setPrefectures(e.target.value)}
              placeholder="JP-13:Tokyo, JP-27:Osaka"
            />
          </div>
          <div className="md:col-span-3">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Saving..." : "Save Zone"}
            </Button>
          </div>
        </form>
        <div className="space-y-2 text-sm text-neutral-600">
          {zones.length === 0 ? (
            <div>No zones yet.</div>
          ) : (
            zones.map((zone) => (
              <div key={zone.id} className="rounded-lg border border-neutral-200 px-3 py-2">
                <div className="text-neutral-900">{zone.name}</div>
                <div className="text-xs text-neutral-500">
                  {zone.domesticOnly ? "Domestic" : "Global"} /{" "}
                  {(zone.prefectures ?? []).length} prefectures
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
