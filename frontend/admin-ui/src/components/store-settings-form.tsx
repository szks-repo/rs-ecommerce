"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { getStoreSettings, updateStoreSettings } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import { useApiCall } from "@/lib/use-api-call";

type StoreSettingsSection =
  | "basic"
  | "profile"
  | "address"
  | "contact-email"
  | "contact-phone"
  | "legal"
  | "sku"
  | "locale"
  | "domain"
  | "payment"
  | "payment-cod"
  | "payment-bank"
  | "tax"
  | "appearance";

export default function StoreSettingsForm({
  sections,
  submitLabel,
}: {
  sections?: StoreSettingsSection[];
  submitLabel?: string;
}) {
  const visible = sections ?? ["basic", "payment", "tax", "appearance"];
  const showBasicSection = (section: StoreSettingsSection) =>
    visible.includes("basic") || visible.includes(section);
  const [storeName, setStoreName] = useState("");
  const [legalName, setLegalName] = useState("");
  const [contactEmail, setContactEmail] = useState("");
  const [contactPhone, setContactPhone] = useState("");
  const [addressPrefecture, setAddressPrefecture] = useState("");
  const [addressCity, setAddressCity] = useState("");
  const [addressLine1, setAddressLine1] = useState("");
  const [addressLine2, setAddressLine2] = useState("");
  const [legalNotice, setLegalNotice] = useState("");
  const [defaultLanguage, setDefaultLanguage] = useState("ja");
  const [primaryDomain, setPrimaryDomain] = useState("");
  const [subdomain, setSubdomain] = useState("");
  const [httpsEnabled, setHttpsEnabled] = useState(false);
  const [timeZone, setTimeZone] = useState("Asia/Tokyo");
  const [currency, setCurrency] = useState("JPY");
  const [taxMode, setTaxMode] = useState("inclusive");
  const [taxRounding, setTaxRounding] = useState("round");
  const [orderInitialStatus, setOrderInitialStatus] = useState("pending_payment");
  const [codEnabled, setCodEnabled] = useState(false);
  const [codFeeAmount, setCodFeeAmount] = useState("0");
  const [codFeeCurrency, setCodFeeCurrency] = useState("JPY");
  const [bankTransferEnabled, setBankTransferEnabled] = useState(false);
  const [bankName, setBankName] = useState("");
  const [bankBranch, setBankBranch] = useState("");
  const [bankAccountType, setBankAccountType] = useState("");
  const [bankAccountNumber, setBankAccountNumber] = useState("");
  const [bankAccountName, setBankAccountName] = useState("");
  const [theme, setTheme] = useState("default");
  const [brandColor, setBrandColor] = useState("#111827");
  const [logoUrl, setLogoUrl] = useState("");
  const [faviconUrl, setFaviconUrl] = useState("");
  const [skuCodeRegex, setSkuCodeRegex] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const { call } = useApiCall();

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    setIsLoading(true);
    call(() => getStoreSettings(), {
      errorTitle: "Load failed",
      errorDescription: "Failed to load store settings",
    })
      .then((data) => {
        if (!data) return;
        const settings = data.settings;
        if (!settings) {
          return;
        }
        setStoreName(settings.storeName);
        setLegalName(settings.legalName);
        setContactEmail(settings.contactEmail);
        setContactPhone(settings.contactPhone);
        setAddressPrefecture(settings.addressPrefecture);
        setAddressCity(settings.addressCity);
        setAddressLine1(settings.addressLine1);
        setAddressLine2(settings.addressLine2 || "");
        setLegalNotice(settings.legalNotice);
        setDefaultLanguage(settings.defaultLanguage);
        setPrimaryDomain(settings.primaryDomain || "");
        setSubdomain(settings.subdomain || "");
        setHttpsEnabled(Boolean(settings.httpsEnabled));
        setTimeZone(settings.timeZone || "Asia/Tokyo");
        window.sessionStorage.setItem("store_time_zone", settings.timeZone || "Asia/Tokyo");
        setCurrency(settings.currency || "JPY");
        setTaxMode(settings.taxMode || "inclusive");
        setTaxRounding(settings.taxRounding || "round");
        setOrderInitialStatus(settings.orderInitialStatus || "pending_payment");
        setCodEnabled(Boolean(settings.codEnabled));
        setCodFeeAmount(settings.codFee ? settings.codFee.amount.toString() : "0");
        setCodFeeCurrency(settings.codFee?.currency || "JPY");
        setBankTransferEnabled(Boolean(settings.bankTransferEnabled));
        setBankName(settings.bankName);
        setBankBranch(settings.bankBranch);
        setBankAccountType(settings.bankAccountType);
        setBankAccountNumber(settings.bankAccountNumber);
        setBankAccountName(settings.bankAccountName);
        setTheme(settings.theme || "default");
        setBrandColor(settings.brandColor || "#111827");
        setLogoUrl(settings.logoUrl || "");
        setFaviconUrl(settings.faviconUrl || "");
        setSkuCodeRegex(settings.skuCodeRegex || "");
        window.sessionStorage.setItem("store_sku_code_regex", settings.skuCodeRegex || "");
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, []);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSaving(true);
    if (!getActiveAccessToken()) {
      await call(
        async () => {
          throw new Error("access_token is missing. Please sign in first.");
        },
        { errorTitle: "Update failed", errorDescription: "Sign in is required." }
      );
      setIsSaving(false);
      return;
    }
    const needsCodValidation = visible.includes("payment") || visible.includes("payment-cod");
    const feeAmount = codFeeAmount.trim().length > 0 ? codFeeAmount.trim() : "0";
    if (needsCodValidation && !/^-?\\d+$/.test(feeAmount)) {
      await call(
        async () => {
          throw new Error("COD fee amount must be an integer.");
        },
        { errorTitle: "Update failed", errorDescription: "COD fee amount must be an integer." }
      );
      setIsSaving(false);
      return;
    }
    const resp = await call(
      () =>
        updateStoreSettings({
          settings: {
            storeName,
            legalName,
            contactEmail,
            contactPhone,
            addressPrefecture,
            addressCity,
            addressLine1,
            addressLine2,
            legalNotice,
            defaultLanguage,
            primaryDomain,
            subdomain,
            httpsEnabled,
            timeZone,
            currency,
            taxMode,
            taxRounding,
            orderInitialStatus,
            codEnabled,
            codFeeAmount: needsCodValidation ? feeAmount : "0",
            codFeeCurrency: codFeeCurrency || "JPY",
            bankTransferEnabled,
            bankName,
            bankBranch,
            bankAccountType,
            bankAccountNumber,
            bankAccountName,
            theme,
            brandColor,
            logoUrl,
            faviconUrl,
            skuCodeRegex,
          },
        }),
      {
        success: {
          title: "Settings updated",
          description: "Store settings have been updated.",
        },
        errorTitle: "Update failed",
        errorDescription: "Unknown error",
      }
    );
    if (resp) {
      window.sessionStorage.setItem("store_time_zone", timeZone || "Asia/Tokyo");
      window.sessionStorage.setItem("store_sku_code_regex", skuCodeRegex || "");
    }
    setIsSaving(false);
  }

  return (
    <form className="space-y-6" onSubmit={handleSubmit}>
      {showBasicSection("profile") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Store Profile</CardTitle>
            <CardDescription className="text-neutral-500">
              Store name and legal name.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="storeName">Store Name</Label>
              <Input id="storeName" value={storeName} onChange={(e) => setStoreName(e.target.value)} required />
            </div>
            <div className="space-y-2">
              <Label htmlFor="legalName">Legal Name</Label>
              <Input id="legalName" value={legalName} onChange={(e) => setLegalName(e.target.value)} required />
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("contact-email") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Contact Email</CardTitle>
            <CardDescription className="text-neutral-500">
              Email used for customer inquiries.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <Label htmlFor="contactEmail">Contact Email</Label>
              <Input id="contactEmail" type="email" value={contactEmail} onChange={(e) => setContactEmail(e.target.value)} required />
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("contact-phone") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Contact Phone</CardTitle>
            <CardDescription className="text-neutral-500">
              Phone number shown in store info.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <Label htmlFor="contactPhone">Contact Phone</Label>
              <Input id="contactPhone" value={contactPhone} onChange={(e) => setContactPhone(e.target.value)} required />
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("address") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Address</CardTitle>
            <CardDescription className="text-neutral-500">
              Business address used for legal notice.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="addressPrefecture">Prefecture</Label>
              <Input id="addressPrefecture" value={addressPrefecture} onChange={(e) => setAddressPrefecture(e.target.value)} required />
            </div>
            <div className="space-y-2">
              <Label htmlFor="addressCity">City</Label>
              <Input id="addressCity" value={addressCity} onChange={(e) => setAddressCity(e.target.value)} required />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="addressLine1">Address Line 1</Label>
              <Input id="addressLine1" value={addressLine1} onChange={(e) => setAddressLine1(e.target.value)} required />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="addressLine2">Address Line 2</Label>
              <Input id="addressLine2" value={addressLine2} onChange={(e) => setAddressLine2(e.target.value)} />
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("legal") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Legal Notice</CardTitle>
            <CardDescription className="text-neutral-500">
              "Tokutei Shotorihiki-ho" disclosure.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <Label htmlFor="legalNotice">Legal Notice</Label>
              <Textarea id="legalNotice" value={legalNotice} onChange={(e) => setLegalNotice(e.target.value)} rows={4} required />
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("sku") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>SKU Rule</CardTitle>
            <CardDescription className="text-neutral-500">
              Client-side validation rule for SKU codes.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <Label htmlFor="skuCodeRegex">SKU Code Rule (Regex)</Label>
              <Input
                id="skuCodeRegex"
                value={skuCodeRegex}
                onChange={(e) => setSkuCodeRegex(e.target.value)}
                placeholder="Optional: ^[A-Z0-9-]{4,}$"
              />
              <p className="text-xs text-neutral-500">
                Used for client-side validation when creating SKUs. Leave blank to skip validation.
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("locale") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Locale</CardTitle>
            <CardDescription className="text-neutral-500">
              Language, timezone, and currency defaults.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="defaultLanguage">Default Language</Label>
              <Select value={defaultLanguage} onValueChange={setDefaultLanguage}>
                <SelectTrigger id="defaultLanguage" className="bg-white">
                  <SelectValue placeholder="Select language" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ja">Japanese (ja)</SelectItem>
                  <SelectItem value="en">English (en)</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="timeZone">Time Zone</Label>
              <Select value={timeZone} onValueChange={setTimeZone}>
                <SelectTrigger id="timeZone" className="bg-white">
                  <SelectValue placeholder="Select time zone" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Asia/Tokyo">Asia/Tokyo</SelectItem>
                  <SelectItem value="UTC">UTC</SelectItem>
                  <SelectItem value="America/New_York">America/New_York</SelectItem>
                  <SelectItem value="Europe/London">Europe/London</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="currency">Currency</Label>
              <Select value={currency} onValueChange={setCurrency}>
                <SelectTrigger id="currency" className="bg-white">
                  <SelectValue placeholder="Select currency" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="JPY">JPY</SelectItem>
                  <SelectItem value="USD">USD</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>
      )}

      {showBasicSection("domain") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Domain</CardTitle>
            <CardDescription className="text-neutral-500">
              Storefront domain settings.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="primaryDomain">Primary Domain</Label>
              <Input id="primaryDomain" value={primaryDomain} onChange={(e) => setPrimaryDomain(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="subdomain">Subdomain</Label>
              <Input id="subdomain" value={subdomain} onChange={(e) => setSubdomain(e.target.value)} />
            </div>
            <div className="flex items-center justify-between md:col-span-2">
              <div>
                <Label htmlFor="httpsEnabled">HTTPS Enabled</Label>
                <p className="text-xs text-neutral-500">Enable HTTPS for storefront domains.</p>
              </div>
              <Switch id="httpsEnabled" checked={httpsEnabled} onCheckedChange={setHttpsEnabled} />
            </div>
          </CardContent>
        </Card>
      )}

      {(visible.includes("payment") || visible.includes("payment-cod")) && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Payment: Cash on Delivery</CardTitle>
            <CardDescription className="text-neutral-500">
              Configure COD (cash on delivery).
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="flex items-center justify-between md:col-span-2">
              <div>
                <Label htmlFor="codEnabled">Cash on Delivery</Label>
                <p className="text-xs text-neutral-500">Enable COD payments.</p>
              </div>
              <Switch id="codEnabled" checked={codEnabled} onCheckedChange={setCodEnabled} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="codFeeAmount">COD Fee Amount</Label>
              <Input id="codFeeAmount" value={codFeeAmount} onChange={(e) => setCodFeeAmount(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="codFeeCurrency">COD Fee Currency</Label>
              <Select value={codFeeCurrency} onValueChange={setCodFeeCurrency}>
                <SelectTrigger id="codFeeCurrency" className="bg-white">
                  <SelectValue placeholder="Select currency" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="JPY">JPY</SelectItem>
                  <SelectItem value="USD">USD</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>
      )}

      {(visible.includes("payment") || visible.includes("payment-bank")) && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Payment: Bank Transfer</CardTitle>
            <CardDescription className="text-neutral-500">
              Configure bank transfer details.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="flex items-center justify-between md:col-span-2">
              <div>
                <Label htmlFor="bankTransferEnabled">Bank Transfer</Label>
                <p className="text-xs text-neutral-500">Enable bank transfer payments.</p>
              </div>
              <Switch
                id="bankTransferEnabled"
                checked={bankTransferEnabled}
                onCheckedChange={setBankTransferEnabled}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="bankName">Bank Name</Label>
              <Input
                id="bankName"
                value={bankName}
                onChange={(e) => setBankName(e.target.value)}
                required
                disabled={!bankTransferEnabled}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="bankBranch">Bank Branch</Label>
              <Input
                id="bankBranch"
                value={bankBranch}
                onChange={(e) => setBankBranch(e.target.value)}
                required
                disabled={!bankTransferEnabled}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="bankAccountType">Account Type</Label>
              <Input
                id="bankAccountType"
                value={bankAccountType}
                onChange={(e) => setBankAccountType(e.target.value)}
                required
                disabled={!bankTransferEnabled}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="bankAccountNumber">Account Number</Label>
              <Input
                id="bankAccountNumber"
                value={bankAccountNumber}
                onChange={(e) => setBankAccountNumber(e.target.value)}
                required
                disabled={!bankTransferEnabled}
              />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="bankAccountName">Account Name</Label>
              <Input
                id="bankAccountName"
                value={bankAccountName}
                onChange={(e) => setBankAccountName(e.target.value)}
                required
                disabled={!bankTransferEnabled}
              />
            </div>
          </CardContent>
        </Card>
      )}

      {visible.includes("tax") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Tax & Order Rules</CardTitle>
            <CardDescription className="text-neutral-500">
              Default tax behavior and initial order status.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="taxMode">Tax Mode</Label>
              <Select value={taxMode} onValueChange={setTaxMode}>
                <SelectTrigger id="taxMode" className="bg-white">
                  <SelectValue placeholder="Select mode" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="inclusive">Inclusive</SelectItem>
                  <SelectItem value="exclusive">Exclusive</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="taxRounding">Tax Rounding</Label>
              <Select value={taxRounding} onValueChange={setTaxRounding}>
                <SelectTrigger id="taxRounding" className="bg-white">
                  <SelectValue placeholder="Select rounding" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="round">Round</SelectItem>
                  <SelectItem value="floor">Floor</SelectItem>
                  <SelectItem value="ceil">Ceil</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="orderInitialStatus">Order Initial Status</Label>
              <Select value={orderInitialStatus} onValueChange={setOrderInitialStatus}>
                <SelectTrigger id="orderInitialStatus" className="bg-white">
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="pending_payment">pending_payment</SelectItem>
                  <SelectItem value="pending_shipment">pending_shipment</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>
      )}

      {visible.includes("appearance") && (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Appearance</CardTitle>
            <CardDescription className="text-neutral-500">
              Theme and brand assets.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="theme">Theme</Label>
              <Input id="theme" value={theme} onChange={(e) => setTheme(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="brandColor">Brand Color</Label>
              <Input id="brandColor" value={brandColor} onChange={(e) => setBrandColor(e.target.value)} />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="logoUrl">Logo URL</Label>
              <Input id="logoUrl" value={logoUrl} onChange={(e) => setLogoUrl(e.target.value)} />
            </div>
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="faviconUrl">Favicon URL</Label>
              <Input id="faviconUrl" value={faviconUrl} onChange={(e) => setFaviconUrl(e.target.value)} />
            </div>
          </CardContent>
        </Card>
      )}

      <div className="flex items-center justify-end">
        <Button type="submit" disabled={isSaving || isLoading}>
          {isSaving ? "Saving..." : submitLabel ?? "Save Settings"}
        </Button>
      </div>
    </form>
  );
}
