import type { ReactNode } from "react";
import LogoutButton from "@/components/logout-button";
import RequireAuth from "@/components/require-auth";
import CurrentAccount from "@/components/current-account";

export default function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <RequireAuth>
      <div className="min-h-screen bg-neutral-50 text-neutral-900">
        <div className="grid min-h-screen grid-cols-1 md:grid-cols-[240px_1fr]">
          <aside className="border-b border-neutral-200 bg-white md:border-b-0 md:border-r">
            <div className="md:sticky md:top-0 md:h-screen md:overflow-y-auto md:p-6">
              <div className="p-6 md:p-0">
                <div className="text-xs uppercase tracking-[0.3em] text-neutral-500">
                  rs-ecommerce
                </div>
                <div className="mt-3 text-lg font-semibold text-neutral-900">Admin Console</div>
                <nav className="mt-6 space-y-2 text-sm text-neutral-600">
                  <a className="block rounded-lg bg-neutral-100 px-3 py-2 text-neutral-900" href="/admin">
                    Overview
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/orders">
                    Orders
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/products">
                    Products
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/auctions">
                    Auctions
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/customers">
                    Customers
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/inventory">
                    Inventory
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/settings">
                    Shop Settings
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/identity">
                    Identity
                  </a>
                  <a className="ml-2 block rounded-lg px-3 py-2 text-xs text-neutral-500 hover:bg-neutral-100" href="/admin/identity/roles">
                    Roles
                  </a>
                  <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/audit">
                    Audit Logs
                  </a>
                </nav>
                <div className="mt-8 flex items-center justify-between border-t border-neutral-200 pt-4">
                  <CurrentAccount />
                  <LogoutButton />
                </div>
              </div>
            </div>
          </aside>
          <main className="bg-neutral-50 p-8">{children}</main>
        </div>
      </div>
    </RequireAuth>
  );
}
