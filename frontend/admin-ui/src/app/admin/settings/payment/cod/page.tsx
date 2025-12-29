"use client";

import StoreSettingsForm from "@/components/store-settings-form";

export default function SettingsPaymentCodPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Payment / COD</h1>
        <p className="mt-2 text-sm text-neutral-600">Cash on delivery settings.</p>
      </div>
      <StoreSettingsForm sections={["payment-cod"]} submitLabel="Save COD Settings" />
    </div>
  );
}
