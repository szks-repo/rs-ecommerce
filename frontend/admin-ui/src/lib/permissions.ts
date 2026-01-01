export type PermissionGroup = {
  label: string;
  description: string;
  permissions: Array<{ key: string; label: string }>;
};

export const PERMISSION_GROUPS: PermissionGroup[] = [
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
    label: "Identity",
    description: "Staff and roles",
    permissions: [{ key: "staff.manage", label: "Manage staff & roles" }],
  },
  {
    label: "Audit",
    description: "Audit log access",
    permissions: [{ key: "audit.read", label: "View audit logs" }],
  },
];

export const DEFAULT_PERMISSION_KEYS = PERMISSION_GROUPS.flatMap((group) =>
  group.permissions.map((permission) => permission.key)
);
