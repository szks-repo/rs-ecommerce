"use client";

import { useEffect } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import StoreBadge from "@/components/store-badge";
import { Button } from "@/components/ui/button";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import { getDashboardSummary } from "@/lib/dashboard";
import { getActiveAccessToken } from "@/lib/auth";
import { toNumber } from "@/lib/number";

export default function AdminDashboard() {
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource(async () => {
    if (!getActiveAccessToken()) {
      throw new Error("access_token is missing. Please sign in first.");
    }
    const res = await getDashboardSummary();
    return res.summary ?? null;
  }, []);

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load dashboard data");
    }
  }, [error, notifyError]);

  const productCount = toNumber(data?.productCount);
  const productActiveCount = toNumber(data?.productActiveCount);
  const orderPendingCount = toNumber(data?.orderPendingCount);
  const orderTodayCount = toNumber(data?.orderTodayCount);
  const customerCount = toNumber(data?.customerCount);
  const lowStockSkuCount = toNumber(data?.lowStockSkuCount);
  const auctionRunningCount = toNumber(data?.auctionRunningCount);

  return (
    <div>
      <section className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold">Dashboard</h1>
          <p className="text-sm text-neutral-600">Admin/Staff operational dashboard.</p>
        </div>
      </section>

      <div className="mt-4">
        <StoreBadge />
      </div>

      {!loading && productCount === 0 ? (
        <section className="mt-6">
          <Card className="border-neutral-200 bg-white text-neutral-900">
            <CardHeader>
              <CardTitle>Get started with your catalog</CardTitle>
              <CardDescription className="text-neutral-500">
                Create your first product to unlock inventory, auctions, and storefront previews.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-neutral-600">
              <ol className="list-decimal space-y-1 pl-5">
                <li>Add a product title and status</li>
                <li>Create SKU(s) with price and fulfillment type</li>
                <li>Publish when ready</li>
              </ol>
              <div>
                <Button asChild className="bg-neutral-900 text-white hover:bg-neutral-800">
                  <Link href="/admin/products/new">Create first product</Link>
                </Button>
              </div>
            </CardContent>
          </Card>
        </section>
      ) : null}

      <section className="mt-8 grid gap-6 md:grid-cols-3">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Products</CardTitle>
            <CardDescription className="text-neutral-500">
              Total ({productActiveCount} active)
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">{productCount}</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Orders</CardTitle>
            <CardDescription className="text-neutral-500">
              Today {orderTodayCount}
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">{orderPendingCount}</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Customers</CardTitle>
            <CardDescription className="text-neutral-500">
              Total customers
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">{customerCount}</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Low stock</CardTitle>
            <CardDescription className="text-neutral-500">
              SKUs at or below 0
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">{lowStockSkuCount}</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Auctions</CardTitle>
            <CardDescription className="text-neutral-500">
              Running now
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">{auctionRunningCount}</CardContent>
        </Card>
      </section>
    </div>
  );
}
