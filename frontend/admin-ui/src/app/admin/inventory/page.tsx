import InventorySetForm from "@/components/inventory-set-form";
import InventoryList from "@/components/inventory-list";
import InventoryMovementList from "@/components/inventory-movement-list";
import AdminPageHeader from "@/components/admin-page-header";

export default function InventoryPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Inventory"
        description="Manage inventory per SKU and location."
      />

      <InventoryList />

      <div className="space-y-3">
        <div>
          <h2 className="text-lg font-semibold text-neutral-900">Inventory movements</h2>
          <p className="text-sm text-neutral-500">Track stock changes per SKU and location.</p>
        </div>
        <InventoryMovementList />
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <InventorySetForm />
      </div>
    </div>
  );
}
