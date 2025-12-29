import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import { SetupService, InitializeStoreRequestSchema } from "@/gen/ecommerce/v1/setup_pb";

const client = createServiceClient(SetupService);

export async function initializeStore(params: {
  storeName: string;
  storeCode: string;
  ownerEmail: string;
  ownerPassword: string;
  ownerLoginId?: string;
}) {
  return client.initializeStore(
    create(InitializeStoreRequestSchema, {
      storeName: params.storeName,
      storeCode: params.storeCode,
      ownerEmail: params.ownerEmail,
      ownerPassword: params.ownerPassword,
      ownerLoginId: params.ownerLoginId || "",
    })
  );
}
