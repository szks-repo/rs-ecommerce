"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { getActiveAccessToken } from "@/lib/auth";
import { listCustomers } from "@/lib/customer";
import type { CustomerSummary } from "@/gen/ecommerce/v1/customer_pb";
import { formatConnectError } from "@/lib/handle-error";

export default function CustomerList() {
  const [customers, setCustomers] = useState<CustomerSummary[]>([]);
  const [query, setQuery] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  async function loadCustomers(nextQuery?: string) {
    if (!getActiveAccessToken()) {
      push({
        variant: "error",
        title: "Load failed",
        description: "access_token is missing. Please sign in first.",
      });
      return;
    }
    setIsLoading(true);
    try {
      const data = await listCustomers({ query: nextQuery ?? query });
      setCustomers(data.customers ?? []);
    } catch (err) {
      const uiError = formatConnectError(err, "Load failed", "Failed to load customers");
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
    void loadCustomers();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <CardTitle>Customers</CardTitle>
          <CardDescription className="text-neutral-500">
            Search by name, email, or phone in this store.
          </CardDescription>
        </div>
        <div className="flex w-full flex-col gap-2 md:w-auto md:flex-row md:items-center">
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search customers"
          />
          <Button variant="outline" onClick={() => loadCustomers()} disabled={isLoading}>
            {isLoading ? "Loading..." : "Search"}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {customers.length === 0 ? (
          <div className="text-sm text-neutral-600">No customers found.</div>
        ) : (
          <div className="space-y-3 text-sm text-neutral-700">
            {customers.map((customer) => (
              <div key={customer.customerId} className="rounded-lg border border-neutral-200 p-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="font-medium text-neutral-900">
                      {customer.name || "Unnamed"}
                    </div>
                    <div className="text-xs text-neutral-500">
                      id: {customer.customerId} / status: {customer.status}
                    </div>
                    <div className="mt-1 text-xs text-neutral-600">
                      {customer.email || "-"} {customer.phone ? ` / ${customer.phone}` : ""}
                    </div>
                  </div>
                  <div className="flex flex-col gap-2 text-xs">
                    <Link
                      className="rounded-md border border-neutral-200 px-3 py-1 text-center font-medium text-neutral-700 hover:bg-neutral-50"
                      href={`/admin/customers/${customer.customerId}`}
                    >
                      Details
                    </Link>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
