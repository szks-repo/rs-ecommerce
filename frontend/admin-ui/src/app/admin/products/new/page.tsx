import Link from "next/link";
import ProductCreateForm from "@/components/product-create-form";
import { Button } from "@/components/ui/button";

export default function ProductCreatePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="mt-2 text-lg font-semibold text-neutral-900">Create Product</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Register product master data and default SKU, or define variant axes first.
        </p>
        <div className="mt-4">
          <Button variant="outline" asChild>
            <Link href="/admin/products">Back to list</Link>
          </Button>
        </div>
      </div>

      <div className="grid gap-6">
        <ProductCreateForm />
      </div>
    </div>
  );
}
