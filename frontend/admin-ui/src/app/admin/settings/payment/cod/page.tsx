"use client";

import StoreSettingsForm from "@/components/store-settings-form";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsPaymentCodPage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader title="Payment / COD" description="Cash on delivery settings." />
      <StoreSettingsForm sections={["payment-cod"]} submitLabel="Save COD Settings" />
    </div>
  );
}
