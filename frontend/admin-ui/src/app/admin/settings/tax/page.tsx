"use client";

import StoreSettingsForm from "@/components/store-settings-form";
import TaxRulesForm from "@/components/tax-rules-form";

export default function SettingsTaxPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Tax</h1>
        <p className="mt-2 text-sm text-neutral-600">Tax mode, rounding, and rules.</p>
      </div>
      <StoreSettingsForm sections={["tax"]} submitLabel="Save Tax Settings" />
      <TaxRulesForm />
    </div>
  );
}
