import Link from "next/link";
import AuctionList from "@/components/auction-list";
import { Button } from "@/components/ui/button";

export default function AuctionsPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Auctions</h1>
      <p className="mt-2 text-sm text-neutral-600">
        Create auctions, review bids, and approve winning offers.
      </p>
      <div className="mt-6 flex flex-wrap items-center gap-3">
        <Button asChild>
          <Link href="/admin/auctions/new">New auction</Link>
        </Button>
        <Button variant="outline" asChild>
          <Link href="/admin/auctions/settings">Settings</Link>
        </Button>
      </div>

      <div className="mt-8 grid gap-6">
        <AuctionList />
      </div>
    </div>
  );
}
