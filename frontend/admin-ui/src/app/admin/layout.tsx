import type { ReactNode } from "react";

export default function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen bg-neutral-50 text-neutral-900">
      <div className="grid min-h-screen grid-cols-1 md:grid-cols-[240px_1fr]">
        <aside className="border-b border-neutral-200 bg-white p-6 md:border-b-0 md:border-r">
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-500">
            rs-ecommerce
          </div>
          <div className="mt-3 text-lg font-semibold text-neutral-900">Admin Console</div>
          <nav className="mt-6 space-y-2 text-sm text-neutral-600">
            <a className="block rounded-lg bg-neutral-100 px-3 py-2 text-neutral-900" href="/admin">
              Overview
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/identity">
              Identity & Staff
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/products">
              Products
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/inventory">
              Inventory
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/settings">
              Shop Settings
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-100" href="/admin/locations">
              Locations
            </a>
          </nav>
        </aside>
        <main className="bg-neutral-50 p-8">{children}</main>
      </div>
    </div>
  );
}
