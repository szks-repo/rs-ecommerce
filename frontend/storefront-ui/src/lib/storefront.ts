import { create } from "@bufbuild/protobuf";
import { DEFAULT_TENANT_ID } from "@/lib/api";
import { createServiceClient } from "@/lib/connect";
import {
  StorefrontService,
  ListProductsRequestSchema,
  GetProductRequestSchema,
  GetPageBySlugRequestSchema,
} from "@/gen/ecommerce/v1/storefront_pb";

const client = createServiceClient(StorefrontService);

export function getTenantId() {
  return DEFAULT_TENANT_ID;
}

export async function listProducts(tenantId: string) {
  return client.listProducts(
    create(ListProductsRequestSchema, {
      tenant: { tenantId },
    })
  );
}

export async function getProduct(tenantId: string, productId: string) {
  return client.getProduct(
    create(GetProductRequestSchema, {
      tenant: { tenantId },
      productId,
    })
  );
}

export async function getPageBySlug(tenantId: string, slug: string) {
  return client.getPageBySlug(
    create(GetPageBySlugRequestSchema, {
      tenant: { tenantId },
      slug,
    })
  );
}
