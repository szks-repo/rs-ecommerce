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

type ProductAdminListResponse = {
  products: ProductAdmin[];
};

type VariantAdminListResponse = {
  variants: VariantAdmin[];
};

export async function listProductsAdmin() {
  return rpcFetch<ProductAdminListResponse>(
    "/rpc/ecommerce.v1.BackofficeService/ListProducts",
    {}
  );
}

export async function listVariantsAdmin(params: { productId: string }) {
  return rpcFetch<VariantAdminListResponse>(
    "/rpc/ecommerce.v1.BackofficeService/ListVariants",
    {
      productId: params.productId,
    }
  );
}

export async function createProduct(params: {
  vendorId?: string;
  title: string;
  description: string;
  status: string;
}) {
  return rpcFetch<{ product: ProductAdmin }>("/rpc/ecommerce.v1.BackofficeService/CreateProduct", {
    vendorId: params.vendorId || "",
    title: params.title,
    description: params.description,
    status: params.status,
  });
}

export async function updateProduct(params: {
  productId: string;
  title: string;
  description: string;
  status: string;
}) {
  return rpcFetch<{ product: ProductAdmin }>("/rpc/ecommerce.v1.BackofficeService/UpdateProduct", {
    productId: params.productId,
    title: params.title,
    description: params.description,
    status: params.status,
  });
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
  return rpcFetch<{ variant: VariantAdmin }>("/rpc/ecommerce.v1.BackofficeService/CreateVariant", {
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

export async function updateVariant(params: {
  variantId: string;
  priceAmount: number;
  compareAtAmount?: number;
  currency: string;
  status: string;
  fulfillmentType?: string;
}) {
  return rpcFetch<{ variant: VariantAdmin }>("/rpc/ecommerce.v1.BackofficeService/UpdateVariant", {
    variantId: params.variantId,
    price: { amount: params.priceAmount, currency: params.currency },
    compareAt:
      typeof params.compareAtAmount === "number"
        ? { amount: params.compareAtAmount, currency: params.currency }
        : undefined,
    status: params.status,
    fulfillmentType: params.fulfillmentType || "",
  });
}

export async function setInventory(params: {
  variantId: string;
  locationId: string;
  stock: number;
  reserved: number;
}) {
  return rpcFetch<{ inventory: InventoryAdmin }>(
    "/rpc/ecommerce.v1.BackofficeService/SetInventory",
    {
      variantId: params.variantId,
      locationId: params.locationId,
      stock: params.stock,
      reserved: params.reserved,
    }
  );
}
