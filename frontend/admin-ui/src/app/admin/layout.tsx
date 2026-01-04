import type { ReactNode } from "react";
import LogoutButton from "@/components/logout-button";
import RequireAuth from "@/components/require-auth";
import CurrentAccount from "@/components/current-account";
import AdminSidebarNav from "@/components/admin-sidebar-nav";

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
                <AdminSidebarNav />
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
