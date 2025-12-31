import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  BackofficeService,
  ListProductsAdminRequestSchema,
  ListVariantsAdminRequestSchema,
  ListSkusAdminRequestSchema,
  CreateProductRequestSchema,
  UpdateProductRequestSchema,
  CreateVariantRequestSchema,
  UpdateVariantRequestSchema,
  SetInventoryRequestSchema,
  ListMediaAssetsRequestSchema,
  CreateMediaAssetRequestSchema,
  CreateMediaUploadUrlRequestSchema,
  ListSkuImagesRequestSchema,
  SetSkuImagesRequestSchema,
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
  variantAxes?: { name: string; position?: number }[];
  defaultVariant?: {
    sku: string;
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
    variantAxes: { name: string; position: number }[];
    defaultVariant?: {
      sku: string;
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
    variantAxes: variantAxes.map((axis, index) => ({
      name: axis.name,
      position: axis.position ?? index + 1,
    })),
  };

  if (params.defaultVariant) {
    payload.defaultVariant = {
      sku: params.defaultVariant.sku,
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
}) {
  return client.updateProduct(
    create(UpdateProductRequestSchema, {
      productId: params.productId,
      title: params.title,
      description: params.description,
      status: params.status,
      taxRuleId: params.taxRuleId || "",
    })
  );
}

export async function createVariant(params: {
  productId: string;
  sku: string;
  fulfillmentType: string;
  priceAmount: number;
  compareAtAmount?: number;
  currency: string;
  status: string;
}) {
  return client.createVariant(
    create(CreateVariantRequestSchema, {
      productId: params.productId,
      sku: params.sku,
      fulfillmentType: params.fulfillmentType,
      price: { amount: BigInt(params.priceAmount), currency: params.currency },
      compareAt:
        typeof params.compareAtAmount === "number"
          ? { amount: BigInt(params.compareAtAmount), currency: params.currency }
          : undefined,
      status: params.status,
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
    })
  );
}

export async function setInventory(params: {
  variantId: string;
  locationId: string;
  stock: number;
  reserved: number;
}) {
  return client.setInventory(
    create(SetInventoryRequestSchema, {
      variantId: params.variantId,
      locationId: params.locationId,
      stock: params.stock,
      reserved: params.reserved,
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
  sizeBytes?: number;
}) {
  return client.createMediaAsset(
    create(CreateMediaAssetRequestSchema, {
      asset: {
        publicUrl: params.publicUrl,
        provider: params.provider || "",
        bucket: params.bucket || "",
        objectKey: params.objectKey || "",
        contentType: params.contentType || "",
        sizeBytes: params.sizeBytes ?? 0,
      },
    })
  );
}

export async function createMediaUploadUrl(params: {
  filename: string;
  contentType?: string;
  sizeBytes?: number;
}) {
  return client.createMediaUploadUrl(
    create(CreateMediaUploadUrlRequestSchema, {
      filename: params.filename,
      contentType: params.contentType || "",
      sizeBytes: params.sizeBytes ?? 0,
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
