import Link from "next/link";
import ProductList from "@/components/product-list";
import { Button } from "@/components/ui/button";

export default function ProductsPage() {
  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Catalog</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Products</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Browse and search products. Create new products in a dedicated page.
          </p>
        </div>
        <Button asChild>
          <Link href="/admin/products/new">New product</Link>
        </Button>
      </div>

      <div className="grid gap-6">
        <ProductList />
      </div>
    </div>
  );
}
