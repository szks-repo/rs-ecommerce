import type { ReactNode } from "react";

export default function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-50">
      <div className="grid min-h-screen grid-cols-1 md:grid-cols-[240px_1fr]">
        <aside className="border-b border-neutral-800 bg-neutral-900 p-6 md:border-b-0 md:border-r">
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">
            rs-ecommerce
          </div>
          <div className="mt-3 text-lg font-semibold">Admin Console</div>
          <nav className="mt-6 space-y-2 text-sm text-neutral-300">
            <a className="block rounded-lg bg-neutral-800 px-3 py-2" href="/admin">
              Overview
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="/admin/identity">
              Identity & Staff
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="/admin/products">
              Products
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="/admin/inventory">
              Inventory
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="/admin/locations">
              Locations
            </a>
          </nav>
        </aside>
        <main className="bg-neutral-950 p-8">{children}</main>
      </div>
    </div>
  );
}
