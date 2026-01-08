import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import type { Timestamp } from "@bufbuild/protobuf/wkt";
import {
  BackofficeService,
  ListProductsAdminRequestSchema,
  ListVariantsAdminRequestSchema,
  ListSkusAdminRequestSchema,
  CreateProductRequestSchema,
  UpdateProductRequestSchema,
  CreateVariantRequestSchema,
  UpdateVariantRequestSchema,
  ListInventoryStocksRequestSchema,
  ListInventoryMovementsRequestSchema,
  SetInventoryRequestSchema,
  AdjustInventoryRequestSchema,
  TransferInventoryRequestSchema,
  ListMediaAssetsRequestSchema,
  CreateMediaAssetRequestSchema,
  CreateMediaUploadUrlRequestSchema,
  UpdateMediaAssetTagsRequestSchema,
  DeleteMediaAssetRequestSchema,
  ListSkuImagesRequestSchema,
  SetSkuImagesRequestSchema,
  ListDigitalAssetsRequestSchema,
  CreateDigitalAssetRequestSchema,
  CreateDigitalUploadUrlRequestSchema,
  CreateDigitalDownloadUrlRequestSchema,
  ListProductMetafieldDefinitionsRequestSchema,
  CreateProductMetafieldDefinitionRequestSchema,
  UpdateProductMetafieldDefinitionRequestSchema,
  ProductMetafieldDefinitionInputSchema,
  ListProductMetafieldValuesRequestSchema,
  UpsertProductMetafieldValueRequestSchema,
} from "@/gen/ecommerce/v1/backoffice_pb";

const client = createServiceClient(BackofficeService);

export async function listProductsAdmin() {
  return client.listProducts(create(ListProductsAdminRequestSchema, {}));
}

export async function listVariantsAdmin(params: { productId: string }) {
  return client.listVariants(
    create(ListVariantsAdminRequestSchema, {
      productId: params.productId,
    })
  );
}

export async function listSkusAdmin(params: { query?: string }) {
  return client.listSkus(
    create(ListSkusAdminRequestSchema, {
      query: params.query || "",
    })
  );
}

export async function createProduct(params: {
  vendorId?: string;
  title: string;
  description: string;
  status: string;
  taxRuleId?: string;
  saleStartAt?: Timestamp;
  saleEndAt?: Timestamp;
  primaryCategoryId: string;
  categoryIds: string[];
  variantAxes?: { name: string; position?: number }[];
  defaultVariant?: {
    sku: string;
    janCode?: string;
    fulfillmentType: string;
    priceAmount: number;
    compareAtAmount?: number;
    currency: string;
    status: string;
  };
}) {
  const variantAxes = params.variantAxes ?? [];
  if (params.defaultVariant) {
    const price = params.defaultVariant.priceAmount;
    if (price === undefined || price === null) {
      throw new Error("default_variant.priceAmount is required.");
    }
    if (!Number.isFinite(price)) {
      throw new Error("default_variant.priceAmount must be a number.");
    }
    if (params.defaultVariant.compareAtAmount !== undefined) {
      if (!Number.isFinite(params.defaultVariant.compareAtAmount)) {
        throw new Error("default_variant.compareAtAmount must be a number.");
      }
    }
    if (!params.defaultVariant.currency) {
      throw new Error("default_variant.currency is required.");
    }
    if (!params.defaultVariant.sku.trim()) {
      throw new Error("default_variant.sku is required.");
    }
    if (!params.defaultVariant.fulfillmentType.trim()) {
      throw new Error("default_variant.fulfillmentType is required.");
    }
    if (!params.defaultVariant.status.trim()) {
      throw new Error("default_variant.status is required.");
    }
  }
  const payload: {
    vendorId: string;
    title: string;
    description: string;
    status: string;
    taxRuleId: string;
    saleStartAt?: Timestamp;
    saleEndAt?: Timestamp;
    primaryCategoryId: string;
    categoryIds: string[];
    variantAxes: { name: string; position: number }[];
    defaultVariant?: {
      sku: string;
      janCode?: string;
      fulfillmentType: string;
      price: { amount: bigint; currency: string };
      compareAt?: { amount: bigint; currency: string };
      status: string;
    };
  } = {
    vendorId: params.vendorId || "",
    title: params.title,
    description: params.description,
    status: params.status,
    taxRuleId: params.taxRuleId || "",
    primaryCategoryId: params.primaryCategoryId,
    categoryIds: params.categoryIds,
    variantAxes: variantAxes.map((axis, index) => ({
      name: axis.name,
      position: axis.position ?? index + 1,
    })),
  };

  if (params.saleStartAt) {
    payload.saleStartAt = params.saleStartAt;
  }
  if (params.saleEndAt) {
    payload.saleEndAt = params.saleEndAt;
  }

  if (params.defaultVariant) {
    payload.defaultVariant = {
      sku: params.defaultVariant.sku,
      janCode: params.defaultVariant.janCode || "",
      fulfillmentType: params.defaultVariant.fulfillmentType,
      price: {
        amount: BigInt(params.defaultVariant.priceAmount),
        currency: params.defaultVariant.currency || "JPY",
      },
      compareAt:
        typeof params.defaultVariant.compareAtAmount === "number"
          ? {
              amount: BigInt(params.defaultVariant.compareAtAmount),
              currency: params.defaultVariant.currency || "JPY",
            }
          : undefined,
      status: params.defaultVariant.status,
    };
  }

  return client.createProduct(create(CreateProductRequestSchema, payload));
}

