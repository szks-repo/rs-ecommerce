import ProductCreateForm from "@/components/product-create-form";
import VariantCreateForm from "@/components/variant-create-form";

export default function ProductsPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Products</h1>
      <p className="mt-2 text-sm text-neutral-600">Create products and SKUs.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <ProductCreateForm />
        <VariantCreateForm />
      </div>
    </div>
  );
}
