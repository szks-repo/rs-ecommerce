import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  BackofficeService,
  ListCategoriesAdminRequestSchema,
  CreateCategoryRequestSchema,
  UpdateCategoryRequestSchema,
  DeleteCategoryRequestSchema,
  ReorderCategoriesRequestSchema,
  ListCategoryProductsRequestSchema,
  ReorderCategoryProductsRequestSchema,
} from "@/gen/ecommerce/v1/backoffice_pb";

const client = createServiceClient(BackofficeService);

export async function listCategoriesAdmin(params?: { status?: string }) {
  return client.listCategories(
    create(ListCategoriesAdminRequestSchema, {
      status: params?.status ?? "",
    })
  );
}

export async function createCategory(params: {
  name: string;
  slug: string;
  description?: string;
  status?: string;
  parentId?: string;
  position?: number;
}) {
  return client.createCategory(
    create(CreateCategoryRequestSchema, {
      category: {
        name: params.name,
        slug: params.slug,
        description: params.description ?? "",
        status: params.status ?? "active",
        parentId: params.parentId ?? "",
        position: params.position ?? 0,
      },
    })
  );
}

export async function updateCategory(params: {
  categoryId: string;
  name: string;
  slug: string;
  description?: string;
  status?: string;
  parentId?: string;
  position?: number;
}) {
  return client.updateCategory(
    create(UpdateCategoryRequestSchema, {
      categoryId: params.categoryId,
      category: {
        name: params.name,
        slug: params.slug,
        description: params.description ?? "",
        status: params.status ?? "active",
        parentId: params.parentId ?? "",
        position: params.position ?? 0,
      },
    })
  );
}

export async function deleteCategory(params: { categoryId: string }) {
  return client.deleteCategory(
    create(DeleteCategoryRequestSchema, { categoryId: params.categoryId })
  );
}

export async function reorderCategories(params: {
  parentId?: string;
  orderedIds: string[];
}) {
  return client.reorderCategories(
    create(ReorderCategoriesRequestSchema, {
      parentId: params.parentId ?? "",
      orderedIds: params.orderedIds,
    })
  );
}

export async function listCategoryProducts(params: { categoryId: string }) {
  return client.listCategoryProducts(
    create(ListCategoryProductsRequestSchema, {
      categoryId: params.categoryId,
    })
  );
}

export async function reorderCategoryProducts(params: {
  categoryId: string;
  orderedProductIds: string[];
}) {
  return client.reorderCategoryProducts(
    create(ReorderCategoryProductsRequestSchema, {
      categoryId: params.categoryId,
      orderedProductIds: params.orderedProductIds,
    })
  );
}
