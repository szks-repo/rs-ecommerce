import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import StoreBadge from "@/components/store-badge";
import StaffCreateForm from "@/components/staff-create-form";
import RoleCreateForm from "@/components/role-create-form";
import RoleAssignForm from "@/components/role-assign-form";
import ProductCreateForm from "@/components/product-create-form";
import VariantCreateForm from "@/components/variant-create-form";
import InventorySetForm from "@/components/inventory-set-form";

export default function AdminDashboard() {
  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-50">
      <div className="grid min-h-screen grid-cols-1 md:grid-cols-[240px_1fr]">
        <aside className="border-b border-neutral-800 bg-neutral-900 p-6 md:border-b-0 md:border-r">
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">
            rs-ecommerce
          </div>
          <div className="mt-3 text-lg font-semibold">Admin Console</div>
          <nav className="mt-6 space-y-2 text-sm text-neutral-300">
            <a className="block rounded-lg bg-neutral-800 px-3 py-2" href="#">
              Overview
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="#">
              Products
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="#">
              Orders
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="#">
              Promotions
            </a>
            <a className="block rounded-lg px-3 py-2 hover:bg-neutral-800" href="#">
              Store Settings
            </a>
          </nav>
        </aside>

        <main className="bg-neutral-950 p-8">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div>
              <h1 className="text-2xl font-semibold">Welcome back</h1>
              <p className="text-sm text-neutral-400">
                Admin/Staff operational dashboard.
              </p>
            </div>
            <Button className="bg-white text-neutral-900 hover:bg-neutral-100">
              Create Product
            </Button>
          </div>

          <div className="mt-4">
            <StoreBadge />
          </div>

          <div className="mt-8 grid gap-6 md:grid-cols-3">
            <Card className="border-neutral-800 bg-neutral-900 text-neutral-50">
              <CardHeader>
                <CardTitle>Orders</CardTitle>
                <CardDescription className="text-neutral-400">
                  Pending shipment
                </CardDescription>
              </CardHeader>
              <CardContent className="text-3xl font-semibold">128</CardContent>
            </Card>
            <Card className="border-neutral-800 bg-neutral-900 text-neutral-50">
              <CardHeader>
                <CardTitle>Revenue</CardTitle>
                <CardDescription className="text-neutral-400">
                  Last 7 days
                </CardDescription>
              </CardHeader>
              <CardContent className="text-3xl font-semibold">JPY 4.2M</CardContent>
            </Card>
            <Card className="border-neutral-800 bg-neutral-900 text-neutral-50">
              <CardHeader>
                <CardTitle>Inventory</CardTitle>
                <CardDescription className="text-neutral-400">
                  Low stock items
                </CardDescription>
              </CardHeader>
              <CardContent className="text-3xl font-semibold">12</CardContent>
            </Card>
          </div>

          <div className="mt-8 grid gap-6 md:grid-cols-2">
            <Card className="border-neutral-800 bg-neutral-900 text-neutral-50">
              <CardHeader>
                <CardTitle>Recent Updates</CardTitle>
                <CardDescription className="text-neutral-400">
                  Audit highlights
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3 text-sm text-neutral-300">
                <div>store_settings.update by admin_123</div>
                <div>product.create by staff_002</div>
                <div>order.update_status by staff_008</div>
              </CardContent>
            </Card>
            <StaffCreateForm />
          </div>

          <div className="mt-8 grid gap-6 md:grid-cols-2">
            <RoleCreateForm />
            <RoleAssignForm />
          </div>

          <div className="mt-8 grid gap-6 md:grid-cols-2">
            <ProductCreateForm />
            <VariantCreateForm />
          </div>

          <div className="mt-8 grid gap-6 md:grid-cols-2">
            <InventorySetForm />
          </div>
        </main>
      </div>
    </div>
  );
}
