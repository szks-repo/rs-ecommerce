import { createClient } from "@/lib/connect";
import { BackofficeService } from "@/gen/ecommerce/v1/backoffice_connect";
import {
  ListProductsAdminRequest,
  ListVariantsAdminRequest,
  CreateProductRequest,
  UpdateProductRequest,
  CreateVariantRequest,
  UpdateVariantRequest,
  SetInventoryRequest,
} from "@/gen/ecommerce/v1/backoffice_pb";

const client = createClient(BackofficeService);

export async function listProductsAdmin() {
  return client.listProducts(new ListProductsAdminRequest({}));
}

export async function listVariantsAdmin(params: { productId: string }) {
  return client.listVariants(
    new ListVariantsAdminRequest({
      productId: params.productId,
    })
  );
}

export async function createProduct(params: {
  vendorId?: string;
  title: string;
  description: string;
  status: string;
}) {
  return client.createProduct(
    new CreateProductRequest({
      vendorId: params.vendorId || "",
      title: params.title,
      description: params.description,
      status: params.status,
    })
  );
}

export async function updateProduct(params: {
  productId: string;
  title: string;
  description: string;
  status: string;
}) {
  return client.updateProduct(
    new UpdateProductRequest({
      productId: params.productId,
      title: params.title,
      description: params.description,
      status: params.status,
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
    new CreateVariantRequest({
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
    new UpdateVariantRequest({
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
    new SetInventoryRequest({
      variantId: params.variantId,
      locationId: params.locationId,
      stock: params.stock,
      reserved: params.reserved,
    })
  );
}
