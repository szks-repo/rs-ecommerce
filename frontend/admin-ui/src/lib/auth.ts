import {
  asStaffId,
  asStoreId,
  asTenantId,
  type StaffId,
  type StoreId,
  type TenantId,
} from "@/lib/ids";

type StoreSession = {
  storeId: StoreId;
  tenantId: TenantId;
  accessToken: string;
};

const STORE_TOKENS_KEY = "store_tokens";
const ACTIVE_STORE_KEY = "active_store_id";
const AUTH_FLASH_KEY = "auth_flash_message";

type AuthFlashMessage = {
  title: string;
  description: string;
  variant: "success" | "error";
};

function readStoreTokens(): Record<string, StoreSession> {
  if (typeof window === "undefined") {
    return {};
  }
  const raw = window.localStorage.getItem(STORE_TOKENS_KEY);
  if (!raw) {
    return {};
  }
  try {
    return JSON.parse(raw) as Record<string, StoreSession>;
  } catch {
    return {};
  }
}

function writeStoreTokens(tokens: Record<string, StoreSession>) {
  window.localStorage.setItem(STORE_TOKENS_KEY, JSON.stringify(tokens));
}

export function saveStoreSession(session: StoreSession) {
  if (typeof window === "undefined") {
    return;
  }
  const tokens = readStoreTokens();
  tokens[session.storeId] = session;
  writeStoreTokens(tokens);
}

export function setActiveStore(storeId: StoreId, tenantId: TenantId, accessToken: string) {
  if (typeof window === "undefined") {
    return;
  }
  window.sessionStorage.setItem("store_id", storeId);
  window.sessionStorage.setItem("tenant_id", tenantId);
  window.sessionStorage.setItem("access_token", accessToken);
  window.localStorage.setItem(ACTIVE_STORE_KEY, storeId);
}

export function clearActiveStoreSession() {
  if (typeof window === "undefined") {
    return;
  }
  const activeStoreId = getActiveStoreId();
  window.sessionStorage.removeItem("store_id");
  window.sessionStorage.removeItem("tenant_id");
  window.sessionStorage.removeItem("access_token");
  if (activeStoreId) {
    const tokens = readStoreTokens();
    delete tokens[activeStoreId];
    writeStoreTokens(tokens);
  }
  window.localStorage.removeItem(ACTIVE_STORE_KEY);
}

export function getActiveStoreId(): StoreId | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw =
    window.sessionStorage.getItem("store_id") || window.localStorage.getItem(ACTIVE_STORE_KEY);
  return raw ? asStoreId(raw) : null;
}

export function getActiveAccessToken(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  const storeId = getActiveStoreId();
  if (!storeId) {
    return window.sessionStorage.getItem("access_token");
  }
  const tokens = readStoreTokens();
  return tokens[storeId]?.accessToken || window.sessionStorage.getItem("access_token");
}

export function getActiveActorInfo():
  | { staffId: StaffId; role: string; storeId: StoreId }
  | null {
  if (typeof window === "undefined") {
    return null;
  }
  const token = getActiveAccessToken();
  if (!token) {
    return null;
  }
  const parts = token.split(".");
  if (parts.length !== 3) {
    return null;
  }
  try {
    const payload = JSON.parse(
      decodeURIComponent(
        atob(parts[1].replace(/-/g, "+").replace(/_/g, "/"))
          .split("")
          .map((char) => `%${char.charCodeAt(0).toString(16).padStart(2, "0")}`)
          .join("")
      )
    ) as {
      sub?: string;
      actor_type?: string;
      store_id?: string;
    };
    if (!payload.sub || !payload.actor_type || !payload.store_id) {
      return null;
    }
    return {
      staffId: asStaffId(payload.sub),
      role: payload.actor_type,
      storeId: asStoreId(payload.store_id),
    };
  } catch {
    return null;
  }
}

export function getActiveTenantId(): TenantId | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw = window.sessionStorage.getItem("tenant_id");
  return raw ? asTenantId(raw) : null;
}

export function setAuthFlashMessage(message: AuthFlashMessage) {
  if (typeof window === "undefined") {
    return;
  }
  window.sessionStorage.setItem(AUTH_FLASH_KEY, JSON.stringify(message));
}

export function consumeAuthFlashMessage(): AuthFlashMessage | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw = window.sessionStorage.getItem(AUTH_FLASH_KEY);
  if (!raw) {
    return null;
  }
  window.sessionStorage.removeItem(AUTH_FLASH_KEY);
  try {
    return JSON.parse(raw) as AuthFlashMessage;
  } catch {
    return null;
  }
}

export async function refreshAccessToken(): Promise<StoreSession | null> {
  if (typeof window === "undefined") {
    return null;
  }
  const storeId = getActiveStoreId();
  const tenantId = getActiveTenantId() || "";
  if (!storeId) {
    return null;
  }

  const { API_BASE } = await import("@/lib/api");
  const resp = await fetch(`${API_BASE}/ecommerce.v1.IdentityService/RefreshToken`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      "Connect-Protocol-Version": "1",
    },
    body: JSON.stringify({
      store: { storeId },
      tenant: tenantId ? { tenantId } : undefined,
    }),
  });

  if (!resp.ok) {
    return null;
  }
  const data = (await resp.json()) as {
    accessToken?: string;
    storeId?: string;
    tenantId?: string;
  };
  if (!data.accessToken || !data.storeId) {
    return null;
  }
  const session: StoreSession = {
    storeId: data.storeId,
    tenantId: data.tenantId || "",
    accessToken: data.accessToken,
  };
  saveStoreSession(session);
  setActiveStore(session.storeId, session.tenantId, session.accessToken);
  return session;
}
