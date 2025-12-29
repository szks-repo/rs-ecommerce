import InventorySetForm from "@/components/inventory-set-form";

export default function InventoryPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Inventory</h1>
      <p className="mt-2 text-sm text-neutral-400">Update stock per location.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <InventorySetForm />
      </div>
    </div>
  );
}
