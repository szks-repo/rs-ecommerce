"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { listStoreLocations, upsertStoreLocation } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import type { StoreLocation } from "@/gen/ecommerce/v1/store_settings_pb";

export default function StoreLocationForm() {
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [status, setStatus] = useState("active");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const statusOptions = ["active", "inactive"] as const;
  const { push } = useToast();

  async function loadLocations() {
    if (!getActiveAccessToken()) {
      return;
    }
    const data = await listStoreLocations();
    setLocations(data.locations || []);
  }

  useEffect(() => {
    void loadLocations();
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      await upsertStoreLocation({ code, name, status });
      push({
        variant: "success",
        title: "Location saved",
        description: "Store location has been saved.",
      });
      setCode("");
      setName("");
      setStatus("active");
      await loadLocations();
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
        <CardTitle>Store Locations</CardTitle>
        <CardDescription className="text-neutral-500">
          Configure inventory locations (warehouses).
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="locationCode">Code</Label>
              <Input
                id="locationCode"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="locationName">Name</Label>
              <Input
                id="locationName"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
              />
            </div>
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="locationStatus">Status</Label>
              <Select value={status} onValueChange={setStatus}>
                <SelectTrigger id="locationStatus" className="bg-white">
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  {statusOptions.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Saving..." : "Save Location"}
            </Button>
          </div>
        </form>

        <div className="space-y-2 text-sm text-neutral-600">
          {locations.length === 0 ? (
            <div>No locations yet.</div>
          ) : (
            locations.map((loc) => (
              <div key={loc.id} className="rounded-lg border border-neutral-200 px-3 py-2">
                <div className="text-neutral-900">
                  {loc.code} — {loc.name}
                </div>
                <div className="text-xs text-neutral-500">
                  id: {loc.id} / status: {loc.status}
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
