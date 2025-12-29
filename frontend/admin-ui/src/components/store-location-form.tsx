"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { listStoreLocations, upsertStoreLocation } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import type { StoreLocation } from "@/gen/ecommerce/v1/store_settings_pb";

export default function StoreLocationForm() {
  const [locations, setLocations] = useState<StoreLocation[]>([]);
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [status, setStatus] = useState("active");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

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
    setError(null);
    setMessage(null);
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      await upsertStoreLocation({ code, name, status });
      setMessage("Location saved.");
      setCode("");
      setName("");
      setStatus("active");
      await loadLocations();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
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
        {error && (
          <Alert>
            <AlertTitle>Save failed</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {message && (
          <Alert>
            <AlertTitle>Success</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        )}
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
              <Input
                id="locationStatus"
                value={status}
                onChange={(e) => setStatus(e.target.value)}
              />
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
