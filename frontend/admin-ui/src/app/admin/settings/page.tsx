"use client";

import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Settings"
        description="Choose a category to configure. Each section is split into focused pages."
      />

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
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Pages</CardTitle>
            <CardDescription className="text-neutral-500">
              Free pages for storefront content.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/pages">
              Open Pages
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Files</CardTitle>
            <CardDescription className="text-neutral-500">
              Upload and manage media assets shared with products.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/files">
              Open Files
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