export async function updateProduct(params: {
  productId: string;
  title: string;
  description: string;
  status: string;
  taxRuleId?: string;
  saleStartAt?: Timestamp;
  saleEndAt?: Timestamp;
  applyTaxRuleToVariants?: boolean;
  primaryCategoryId: string;
  categoryIds: string[];
}) {
  const payload: {
    productId: string;
    title: string;
    description: string;
    status: string;
    taxRuleId: string;
    saleStartAt?: Timestamp;
    saleEndAt?: Timestamp;
    applyTaxRuleToVariants: boolean;
    primaryCategoryId: string;
    categoryIds: string[];
  } = {
    productId: params.productId,
    title: params.title,
    description: params.description,
    status: params.status,
    taxRuleId: params.taxRuleId || "",
    applyTaxRuleToVariants: Boolean(params.applyTaxRuleToVariants),
    primaryCategoryId: params.primaryCategoryId,
    categoryIds: params.categoryIds,
  };

  if (params.saleStartAt) {
    payload.saleStartAt = params.saleStartAt;
  }
  if (params.saleEndAt) {
    payload.saleEndAt = params.saleEndAt;
  }

  return client.updateProduct(create(UpdateProductRequestSchema, payload));
}

export async function createVariant(params: {
  productId: string;
  sku: string;
  janCode?: string;
  fulfillmentType: string;
  priceAmount: number;
  compareAtAmount?: number;
  currency: string;
  status: string;
  axisValues?: { name: string; value: string }[];
}) {
  return client.createVariant(
    create(CreateVariantRequestSchema, {
      productId: params.productId,
      sku: params.sku,
      janCode: params.janCode || "",
      fulfillmentType: params.fulfillmentType,
      price: { amount: BigInt(params.priceAmount), currency: params.currency },
      compareAt:
        typeof params.compareAtAmount === "number"
          ? { amount: BigInt(params.compareAtAmount), currency: params.currency }
          : undefined,
      status: params.status,
      axisValues: params.axisValues ?? [],
    })
  );
}

export async function updateVariant(params: {
  variantId: string;
  priceAmount: number;
  compareAtAmount?: number;
  currency: string;
  status: string;
  fulfillmentType?: string;
  janCode?: string;
  axisValues?: { name: string; value: string }[];
}) {
  return client.updateVariant(
    create(UpdateVariantRequestSchema, {
      variantId: params.variantId,
      price: { amount: BigInt(params.priceAmount), currency: params.currency },
      compareAt:
        typeof params.compareAtAmount === "number"
          ? { amount: BigInt(params.compareAtAmount), currency: params.currency }
          : undefined,
      status: params.status,
      fulfillmentType: params.fulfillmentType || "",
      janCode: params.janCode || "",
      axisValues: params.axisValues ?? [],
    })
  );
}

