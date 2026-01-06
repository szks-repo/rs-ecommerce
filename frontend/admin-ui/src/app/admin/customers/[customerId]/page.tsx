"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import {
  getCustomer,
  updateCustomer,
  upsertCustomerIdentity,
  upsertCustomerAddress,
  listCustomerMetafieldDefinitions,
  listCustomerMetafieldValues,
  upsertCustomerMetafieldValue,
} from "@/lib/customer";
import type {
  CustomerAddress,
  CustomerIdentity,
  MetafieldDefinition,
} from "@/gen/ecommerce/v1/customer_pb";
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

type MetafieldValueState = string | string[] | boolean;

function normalizeMetafieldValue(valueJson?: string): MetafieldValueState {
  if (!valueJson) {
    return "";
  }
  try {
    const parsed = JSON.parse(valueJson);
    if (Array.isArray(parsed)) {
      return parsed.map((item) => String(item));
    }
    if (typeof parsed === "boolean") {
      return parsed;
    }
    if (typeof parsed === "string") {
      return parsed;
    }
    if (typeof parsed === "number") {
      return String(parsed);
    }
    return JSON.stringify(parsed);
  } catch {
    return valueJson;
  }
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
  const [metafieldDefs, setMetafieldDefs] = useState<MetafieldDefinition[]>([]);
  const [metafieldValues, setMetafieldValues] = useState<Record<string, MetafieldValueState>>({});
  const [isSavingMetafield, setIsSavingMetafield] = useState<string | null>(null);

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
        const [defsResp, valuesResp] = await Promise.all([
          listCustomerMetafieldDefinitions(),
          listCustomerMetafieldValues(customerId),
        ]);
        setMetafieldDefs(defsResp.definitions ?? []);
        const valuesMap: Record<string, MetafieldValueState> = {};
        (valuesResp.values ?? []).forEach((value) => {
          if (value.definitionId) {
            valuesMap[value.definitionId] = normalizeMetafieldValue(value.valueJson);
          }
        });
        setMetafieldValues(valuesMap);
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
    const valuesResp = await listCustomerMetafieldValues(customerId);
    const valuesMap: Record<string, MetafieldValueState> = {};
    (valuesResp.values ?? []).forEach((value) => {
      if (value.definitionId) {
        valuesMap[value.definitionId] = normalizeMetafieldValue(value.valueJson);
      }
    });
    setMetafieldValues(valuesMap);
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

  function validateMetafieldValue(definition: MetafieldDefinition, rawValue: MetafieldValueState) {
    let validations: { min?: unknown; max?: unknown; required?: unknown } = {};
    if (definition.validationsJson) {
      try {
        validations = JSON.parse(definition.validationsJson);
      } catch {
        validations = {};
      }
    }
    const required = validations.required === true;
    const isDateType = definition.valueType === "date" || definition.valueType === "dateTime";
    const isBooleanType =
      definition.valueType === "bool" || definition.valueType === "boolean";
    const isNumberType = definition.valueType === "number";

    const isEmpty =
      rawValue == null ||
      (typeof rawValue === "string" && rawValue.trim() === "") ||
      (Array.isArray(rawValue) && rawValue.length === 0);

    if (required && isEmpty) {
      return "This field is required.";
    }

    if (isBooleanType) {
      return null;
    }

    if (isDateType) {
      const minValue = typeof validations.min === "string" ? validations.min : undefined;
      const maxValue = typeof validations.max === "string" ? validations.max : undefined;
      const parseDate = (value: string) => {
        if (definition.valueType === "date") {
          return new Date(`${value}T00:00:00`);
        }
        return new Date(value);
      };
      const minDate = minValue ? parseDate(minValue) : null;
      const maxDate = maxValue ? parseDate(maxValue) : null;
      const values: string[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => String(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of date strings.";
            }
            values.push(...parsed.map((v: unknown) => String(v)));
          } catch {
            return "Value must be a JSON array of date strings.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(rawValue.trim());
      }

      if (values.length === 0) {
        return null;
      }

      for (const value of values) {
        const date = parseDate(value);
        if (!Number.isFinite(date.getTime())) {
          return "Value must be a valid date.";
        }
        if (minDate && Number.isFinite(minDate.getTime()) && date < minDate) {
          return `Date must be on or after ${minValue}.`;
        }
        if (maxDate && Number.isFinite(maxDate.getTime()) && date > maxDate) {
          return `Date must be on or before ${maxValue}.`;
        }
      }
    }

    if (isNumberType) {
      const minValue = typeof validations.min === "number" ? validations.min : undefined;
      const maxValue = typeof validations.max === "number" ? validations.max : undefined;
      const values: number[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => Number(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of numbers.";
            }
            values.push(...parsed.map((v: unknown) => Number(v)));
          } catch {
            return "Value must be a JSON array of numbers.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(Number(rawValue.trim()));
      }

      if (values.length === 0) {
        return null;
      }

      for (const num of values) {
        if (!Number.isFinite(num)) {
          return "Value must be a number.";
        }
        if (minValue != null && num < minValue) {
          return `Value must be at least ${minValue}.`;
        }
        if (maxValue != null && num > maxValue) {
          return `Value must be at most ${maxValue}.`;
        }
      }
    }

    if (
      definition.valueType === "string" ||
      definition.valueType === "text" ||
      definition.valueType === "json"
    ) {
      const minValue = typeof validations.min === "number" ? validations.min : undefined;
      const maxValue = typeof validations.max === "number" ? validations.max : undefined;
      const regexValue = typeof validations.regex === "string" ? validations.regex : undefined;
      let regex: RegExp | null = null;
      if (regexValue) {
        try {
          regex = new RegExp(regexValue);
        } catch {
          regex = null;
        }
      }
      const values: string[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => String(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of strings.";
            }
            values.push(...parsed.map((v: unknown) => String(v)));
          } catch {
            return "Value must be a JSON array of strings.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(rawValue.trim());
      }

      if (values.length === 0) {
        return null;
      }

      for (const text of values) {
        if (minValue != null && text.length < minValue) {
          return `Value must be at least ${minValue} characters.`;
        }
        if (maxValue != null && text.length > maxValue) {
          return `Value must be at most ${maxValue} characters.`;
        }
        if (regex && !regex.test(text)) {
          return "Value does not match the required pattern.";
        }
      }
    }

    return null;
  }

  async function handleSaveMetafield(definition: MetafieldDefinition) {
    if (!definition.id || isSavingMetafield) {
      return;
    }
    setIsSavingMetafield(definition.id);
    try {
      const rawValue = metafieldValues[definition.id];
      const validationError = validateMetafieldValue(definition, rawValue);
      if (validationError) {
        push({
          variant: "error",
          title: "Validation failed",
          description: validationError,
        });
        return;
      }

      let valueJson = "\"\"";
      const isBooleanType =
        definition.valueType === "bool" || definition.valueType === "boolean";
      if (isBooleanType) {
        const normalized =
          typeof rawValue === "string"
            ? rawValue.trim().toLowerCase() === "true"
            : Boolean(rawValue);
        valueJson = JSON.stringify(normalized);
      } else if (definition.isList) {
        if (Array.isArray(rawValue)) {
          valueJson = JSON.stringify(rawValue);
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          valueJson = rawValue.trim();
        } else {
          valueJson = "[]";
        }
      } else {
        valueJson = JSON.stringify(typeof rawValue === "string" ? rawValue : "");
      }
      await upsertCustomerMetafieldValue({
        customerId,
        definitionId: definition.id,
        valueJson,
      });
      push({
        variant: "success",
        title: "Custom attribute saved",
        description: "Metafield value has been updated.",
      });
      await reloadCustomer();
    } catch (err) {
      notifyError(err, "Save failed", "Failed to save custom attribute");
    } finally {
      setIsSavingMetafield(null);
    }
  }

  function renderMetafieldInput(definition: MetafieldDefinition) {
    let value: MetafieldValueState = metafieldValues[definition.id] ?? "";
    if (
      (definition.valueType === "bool" || definition.valueType === "boolean") &&
      typeof value === "string"
    ) {
      if (value.toLowerCase() === "true") {
        value = true;
      } else if (value.toLowerCase() === "false") {
        value = false;
      }
    }
    const handleChange = (nextValue: MetafieldValueState) => {
      setMetafieldValues((prev) => ({
        ...prev,
        [definition.id]: nextValue,
      }));
    };

    const validationError = validateMetafieldValue(definition, value);
    let enumOptions: string[] = [];
    if (definition.valueType === "enum") {
      try {
        const parsed = definition.validationsJson
          ? JSON.parse(definition.validationsJson)
          : {};
        if (parsed && Array.isArray(parsed.enum)) {
          enumOptions = parsed.enum.map((item: unknown) => String(item));
        }
      } catch {
        enumOptions = [];
      }
    }

    if (definition.valueType === "enum") {
      if (definition.isList) {
        const selected = Array.isArray(value) ? value : [];
        return (
          <div className="space-y-2">
            {enumOptions.length === 0 ? (
              <div className="text-xs text-neutral-500">No enum options configured.</div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {enumOptions.map((option) => {
                  const checked = selected.includes(option);
                  return (
                    <label
                      key={option}
                      className="flex items-center gap-2 rounded-full border border-neutral-200 px-3 py-1 text-xs text-neutral-700"
                    >
                      <input
                        type="checkbox"
                        className="h-4 w-4 accent-neutral-900"
                        checked={checked}
                        onChange={(event) => {
                          if (event.target.checked) {
                            handleChange([...selected, option]);
                          } else {
                            handleChange(selected.filter((item) => item !== option));
                          }
                        }}
                      />
                      {option}
                    </label>
                  );
                })}
              </div>
            )}
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
      }
      return (
        <div className="space-y-2">
          <Select
            value={typeof value === "string" ? value : ""}
            onValueChange={(next) => handleChange(next)}
          >
            <SelectTrigger className="bg-white">
              <SelectValue placeholder="Select value" />
            </SelectTrigger>
            <SelectContent>
              {enumOptions.map((option) => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {validationError ? (
            <p className="text-xs text-red-600">{validationError}</p>
          ) : null}
        </div>
      );
    }

    if (definition.valueType === "bool" || definition.valueType === "boolean") {
      return (
        <div className="flex items-center justify-between gap-3">
          <div className="text-xs text-neutral-500">Toggle on/off</div>
          <Switch
            checked={Boolean(value)}
            onCheckedChange={(checked) => handleChange(checked)}
          />
          {validationError ? (
            <p className="text-xs text-red-600">{validationError}</p>
          ) : null}
        </div>
      );
    }

    if (definition.isList) {
      return (
        <div className="space-y-2">
          <Textarea
            id={`metafield-${definition.id}`}
            value={typeof value === "string" ? value : JSON.stringify(value)}
            onChange={(event) => handleChange(event.target.value)}
            placeholder='e.g. ["value1","value2"]'
          />
          {validationError ? (
            <p className="text-xs text-red-600">{validationError}</p>
          ) : null}
        </div>
      );
    }

    switch (definition.valueType) {
      case "date":
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              type="date"
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
            />
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
      case "dateTime":
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              type="datetime-local"
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
            />
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
      case "color":
        return (
          <div className="flex items-center gap-3">
            <Input
              id={`metafield-${definition.id}`}
              type="color"
              value={typeof value === "string" && value ? value : "#000000"}
              onChange={(event) => handleChange(event.target.value)}
              className="h-10 w-16 p-1"
            />
            <Input
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
              placeholder="#000000"
            />
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
      default:
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
              placeholder="Enter value"
            />
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
    }
  }

  if (isLoading) {
    return <div className="text-sm text-neutral-600">Loading customer...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="mt-2 text-lg font-semibold text-neutral-900">Customer Detail</h1>
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

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Custom attributes</CardTitle>
          <CardDescription className="text-neutral-500">
            Optional metadata defined for customers (metafields).
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 text-sm text-neutral-700">
          {metafieldDefs.length === 0 ? (
            <div className="text-sm text-neutral-600">No metafield definitions configured.</div>
          ) : (
            metafieldDefs.map((definition) => (
              <div key={definition.id} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="font-medium text-neutral-900">
                      {definition.name}
                      <span className="ml-2 text-xs font-normal text-neutral-500">
                        {definition.namespace}.{definition.key}
                      </span>
                    </div>
                    <div className="text-xs text-neutral-500">
                      type: {definition.valueType}
                      {definition.description ? ` · ${definition.description}` : ""}
                    </div>
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    onClick={() => handleSaveMetafield(definition)}
                    disabled={
                      isSavingMetafield === definition.id ||
                      Boolean(
                        validateMetafieldValue(
                          definition,
                          metafieldValues[definition.id] ?? ""
                        )
                      )
                    }
                  >
                    {isSavingMetafield === definition.id ? "Saving..." : "Save"}
                  </Button>
                </div>
                <div className="mt-3 space-y-2">
                  <Label htmlFor={`metafield-${definition.id}`}>Value</Label>
                  {renderMetafieldInput(definition)}
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
