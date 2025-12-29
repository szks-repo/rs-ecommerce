import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import StaffCreateForm from "@/components/staff-create-form";
import RoleCreateForm from "@/components/role-create-form";
import RoleAssignForm from "@/components/role-assign-form";

export default function IdentityPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Identity & Staff</h1>
      <p className="mt-2 text-sm text-neutral-600">Manage staff access and roles.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>Recent Updates</CardTitle>
            <CardDescription className="text-neutral-500">
              Audit highlights
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-neutral-600">
            <div>store_settings.update by admin_123</div>
            <div>product.create by staff_002</div>
            <div>order.update_status by staff_008</div>
          </CardContent>
        </Card>
        <StaffCreateForm />
      </div>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <RoleCreateForm />
        <RoleAssignForm />
      </div>
    </div>
  );
}
