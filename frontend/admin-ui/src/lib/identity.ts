import { saveStoreSession, setActiveStore } from "@/lib/auth";
import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  IdentityService,
  IdentitySignInRequestSchema,
  IdentityCreateStaffRequestSchema,
  IdentityListRolesRequestSchema,
  IdentityCreateRoleRequestSchema,
  IdentityAssignRoleRequestSchema,
} from "@/gen/ecommerce/v1/identity_pb";

const client = createServiceClient(IdentityService);

export async function identitySignIn(params: {
  storeId: string;
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
}) {
  const resp = await client.signIn(
    create(IdentitySignInRequestSchema, {
      store: { storeId: params.storeId },
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

export async function identityCreateStaff(params: {
  storeId: string;
  email?: string;
  loginId?: string;
  phone?: string;
  password: string;
  role: string;
}) {
  return client.createStaff(
    create(IdentityCreateStaffRequestSchema, {
      store: { storeId: params.storeId },
      email: params.email || "",
      loginId: params.loginId || "",
      phone: params.phone || "",
      password: params.password,
      role: params.role,
    })
  );
}

export async function identityListRoles(params: { storeId: string }) {
  return client.listRoles(
    create(IdentityListRolesRequestSchema, {
      store: { storeId: params.storeId },
    })
  );
}

export async function identityCreateRole(params: {
  storeId: string;
  key: string;
  name: string;
  description?: string;
  permissionKeys: string[];
}) {
  return client.createRole(
    create(IdentityCreateRoleRequestSchema, {
      store: { storeId: params.storeId },
      key: params.key,
      name: params.name,
      description: params.description || "",
      permissionKeys: params.permissionKeys,
    })
  );
}

export async function identityAssignRole(params: {
  storeId: string;
  staffId: string;
  roleId: string;
}) {
  return client.assignRoleToStaff(
    create(IdentityAssignRoleRequestSchema, {
      store: { storeId: params.storeId },
      staffId: params.staffId,
      roleId: params.roleId,
    })
  );
}
