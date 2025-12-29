import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import { AuditService, ListAuditLogsRequestSchema } from "@/gen/ecommerce/v1/audit_pb";
import { getActiveTenantId } from "@/lib/auth";

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
  const tenantId = getActiveTenantId();
  if (!tenantId) {
    throw new Error("tenant_id is missing. Please sign in again.");
  }
  return client.listAuditLogs(
    create(ListAuditLogsRequestSchema, {
      tenant: { tenantId },
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
