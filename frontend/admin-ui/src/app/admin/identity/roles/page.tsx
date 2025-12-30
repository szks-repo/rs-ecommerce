import RoleCreateForm from "@/components/role-create-form";
import RoleList from "@/components/role-list";

export default function RolesPage() {
  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Admin</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Roles</h1>
        <p className="mt-2 text-sm text-neutral-600">Create and browse roles.</p>
      </div>

      <div className="grid gap-6">
        <RoleCreateForm />
        <RoleList />
      </div>
    </div>
  );
}
