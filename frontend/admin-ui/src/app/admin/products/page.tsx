import Link from "next/link";
import ProductList from "@/components/product-list";
import { Button } from "@/components/ui/button";

export default function ProductsPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Products</h1>
      <p className="mt-2 text-sm text-neutral-600">
        Browse and search products. Create new products in a dedicated page.
      </p>
      <div className="mt-6 flex flex-wrap items-center gap-3">
        <Button asChild>
          <Link href="/admin/products/new">New product</Link>
        </Button>
      </div>

      <div className="mt-8 grid gap-6">
        <ProductList />
      </div>
    </div>
  );
}
