"use client";

import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import {
  Boxes,
  FileText,
  Gavel,
  Home,
  Settings,
  Shield,
  ShoppingCart,
  UserCog,
  Users,
} from "lucide-react";
import { usePermissions } from "@/lib/use-permissions";
import type { PermissionKeyLiteral } from "@/lib/permissions";

type NavItem = {
  label: string;
  href: string;
  permission?: PermissionKeyLiteral;
  indent?: boolean;
  icon?: React.ComponentType<{ className?: string }>;
};

const NAV_ITEMS: NavItem[] = [
  { label: "Dashboard", href: "/admin", icon: Home },
  { label: "Orders", href: "/admin/orders", permission: "orders.read", icon: ShoppingCart },
  { label: "Auctions", href: "/admin/auctions", permission: "auction.read", icon: Gavel },
  { label: "Customers", href: "/admin/customers", permission: "customers.read", icon: Users },
  { label: "Audit Logs", href: "/admin/audit", permission: "audit.read", icon: Shield },
];

const CATALOG_ITEMS: NavItem[] = [
  { label: "Products", href: "/admin/products", permission: "catalog.read" },
  { label: "Categories", href: "/admin/categories", permission: "catalog.read" },
  { label: "Inventory", href: "/admin/inventory", permission: "catalog.read" },
];

const SETTINGS_ITEMS: NavItem[] = [
  { label: "Overview", href: "/admin/settings", permission: "settings.read" },
];

const CONTENT_ITEMS: NavItem[] = [
  { label: "Pages", href: "/admin/settings/pages", permission: "pages.read" },
  { label: "Metafields", href: "/admin/settings/metafields", permission: "customers.write" },
  { label: "Files", href: "/admin/settings/files", permission: "catalog.read" },
];

const IDENTITY_ITEMS: NavItem[] = [
  { label: "Staff", href: "/admin/identity", permission: "staff.manage" },
  { label: "Roles", href: "/admin/identity/roles", permission: "staff.manage" },
];

export default function AdminSidebarNav() {
  const pathname = usePathname();
  const permissions = usePermissions();
  const [catalogOpen, setCatalogOpen] = useState(() => pathname?.startsWith("/admin/products") ?? false);
  const [settingsOpen, setSettingsOpen] = useState(() => pathname?.startsWith("/admin/settings") ?? false);
  const [contentOpen, setContentOpen] = useState(() =>
    pathname?.startsWith("/admin/settings/pages") ||
    pathname?.startsWith("/admin/settings/metafields") ||
    pathname?.startsWith("/admin/settings/files")
  );
  const [identityOpen, setIdentityOpen] = useState(() => pathname?.startsWith("/admin/identity") ?? false);

  useEffect(() => {
    if (pathname?.startsWith("/admin/products") || pathname?.startsWith("/admin/categories") || pathname?.startsWith("/admin/inventory")) {
      setCatalogOpen(true);
    }
  }, [pathname]);

  useEffect(() => {
    if (pathname?.startsWith("/admin/settings")) {
      setSettingsOpen(true);
    }
  }, [pathname]);

  useEffect(() => {
    if (
      pathname?.startsWith("/admin/settings/pages") ||
      pathname?.startsWith("/admin/settings/metafields") ||
      pathname?.startsWith("/admin/settings/files")
    ) {
      setContentOpen(true);
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
    <nav className="mt-5 space-y-1 text-[13px] text-neutral-600">
      {NAV_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
        const isActive =
          item.href === "/admin"
            ? pathname === "/admin"
            : pathname?.startsWith(item.href);
        const base = item.indent
          ? "ml-2 block rounded-lg px-2.5 py-1.5 text-[12px]"
          : "block rounded-lg px-2.5 py-1.5";
        const state = isActive
          ? "bg-neutral-100 text-neutral-900"
          : "hover:bg-neutral-100";
        const Icon = item.icon;
        return (
          <a key={item.href} className={`${base} ${state} flex items-center gap-2`} href={item.href}>
            {Icon ? <Icon className="h-4 w-4 text-neutral-500" /> : null}
            <span>{item.label}</span>
          </a>
        );
      })}
      {CATALOG_ITEMS.some((item) => permissions.has(item.permission)) && (
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => setCatalogOpen((prev) => !prev)}
            className="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-[13px] text-neutral-700 hover:bg-neutral-100"
            aria-expanded={catalogOpen}
          >
            <span className="flex items-center gap-2">
              <Boxes className="h-4 w-4 text-neutral-500" />
              Catalog
            </span>
            <span className={`text-xs text-neutral-400 transition ${catalogOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {catalogOpen && (
            <div className="space-y-1 pl-2">
              {CATALOG_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-2.5 py-1.5 text-[12px] ${
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
      {SETTINGS_ITEMS.some((item) => permissions.has(item.permission)) && (
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => setSettingsOpen((prev) => !prev)}
            className="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-[13px] text-neutral-700 hover:bg-neutral-100"
            aria-expanded={settingsOpen}
          >
            <span className="flex items-center gap-2">
              <Settings className="h-4 w-4 text-neutral-500" />
              Settings
            </span>
            <span className={`text-xs text-neutral-400 transition ${settingsOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {settingsOpen && (
            <div className="space-y-1 pl-2">
              {SETTINGS_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-2.5 py-1.5 text-[12px] ${
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
      {CONTENT_ITEMS.some((item) => permissions.has(item.permission)) && (
        <div className="space-y-1">
          <button
            type="button"
            onClick={() => setContentOpen((prev) => !prev)}
            className="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-[13px] text-neutral-700 hover:bg-neutral-100"
            aria-expanded={contentOpen}
          >
            <span className="flex items-center gap-2">
              <FileText className="h-4 w-4 text-neutral-500" />
              Contents
            </span>
            <span className={`text-xs text-neutral-400 transition ${contentOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {contentOpen && (
            <div className="space-y-1 pl-2">
              {CONTENT_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-2.5 py-1.5 text-[12px] ${
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
            className="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-[13px] text-neutral-700 hover:bg-neutral-100"
            aria-expanded={identityOpen}
          >
            <span className="flex items-center gap-2">
              <UserCog className="h-4 w-4 text-neutral-500" />
              Identity
            </span>
            <span className={`text-xs text-neutral-400 transition ${identityOpen ? "rotate-90" : ""}`}>›</span>
          </button>
          {identityOpen && (
            <div className="space-y-1 pl-2">
              {IDENTITY_ITEMS.filter((item) => permissions.has(item.permission)).map((item) => {
                const isActive = pathname?.startsWith(item.href);
                return (
                  <a
                    key={item.href}
                    className={`block rounded-lg px-2.5 py-1.5 text-[12px] ${
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
