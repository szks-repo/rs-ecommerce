import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  BackofficeService,
  ListProductsAdminRequestSchema,
  ListVariantsAdminRequestSchema,
  CreateProductRequestSchema,
  UpdateProductRequestSchema,
  CreateVariantRequestSchema,
  UpdateVariantRequestSchema,
  SetInventoryRequestSchema,
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

export async function createProduct(params: {
  vendorId?: string;
  title: string;
  description: string;
  status: string;
  taxRuleId?: string;
}) {
  return client.createProduct(
    create(CreateProductRequestSchema, {
      vendorId: params.vendorId || "",
      title: params.title,
      description: params.description,
      status: params.status,
      taxRuleId: params.taxRuleId || "",
    })
  );
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
      price: { amount: params.priceAmount, currency: params.currency },
      compareAt:
        typeof params.compareAtAmount === "number"
          ? { amount: params.compareAtAmount, currency: params.currency }
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
      price: { amount: params.priceAmount, currency: params.currency },
      compareAt:
        typeof params.compareAtAmount === "number"
          ? { amount: params.compareAtAmount, currency: params.currency }
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
