import Link from "next/link";
import AuctionList from "@/components/auction-list";
import { Button } from "@/components/ui/button";

export default function AuctionsPage() {
  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Auctions</div>
          <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Auctions</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Create auctions, review bids, and approve winning offers.
          </p>
        </div>
        <Button asChild>
          <Link href="/admin/auctions/new">New auction</Link>
        </Button>
      </div>

      <div className="grid gap-6">
        <AuctionList />
      </div>
    </div>
  );
}
