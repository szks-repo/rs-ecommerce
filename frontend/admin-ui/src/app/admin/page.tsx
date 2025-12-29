import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import StoreBadge from "@/components/store-badge";
import { Button } from "@/components/ui/button";

export default function AdminDashboard() {
  return (
    <div>
      <section className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Welcome back</h1>
          <p className="text-sm text-neutral-600">
            Admin/Staff operational dashboard.
          </p>
        </div>
        <Button className="bg-neutral-900 text-white hover:bg-neutral-800">
          Create Product
        </Button>
      </section>

      <div className="mt-4">
        <StoreBadge />
      </div>

      <section className="mt-8 grid gap-6 md:grid-cols-3">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Orders</CardTitle>
            <CardDescription className="text-neutral-500">
              Pending shipment
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">128</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Revenue</CardTitle>
            <CardDescription className="text-neutral-500">
              Last 7 days
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">JPY 4.2M</CardContent>
        </Card>
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Inventory</CardTitle>
            <CardDescription className="text-neutral-500">
              Low stock items
            </CardDescription>
          </CardHeader>
          <CardContent className="text-3xl font-semibold">12</CardContent>
        </Card>
      </section>
    </div>
  );
}
