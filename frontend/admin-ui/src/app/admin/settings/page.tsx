"use client";

import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function SettingsPage() {
  return (
    <div className="space-y-8">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Settings</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Choose a category to configure. Each section is split into focused pages.
        </p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Basic</CardTitle>
            <CardDescription className="text-neutral-500">
              Store profile, contact, domain, and defaults.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/basic">
              Open Basic Settings
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Payment</CardTitle>
            <CardDescription className="text-neutral-500">
              Bank transfer and cash on delivery.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/payment">
              Open Payment Settings
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Tax</CardTitle>
            <CardDescription className="text-neutral-500">
              Tax modes, rounding, and rules.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/tax">
              Open Tax Settings
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Shipping</CardTitle>
            <CardDescription className="text-neutral-500">
              Zones, rates, and locations.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/shipping">
              Open Shipping Settings
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
