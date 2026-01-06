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
  AdminTable,
  AdminTableCell,
  AdminTableHeaderCell,
  AdminTablePagination,
  AdminTableToolbar,
} from "@/components/admin-table";
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
      </CardHeader>
      <CardContent>
        <AdminTableToolbar
          left={`Showing ${customers.length} customers`}
          right={
            <>
              <Input
                value={query}
                onChange={(event) => {
                  setQuery(event.target.value);
                  setPageToken("");
                }}
                placeholder="Search customers"
                className="h-9 w-full min-w-[220px] max-w-[320px]"
              />
              <Select
                value={String(pageSize)}
                onValueChange={(value) => {
                  setPageSize(Number(value));
                  setPageToken("");
                }}
              >
                <SelectTrigger className="h-9 w-[120px] bg-white">
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
              <Button variant="outline" onClick={reload} disabled={loading} size="sm">
                {loading ? "Loading..." : "Refresh"}
              </Button>
            </>
          }
        />
        {customers.length === 0 ? (
          <div className="text-sm text-neutral-600">No customers found.</div>
        ) : (
          <AdminTable>
            <thead className="sticky top-0 bg-neutral-50">
              <tr>
                <AdminTableHeaderCell>Customer</AdminTableHeaderCell>
                <AdminTableHeaderCell>Contact</AdminTableHeaderCell>
                <AdminTableHeaderCell>Status</AdminTableHeaderCell>
                <AdminTableHeaderCell>Created</AdminTableHeaderCell>
                <AdminTableHeaderCell align="right">Detail</AdminTableHeaderCell>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-200">
              {customers.map((customer) => (
                <tr key={customer.customerId}>
                  <AdminTableCell>
                    <div className="text-sm font-medium text-neutral-900">
                      {customer.name || "Unnamed"}
                    </div>
                    <div className="text-[11px] text-neutral-500">
                      id: {customer.customerId}
                    </div>
                  </AdminTableCell>
                  <AdminTableCell>
                    {customer.email ? <div>{customer.email}</div> : null}
                    {customer.phone ? <div>{customer.phone}</div> : null}
                  </AdminTableCell>
                  <AdminTableCell className="text-neutral-500">{customer.status}</AdminTableCell>
                  <AdminTableCell className="text-neutral-500">
                    <DateCell value={toIsoString(customer.createdAt)} />
                  </AdminTableCell>
                  <AdminTableCell align="right">
                    <Button asChild type="button" size="sm" variant="outline">
                      <Link href={`/admin/customers/${customer.customerId}`}>Open</Link>
                    </Button>
                  </AdminTableCell>
                </tr>
              ))}
            </tbody>
          </AdminTable>
        )}
        <AdminTablePagination
          label={`Showing ${customers.length} customers`}
          onPrev={() => {
            const offset = Number.parseInt(pageToken || "0", 10);
            const prev = Math.max(0, offset - pageSize);
            setPageToken(prev > 0 ? String(prev) : "");
          }}
          onNext={() => setPageToken(nextPageToken)}
          canPrev={!!pageToken && Number.parseInt(pageToken || "0", 10) > 0}
          canNext={!!nextPageToken}
        />
      </CardContent>
    </Card>
  );
}
