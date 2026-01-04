"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams, useRouter } from "next/navigation";
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
import {
  listCustomerMetafieldDefinitions,
  updateCustomerMetafieldDefinition,
} from "@/lib/customer";
import { identityListRoles } from "@/lib/identity";
import type { MetafieldDefinition } from "@/gen/ecommerce/v1/customer_pb";
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

const OWNER_TYPE_LABELS: Record<string, string> = {
  customer: "Customers",
  product: "Products",
  order: "Orders",
};

export default function MetafieldDetailPage() {
  const params = useParams<{ definitionId: string }>();
  const router = useRouter();
  const { push } = useToast();
  const { notifyError } = useApiCall();

  const definitionId = params.definitionId;
  const [definition, setDefinition] = useState<MetafieldDefinition | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  const [namespace, setNamespace] = useState("");
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [valueType, setValueType] = useState("string");
  const [isList, setIsList] = useState(false);
  const [validationsJson, setValidationsJson] = useState("{}");
  const [visibilityJson, setVisibilityJson] = useState("{}");
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
    async function load() {
      setIsLoading(true);
      try {
        const resp = await listCustomerMetafieldDefinitions();
        const found = (resp.definitions ?? []).find((item) => item.id === definitionId) || null;
        if (!found) {
          push({
            variant: "error",
            title: "Not found",
            description: "Metafield definition not found.",
          });
          router.push("/admin/metafields");
          return;
        }
        setDefinition(found);
        setNamespace(found.namespace);
        setKey(found.key);
        setName(found.name);
        setDescription(found.description || "");
        setValueType(found.valueType || "string");
        setIsList(found.isList ?? false);
        setValidationsJson(found.validationsJson || "{}");
        setVisibilityJson(found.visibilityJson || "{}");
        try {
          const parsed = found.validationsJson ? JSON.parse(found.validationsJson) : {};
          if (typeof parsed === "object" && parsed) {
            setValidationRequired(Boolean(parsed.required));
            setValidationMin(parsed.min != null ? String(parsed.min) : "");
            setValidationMax(parsed.max != null ? String(parsed.max) : "");
            setValidationRegex(parsed.regex ?? "");
            setValidationEnumValues(Array.isArray(parsed.enum) ? parsed.enum.map(String) : []);
          }
        } catch {
          setValidationRequired(false);
          setValidationMin("");
          setValidationMax("");
          setValidationRegex("");
          setValidationEnumValues([]);
        }
        try {
          const parsed = found.visibilityJson ? JSON.parse(found.visibilityJson) : {};
          if (typeof parsed === "object" && parsed) {
            setVisibilityAdminOnly(Boolean(parsed.adminOnly));
            setVisibilityPublic(Boolean(parsed.public));
            setVisibilityRoles(Array.isArray(parsed.roles) ? parsed.roles.map(String) : []);
          }
        } catch {
          setVisibilityAdminOnly(true);
          setVisibilityPublic(false);
          setVisibilityRoles([]);
        }
      } catch (err) {
        notifyError(err, "Load failed", "Failed to load metafield definition");
      } finally {
        setIsLoading(false);
      }
    }
    void load();
  }, [definitionId, push]);

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

  useEffect(() => {
    if (valueType === "bool" || valueType === "boolean") {
      setIsList(false);
    }
  }, [valueType]);

  const valueTypeLabel = useMemo(() => {
    const entry = VALUE_TYPES.find((option) => option.value === valueType);
    return entry?.label ?? valueType;
  }, [valueType]);

  const ownerTypeLabel = useMemo(() => {
    if (!definition) {
      return "-";
    }
    return OWNER_TYPE_LABELS[definition.ownerType] ?? (definition.ownerType || "-");
  }, [definition]);

  const computedValidationsJson = useMemo(
    () =>
      JSON.stringify(
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
      ),
    [
      valueType,
      validationRequired,
      validationMin,
      validationMax,
      validationRegex,
      validationEnumValues,
    ]
  );

  const computedVisibilityJson = useMemo(
    () =>
      JSON.stringify(
        {
          adminOnly: visibilityAdminOnly || undefined,
          public: visibilityPublic || undefined,
          roles: visibilityRoles.length === 0 ? undefined : visibilityRoles,
        },
        null,
        2
      ),
    [visibilityAdminOnly, visibilityPublic, visibilityRoles]
  );

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSaving || !definitionId) {
      return;
    }
    setIsSaving(true);
    try {
      setValidationsJson(computedValidationsJson);
      setVisibilityJson(computedVisibilityJson);
      await updateCustomerMetafieldDefinition({
        definitionId,
        namespace,
        key,
        name,
        description: description || undefined,
        valueType,
        isList,
        validationsJson: computedValidationsJson,
        visibilityJson: computedVisibilityJson,
      });
      push({
        variant: "success",
        title: "Definition updated",
        description: "Metafield definition has been saved.",
      });
    } catch (err) {
      notifyError(err, "Save failed", "Failed to save metafield definition");
    } finally {
      setIsSaving(false);
    }
  }

  if (isLoading) {
    return <div className="text-sm text-neutral-600">Loading definition...</div>;
  }

  if (!definition) {
    return null;
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Metafields</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Definition detail</h1>
          <p className="mt-2 text-sm text-neutral-500">
            Owner type: <span className="font-medium text-neutral-900">{ownerTypeLabel}</span> · Value type: {valueTypeLabel}
          </p>
        </div>
        <Button variant="outline" onClick={() => router.push("/admin/metafields")}>
          Back to list
        </Button>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Edit definition</CardTitle>
          <CardDescription className="text-neutral-500">
            Update namespace, key, labels, and validation rules.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="namespace">Namespace</Label>
                <Input
                  id="namespace"
                  value={namespace}
                  onChange={(event) => setNamespace(event.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="key">Key</Label>
                <Input id="key" value={key} onChange={(event) => setKey(event.target.value)} />
              </div>
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="name">Name</Label>
                <Input id="name" value={name} onChange={(event) => setName(event.target.value)} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="valueType">Value type</Label>
                <Select value={valueType} onValueChange={setValueType}>
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
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
              />
            </div>
            <div className="flex items-center gap-3">
              <input
                id="isList"
                type="checkbox"
                className="h-4 w-4 accent-neutral-900"
                checked={isList}
                onChange={(event) => setIsList(event.target.checked)}
                disabled={valueType === "bool" || valueType === "boolean"}
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
                    />
                    Required
                  </label>
                  {valueType === "number" ? (
                    <div className="grid gap-2 md:grid-cols-2">
                      <Input
                        placeholder="Min value"
                        value={validationMin}
                        onChange={(event) => setValidationMin(event.target.value)}
                      />
                      <Input
                        placeholder="Max value"
                        value={validationMax}
                        onChange={(event) => setValidationMax(event.target.value)}
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
                      />
                      <Input
                        type={valueType === "date" ? "date" : "datetime-local"}
                        placeholder="Max"
                        value={validationMax}
                        onChange={(event) => setValidationMax(event.target.value)}
                      />
                    </div>
                  ) : null}
                  {valueType === "string" ? (
                    <Input
                      placeholder="Regex (optional)"
                      value={validationRegex}
                      onChange={(event) => setValidationRegex(event.target.value)}
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
                      />
                    </div>
                  ) : null}
                  <Textarea value={computedValidationsJson} readOnly className="bg-white text-xs" />
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
                    />
                    Admin only
                  </label>
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-neutral-900"
                      checked={visibilityPublic}
                      onChange={(event) => setVisibilityPublic(event.target.checked)}
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
                              />
                              {role.name}
                            </label>
                          );
                        })}
                      </div>
                    )}
                  </div>
                  <Textarea value={computedVisibilityJson} readOnly className="bg-white text-xs" />
                </div>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button type="submit" disabled={isSaving}>
                {isSaving ? "Saving..." : "Save changes"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
