import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import StoreBadge from "@/components/store-badge";
import { Button } from "@/components/ui/button";

export default function AdminDashboard() {
  return (
    <div>
      <section className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Welcome back</h1>
          <p className="text-sm text-neutral-400">
            Admin/Staff operational dashboard.
          </p>
        </div>
        <Button className="bg-white text-neutral-900 hover:bg-neutral-100">
          Create Product
        </Button>
      </section>

      <div className="mt-4">
        <StoreBadge />
      </div>

      <section className="mt-8 grid gap-6 md:grid-cols-3">
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
      </section>
    </div>
  );
}
