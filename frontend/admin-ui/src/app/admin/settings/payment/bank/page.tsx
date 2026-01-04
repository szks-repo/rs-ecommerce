"use client";

import StoreSettingsForm from "@/components/store-settings-form";

export default function SettingsPaymentBankPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Payment / Bank Transfer</h1>
        <p className="mt-2 text-sm text-neutral-600">Bank transfer account details.</p>
      </div>
      <StoreSettingsForm sections={["payment-bank"]} submitLabel="Save Bank Settings" />
    </div>
  );
}
