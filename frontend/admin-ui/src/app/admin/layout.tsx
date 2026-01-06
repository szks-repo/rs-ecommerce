import type { ReactNode } from "react";
import LogoutButton from "@/components/logout-button";
import RequireAuth from "@/components/require-auth";
import CurrentAccount from "@/components/current-account";
import AdminSidebarNav from "@/components/admin-sidebar-nav";

export default function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <RequireAuth>
      <div className="min-h-screen bg-neutral-50 text-neutral-900">
        <div className="grid min-h-screen grid-cols-1 md:grid-cols-[220px_1fr]">
          <aside className="border-b border-neutral-200 bg-white md:border-b-0 md:border-r">
            <div className="md:sticky md:top-0 md:h-screen md:overflow-y-auto md:p-5">
              <div className="p-5 md:p-0">
                <div className="flex items-center gap-2">
                  <div className="rounded-full border border-neutral-200 bg-neutral-50 px-2 py-0.5 text-[10px] uppercase tracking-wide text-neutral-500">
                    rs-ecommerce Admin
                  </div>
                </div>
                <AdminSidebarNav />
                <div className="mt-8 flex items-center justify-between border-t border-neutral-200 pt-4">
                  <CurrentAccount />
                  <LogoutButton />
                </div>
              </div>
            </div>
          </aside>
          <main className="bg-neutral-50 p-8 text-sm">{children}</main>
        </div>
      </div>
    </RequireAuth>
  );
}
