"use client";

import { useParams } from "next/navigation";
import RoleDetailForm from "@/components/role-detail-form";

export default function RoleDetailPage() {
  const params = useParams();
  const roleId = Array.isArray(params.roleId) ? params.roleId[0] : params.roleId;

  if (!roleId) {
    return <div className="text-sm text-neutral-600">Role not found.</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Admin</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Role Detail</h1>
        <p className="mt-2 text-sm text-neutral-600">Update role name and permissions.</p>
      </div>
      <RoleDetailForm roleId={roleId} />
    </div>
  );
}
