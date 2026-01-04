"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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
import { listCustomerMetafieldDefinitions } from "@/lib/customer";
import type { MetafieldDefinition } from "@/gen/ecommerce/v1/customer_pb";
import { CalendarDays, Clock, Palette, Type } from "lucide-react";

const VALUE_TYPES = [
  { value: "string", label: "string", Icon: Type },
  { value: "number", label: "number", Icon: Type },
  { value: "boolean", label: "boolean", Icon: Type },
  { value: "json", label: "json", Icon: Type },
  { value: "date", label: "date", Icon: CalendarDays },
  { value: "dateTime", label: "dateTime", Icon: Clock },
  { value: "color", label: "color", Icon: Palette },
];

const OWNER_TYPES = [
  { value: "customer", label: "Customers", enabled: true },
  { value: "product", label: "Products", enabled: false },
  { value: "order", label: "Orders", enabled: false },
];

const OWNER_TYPE_LABELS: Record<string, string> = {
  customer: "Customers",
  product: "Products",
  order: "Orders",
};

export default function MetafieldsPage() {
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const [definitions, setDefinitions] = useState<MetafieldDefinition[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [ownerType, setOwnerType] = useState("customer");

  useEffect(() => {
    async function load() {
      setIsLoading(true);
      try {
        if (ownerType !== "customer") {
          setDefinitions([]);
          return;
        }
        const resp = await listCustomerMetafieldDefinitions();
        setDefinitions(resp.definitions ?? []);
      } catch (err) {
        notifyError(err, "Load failed", "Failed to load metafield definitions");
      } finally {
        setIsLoading(false);
      }
    }
    void load();
  }, [push, ownerType]);

  const newLink = useMemo(() => `/admin/metafields/new?ownerType=${ownerType}`, [ownerType]);

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Metafields</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Definitions</h1>
          <p className="mt-2 text-sm text-neutral-500">
            Define custom fields per resource type. Use namespace + key for stable identifiers.
          </p>
        </div>
        <Button asChild>
          <Link href={newLink}>New definition</Link>
        </Button>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Target resource</CardTitle>
          <CardDescription className="text-neutral-500">
            Choose the resource that the metafield applies to.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
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
          {ownerType !== "customer" ? (
            <div className="rounded-lg border border-dashed border-neutral-200 bg-neutral-50 p-3 text-sm text-neutral-600">
              This resource is planned but not available yet. Switch back to Customers to manage
              definitions today.
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Definitions</CardTitle>
          <CardDescription className="text-neutral-500">
            {isLoading ? "Loading definitions..." : `Existing definitions for ${ownerType}.`}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {ownerType !== "customer" ? (
            <div className="text-sm text-neutral-600">No definitions available yet.</div>
          ) : definitions.length === 0 ? (
            <div className="text-sm text-neutral-600">No definitions yet.</div>
          ) : (
            definitions.map((definition) => {
              const entry = VALUE_TYPES.find((option) => option.value === definition.valueType);
              const Icon = entry?.Icon ?? Type;
              const ownerLabel =
                OWNER_TYPE_LABELS[definition.ownerType] ?? (definition.ownerType || "-");
              return (
                <Link
                  key={definition.id}
                  href={`/admin/metafields/${definition.id}`}
                  className="block rounded-lg border border-neutral-200 p-3 transition hover:border-neutral-300"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <div className="font-medium text-neutral-900">
                        {definition.name}
                        <span className="ml-2 text-xs font-normal text-neutral-500">
                          {definition.namespace}.{definition.key}
                        </span>
                      </div>
                      <div className="text-xs text-neutral-500">
                        owner: {ownerLabel} ·{" "}
                        <span className="inline-flex items-center gap-1">
                          <Icon className="h-3.5 w-3.5" />
                          <span>type: {definition.valueType}</span>
                        </span>
                        {definition.isList ? " (list)" : ""}
                      </div>
                    </div>
                    <div className="text-xs text-neutral-400">View details</div>
                  </div>
                </Link>
              );
            })
          )}
        </CardContent>
      </Card>
    </div>
  );
}
