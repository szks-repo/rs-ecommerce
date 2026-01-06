import Link from "next/link";
import AuctionList from "@/components/auction-list";
import { Button } from "@/components/ui/button";
import AdminPageHeader from "@/components/admin-page-header";

export default function AuctionsPage() {
  return (
    <div className="space-y-8">
      <AdminPageHeader
        title="Auctions"
        description="Create auctions, review bids, and approve winning offers."
        actions={
          <Button asChild>
            <Link href="/admin/auctions/new">New auction</Link>
          </Button>
        }
      />

      <div className="grid gap-6">
        <AuctionList />
      </div>
    </div>
  );
}
