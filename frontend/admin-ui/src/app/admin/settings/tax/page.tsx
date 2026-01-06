"use client";

import StoreSettingsForm from "@/components/store-settings-form";
import TaxRulesForm from "@/components/tax-rules-form";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsTaxPage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader title="Tax" description="Tax mode, rounding, and rules." />
      <StoreSettingsForm sections={["tax"]} submitLabel="Save Tax Settings" />
      <TaxRulesForm />
    </div>
  );
}
