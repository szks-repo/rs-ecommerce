import Link from "next/link";
import { Button } from "@/components/ui/button";
import CustomerList from "@/components/customer-list";

export default function CustomersPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold">Customers</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Manage customer profiles and identities across stores.
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <Button asChild>
          <Link href="/admin/customers/new">New customer</Link>
        </Button>
      </div>
      <CustomerList />
    </div>
  );
}
