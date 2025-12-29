"use client";

import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function SettingsPaymentPage() {
  return (
    <div className="space-y-8">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Payment</h1>
        <p className="mt-2 text-sm text-neutral-600">Choose a payment method to configure.</p>
      </div>
      <div className="grid gap-6 md:grid-cols-2">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Bank Transfer</CardTitle>
            <CardDescription className="text-neutral-500">
              Bank account information for transfer payments.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/payment/bank">
              Configure Bank Transfer
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Cash on Delivery</CardTitle>
            <CardDescription className="text-neutral-500">
              COD enable/fee settings.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/payment/cod">
              Configure COD
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
