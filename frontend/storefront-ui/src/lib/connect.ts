import { createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { API_BASE } from "@/lib/api";

const transport = createConnectTransport({
  baseUrl: API_BASE,
  useBinaryFormat: false,
  fetchOptions: {
    credentials: "include",
  },
  fetch: (input, init) => {
    return fetch(input, { ...init, credentials: "include" });
  },
});

export function createServiceClient<T>(service: T): ReturnType<typeof createClient<T>> {
  return createClient(service, transport);
}
