"use client";

import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { usePermissions } from "@/lib/use-permissions";
import type { PermissionKeyLiteral } from "@/lib/permissions";

type NavItem = {
  label: string;
  href: string;
  permission?: PermissionKeyLiteral;
  indent?: boolean;
};

const NAV_ITEMS: NavItem[] = [
  { label: "Overview", href: "/admin" },
  { label: "Orders", href: "/admin/orders", permission: "orders.read" },
  { label: "Products", href: "/admin/products", permission: "catalog.read" },
  { label: "Auctions", href: "/admin/auctions", permission: "auction.read" },
  { label: "Customers", href: "/admin/customers", permission: "customers.read" },
  { label: "Inventory", href: "/admin/inventory", permission: "catalog.read" },
  { label: "Audit Logs", href: "/admin/audit", permission: "audit.read" },
];

const SETTINGS_ITEMS: NavItem[] = [
  { label: "Overview", href: "/admin/settings", permission: "settings.read" },
  { label: "Metafields", href: "/admin/settings/metafields", permission: "customers.write" },
];

const IDENTITY_ITEMS: NavItem[] = [
  { label: "Staff", href: "/admin/identity", permission: "staff.manage" },
  { label: "Roles", href: "/admin/identity/roles", permission: "staff.manage" },
];

export default function AdminSidebarNav() {
  const pathname = usePathname();
  const permissions = usePermissions();
  const [settingsOpen, setSettingsOpen] = useState(() => pathname?.startsWith("/admin/settings") ?? false);
  const [identityOpen, setIdentityOpen] = useState(() => pathname?.startsWith("/admin/identity") ?? false);

  useEffect(() => {
    if (pathname?.startsWith("/admin/settings")) {
      setSettingsOpen(true);
    }
  }, [pathname]);

  useEffect(() => {
    if (pathname?.startsWith("/admin/identity")) {
      setIdentityOpen(true);
    }
  }, [pathname]);

  if (permissions.status === "loading") {
    return (
      <nav className="mt-6 space-y-2 text-sm text-neutral-600">
        <div className="text-xs text-neutral-400">Loading menu…</div>
      </nav>
    );
  }

  return (
    <nav className="mt-6 space-y-2 text-sm text-neutral-600">
      {NAV_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
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
      {SETTINGS_ITEMS.some((item) => permissions.has(item.permission)) && (
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => setSettingsOpen((prev) => !prev)}
            className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm text-neutral-700 hover:bg-neutral-100"
            aria-expanded={settingsOpen}
          >
            <span>Settings</span>
            <span className={`text-xs text-neutral-400 transition ${settingsOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {settingsOpen && (
            <div className="space-y-1 pl-2">
              {SETTINGS_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-3 py-2 text-xs ${
                      isActive ? "bg-neutral-100 text-neutral-900" : "text-neutral-600 hover:bg-neutral-100"
                    }`}
                    href={item.href}
                  >
                    {item.label}
                  </a>
                );
              })}
            </div>
          )}
        </div>
      )}
      {IDENTITY_ITEMS.some((item) => permissions.has(item.permission)) && (
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => setIdentityOpen((prev) => !prev)}
            className="flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm text-neutral-700 hover:bg-neutral-100"
            aria-expanded={identityOpen}
          >
            <span>Identity</span>
            <span className={`text-xs text-neutral-400 transition ${identityOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {identityOpen && (
            <div className="space-y-1 pl-2">
              {IDENTITY_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-3 py-2 text-xs ${
                      isActive ? "bg-neutral-100 text-neutral-900" : "text-neutral-600 hover:bg-neutral-100"
                    }`}
                    href={item.href}
                  >
                    {item.label}
                  </a>
                );
              })}
            </div>
          )}
        </div>
      )}
    </nav>
  );
}
