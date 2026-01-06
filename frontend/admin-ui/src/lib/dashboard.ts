import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  BackofficeService,
  GetDashboardSummaryRequestSchema,
} from "@/gen/ecommerce/v1/backoffice_pb";

const client = createServiceClient(BackofficeService);

export async function getDashboardSummary() {
  return client.getDashboardSummary(
    create(GetDashboardSummaryRequestSchema, {})
  );
}
