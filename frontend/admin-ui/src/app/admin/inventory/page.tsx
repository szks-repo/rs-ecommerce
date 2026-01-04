import InventorySetForm from "@/components/inventory-set-form";

export default function InventoryPage() {
  return (
    <div className="space-y-8">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Catalog</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Inventory</h1>
        <p className="mt-2 text-sm text-neutral-600">Update stock per location.</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <InventorySetForm />
      </div>
    </div>
  );
}
