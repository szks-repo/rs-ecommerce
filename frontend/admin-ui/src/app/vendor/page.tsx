import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function VendorDashboard() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto max-w-6xl px-6 py-8">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
            <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Vendor Console</h1>
            <p className="mt-2 text-sm text-neutral-600">
              Manage your products and shipments inside the mall.
            </p>
          </div>
          <Button>Create Product</Button>
        </div>

        <div className="mt-8 grid gap-6 md:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle>Orders</CardTitle>
              <CardDescription>Pending shipment</CardDescription>
            </CardHeader>
            <CardContent className="text-3xl font-semibold">24</CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Revenue</CardTitle>
              <CardDescription>Last 7 days</CardDescription>
            </CardHeader>
            <CardContent className="text-3xl font-semibold">JPY 1.1M</CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Inventory</CardTitle>
              <CardDescription>Low stock items</CardDescription>
            </CardHeader>
            <CardContent className="text-3xl font-semibold">4</CardContent>
          </Card>
        </div>

        <div className="mt-8 grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>Common vendor tasks</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-neutral-600">
              <div>Update product price</div>
              <div>Print shipment labels</div>
              <div>Check inventory alerts</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
              <CardDescription>Last updates</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-neutral-600">
              <div>variant.update on SKU-001</div>
              <div>shipment.create for order #10021</div>
              <div>inventory.set for SKU-014</div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
