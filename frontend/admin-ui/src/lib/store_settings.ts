import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  StoreSettingsService,
  GetStoreSettingsRequestSchema,
  UpdateStoreSettingsRequestSchema,
  ListStoreLocationsRequestSchema,
  UpsertStoreLocationRequestSchema,
  ListShippingZonesRequestSchema,
  UpsertShippingZoneRequestSchema,
  ListShippingRatesRequestSchema,
  UpsertShippingRateRequestSchema,
  ListTaxRulesRequestSchema,
  UpsertTaxRuleRequestSchema,
  StoreSettingsSchema,
  type StoreSettings,
} from "@/gen/ecommerce/v1/store_settings_pb";
import { MoneySchema } from "@/gen/ecommerce/v1/common_pb";

const client = createServiceClient(StoreSettingsService);

export type StoreSettingsFormValues = {
  storeName: string;
  legalName: string;
  legalNotice: string;
  contactEmail: string;
  contactPhone: string;
  addressPrefecture: string;
  addressCity: string;
  addressLine1: string;
  addressLine2?: string;
  defaultLanguage: string;
  primaryDomain?: string;
  subdomain?: string;
  httpsEnabled: boolean;
  timeZone: string;
  currency: string;
  taxMode: string;
  taxRounding: string;
  orderInitialStatus: string;
  codEnabled: boolean;
  codFee?: { amount?: string; currency?: string };
  codFeeAmount?: string;
  codFeeCurrency?: string;
  bankTransferEnabled: boolean;
  bankName: string;
  bankBranch: string;
  bankAccountType: string;
  bankAccountNumber: string;
  bankAccountName: string;
  theme: string;
  brandColor: string;
  logoUrl?: string;
  faviconUrl?: string;
  skuCodeRegex?: string;
};

export function toStoreSettingsForm(settings: StoreSettings): StoreSettingsFormValues {
  const profile = settings.profile ?? {};
  const contact = settings.contact ?? {};
  const address = settings.address ?? {};
  const domain = settings.domain ?? {};
  const locale = settings.locale ?? {};
  const tax = settings.tax ?? {};
  const order = settings.order ?? {};
  const payment = settings.payment ?? {};
  const bank = payment.bankAccount ?? {};
  const branding = settings.branding ?? {};
  const catalog = settings.catalog ?? {};
  return {
    storeName: profile.storeName || "",
    legalName: profile.legalName || "",
    legalNotice: profile.legalNotice || "",
    contactEmail: contact.contactEmail || "",
    contactPhone: contact.contactPhone || "",
    addressPrefecture: address.addressPrefecture || "",
    addressCity: address.addressCity || "",
    addressLine1: address.addressLine1 || "",
    addressLine2: address.addressLine2 || "",
    defaultLanguage: locale.defaultLanguage || "ja",
    primaryDomain: domain.primaryDomain || "",
    subdomain: domain.subdomain || "",
    httpsEnabled: Boolean(domain.httpsEnabled),
    timeZone: locale.timeZone || "Asia/Tokyo",
    currency: locale.currency || "JPY",
    taxMode: tax.taxMode || "inclusive",
    taxRounding: tax.taxRounding || "round",
    orderInitialStatus: order.orderInitialStatus || "pending_payment",
    codEnabled: Boolean(payment.codEnabled),
    codFee: payment.codFee,
    codFeeAmount: payment.codFee?.amount?.toString() || "",
    codFeeCurrency: payment.codFee?.currency || "",
    bankTransferEnabled: Boolean(payment.bankTransferEnabled),
    bankName: bank.bankName || "",
    bankBranch: bank.bankBranch || "",
    bankAccountType: bank.bankAccountType || "",
    bankAccountNumber: bank.bankAccountNumber || "",
    bankAccountName: bank.bankAccountName || "",
    theme: branding.theme || "default",
    brandColor: branding.brandColor || "#111827",
    logoUrl: branding.logoUrl || "",
    faviconUrl: branding.faviconUrl || "",
    skuCodeRegex: catalog.skuCodeRegex || "",
  };
}

export async function getStoreSettings() {
  const resp = await client.getStoreSettings(create(GetStoreSettingsRequestSchema, {}));
  const settings = resp.settings;
  if (!settings) {
    return { settings: undefined };
  }
  return {
    settings: toStoreSettingsForm(settings),
  };
}

