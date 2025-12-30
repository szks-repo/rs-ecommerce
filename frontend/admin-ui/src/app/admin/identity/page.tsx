import StaffCreateForm from "@/components/staff-create-form";
import StaffListForm from "@/components/staff-list-form";

export default function IdentityPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Identity & Staff</h1>
      <p className="mt-2 text-sm text-neutral-600">Manage staff access and roles.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <StaffCreateForm />
        <StaffListForm />
      </div>

      <div className="mt-8 rounded-lg border border-neutral-200 bg-white p-6 text-sm text-neutral-600">
        Manage roles in the Identity section.{" "}
        <a className="font-medium text-neutral-900 underline" href="/admin/identity/roles">
          Open roles
        </a>
      </div>
    </div>
  );
}
