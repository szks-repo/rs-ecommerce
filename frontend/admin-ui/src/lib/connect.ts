import { createClient, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { API_BASE } from "@/lib/api";
import { clearActiveStoreSession, getActiveAccessToken, refreshAccessToken, setAuthFlashMessage } from "@/lib/auth";
import { parseErrorCode } from "@/lib/errors";
import { ErrorCode } from "@/gen/ecommerce/v1/common_pb";

const authInterceptor: Interceptor = (next) => async (req) => {
  const token = getActiveAccessToken();
  if (token) {
    req.header.set("Authorization", `Bearer ${token}`);
  }
  try {
    return await next(req);
  } catch (err) {
    const code = parseErrorCode((err as { code?: string })?.code);
    const rawResponse = (err as { rawResponse?: Response })?.rawResponse;
    const status =
      rawResponse?.status ??
      (err as { status?: number })?.status ??
      (err as { httpStatus?: number })?.httpStatus;
    const shouldRefresh = code === ErrorCode.ERROR_CODE_UNAUTHENTICATED || status === 401;
    const isRefreshRequest = typeof req.url === "string" && req.url.includes("IdentityService/RefreshToken");
    const alreadyRetried = req.header.get("x-refresh-attempt") === "1";

    if (shouldRefresh && !isRefreshRequest && !alreadyRetried) {
      const refreshed = await refreshAccessToken();
      if (refreshed?.accessToken) {
        req.header.set("Authorization", `Bearer ${refreshed.accessToken}`);
        req.header.set("x-refresh-attempt", "1");
        return next(req);
      }
    }

    if (shouldRefresh && typeof window !== "undefined") {
      setAuthFlashMessage({
        variant: "error",
        title: "Session expired",
        description: "Please sign in again to continue.",
      });
      clearActiveStoreSession();
      if (!window.location.pathname.startsWith("/login") && !window.location.pathname.startsWith("/vendor/login")) {
        window.location.href = "/login";
      }
    }
    throw err;
  }
};

const transport = createConnectTransport({
  baseUrl: API_BASE,
  useBinaryFormat: false,
  interceptors: [authInterceptor],
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
