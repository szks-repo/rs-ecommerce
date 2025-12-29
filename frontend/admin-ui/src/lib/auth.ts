type StoreSession = {
  storeId: string;
  tenantId: string;
  accessToken: string;
};

const STORE_TOKENS_KEY = "store_tokens";
const ACTIVE_STORE_KEY = "active_store_id";

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

export function setActiveStore(storeId: string, tenantId: string, accessToken: string) {
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

export function getActiveStoreId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return window.sessionStorage.getItem("store_id") || window.localStorage.getItem(ACTIVE_STORE_KEY);
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

export function getActiveTenantId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return window.sessionStorage.getItem("tenant_id");
}
