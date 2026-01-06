import RoleCreateForm from "@/components/role-create-form";
import RoleList from "@/components/role-list";
import AdminPageHeader from "@/components/admin-page-header";

export default function RolesPage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader title="Roles" description="Create and browse roles." />

      <div className="grid gap-6">
        <RoleCreateForm />
        <RoleList />
      </div>
    </div>
  );
}
