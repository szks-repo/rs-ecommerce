import { createClient } from "@/lib/connect";
import { StoreSettingsService } from "@/gen/ecommerce/v1/store_settings_connect";
import {
  ListStoreLocationsRequest,
  UpsertStoreLocationRequest,
} from "@/gen/ecommerce/v1/store_settings_pb";

const client = createClient(StoreSettingsService);

export async function listStoreLocations() {
  return client.listStoreLocations(new ListStoreLocationsRequest({}));
}

export async function upsertStoreLocation(params: {
  code: string;
  name: string;
  status: string;
}) {
  return client.upsertStoreLocation(
    new UpsertStoreLocationRequest({
      location: {
        code: params.code,
        name: params.name,
        status: params.status,
      },
    })
  );
}
