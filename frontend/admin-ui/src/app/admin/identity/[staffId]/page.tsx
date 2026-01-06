import StaffDetailForm from "@/components/staff-detail-form";

export default async function StaffDetailPage({
  params,
}: {
  params: Promise<{ staffId: string }>;
}) {
  const { staffId } = await params;
  return (
    <div>
      <h1 className="text-lg font-semibold">Staff Detail</h1>
      <p className="mt-2 text-sm text-neutral-600">Review and update staff settings.</p>

      <div className="mt-8">
        <StaffDetailForm staffId={staffId} />
      </div>
    </div>
  );
}
