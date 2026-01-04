"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { getCustomer, updateCustomer, upsertCustomerIdentity, upsertCustomerAddress } from "@/lib/customer";
import type { CustomerAddress, CustomerIdentity } from "@/gen/ecommerce/v1/customer_pb";
import { useApiCall } from "@/lib/use-api-call";

const COUNTRY_OPTIONS = [
  { code: "JP", label: "Japan (JP)" },
  { code: "US", label: "United States (US)" },
  { code: "GB", label: "United Kingdom (GB)" },
];

const JP_POSTAL_LOOKUP: Record<string, { prefecture: string; city: string; line1?: string }> = {
  "100-0001": { prefecture: "東京都", city: "千代田区" },
  "150-0001": { prefecture: "東京都", city: "渋谷区" },
  "530-0001": { prefecture: "大阪府", city: "大阪市北区" },
};

function normalizePostalCodeJP(value: string): string | null {
  const digits = value.replace(/[^0-9]/g, "");
  if (digits.length !== 7) {
    return null;
  }
  return `${digits.slice(0, 3)}-${digits.slice(3)}`;
}

export default function CustomerDetailPage() {
  const params = useParams<{ customerId: string }>();
  const router = useRouter();
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const customerId = params.customerId;

  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [notes, setNotes] = useState("");
  const [status, setStatus] = useState("active");
  const [countryCode, setCountryCode] = useState("JP");
  const [customerStatus, setCustomerStatus] = useState("active");
  const [identities, setIdentities] = useState<CustomerIdentity[]>([]);
  const [addresses, setAddresses] = useState<CustomerAddress[]>([]);
  const [identityId, setIdentityId] = useState("");
  const [identityType, setIdentityType] = useState("email");
  const [identityValue, setIdentityValue] = useState("");
  const [identityVerified, setIdentityVerified] = useState(false);
  const [isSavingIdentity, setIsSavingIdentity] = useState(false);
  const [addressId, setAddressId] = useState("");
  const [addressType, setAddressType] = useState("shipping");
  const [addressName, setAddressName] = useState("");
  const [addressPostalCode, setAddressPostalCode] = useState("");
  const [addressPrefecture, setAddressPrefecture] = useState("");
  const [addressCity, setAddressCity] = useState("");
  const [addressLine1, setAddressLine1] = useState("");
  const [addressLine2, setAddressLine2] = useState("");
  const [addressPhone, setAddressPhone] = useState("");
  const [addressCountryCode, setAddressCountryCode] = useState("JP");
  const [isSavingAddress, setIsSavingAddress] = useState(false);

  useEffect(() => {
    async function load() {
      setIsLoading(true);
      try {
        const resp = await getCustomer(customerId);
        const profile = resp.profile;
        setName(profile?.name ?? "");
        setEmail(profile?.email ?? "");
        setPhone(profile?.phone ?? "");
        setNotes(profile?.notes ?? "");
        setStatus(profile?.status ?? "active");
        setCountryCode(profile?.countryCode || "JP");
        setCustomerStatus(resp.customer?.status ?? "active");
        setIdentities(resp.identities ?? []);
        setAddresses(resp.addresses ?? []);
      } catch (err) {
        notifyError(err, "Load failed", "Failed to load customer");
      } finally {
        setIsLoading(false);
      }
    }
    void load();
  }, [customerId, push]);

  async function reloadCustomer() {
    const resp = await getCustomer(customerId);
    setIdentities(resp.identities ?? []);
    setAddresses(resp.addresses ?? []);
  }

  async function handleSave(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSaving) {
      return;
    }
    setIsSaving(true);
    try {
      await updateCustomer({
        customerId,
        name,
        email: email || undefined,
        phone: phone || undefined,
        status,
        notes: notes || undefined,
        countryCode,
        customerStatus,
      });
      push({
        variant: "success",
        title: "Customer updated",
        description: "Customer profile has been saved.",
      });
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update customer");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleSaveIdentity(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSavingIdentity) {
      return;
    }
    setIsSavingIdentity(true);
    try {
      await upsertCustomerIdentity({
        customerId,
        id: identityId || undefined,
        identityType,
        identityValue,
        verified: identityVerified,
      });
      push({
        variant: "success",
        title: "Identity saved",
        description: "Customer identity has been updated.",
      });
      setIdentityId("");
      setIdentityType("email");
      setIdentityValue("");
      setIdentityVerified(false);
      await reloadCustomer();
    } catch (err) {
      notifyError(err, "Identity save failed", "Failed to save identity");
    } finally {
      setIsSavingIdentity(false);
    }
  }

  async function handleSaveAddress(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSavingAddress) {
      return;
    }
    setIsSavingAddress(true);
    try {
      await upsertCustomerAddress({
        customerId,
        id: addressId || undefined,
        type: addressType,
        name: addressName,
        postalCode: addressPostalCode,
        prefecture: addressPrefecture,
        city: addressCity,
        line1: addressLine1,
        line2: addressLine2 || undefined,
        phone: addressPhone || undefined,
        countryCode: addressCountryCode,
      });
      push({
        variant: "success",
        title: "Address saved",
        description: "Customer address has been updated.",
      });
      setAddressId("");
      setAddressType("shipping");
      setAddressName("");
      setAddressPostalCode("");
      setAddressPrefecture("");
      setAddressCity("");
      setAddressLine1("");
      setAddressLine2("");
      setAddressPhone("");
      setAddressCountryCode("JP");
      await reloadCustomer();
    } catch (err) {
      notifyError(err, "Address save failed", "Failed to save address");
    } finally {
      setIsSavingAddress(false);
    }
  }

  if (isLoading) {
    return <div className="text-sm text-neutral-600">Loading customer...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Customers</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Customer Detail</h1>
          <p className="mt-2 text-sm text-neutral-600">Manage profile and identity data.</p>
        </div>
        <Button variant="outline" onClick={() => router.push("/admin/customers")}>
          Back to list
        </Button>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Profile</CardTitle>
          <CardDescription className="text-neutral-500">
            Store-level profile information.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-4" onSubmit={handleSave}>
            <div className="space-y-2">
              <Label htmlFor="profileName">Name</Label>
              <Input
                id="profileName"
                value={name}
                onChange={(event) => setName(event.target.value)}
              />
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="profileEmail">Email</Label>
                <Input
                  id="profileEmail"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="profilePhone">Phone</Label>
                <Input
                  id="profilePhone"
                  value={phone}
                  onChange={(event) => setPhone(event.target.value)}
                />
              </div>
            </div>
            <div className="grid gap-4 md:grid-cols-3">
              <div className="space-y-2">
                <Label>Profile Status</Label>
                <Select value={status} onValueChange={setStatus}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="active">active</SelectItem>
                    <SelectItem value="inactive">inactive</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>Customer Status</Label>
                <Select value={customerStatus} onValueChange={setCustomerStatus}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="active">active</SelectItem>
                    <SelectItem value="inactive">inactive</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>Country</Label>
                <Select value={countryCode} onValueChange={setCountryCode}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select country" />
                  </SelectTrigger>
                  <SelectContent>
                    {COUNTRY_OPTIONS.map((country) => (
                      <SelectItem key={country.code} value={country.code}>
                        {country.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="profileNotes">Notes</Label>
              <Textarea
                id="profileNotes"
                value={notes}
                onChange={(event) => setNotes(event.target.value)}
              />
            </div>
            <Button type="submit" disabled={isSaving}>
              {isSaving ? "Saving..." : "Save profile"}
            </Button>
          </form>
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Identities</CardTitle>
          <CardDescription className="text-neutral-500">
            Linked identifiers for cross-store matching.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 text-sm text-neutral-700">
          <form className="grid gap-3 rounded-lg border border-neutral-200 bg-neutral-50 p-3" onSubmit={handleSaveIdentity}>
            <div className="grid gap-2 md:grid-cols-4">
              <div className="space-y-2">
                <Label>Type</Label>
                <Select value={identityType} onValueChange={setIdentityType}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="email">email</SelectItem>
                    <SelectItem value="phone">phone</SelectItem>
                    <SelectItem value="external">external</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="identityValue">Value</Label>
                <Input
                  id="identityValue"
                  value={identityValue}
                  onChange={(event) => setIdentityValue(event.target.value)}
                  placeholder="email / phone / external id"
                />
              </div>
              <div className="space-y-2">
                <Label>Verified</Label>
                <Select value={identityVerified ? "yes" : "no"} onValueChange={(value) => setIdentityVerified(value === "yes")}>
                  <SelectTrigger className="bg-white">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="yes">yes</SelectItem>
                    <SelectItem value="no">no</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div className="text-xs text-neutral-500">
                {identityId ? `Editing identity ${identityId}` : "Create new identity"}
              </div>
              <div className="flex items-center gap-2">
                {identityId ? (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setIdentityId("");
                      setIdentityType("email");
                      setIdentityValue("");
                      setIdentityVerified(false);
                    }}
                  >
                    Cancel
                  </Button>
                ) : null}
                <Button type="submit" disabled={isSavingIdentity}>
                  {isSavingIdentity ? "Saving..." : "Save identity"}
                </Button>
              </div>
            </div>
          </form>
          {identities.length === 0 ? (
            <div className="text-sm text-neutral-600">No identities recorded.</div>
          ) : (
            identities.map((identity) => (
              <div key={identity.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <div className="font-medium text-neutral-900">
                      {identity.identityType}: {identity.identityValue}
                    </div>
                    <div className="text-xs text-neutral-500">
                      verified: {identity.verified ? "yes" : "no"} / source: {identity.source}
                    </div>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setIdentityId(identity.id);
                      setIdentityType(identity.identityType);
                      setIdentityValue(identity.identityValue);
                      setIdentityVerified(identity.verified);
                    }}
                  >
                    Edit
                  </Button>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Addresses</CardTitle>
          <CardDescription className="text-neutral-500">
            Shipping and billing addresses linked to the customer.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 text-sm text-neutral-700">
          <form className="grid gap-3 rounded-lg border border-neutral-200 bg-neutral-50 p-3" onSubmit={handleSaveAddress}>
            <div className="grid gap-3 md:grid-cols-4">
              <div className="space-y-2">
                <Label>Type</Label>
                <Select value={addressType} onValueChange={setAddressType}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="shipping">shipping</SelectItem>
                    <SelectItem value="billing">billing</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>Country</Label>
                <Select value={addressCountryCode} onValueChange={setAddressCountryCode}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select country" />
                  </SelectTrigger>
                  <SelectContent>
                    {COUNTRY_OPTIONS.map((country) => (
                      <SelectItem key={country.code} value={country.code}>
                        {country.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2 md:col-span-2">
                <Label htmlFor="addressName">Name</Label>
                <Input
                  id="addressName"
                  value={addressName}
                  onChange={(event) => setAddressName(event.target.value)}
                />
              </div>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              <div className="space-y-2">
                <Label htmlFor="postalCode">Postal code</Label>
                <Input
                  id="postalCode"
                  value={addressPostalCode}
                  onChange={(event) => setAddressPostalCode(event.target.value)}
                  onBlur={() => {
                    if (addressCountryCode !== "JP") {
                      return;
                    }
                    const normalized = normalizePostalCodeJP(addressPostalCode);
                    if (!normalized) {
                      return;
                    }
                    setAddressPostalCode(normalized);
                    const hit = JP_POSTAL_LOOKUP[normalized];
                    if (hit) {
                      setAddressPrefecture(hit.prefecture);
                      setAddressCity(hit.city);
                      if (hit.line1) {
                        setAddressLine1(hit.line1);
                      }
                    }
                  }}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="prefecture">
                  {addressCountryCode === "JP" ? "Prefecture" : "State/Region"}
                </Label>
                <Input
                  id="prefecture"
                  value={addressPrefecture}
                  onChange={(event) => setAddressPrefecture(event.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="city">City</Label>
                <Input
                  id="city"
                  value={addressCity}
                  onChange={(event) => setAddressCity(event.target.value)}
                />
              </div>
            </div>
            <div className="grid gap-3 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="line1">Address line 1</Label>
                <Input
                  id="line1"
                  value={addressLine1}
                  onChange={(event) => setAddressLine1(event.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="line2">Address line 2</Label>
                <Input
                  id="line2"
                  value={addressLine2}
                  onChange={(event) => setAddressLine2(event.target.value)}
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="addressPhone">Phone</Label>
              <Input
                id="addressPhone"
                value={addressPhone}
                onChange={(event) => setAddressPhone(event.target.value)}
              />
            </div>
            <div className="flex items-center justify-between">
              <div className="text-xs text-neutral-500">
                {addressId ? `Editing address ${addressId}` : "Create new address"}
              </div>
              <div className="flex items-center gap-2">
                {addressId ? (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setAddressId("");
                      setAddressType("shipping");
                      setAddressName("");
                      setAddressPostalCode("");
                      setAddressPrefecture("");
                      setAddressCity("");
                      setAddressLine1("");
                      setAddressLine2("");
                      setAddressPhone("");
                      setAddressCountryCode("JP");
                    }}
                  >
                    Cancel
                  </Button>
                ) : null}
                <Button type="submit" disabled={isSavingAddress}>
                  {isSavingAddress ? "Saving..." : "Save address"}
                </Button>
              </div>
            </div>
          </form>
          {addresses.length === 0 ? (
            <div className="text-sm text-neutral-600">No addresses recorded.</div>
          ) : (
            addresses.map((address) => (
              <div key={address.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="font-medium text-neutral-900">
                      {address.type.toUpperCase()} - {address.name}
                    </div>
                    <div className="text-xs text-neutral-500">
                      {address.countryCode || "JP"} {address.postalCode} {address.prefecture} {address.city} {address.line1}{" "}
                      {address.line2 || ""}
                    </div>
                    <div className="text-xs text-neutral-500">phone: {address.phone || "-"}</div>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      setAddressId(address.id);
                      setAddressType(address.type);
                      setAddressName(address.name);
                      setAddressPostalCode(address.postalCode);
                      setAddressPrefecture(address.prefecture);
                      setAddressCity(address.city);
                      setAddressLine1(address.line1);
                      setAddressLine2(address.line2 || "");
                      setAddressPhone(address.phone || "");
                      setAddressCountryCode(address.countryCode || "JP");
                    }}
                  >
                    Edit
                  </Button>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <div className="text-sm text-neutral-500">
        Customer ID: {customerId}{" "}
        <Link className="text-neutral-900 underline" href={`/admin/customers/${customerId}`}>
          permalink
        </Link>
      </div>
    </div>
  );
}
