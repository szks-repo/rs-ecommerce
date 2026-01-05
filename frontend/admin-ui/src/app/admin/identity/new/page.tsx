import StaffCreateForm from "@/components/staff-create-form";
import StaffInviteForm from "@/components/staff-invite-form";
import OwnerTransferForm from "@/components/owner-transfer-form";
import { Button } from "@/components/ui/button";

export default function StaffNewPage() {
  return (
    <div>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Add Staff</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Create or invite staff, and manage ownership transfer.
          </p>
        </div>
        <Button asChild size="sm" variant="outline" className="mt-1">
          <a href="/admin/identity">Back to list</a>
        </Button>
      </div>

      <div className="mt-8 space-y-6">
        <StaffCreateForm />
        <StaffInviteForm />
        <OwnerTransferForm />
      </div>
    </div>
  );
}
