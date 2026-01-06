import StaffListForm from "@/components/staff-list-form";
import StaffSessionList from "@/components/staff-session-list";
import { Button } from "@/components/ui/button";
import AdminPageHeader from "@/components/admin-page-header";

export default function IdentityPage() {
  return (
    <div>
      <AdminPageHeader
        title="Identity & Staff"
        description="Manage staff access and roles."
        actions={
          <Button asChild size="sm">
            <a href="/admin/identity/new">Add staff</a>
          </Button>
        }
      />

      <div className="mt-8">
        <StaffListForm />
      </div>

      <div className="mt-8">
        <StaffSessionList />
      </div>

    </div>
  );
}
