import { createClient, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { API_BASE } from "@/lib/api";
import { getActiveAccessToken } from "@/lib/auth";

const authInterceptor: Interceptor = (next) => async (req) => {
  const token = getActiveAccessToken();
  if (token) {
    req.header.set("Authorization", `Bearer ${token}`);
  }
  return next(req);
};

const transport = createConnectTransport({
  baseUrl: API_BASE,
  useBinaryFormat: false,
  interceptors: [authInterceptor],
});

export function createServiceClient<T>(service: T): ReturnType<typeof createClient<T>> {
  return createClient(service, transport);
}
