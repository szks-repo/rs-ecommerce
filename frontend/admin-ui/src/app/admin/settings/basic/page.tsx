"use client";

import StoreSettingsForm from "@/components/store-settings-form";

export default function SettingsBasicPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Shop Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Basic</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Store profile, contact, domain, and defaults.
        </p>
      </div>
      <StoreSettingsForm sections={["basic", "appearance"]} submitLabel="Save Basic Settings" />
    </div>
  );
}