export async function listInventoryStocks(params: {
  skuId?: string;
  locationId?: string;
  pageToken?: string;
  pageSize?: number;
}) {
  return client.listInventoryStocks(
    create(ListInventoryStocksRequestSchema, {
      skuId: params.skuId || "",
      locationId: params.locationId || "",
      page: {
        pageToken: params.pageToken || "",
        pageSize: params.pageSize ?? 50,
      },
    })
  );
}

export async function listInventoryMovements(params: {
  skuId?: string;
  locationId?: string;
  movementType?: string;
  pageToken?: string;
  pageSize?: number;
}) {
  return client.listInventoryMovements(
    create(ListInventoryMovementsRequestSchema, {
      skuId: params.skuId || "",
      locationId: params.locationId || "",
      movementType: params.movementType || "",
      page: {
        pageToken: params.pageToken || "",
        pageSize: params.pageSize ?? 50,
      },
    })
  );
}

export async function setInventory(params: {
  skuId: string;
  locationId: string;
  onHand: number;
  reserved: number;
}) {
  return client.setInventory(
    create(SetInventoryRequestSchema, {
      skuId: params.skuId,
      locationId: params.locationId,
      onHand: params.onHand,
      reserved: params.reserved,
    })
  );
}

export async function adjustInventory(params: {
  skuId: string;
  locationId: string;
  delta: number;
  reason?: string;
}) {
  return client.adjustInventory(
    create(AdjustInventoryRequestSchema, {
      skuId: params.skuId,
      locationId: params.locationId,
      delta: params.delta,
      reason: params.reason || "",
    })
  );
}

export async function transferInventory(params: {
  skuId: string;
  fromLocationId: string;
  toLocationId: string;
  quantity: number;
  reason?: string;
}) {
  return client.transferInventory(
    create(TransferInventoryRequestSchema, {
      skuId: params.skuId,
      fromLocationId: params.fromLocationId,
      toLocationId: params.toLocationId,
      quantity: params.quantity,
      reason: params.reason || "",
    })
  );
}

export async function listMediaAssets(params: { query?: string }) {
  return client.listMediaAssets(
    create(ListMediaAssetsRequestSchema, {
      query: params.query || "",
    })
  );
}

export async function createMediaAsset(params: {
  publicUrl: string;
  provider?: string;
  bucket?: string;
  objectKey?: string;
  contentType?: string;
  sizeBytes?: number | bigint;
}) {
  return client.createMediaAsset(
    create(CreateMediaAssetRequestSchema, {
      asset: {
        publicUrl: params.publicUrl,
        provider: params.provider || "",
        bucket: params.bucket || "",
        objectKey: params.objectKey || "",
        contentType: params.contentType || "",
        sizeBytes:
          typeof params.sizeBytes === "bigint" ? params.sizeBytes : BigInt(params.sizeBytes ?? 0),
      },
    })
  );
}

export async function createMediaUploadUrl(params: {
  filename: string;
  contentType?: string;
  sizeBytes?: number | bigint;
}) {
  return client.createMediaUploadUrl(
    create(CreateMediaUploadUrlRequestSchema, {
      filename: params.filename,
      contentType: params.contentType || "",
      sizeBytes:
        typeof params.sizeBytes === "bigint" ? params.sizeBytes : BigInt(params.sizeBytes ?? 0),
    })
  );
}

export async function updateMediaAssetTags(params: { assetId: string; tags: string[] }) {
  return client.updateMediaAssetTags(
    create(UpdateMediaAssetTagsRequestSchema, {
      assetId: params.assetId,
      tags: params.tags,
    })
  );
}

export async function deleteMediaAsset(params: { assetId: string }) {
  return client.deleteMediaAsset(
    create(DeleteMediaAssetRequestSchema, {
      assetId: params.assetId,
    })
  );
}

export async function listDigitalAssets(params: { skuId: string }) {
  return client.listDigitalAssets(
    create(ListDigitalAssetsRequestSchema, {
      skuId: params.skuId,
    })
  );
}

