"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { getStoreSettings, updateStoreSettings } from "@/lib/store_settings";
import { useToast } from "@/components/ui/toast";
import { useApiCall } from "@/lib/use-api-call";

export default function SettingsPaymentPage() {
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [settings, setSettings] = useState<Awaited<ReturnType<typeof getStoreSettings>>["settings"] | null>(null);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  useEffect(() => {
    setIsLoading(true);
    getStoreSettings()
      .then((data) => {
        setSettings(data.settings ?? null);
      })
      .catch((err) => {
        notifyError(err, "Load failed", "Failed to load payment settings");
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [push]);

  async function updatePaymentSettings(patch: { codEnabled?: boolean; bankTransferEnabled?: boolean }) {
    if (!settings || isSaving) {
      return;
    }
    setIsSaving(true);
    try {
      const codFeeAmount = settings.codFee?.amount?.toString() ?? "0";
      const codFeeCurrency = settings.codFee?.currency ?? "JPY";
      const resp = await updateStoreSettings({
        settings: {
          storeName: settings.storeName,
          legalName: settings.legalName,
          contactEmail: settings.contactEmail,
          contactPhone: settings.contactPhone,
          addressPrefecture: settings.addressPrefecture,
          addressCity: settings.addressCity,
          addressLine1: settings.addressLine1,
          addressLine2: settings.addressLine2,
          legalNotice: settings.legalNotice,
          defaultLanguage: settings.defaultLanguage,
          primaryDomain: settings.primaryDomain,
          subdomain: settings.subdomain,
          httpsEnabled: Boolean(settings.httpsEnabled),
          timeZone: settings.timeZone || "Asia/Tokyo",
          currency: settings.currency,
          taxMode: settings.taxMode,
          taxRounding: settings.taxRounding,
          orderInitialStatus: settings.orderInitialStatus,
          codEnabled: patch.codEnabled ?? Boolean(settings.codEnabled),
          codFeeAmount,
          codFeeCurrency,
          bankTransferEnabled: patch.bankTransferEnabled ?? Boolean(settings.bankTransferEnabled),
          bankName: settings.bankName,
          bankBranch: settings.bankBranch,
          bankAccountType: settings.bankAccountType,
          bankAccountNumber: settings.bankAccountNumber,
          bankAccountName: settings.bankAccountName,
          theme: settings.theme,
          brandColor: settings.brandColor,
          logoUrl: settings.logoUrl,
          faviconUrl: settings.faviconUrl,
        },
      });
      setSettings(resp.settings ?? settings);
      push({
        variant: "success",
        title: "Payment settings updated",
        description: "Available payment methods have been updated.",
      });
    } catch (err) {
      notifyError(err, "Update failed", "Failed to update payment settings");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-8">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Settings</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Payment</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Enable the payment methods available in your store.
        </p>
      </div>
      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Available Methods</CardTitle>
          <CardDescription className="text-neutral-500">
            Toggle which payment methods can be used by customers.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between rounded-lg border border-neutral-200 px-4 py-3">
            <div>
              <div className="text-sm font-medium text-neutral-900">Bank Transfer</div>
              <div className="text-xs text-neutral-500">Manual bank transfers.</div>
            </div>
            <Switch
              checked={Boolean(settings?.bankTransferEnabled)}
              disabled={isLoading || isSaving || !settings}
              onCheckedChange={(checked) => updatePaymentSettings({ bankTransferEnabled: checked })}
            />
          </div>
          <div className="flex items-center justify-between rounded-lg border border-neutral-200 px-4 py-3">
            <div>
              <div className="text-sm font-medium text-neutral-900">Cash on Delivery</div>
              <div className="text-xs text-neutral-500">Collect payment on delivery.</div>
            </div>
            <Switch
              checked={Boolean(settings?.codEnabled)}
              disabled={isLoading || isSaving || !settings}
              onCheckedChange={(checked) => updatePaymentSettings({ codEnabled: checked })}
            />
          </div>
        </CardContent>
      </Card>
      <div className="grid gap-6 md:grid-cols-2">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Bank Transfer</CardTitle>
            <CardDescription className="text-neutral-500">
              Bank account information for transfer payments.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/payment/bank">
              Configure Bank Transfer
            </Link>
          </CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Cash on Delivery</CardTitle>
            <CardDescription className="text-neutral-500">
              COD enable/fee settings.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link className="text-sm font-medium text-neutral-900 hover:underline" href="/admin/settings/payment/cod">
              Configure COD
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
