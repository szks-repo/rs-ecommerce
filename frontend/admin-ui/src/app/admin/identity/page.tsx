import StaffListForm from "@/components/staff-list-form";
import StaffSessionList from "@/components/staff-session-list";
import { Button } from "@/components/ui/button";

export default function IdentityPage() {
  return (
    <div>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Identity & Staff</h1>
          <p className="mt-2 text-sm text-neutral-600">Manage staff access and roles.</p>
        </div>
        <Button asChild size="sm" className="mt-1">
          <a href="/admin/identity/new">Add staff</a>
        </Button>
      </div>

      <div className="mt-8">
        <StaffListForm />
      </div>

      <div className="mt-8">
        <StaffSessionList />
      </div>

    </div>
  );
}
