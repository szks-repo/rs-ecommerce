"use client";

import ShippingZonesForm from "@/components/shipping-zones-form";
import ShippingRatesForm from "@/components/shipping-rates-form";
import StoreLocationForm from "@/components/store-location-form";

export default function SettingsShippingPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Shipping</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Zones, rates, and inventory locations.
        </p>
      </div>
      <div className="grid gap-6 lg:grid-cols-2">
        <ShippingZonesForm />
        <ShippingRatesForm />
      </div>
      <StoreLocationForm />
    </div>
  );
}
