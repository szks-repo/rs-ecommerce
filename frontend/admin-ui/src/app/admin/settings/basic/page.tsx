"use client";

import StoreSettingsForm from "@/components/store-settings-form";

export default function SettingsBasicPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Basic</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Update basic information in smaller sections.
        </p>
      </div>
      <StoreSettingsForm sections={["profile"]} submitLabel="Save Store Profile" />
      <StoreSettingsForm sections={["address"]} submitLabel="Save Address" />
      <StoreSettingsForm sections={["contact-email"]} submitLabel="Save Contact Email" />
      <StoreSettingsForm sections={["contact-phone"]} submitLabel="Save Contact Phone" />
      <StoreSettingsForm sections={["legal"]} submitLabel="Save Legal Notice" />
      <StoreSettingsForm sections={["sku"]} submitLabel="Save SKU Rule" />
      <StoreSettingsForm sections={["locale"]} submitLabel="Save Locale" />
      <StoreSettingsForm sections={["domain"]} submitLabel="Save Domain" />
      <StoreSettingsForm sections={["appearance"]} submitLabel="Save Appearance" />
    </div>
  );
}
