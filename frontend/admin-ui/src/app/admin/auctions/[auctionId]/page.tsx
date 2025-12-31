import Link from "next/link";
import AuctionDetail from "@/components/auction-detail";
import { Button } from "@/components/ui/button";

export default async function AuctionDetailPage({
  params,
}: {
  params: Promise<{ auctionId: string }>;
}) {
  const { auctionId } = await params;
  return (
    <div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold">Auction Detail</h1>
          <p className="mt-2 text-sm text-neutral-600">Review bids and approve the winner.</p>
        </div>
        <Button variant="outline" asChild>
          <Link href="/admin/auctions">Back to list</Link>
        </Button>
      </div>
      <div className="mt-8">
        <AuctionDetail auctionId={auctionId} />
      </div>
    </div>
  );
}
