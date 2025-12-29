"use client";

import StoreSettingsForm from "@/components/store-settings-form";
import TaxRulesForm from "@/components/tax-rules-form";
import ShippingZonesForm from "@/components/shipping-zones-form";
import ShippingRatesForm from "@/components/shipping-rates-form";
import StoreLocationForm from "@/components/store-location-form";

export default function SettingsPage() {
  return (
    <div className="space-y-8">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Shop Settings</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Configure store profile, payments, taxes, and shipping.
        </p>
      </div>

      <StoreSettingsForm />

      <TaxRulesForm />

      <div className="grid gap-6 lg:grid-cols-2">
        <ShippingZonesForm />
        <ShippingRatesForm />
      </div>

      <StoreLocationForm />
    </div>
  );
}
