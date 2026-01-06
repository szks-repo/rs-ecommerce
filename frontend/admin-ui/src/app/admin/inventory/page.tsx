import InventorySetForm from "@/components/inventory-set-form";
import AdminPageHeader from "@/components/admin-page-header";

export default function InventoryPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Inventory"
        description="Update stock per location."
      />

      <div className="grid gap-6 md:grid-cols-2">
        <InventorySetForm />
      </div>
    </div>
  );
}
