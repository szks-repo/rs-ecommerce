import StaffCreateForm from "@/components/staff-create-form";
import RoleCreateForm from "@/components/role-create-form";
import RoleAssignForm from "@/components/role-assign-form";

export default function IdentityPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Identity & Staff</h1>
      <p className="mt-2 text-sm text-neutral-600">Manage staff access and roles.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <StaffCreateForm />
      </div>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <RoleCreateForm />
        <RoleAssignForm />
      </div>
    </div>
  );
}
