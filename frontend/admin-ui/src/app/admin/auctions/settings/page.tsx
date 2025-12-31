import Link from "next/link";
import AuctionSettingsForm from "@/components/auction-settings-form";
import { Button } from "@/components/ui/button";

export default function AuctionSettingsPage() {
  return (
    <div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold">Auction Settings</h1>
          <p className="mt-2 text-sm text-neutral-600">Configure store-level auction rules.</p>
        </div>
        <Button variant="outline" asChild>
          <Link href="/admin/auctions">Back to auctions</Link>
        </Button>
      </div>
      <div className="mt-8">
        <AuctionSettingsForm />
      </div>
    </div>
  );
}
