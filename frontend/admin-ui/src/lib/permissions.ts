import { brand, type Brand } from "@/lib/types";

const PERMISSION_GROUPS_CONST = [
  {
    label: "Catalog",
    description: "Products, variants, inventory",
    permissions: [
      { key: "catalog.read", label: "Read catalog" },
      { key: "catalog.write", label: "Write catalog" },
    ],
  },
  {
    label: "Orders",
    description: "Orders and fulfillment",
    permissions: [
      { key: "orders.read", label: "Read orders" },
      { key: "orders.write", label: "Write orders" },
    ],
  },
  {
    label: "Promotions",
    description: "Discounts, campaigns",
    permissions: [
      { key: "promotions.read", label: "Read promotions" },
      { key: "promotions.write", label: "Write promotions" },
    ],
  },
  {
    label: "Auctions",
    description: "Auction settings, listings, and bids",
    permissions: [
      { key: "auction.read", label: "Read auctions" },
      { key: "auction.write", label: "Write auctions" },
    ],
  },
  {
    label: "Settings",
    description: "Store settings and configurations",
    permissions: [
      { key: "settings.read", label: "Read settings" },
      { key: "settings.write", label: "Write settings" },
    ],
  },
  {
    label: "Pages",
    description: "Free pages and content",
    permissions: [
      { key: "pages.read", label: "Read pages" },
      { key: "pages.write", label: "Write pages" },
    ],
  },
  {
    label: "Identity",
    description: "Staff and roles",
    permissions: [{ key: "staff.manage", label: "Manage staff & roles" }],
  },
  {
    label: "Audit",
    description: "Audit log access",
    permissions: [{ key: "audit.read", label: "View audit logs" }],
  },
 ] as const;

export type PermissionKeyLiteral =
  (typeof PERMISSION_GROUPS_CONST)[number]["permissions"][number]["key"];
export type PermissionKey = Brand<PermissionKeyLiteral, "PermissionKey">;

export type PermissionGroup = {
  label: string;
  description: string;
  permissions: Array<{ key: PermissionKeyLiteral; label: string }>;
};

export const PERMISSION_GROUPS: PermissionGroup[] =
  PERMISSION_GROUPS_CONST as PermissionGroup[];

export const DEFAULT_PERMISSION_KEYS = PERMISSION_GROUPS.flatMap((group) =>
  group.permissions.map((permission) => permission.key)
);

export function isPermissionKey(value: string): value is PermissionKeyLiteral {
  return DEFAULT_PERMISSION_KEYS.includes(value as PermissionKeyLiteral);
}

export function toPermissionKey(value: string): PermissionKey | null {
  return isPermissionKey(value) ? brand(value) : null;
}

export function normalizePermissionKeys(values: string[]): PermissionKey[] {
  return values
    .map((value) => toPermissionKey(value))
    .filter((value): value is PermissionKey => Boolean(value));
}
