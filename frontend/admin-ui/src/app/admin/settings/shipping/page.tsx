"use client";

import ShippingZonesForm from "@/components/shipping-zones-form";
import ShippingRatesForm from "@/components/shipping-rates-form";
import StoreLocationForm from "@/components/store-location-form";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsShippingPage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader title="Shipping" description="Zones, rates, and inventory locations." />
      <div className="grid gap-6 lg:grid-cols-2">
        <ShippingZonesForm />
        <ShippingRatesForm />
      </div>
      <StoreLocationForm />
    </div>
  );
}
