import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  AuditService,
  ListAuditLogsRequestSchema,
  ListAuditActionsRequestSchema,
} from "@/gen/ecommerce/v1/audit_pb";
import { getActiveStoreId } from "@/lib/auth";

const client = createServiceClient(AuditService);

export async function listAuditLogs(params?: {
  action?: string;
  actorId?: string;
  actorType?: string;
  targetType?: string;
  targetId?: string;
  pageToken?: string;
  pageSize?: number;
}) {
  const storeId = getActiveStoreId();
  if (!storeId) {
    throw new Error("store_id is missing. Please sign in again.");
  }
  return client.listAuditLogs(
    create(ListAuditLogsRequestSchema, {
      store: { storeId },
      action: params?.action ?? "",
      actorId: params?.actorId ?? "",
      actorType: params?.actorType ?? "",
      targetType: params?.targetType ?? "",
      targetId: params?.targetId ?? "",
      page: {
        pageToken: params?.pageToken ?? "",
        pageSize: params?.pageSize ?? 50,
      },
    })
  );
}

export async function listAuditActions() {
  return client.listAuditActions(create(ListAuditActionsRequestSchema, {}));
}
