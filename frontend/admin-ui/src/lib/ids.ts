import { brand, type Brand } from "@/lib/types";

export type StoreId = Brand<string, "StoreId">;
export type TenantId = Brand<string, "TenantId">;
export type StaffId = Brand<string, "StaffId">;
export type RoleId = Brand<string, "RoleId">;
export type ProductId = Brand<string, "ProductId">;
export type SkuId = Brand<string, "SkuId">;

export function asStoreId(value: string): StoreId {
  return brand<string, "StoreId">(value);
}

export function asTenantId(value: string): TenantId {
  return brand<string, "TenantId">(value);
}

export function asStaffId(value: string): StaffId {
  return brand<string, "StaffId">(value);
}

export function asRoleId(value: string): RoleId {
  return brand<string, "RoleId">(value);
}

export function asProductId(value: string): ProductId {
  return brand<string, "ProductId">(value);
}

export function asSkuId(value: string): SkuId {
  return brand<string, "SkuId">(value);
}
