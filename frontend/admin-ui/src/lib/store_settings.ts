import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  StoreSettingsService,
  ListStoreLocationsRequestSchema,
  UpsertStoreLocationRequestSchema,
} from "@/gen/ecommerce/v1/store_settings_pb";

const client = createServiceClient(StoreSettingsService);

export async function listStoreLocations() {
  return client.listStoreLocations(create(ListStoreLocationsRequestSchema, {}));
}

export async function upsertStoreLocation(params: {
  code: string;
  name: string;
  status: string;
}) {
  return client.upsertStoreLocation(
    create(UpsertStoreLocationRequestSchema, {
      location: {
        code: params.code,
        name: params.name,
        status: params.status,
      },
    })
  );
}
