import Link from "next/link";
import AuctionCreateForm from "@/components/auction-create-form";
import { Button } from "@/components/ui/button";

export default function AuctionCreatePage() {
  return (
    <div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-lg font-semibold">New Auction</h1>
          <p className="mt-2 text-sm text-neutral-600">Set auction details and schedule bidding.</p>
        </div>
        <Button variant="outline" asChild>
          <Link href="/admin/auctions">Back to list</Link>
        </Button>
      </div>
      <div className="mt-8">
        <AuctionCreateForm />
      </div>
    </div>
  );
}
