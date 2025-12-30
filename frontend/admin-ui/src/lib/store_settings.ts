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
} from "@/gen/ecommerce/v1/store_settings_pb";
import { MoneySchema } from "@/gen/ecommerce/v1/common_pb";

const client = createServiceClient(StoreSettingsService);

export async function getStoreSettings() {
  return client.getStoreSettings(create(GetStoreSettingsRequestSchema, {}));
}

export async function updateStoreSettings(params: {
  settings: {
    storeName: string;
    legalName: string;
    contactEmail: string;
    contactPhone: string;
    addressPrefecture: string;
    addressCity: string;
    addressLine1: string;
    addressLine2?: string;
    legalNotice: string;
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
    codFeeAmount: string;
    codFeeCurrency: string;
    bankName: string;
    bankBranch: string;
    bankAccountType: string;
    bankAccountNumber: string;
    bankAccountName: string;
    theme: string;
    brandColor: string;
    logoUrl?: string;
    faviconUrl?: string;
  };
}) {
  const settings = create(StoreSettingsSchema, {
    storeName: params.settings.storeName,
    legalName: params.settings.legalName,
    contactEmail: params.settings.contactEmail,
    contactPhone: params.settings.contactPhone,
    addressPrefecture: params.settings.addressPrefecture,
    addressCity: params.settings.addressCity,
    addressLine1: params.settings.addressLine1,
    addressLine2: params.settings.addressLine2 || "",
    legalNotice: params.settings.legalNotice,
    defaultLanguage: params.settings.defaultLanguage,
    primaryDomain: params.settings.primaryDomain || "",
    subdomain: params.settings.subdomain || "",
    httpsEnabled: params.settings.httpsEnabled,
    timeZone: params.settings.timeZone || "Asia/Tokyo",
    currency: params.settings.currency,
    taxMode: params.settings.taxMode,
    taxRounding: params.settings.taxRounding,
    orderInitialStatus: params.settings.orderInitialStatus,
    codEnabled: params.settings.codEnabled,
    codFee: create(MoneySchema, {
      amount: BigInt(params.settings.codFeeAmount || "0"),
      currency: params.settings.codFeeCurrency || "JPY",
    }),
    bankName: params.settings.bankName,
    bankBranch: params.settings.bankBranch,
    bankAccountType: params.settings.bankAccountType,
    bankAccountNumber: params.settings.bankAccountNumber,
    bankAccountName: params.settings.bankAccountName,
    theme: params.settings.theme,
    brandColor: params.settings.brandColor,
    logoUrl: params.settings.logoUrl || "",
    faviconUrl: params.settings.faviconUrl || "",
  });
  return client.updateStoreSettings(create(UpdateStoreSettingsRequestSchema, { settings }));
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
