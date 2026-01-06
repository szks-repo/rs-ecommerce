import Link from "next/link";
import ProductList from "@/components/product-list";
import { Button } from "@/components/ui/button";
import AdminPageHeader from "@/components/admin-page-header";

export default function ProductsPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Products"
        description="Browse and search products. Create new products in a dedicated page."
        actions={
          <Button asChild>
            <Link href="/admin/products/new">New product</Link>
          </Button>
        }
      />

      <div className="grid gap-6">
        <ProductList />
      </div>
    </div>
  );
}
