import Link from "next/link";
import CustomerCreateForm from "@/components/customer-create-form";
import { Button } from "@/components/ui/button";

export default function CustomerCreatePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="mt-2 text-lg font-semibold text-neutral-900">Create Customer</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Create a customer and link identities for cross-store matching.
        </p>
        <div className="mt-4">
          <Button variant="outline" asChild>
            <Link href="/admin/customers">Back to list</Link>
          </Button>
        </div>
      </div>
      <CustomerCreateForm />
    </div>
  );
}
