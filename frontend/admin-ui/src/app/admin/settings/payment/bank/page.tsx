"use client";

import StoreSettingsForm from "@/components/store-settings-form";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsPaymentBankPage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader
        title="Payment / Bank Transfer"
        description="Bank transfer account details."
      />
      <StoreSettingsForm sections={["payment-bank"]} submitLabel="Save Bank Settings" />
    </div>
  );
}
