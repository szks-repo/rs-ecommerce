import { rpcFetch } from "@/lib/api";
import { saveStoreSession, setActiveStore } from "@/lib/auth";

type IdentitySignInResponse = {
  accessToken: string;
  storeId: string;
  tenantId: string;
};

type IdentityCreateStaffResponse = {
  staffId: string;
};

type IdentityRole = {
  id: string;
  key: string;
  name: string;
  description?: string;
};

type IdentityListRolesResponse = {
  roles: IdentityRole[];
};

export async function identitySignIn(params: {
  storeId: string;
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
}) {
  const resp = await rpcFetch<IdentitySignInResponse>("/rpc/ecommerce.v1.IdentityService/SignIn", {
    store: { storeId: params.storeId },
    email: params.email || "",
    loginId: params.loginId || "",
    phone: params.phone || "",
    password: params.password,
  });
  saveStoreSession({
    storeId: resp.storeId,
    tenantId: resp.tenantId,
    accessToken: resp.accessToken,
  });
  setActiveStore(resp.storeId, resp.tenantId, resp.accessToken);
  return resp;
}

export async function identityCreateStaff(params: {
  storeId: string;
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
  role: string;
}) {
  return rpcFetch<IdentityCreateStaffResponse>(
    "/rpc/ecommerce.v1.IdentityService/CreateStaff",
    {
      store: { storeId: params.storeId },
      email: params.email || "",
      loginId: params.loginId || "",
      phone: params.phone || "",
      password: params.password,
      role: params.role,
    }
  );
}

export async function identityListRoles(params: { storeId: string }) {
  return rpcFetch<IdentityListRolesResponse>("/rpc/ecommerce.v1.IdentityService/ListRoles", {
    store: { storeId: params.storeId },
  });
}