export async function createDigitalAsset(params: {
  skuId: string;
  provider: string;
  bucket: string;
  objectKey: string;
  contentType?: string;
  sizeBytes?: number | bigint;
}) {
  return client.createDigitalAsset(
    create(CreateDigitalAssetRequestSchema, {
      skuId: params.skuId,
      asset: {
        provider: params.provider,
        bucket: params.bucket,
        objectKey: params.objectKey,
        contentType: params.contentType || "",
        sizeBytes:
          typeof params.sizeBytes === "bigint" ? params.sizeBytes : BigInt(params.sizeBytes ?? 0),
      },
    })
  );
}

export async function createDigitalUploadUrl(params: {
  skuId: string;
  filename: string;
  contentType?: string;
  sizeBytes?: number | bigint;
}) {
  return client.createDigitalUploadUrl(
    create(CreateDigitalUploadUrlRequestSchema, {
      skuId: params.skuId,
      filename: params.filename,
      contentType: params.contentType || "",
      sizeBytes:
        typeof params.sizeBytes === "bigint" ? params.sizeBytes : BigInt(params.sizeBytes ?? 0),
    })
  );
}

export async function createDigitalDownloadUrl(params: { assetId: string }) {
  return client.createDigitalDownloadUrl(
    create(CreateDigitalDownloadUrlRequestSchema, {
      assetId: params.assetId,
    })
  );
}

export async function listSkuImages(params: { skuId: string }) {
  return client.listSkuImages(
    create(ListSkuImagesRequestSchema, {
      skuId: params.skuId,
    })
  );
}

export async function setSkuImages(params: { skuId: string; images: { assetId: string; position: number }[] }) {
  return client.setSkuImages(
    create(SetSkuImagesRequestSchema, {
      skuId: params.skuId,
      images: params.images,
    })
  );
}

export async function listProductMetafieldDefinitions() {
  return client.listProductMetafieldDefinitions(
    create(ListProductMetafieldDefinitionsRequestSchema, {})
  );
}

export async function createProductMetafieldDefinition(params: {
  namespace: string;
  key: string;
  name: string;
  description?: string;
  valueType: string;
  isList: boolean;
  validationsJson?: string;
  visibilityJson?: string;
}) {
  const definition = create(ProductMetafieldDefinitionInputSchema, {
    namespace: params.namespace,
    key: params.key,
    name: params.name,
    description: params.description ?? "",
    valueType: params.valueType,
    isList: params.isList,
    validationsJson: params.validationsJson ?? "{}",
    visibilityJson: params.visibilityJson ?? "{}",
  });
  return client.createProductMetafieldDefinition(
    create(CreateProductMetafieldDefinitionRequestSchema, { definition })
  );
}

export async function updateProductMetafieldDefinition(params: {
  definitionId: string;
  namespace: string;
  key: string;
  name: string;
  description?: string;
  valueType: string;
  isList: boolean;
  validationsJson?: string;
  visibilityJson?: string;
}) {
  const definition = create(ProductMetafieldDefinitionInputSchema, {
    namespace: params.namespace,
    key: params.key,
    name: params.name,
    description: params.description ?? "",
    valueType: params.valueType,
    isList: params.isList,
    validationsJson: params.validationsJson ?? "{}",
    visibilityJson: params.visibilityJson ?? "{}",
  });
  return client.updateProductMetafieldDefinition(
    create(UpdateProductMetafieldDefinitionRequestSchema, {
      definitionId: params.definitionId,
      definition,
    })
  );
}

export async function listProductMetafieldValues(productId: string) {
  return client.listProductMetafieldValues(
    create(ListProductMetafieldValuesRequestSchema, { productId })
  );
}

export async function upsertProductMetafieldValue(params: {
  productId: string;
  definitionId: string;
  valueJson: string;
}) {
  return client.upsertProductMetafieldValue(
    create(UpsertProductMetafieldValueRequestSchema, {
      productId: params.productId,
      definitionId: params.definitionId,
      valueJson: params.valueJson,
    })
  );
}
