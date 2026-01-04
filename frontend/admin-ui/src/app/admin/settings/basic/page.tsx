"use client";

import { useState } from "react";
import StoreSettingsForm from "@/components/store-settings-form";

type BasicSection =
  | "profile"
  | "address"
  | "contact-email"
  | "contact-phone"
  | "legal"
  | "sku"
  | "locale"
  | "domain"
  | "appearance";

const BASIC_SECTIONS: { id: BasicSection; label: string; description: string; submitLabel: string }[] =
  [
    { id: "profile", label: "Store Profile", description: "Store name and legal name.", submitLabel: "Save Store Profile" },
    { id: "address", label: "Address", description: "Business address for legal disclosure.", submitLabel: "Save Address" },
    { id: "contact-email", label: "Contact Email", description: "Email shown for customer inquiries.", submitLabel: "Save Contact Email" },
    { id: "contact-phone", label: "Contact Phone", description: "Phone number shown in store info.", submitLabel: "Save Contact Phone" },
    { id: "legal", label: "Legal Notice", description: "Tokutei Shotorihiki-ho disclosure.", submitLabel: "Save Legal Notice" },
    { id: "sku", label: "SKU Rule", description: "Client-side SKU validation rule.", submitLabel: "Save SKU Rule" },
    { id: "locale", label: "Locale", description: "Language, timezone, and currency.", submitLabel: "Save Locale" },
    { id: "domain", label: "Domain", description: "Storefront domain settings.", submitLabel: "Save Domain" },
    { id: "appearance", label: "Appearance", description: "Theme and brand assets.", submitLabel: "Save Appearance" },
  ];

export default function SettingsBasicPage() {
  const [activeSection, setActiveSection] = useState<BasicSection>("profile");
  const active = BASIC_SECTIONS.find((section) => section.id === activeSection) ?? BASIC_SECTIONS[0];
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Basic</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Update each section without scrolling through a long form.
        </p>
      </div>
      <div className="grid gap-6 lg:grid-cols-[240px_minmax(0,1fr)]">
        <aside className="space-y-2">
          {BASIC_SECTIONS.map((section) => (
            <button
              key={section.id}
              type="button"
              onClick={() => setActiveSection(section.id)}
              className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition ${
                activeSection === section.id
                  ? "border-neutral-900 bg-neutral-900 text-white"
                  : "border-neutral-200 bg-white text-neutral-700 hover:border-neutral-300 hover:bg-neutral-50"
              }`}
            >
              <div className="font-medium">{section.label}</div>
              <div
                className={`mt-1 text-xs ${
                  activeSection === section.id ? "text-neutral-100" : "text-neutral-500"
                }`}
              >
                {section.description}
              </div>
            </button>
          ))}
        </aside>
        <section className="space-y-4">
          <div>
            <h2 className="text-xl font-semibold text-neutral-900">{active.label}</h2>
            <p className="mt-1 text-sm text-neutral-600">{active.description}</p>
          </div>
          <StoreSettingsForm sections={[active.id]} submitLabel={active.submitLabel} />
        </section>
      </div>
    </div>
  );
}
