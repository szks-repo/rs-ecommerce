import Link from "next/link";
import { Button } from "@/components/ui/button";
import CustomerList from "@/components/customer-list";

export default function CustomersPage() {
  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Customers</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Customers</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Manage customer profiles and identities across stores.
          </p>
        </div>
        <Button asChild>
          <Link href="/admin/customers/new">New customer</Link>
        </Button>
      </div>
      <CustomerList />
    </div>
  );
}
