import { saveStoreSession, setActiveStore } from "@/lib/auth";
import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  IdentityService,
  IdentitySignInRequestSchema,
  IdentitySignOutRequestSchema,
  IdentityCreateStaffRequestSchema,
  IdentityListRolesRequestSchema,
  IdentityCreateRoleRequestSchema,
  IdentityAssignRoleRequestSchema,
} from "@/gen/ecommerce/v1/identity_pb";

const client = createServiceClient(IdentityService);

export async function identitySignIn(params: {
  storeCode: string;
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
}) {
  const resp = await client.signIn(
    create(IdentitySignInRequestSchema, {
      store: { storeCode: params.storeCode },
      email: params.email || "",
      loginId: params.loginId || "",
      phone: params.phone || "",
      password: params.password,
    })
  );
  saveStoreSession({
    storeId: resp.storeId,
    tenantId: resp.tenantId,
    accessToken: resp.accessToken,
  });
  setActiveStore(resp.storeId, resp.tenantId, resp.accessToken);
  return resp;
}

export async function identitySignOut(params?: { storeId?: string; tenantId?: string }) {
  return client.signOut(
    create(IdentitySignOutRequestSchema, {
      store: params?.storeId ? { storeId: params.storeId } : undefined,
      tenant: params?.tenantId ? { tenantId: params.tenantId } : undefined,
    })
  );
}

export async function identityCreateStaff(params: {
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
  role: string;
}) {
  return client.createStaff(
    create(IdentityCreateStaffRequestSchema, {
      email: params.email || "",
      loginId: params.loginId || "",
      phone: params.phone || "",
      password: params.password,
      role: params.role,
    })
  );
}

export async function identityListRoles() {
  return client.listRoles(
    create(IdentityListRolesRequestSchema, {})
  );
}

export async function identityCreateRole(params: {
  key: string;
  name: string;
  description?: string;
  permissionKeys: string[];
}) {
  return client.createRole(
    create(IdentityCreateRoleRequestSchema, {
      key: params.key,
      name: params.name,
      description: params.description || "",
      permissionKeys: params.permissionKeys,
    })
  );
}

export async function identityAssignRole(params: {
  staffId: string;
  roleId: string;
}) {
  return client.assignRoleToStaff(
    create(IdentityAssignRoleRequestSchema, {
      staffId: params.staffId,
      roleId: params.roleId,
    })
  );
}
