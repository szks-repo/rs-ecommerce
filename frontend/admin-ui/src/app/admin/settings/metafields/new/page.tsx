"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
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
import { useApiCall } from "@/lib/use-api-call";
import { createCustomerMetafieldDefinition } from "@/lib/customer";
import { identityListRoles } from "@/lib/identity";
import { CalendarDays, Clock, Palette, Type } from "lucide-react";

const VALUE_TYPES = [
  { value: "string", label: "string", Icon: Type },
  { value: "number", label: "number", Icon: Type },
  { value: "boolean", label: "boolean", Icon: Type },
  { value: "json", label: "json", Icon: Type },
  { value: "enum", label: "enum", Icon: Type },
  { value: "date", label: "date", Icon: CalendarDays },
  { value: "dateTime", label: "dateTime", Icon: Clock },
  { value: "color", label: "color", Icon: Palette },
];

const OWNER_TYPES = [
  { value: "customer", label: "Customers", enabled: true },
  { value: "product", label: "Products", enabled: false },
  { value: "order", label: "Orders", enabled: false },
];

export default function NewMetafieldPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { push } = useToast();
  const { notifyError } = useApiCall();

  const [ownerType, setOwnerType] = useState("customer");
  const [namespace, setNamespace] = useState("");
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [valueType, setValueType] = useState("string");
  const [isList, setIsList] = useState(false);
  const [validationsJson, setValidationsJson] = useState("{}");
  const [visibilityJson, setVisibilityJson] = useState("{}");
  const [isSaving, setIsSaving] = useState(false);
  const [validationRequired, setValidationRequired] = useState(false);
  const [validationMin, setValidationMin] = useState("");
  const [validationMax, setValidationMax] = useState("");
  const [validationRegex, setValidationRegex] = useState("");
  const [validationEnumInput, setValidationEnumInput] = useState("");
  const [validationEnumValues, setValidationEnumValues] = useState<string[]>([]);
  const [visibilityAdminOnly, setVisibilityAdminOnly] = useState(true);
  const [visibilityPublic, setVisibilityPublic] = useState(false);
  const [visibilityRoles, setVisibilityRoles] = useState<string[]>([]);
  const [roleOptions, setRoleOptions] = useState<{ key: string; name: string }[]>([]);

  useEffect(() => {
    const param = searchParams?.get("ownerType");
    if (param) {
      setOwnerType(param);
    }
  }, [searchParams]);

  useEffect(() => {
    if (valueType === "bool" || valueType === "boolean") {
      setIsList(false);
    }
  }, [valueType]);

  useEffect(() => {
    async function loadRoles() {
      try {
        const resp = await identityListRoles();
        setRoleOptions(
          (resp.roles ?? []).map((role) => ({
            key: role.key,
            name: role.name || role.key,
          }))
        );
      } catch (err) {
        notifyError(err, "Load failed", "Failed to load roles");
      }
    }
    void loadRoles();
  }, [push]);

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSaving) {
      return;
    }
    if (ownerType !== "customer") {
      push({
        variant: "error",
        title: "Not supported yet",
        description: "Metafields are only available for customers right now.",
      });
      return;
    }
    setIsSaving(true);
    try {
      setValidationsJson(computedValidationsJson);
      setVisibilityJson(computedVisibilityJson);
      const resp = await createCustomerMetafieldDefinition({
        namespace,
        key,
        name,
        description: description || undefined,
        valueType,
        isList,
        validationsJson: computedValidationsJson,
        visibilityJson: computedVisibilityJson,
      });
      const id = resp.definition?.id;
      push({
        variant: "success",
        title: "Definition created",
        description: "Customer metafield definition has been created.",
      });
      if (id) {
        router.push(`/admin/settings/metafields/${id}`);
      } else {
        router.push("/admin/settings/metafields");
      }
    } catch (err) {
      notifyError(err, "Save failed", "Failed to save metafield definition");
    } finally {
      setIsSaving(false);
    }
  }

  const formDisabled = ownerType !== "customer";
  const computedValidationsJson = JSON.stringify(
    {
      required: validationRequired || undefined,
      min:
        valueType === "number"
          ? validationMin.trim() === ""
            ? undefined
            : Number(validationMin)
          : valueType === "date" || valueType === "dateTime"
              ? validationMin.trim() === ""
                ? undefined
                : validationMin.trim()
              : undefined,
      max:
        valueType === "number"
          ? validationMax.trim() === ""
            ? undefined
            : Number(validationMax)
          : valueType === "date" || valueType === "dateTime"
              ? validationMax.trim() === ""
                ? undefined
                : validationMax.trim()
              : undefined,
      regex:
        valueType === "string"
          ? validationRegex.trim() === ""
            ? undefined
            : validationRegex.trim()
          : undefined,
      enum:
        valueType === "enum" && validationEnumValues.length > 0
          ? validationEnumValues
          : undefined,
    },
    null,
    2
  );
  const computedVisibilityJson = JSON.stringify(
    {
      adminOnly: visibilityAdminOnly || undefined,
      public: visibilityPublic || undefined,
      roles: visibilityRoles.length === 0 ? undefined : visibilityRoles,
    },
    null,
    2
  );

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Metafields</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">New definition</h1>
          <p className="mt-2 text-sm text-neutral-500">
            Create a new metafield definition for a resource type.
          </p>
        </div>
        <Button variant="outline" onClick={() => router.push("/admin/settings/metafields")}>Back to list</Button>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Definition</CardTitle>
          <CardDescription className="text-neutral-500">
            Namespace + key must be unique. JSON fields should be valid JSON objects.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label>Resource</Label>
                <Select value={ownerType} onValueChange={setOwnerType}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select resource" />
                  </SelectTrigger>
                  <SelectContent>
                    {OWNER_TYPES.map((option) => (
                      <SelectItem key={option.value} value={option.value} disabled={!option.enabled}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="valueType">Value type</Label>
                <Select value={valueType} onValueChange={setValueType} disabled={formDisabled}>
                  <SelectTrigger className="bg-white">
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    {VALUE_TYPES.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        <span className="flex items-center gap-2">
                          <option.Icon className="h-4 w-4 text-neutral-500" />
                          <span>{option.label}</span>
                        </span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            {formDisabled ? (
              <div className="rounded-lg border border-dashed border-neutral-200 bg-neutral-50 p-3 text-sm text-neutral-600">
                This resource is planned but not available yet. Switch back to Customers to create a
                definition.
              </div>
            ) : null}
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="namespace">Namespace</Label>
                <Input
                  id="namespace"
                  value={namespace}
                  onChange={(event) => setNamespace(event.target.value)}
                  placeholder="profile"
                  disabled={formDisabled}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="key">Key</Label>
                <Input
                  id="key"
                  value={key}
                  onChange={(event) => setKey(event.target.value)}
                  placeholder="membership_rank"
                  disabled={formDisabled}
                />
              </div>
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="name">Name</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Membership Rank"
                  disabled={formDisabled}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="description">Description</Label>
                <Input
                  id="description"
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                  placeholder="Optional description"
                  disabled={formDisabled}
                />
              </div>
            </div>
            <div className="flex items-center gap-3">
              <input
                id="isList"
                type="checkbox"
                className="h-4 w-4 accent-neutral-900"
                checked={isList}
                onChange={(event) => setIsList(event.target.checked)}
                disabled={formDisabled || valueType === "bool" || valueType === "boolean"}
              />
              <Label htmlFor="isList">Treat as list (array)</Label>
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="validations">Validations (JSON)</Label>
                <div className="space-y-2 rounded-lg border border-neutral-200 bg-neutral-50 p-3 text-sm text-neutral-700">
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-neutral-900"
                      checked={validationRequired}
                      onChange={(event) => setValidationRequired(event.target.checked)}
                      disabled={formDisabled}
                    />
                    Required
                  </label>
                  {valueType === "number" ? (
                    <div className="grid gap-2 md:grid-cols-2">
                      <Input
                        placeholder="Min value"
                        value={validationMin}
                        onChange={(event) => setValidationMin(event.target.value)}
                        disabled={formDisabled}
                      />
                      <Input
                        placeholder="Max value"
                        value={validationMax}
                        onChange={(event) => setValidationMax(event.target.value)}
                        disabled={formDisabled}
                      />
                    </div>
                  ) : null}
                  {valueType === "date" || valueType === "dateTime" ? (
                    <div className="grid gap-2 md:grid-cols-2">
                      <Input
                        type={valueType === "date" ? "date" : "datetime-local"}
                        placeholder="Min"
                        value={validationMin}
                        onChange={(event) => setValidationMin(event.target.value)}
                        disabled={formDisabled}
                      />
                      <Input
                        type={valueType === "date" ? "date" : "datetime-local"}
                        placeholder="Max"
                        value={validationMax}
                        onChange={(event) => setValidationMax(event.target.value)}
                        disabled={formDisabled}
                      />
                    </div>
                  ) : null}
                  {valueType === "string" ? (
                    <Input
                      placeholder="Regex (optional)"
                      value={validationRegex}
                      onChange={(event) => setValidationRegex(event.target.value)}
                      disabled={formDisabled}
                    />
                  ) : null}
                  {valueType === "enum" ? (
                    <div className="flex flex-wrap items-center gap-2">
                      {validationEnumValues.map((value) => (
                        <button
                          key={value}
                          type="button"
                          className="rounded-full border border-neutral-200 bg-white px-3 py-1 text-xs text-neutral-700"
                          onClick={() =>
                            setValidationEnumValues((prev) =>
                              prev.filter((item) => item !== value)
                            )
                          }
                        >
                          {value} ×
                        </button>
                      ))}
                      <Input
                        placeholder="Add enum value"
                        value={validationEnumInput}
                        onChange={(event) => setValidationEnumInput(event.target.value)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            const next = validationEnumInput.trim();
                            if (next && !validationEnumValues.includes(next)) {
                              setValidationEnumValues((prev) => [...prev, next]);
                            }
                            setValidationEnumInput("");
                          }
                        }}
                        disabled={formDisabled}
                      />
                    </div>
                  ) : null}
                  <Textarea
                    value={computedValidationsJson}
                    readOnly
                    className="bg-white text-xs"
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="visibility">Visibility (JSON)</Label>
                <div className="space-y-2 rounded-lg border border-neutral-200 bg-neutral-50 p-3 text-sm text-neutral-700">
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-neutral-900"
                      checked={visibilityAdminOnly}
                      onChange={(event) => setVisibilityAdminOnly(event.target.checked)}
                      disabled={formDisabled}
                    />
                    Admin only
                  </label>
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-neutral-900"
                      checked={visibilityPublic}
                      onChange={(event) => setVisibilityPublic(event.target.checked)}
                      disabled={formDisabled}
                    />
                    Public
                  </label>
                  <div className="rounded-lg border border-neutral-200 bg-white p-2">
                    {roleOptions.length === 0 ? (
                      <div className="text-xs text-neutral-500">No roles available.</div>
                    ) : (
                      <div className="grid gap-2 sm:grid-cols-2">
                        {roleOptions.map((role) => {
                          const checked = visibilityRoles.includes(role.key);
                          return (
                            <label
                              key={role.key}
                              className="flex items-center gap-2 text-xs text-neutral-700"
                            >
                              <input
                                type="checkbox"
                                className="h-4 w-4 accent-neutral-900"
                                checked={checked}
                                onChange={(event) => {
                                  if (event.target.checked) {
                                    setVisibilityRoles((prev) => [...prev, role.key]);
                                  } else {
                                    setVisibilityRoles((prev) =>
                                      prev.filter((item) => item !== role.key)
                                    );
                                  }
                                }}
                                disabled={formDisabled}
                              />
                              {role.name}
                            </label>
                          );
                        })}
                      </div>
                    )}
                  </div>
                  <Textarea
                    value={computedVisibilityJson}
                    readOnly
                    className="bg-white text-xs"
                  />
                </div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button type="submit" disabled={isSaving || formDisabled}>
                {isSaving ? "Saving..." : "Create definition"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
