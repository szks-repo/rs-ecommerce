"use client";

import { useDeferredValue, useEffect, useState } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { getActiveAccessToken } from "@/lib/auth";
import { listCustomers } from "@/lib/customer";
import type { CustomerSummary } from "@/gen/ecommerce/v1/customer_pb";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import DateCell from "@/components/date-cell";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export default function CustomerList() {
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");
  const [pageSize, setPageSize] = useState(50);
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<{
    customers: CustomerSummary[];
    nextPageToken: string;
  }>(
    async () => {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const data = await listCustomers({
        query: deferredQuery.trim(),
        pageSize,
        pageToken,
      });
      return {
        customers: data.customers ?? [],
        nextPageToken: data.page?.nextPageToken ?? "",
      };
    },
    [deferredQuery, pageToken, pageSize]
  );

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load customers");
      return;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, error, notifyError]);

  useEffect(() => {
    if (!data) {
      return;
    }
    setNextPageToken(data.nextPageToken);
  }, [data]);

  const customers = data?.customers ?? [];

  function toIsoString(ts?: { seconds?: string | number | bigint; nanos?: number }) {
    if (!ts?.seconds) {
      return "";
    }
    const seconds = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
    if (!Number.isFinite(seconds)) {
      return "";
    }
    const date = new Date(seconds * 1000);
    return Number.isNaN(date.getTime()) ? "" : date.toISOString();
  }

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
            onChange={(event) => {
              setQuery(event.target.value);
              setPageToken("");
            }}
            placeholder="Search customers"
          />
          <Select
            value={String(pageSize)}
            onValueChange={(value) => {
              setPageSize(Number(value));
              setPageToken("");
            }}
          >
            <SelectTrigger className="bg-white">
              <SelectValue placeholder="Rows" />
            </SelectTrigger>
            <SelectContent>
              {[25, 50, 100].map((size) => (
                <SelectItem key={size} value={String(size)}>
                  {size} / page
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="outline" onClick={reload} disabled={loading}>
            {loading ? "Loading..." : "Refresh"}
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {customers.length === 0 ? (
          <div className="text-sm text-neutral-600">No customers found.</div>
        ) : (
          <div className="overflow-hidden rounded-lg border border-neutral-200 bg-white">
            <div className="max-h-[520px] overflow-auto">
              <table className="min-w-full text-sm">
                <thead className="sticky top-0 bg-neutral-50 text-xs uppercase text-neutral-500">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">Customer</th>
                    <th className="px-3 py-2 text-left font-medium">Contact</th>
                    <th className="px-3 py-2 text-left font-medium">Status</th>
                    <th className="px-3 py-2 text-left font-medium">Created</th>
                    <th className="px-3 py-2 text-right font-medium">Detail</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-neutral-200">
                  {customers.map((customer) => (
                    <tr key={customer.customerId} className="align-top">
                      <td className="px-3 py-2">
                        <div className="text-sm font-medium text-neutral-900">
                          {customer.name || "Unnamed"}
                        </div>
                        <div className="text-[11px] text-neutral-500">
                          id: {customer.customerId}
                        </div>
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-600">
                        {customer.email ? <div>{customer.email}</div> : null}
                        {customer.phone ? <div>{customer.phone}</div> : null}
                      </td>
                      <td className="px-3 py-2 text-[11px] text-neutral-500">{customer.status}</td>
                      <td className="px-3 py-2 text-[11px] text-neutral-500">
                        <DateCell value={toIsoString(customer.createdAt)} />
                      </td>
                      <td className="px-3 py-2 text-right">
                        <Button asChild type="button" size="sm" variant="outline">
                          <Link href={`/admin/customers/${customer.customerId}`}>Open</Link>
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
        <div className="mt-4 flex flex-wrap items-center justify-between gap-2 text-sm">
          <div className="text-neutral-500">
            Showing {customers.length} customers
          </div>
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => {
                const offset = Number.parseInt(pageToken || "0", 10);
                const prev = Math.max(0, offset - pageSize);
                setPageToken(prev > 0 ? String(prev) : "");
              }}
              disabled={!pageToken || Number.parseInt(pageToken || "0", 10) <= 0}
            >
              Prev
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setPageToken(nextPageToken)}
              disabled={!nextPageToken}
            >
              Next
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
