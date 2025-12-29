import { createClient } from "@/lib/connect";
import { SetupService } from "@/gen/ecommerce/v1/setup_connect";
import { InitializeStoreRequest } from "@/gen/ecommerce/v1/setup_pb";

const client = createClient(SetupService);

export async function initializeStore(params: {
  storeName: string;
  ownerEmail: string;
  ownerPassword: string;
  ownerLoginId?: string;
}) {
  return client.initializeStore(
    new InitializeStoreRequest({
      storeName: params.storeName,
      ownerEmail: params.ownerEmail,
      ownerPassword: params.ownerPassword,
      ownerLoginId: params.ownerLoginId || "",
    })
  );
}
