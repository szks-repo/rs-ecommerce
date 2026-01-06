import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  BackofficeService,
  ListPagesRequestSchema,
  GetPageRequestSchema,
  CreatePageRequestSchema,
  UpdatePageRequestSchema,
  DeletePageRequestSchema,
  type PageInput,
} from "@/gen/ecommerce/v1/backoffice_pb";

const client = createServiceClient(BackofficeService);

export async function listPages(params?: { pageSize?: number; pageToken?: string }) {
  return client.listPages(
    create(ListPagesRequestSchema, {
      page: {
        pageSize: params?.pageSize ?? 50,
        pageToken: params?.pageToken ?? "",
      },
    })
  );
}

export async function getPage(params: { pageId: string }) {
  return client.getPage(
    create(GetPageRequestSchema, {
      pageId: params.pageId,
    })
  );
}

export async function createPage(params: { page: PageInput }) {
  return client.createPage(
    create(CreatePageRequestSchema, {
      page: params.page,
    })
  );
}

export async function updatePage(params: { pageId: string; page: PageInput }) {
  return client.updatePage(
    create(UpdatePageRequestSchema, {
      pageId: params.pageId,
      page: params.page,
    })
  );
}

export async function deletePage(params: { pageId: string }) {
  return client.deletePage(
    create(DeletePageRequestSchema, {
      pageId: params.pageId,
    })
  );
}