export async function updateStoreSettings(params: {
  settings: StoreSettingsFormValues;
}) {
  const settings = create(StoreSettingsSchema, {
    profile: {
      storeName: params.settings.storeName,
      legalName: params.settings.legalName,
      legalNotice: params.settings.legalNotice,
    },
    contact: {
      contactEmail: params.settings.contactEmail,
      contactPhone: params.settings.contactPhone,
    },
    address: {
      addressPrefecture: params.settings.addressPrefecture,
      addressCity: params.settings.addressCity,
      addressLine1: params.settings.addressLine1,
      addressLine2: params.settings.addressLine2 || "",
    },
    domain: {
      primaryDomain: params.settings.primaryDomain || "",
      subdomain: params.settings.subdomain || "",
      httpsEnabled: params.settings.httpsEnabled,
    },
    locale: {
      defaultLanguage: params.settings.defaultLanguage,
      currency: params.settings.currency,
      timeZone: params.settings.timeZone || "Asia/Tokyo",
    },
    tax: {
      taxMode: params.settings.taxMode,
      taxRounding: params.settings.taxRounding,
    },
    order: {
      orderInitialStatus: params.settings.orderInitialStatus,
    },
    payment: {
      codEnabled: params.settings.codEnabled,
      codFee: create(MoneySchema, {
        amount: BigInt(params.settings.codFeeAmount || "0"),
        currency: params.settings.codFeeCurrency || "JPY",
      }),
      bankTransferEnabled: params.settings.bankTransferEnabled,
      bankAccount: {
        bankName: params.settings.bankName,
        bankBranch: params.settings.bankBranch,
        bankAccountType: params.settings.bankAccountType,
        bankAccountNumber: params.settings.bankAccountNumber,
        bankAccountName: params.settings.bankAccountName,
      },
    },
    branding: {
      theme: params.settings.theme,
      brandColor: params.settings.brandColor,
      logoUrl: params.settings.logoUrl || "",
      faviconUrl: params.settings.faviconUrl || "",
    },
    catalog: {
      skuCodeRegex: params.settings.skuCodeRegex || "",
    },
  });
  const resp = await client.updateStoreSettings(create(UpdateStoreSettingsRequestSchema, { settings }));
  return { settings: resp.settings ? toStoreSettingsForm(resp.settings) : undefined };
}

export async function listStoreLocations() {
  return client.listStoreLocations(create(ListStoreLocationsRequestSchema, {}));
}

export async function upsertStoreLocation(params: {
  code: string;
  name: string;
  status: string;
}) {
  return client.upsertStoreLocation(
    create(UpsertStoreLocationRequestSchema, {
      location: {
        code: params.code,
        name: params.name,
        status: params.status,
      },
    })
  );
}

export async function listTaxRules() {
  return client.listTaxRules(create(ListTaxRulesRequestSchema, {}));
}

export async function upsertTaxRule(params: {
  id?: string;
  name: string;
  rate: number;
  appliesTo: string;
}) {
  return client.upsertTaxRule(
    create(UpsertTaxRuleRequestSchema, {
      rule: {
        id: params.id || "",
        name: params.name,
        rate: params.rate,
        appliesTo: params.appliesTo,
      },
    })
  );
}

export async function listShippingZones() {
  return client.listShippingZones(create(ListShippingZonesRequestSchema, {}));
}

export async function upsertShippingZone(params: {
  id?: string;
  name: string;
  domesticOnly: boolean;
  prefectures: Array<{ code: string; name: string }>;
}) {
  return client.upsertShippingZone(
    create(UpsertShippingZoneRequestSchema, {
      zone: {
        id: params.id || "",
        name: params.name,
        domesticOnly: params.domesticOnly,
        prefectures: params.prefectures,
      },
    })
  );
}

export async function listShippingRates(params: { zoneId: string }) {
  return client.listShippingRates(
    create(ListShippingRatesRequestSchema, {
      zoneId: params.zoneId,
    })
  );
}

export async function upsertShippingRate(params: {
  id?: string;
  zoneId: string;
  name: string;
  minSubtotal?: string;
  maxSubtotal?: string;
  feeAmount: string;
  currency: string;
}) {
  return client.upsertShippingRate(
    create(UpsertShippingRateRequestSchema, {
      rate: {
        id: params.id || "",
        zoneId: params.zoneId,
        name: params.name,
        minSubtotal: params.minSubtotal
          ? create(MoneySchema, {
              amount: BigInt(params.minSubtotal),
              currency: params.currency,
            })
          : undefined,
        maxSubtotal: params.maxSubtotal
          ? create(MoneySchema, {
              amount: BigInt(params.maxSubtotal),
              currency: params.currency,
            })
          : undefined,
        fee: create(MoneySchema, {
          amount: BigInt(params.feeAmount),
          currency: params.currency,
        }),
      },
    })
  );
}
