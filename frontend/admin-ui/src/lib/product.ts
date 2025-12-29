import { rpcFetch } from "@/lib/api";

type ProductAdmin = {
  id: string;
  storeId: string;
  vendorId?: string;
  title: string;
  description: string;
  status: string;
};

type VariantAdmin = {
  id: string;
  productId: string;
  sku: string;
  fulfillmentType: string;
  status: string;
};

type InventoryAdmin = {
  variantId: string;
  locationId: string;
  stock: number;
  reserved: number;
};

export async function createProduct(params: {
  storeId: string;
  vendorId?: string;
  title: string;
  description: string;
  status: string;
}) {
  return rpcFetch<{ product: ProductAdmin }>("/rpc/ecommerce.v1.BackofficeService/CreateProduct", {
    store: { storeId: params.storeId },
    vendorId: params.vendorId || "",
    title: params.title,
    description: params.description,
    status: params.status,
  });
}

export async function createVariant(params: {
  storeId: string;
  productId: string;
  sku: string;
  fulfillmentType: string;
  priceAmount: number;
  compareAtAmount?: number;
  currency: string;
  status: string;
}) {
  return rpcFetch<{ variant: VariantAdmin }>("/rpc/ecommerce.v1.BackofficeService/CreateVariant", {
    store: { storeId: params.storeId },
    productId: params.productId,
    sku: params.sku,
    fulfillmentType: params.fulfillmentType,
    price: { amount: params.priceAmount, currency: params.currency },
    compareAt:
      typeof params.compareAtAmount === "number"
        ? { amount: params.compareAtAmount, currency: params.currency }
        : undefined,
    status: params.status,
  });
}

export async function setInventory(params: {
  storeId: string;
  variantId: string;
  locationId: string;
  stock: number;
  reserved: number;
}) {
  return rpcFetch<{ inventory: InventoryAdmin }>(
    "/rpc/ecommerce.v1.BackofficeService/SetInventory",
    {
      store: { storeId: params.storeId },
      variantId: params.variantId,
      locationId: params.locationId,
      stock: params.stock,
      reserved: params.reserved,
    }
  );
}
