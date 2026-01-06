import Link from "next/link";
import { Button } from "@/components/ui/button";
import CustomerList from "@/components/customer-list";
import AdminPageHeader from "@/components/admin-page-header";

export default function CustomersPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Customers"
        description="Manage customer profiles and identities across stores."
        actions={
          <Button asChild>
            <Link href="/admin/customers/new">New customer</Link>
          </Button>
        }
      />
      <CustomerList />
    </div>
  );
}
