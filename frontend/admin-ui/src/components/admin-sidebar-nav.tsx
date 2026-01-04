"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname } from "next/navigation";
import { getActiveActorInfo } from "@/lib/auth";
import { identityListMyPermissions } from "@/lib/identity";

type NavItem = {
  label: string;
  href: string;
  permission?: string;
  indent?: boolean;
};

const NAV_ITEMS: NavItem[] = [
  { label: "Overview", href: "/admin" },
  { label: "Orders", href: "/admin/orders", permission: "orders.read" },
  { label: "Products", href: "/admin/products", permission: "catalog.read" },
  { label: "Auctions", href: "/admin/auctions", permission: "auction.read" },
  { label: "Customers", href: "/admin/customers", permission: "customers.read" },
  { label: "Inventory", href: "/admin/inventory", permission: "catalog.read" },
  { label: "Shop Settings", href: "/admin/settings", permission: "settings.read" },
  { label: "Identity", href: "/admin/identity", permission: "staff.manage" },
  { label: "Roles", href: "/admin/identity/roles", permission: "staff.manage", indent: true },
  { label: "Audit Logs", href: "/admin/audit", permission: "audit.read" },
];

export default function AdminSidebarNav() {
  const pathname = usePathname();
  const [permissionKeys, setPermissionKeys] = useState<string[] | null>(null);

  useEffect(() => {
    const actor = getActiveActorInfo();
    if (!actor) {
      setPermissionKeys([]);
      return;
    }
    if (actor.role === "owner") {
      setPermissionKeys(["*"]);
      return;
    }
    identityListMyPermissions()
      .then((resp) => {
        setPermissionKeys(resp.permissionKeys || []);
      })
      .catch(() => {
        setPermissionKeys([]);
      });
  }, []);

  const allowed = useMemo(() => {
    const keys = permissionKeys ?? [];
    return (permission?: string) => {
      if (!permission) {
        return true;
      }
      if (keys.includes("*")) {
        return true;
      }
      return keys.includes(permission);
    };
  }, [permissionKeys]);

  if (permissionKeys === null) {
    return (
      <nav className="mt-6 space-y-2 text-sm text-neutral-600">
        <div className="text-xs text-neutral-400">Loading menu…</div>
      </nav>
    );
  }

  return (
    <nav className="mt-6 space-y-2 text-sm text-neutral-600">
      {NAV_ITEMS.filter((item) => allowed(item.permission)).map((item) => {
        const isActive =
          item.href === "/admin"
            ? pathname === "/admin"
            : pathname?.startsWith(item.href);
        const base = item.indent
          ? "ml-2 block rounded-lg px-3 py-2 text-xs"
          : "block rounded-lg px-3 py-2";
        const state = isActive
          ? "bg-neutral-100 text-neutral-900"
          : "hover:bg-neutral-100";
        return (
          <a key={item.href} className={`${base} ${state}`} href={item.href}>
            {item.label}
          </a>
        );
      })}
    </nav>
  );
}
