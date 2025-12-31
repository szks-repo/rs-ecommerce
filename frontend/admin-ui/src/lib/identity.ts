import { saveStoreSession, setActiveStore } from "@/lib/auth";
import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  IdentityService,
  IdentitySignInRequestSchema,
  IdentitySignOutRequestSchema,
  IdentityCreateStaffRequestSchema,
  IdentityListStaffRequestSchema,
  IdentityUpdateStaffRequestSchema,
  IdentityInviteStaffRequestSchema,
  IdentityTransferOwnerRequestSchema,
  IdentityListStaffSessionsRequestSchema,
  IdentityForceSignOutStaffRequestSchema,
  IdentityListRolesRequestSchema,
  IdentityListRolesWithPermissionsRequestSchema,
  IdentityUpdateRoleRequestSchema,
  IdentityDeleteRoleRequestSchema,
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
  roleId: string;
  displayName?: string;
}) {
  return client.createStaff(
    create(IdentityCreateStaffRequestSchema, {
      email: params.email || "",
      loginId: params.loginId || "",
      phone: params.phone || "",
      password: params.password,
      roleId: params.roleId,
      displayName: params.displayName || "",
    })
  );
}

export async function identityListStaff() {
  return client.listStaff(
    create(IdentityListStaffRequestSchema, {})
  );
}

export async function identityListStaffSessions() {
  return client.listStaffSessions(
    create(IdentityListStaffSessionsRequestSchema, {})
  );
}

export async function identityForceSignOutStaff(params: { staffId: string }) {
  return client.forceSignOutStaff(
    create(IdentityForceSignOutStaffRequestSchema, {
      staffId: params.staffId,
    })
  );
}

export async function identityUpdateStaff(params: {
  staffId: string;
  roleId?: string;
  status?: string;
  displayName?: string;
}) {
  return client.updateStaff(
    create(IdentityUpdateStaffRequestSchema, {
      staffId: params.staffId,
      roleId: params.roleId || "",
      status: params.status || "",
      displayName: params.displayName || "",
    })
  );
}

export async function identityInviteStaff(params: {
  email: string;
  roleId: string;
  displayName?: string;
}) {
  return client.inviteStaff(
    create(IdentityInviteStaffRequestSchema, {
      email: params.email,
      roleId: params.roleId,
      displayName: params.displayName || "",
    })
  );
}

export async function identityTransferOwner(params: {
  newOwnerStaffId: string;
}) {
  return client.transferOwner(
    create(IdentityTransferOwnerRequestSchema, {
      newOwnerStaffId: params.newOwnerStaffId,
    })
  );
}

export async function identityListRoles() {
  return client.listRoles(
    create(IdentityListRolesRequestSchema, {})
  );
}

export async function identityListRolesWithPermissions() {
  return client.listRolesWithPermissions(
    create(IdentityListRolesWithPermissionsRequestSchema, {})
  );
}

export async function identityUpdateRole(params: {
  roleId: string;
  name?: string;
  description?: string;
  permissionKeys: string[];
}) {
  return client.updateRole(
    create(IdentityUpdateRoleRequestSchema, {
      roleId: params.roleId,
      name: params.name || "",
      description: params.description || "",
      permissionKeys: params.permissionKeys,
    })
  );
}

export async function identityDeleteRole(params: { roleId: string }) {
  return client.deleteRole(
    create(IdentityDeleteRoleRequestSchema, {
      roleId: params.roleId,
    })
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
