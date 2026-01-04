import { create } from "@bufbuild/protobuf";
import { timestampFromDate } from "@bufbuild/protobuf/wkt";
import { createServiceClient } from "@/lib/connect";
import {
  AuctionService,
  ListAuctionsRequestSchema,
  GetAuctionRequestSchema,
  CreateAuctionRequestSchema,
  UpdateAuctionRequestSchema,
  ListBidsRequestSchema,
  ListAutoBidsRequestSchema,
  CloseAuctionRequestSchema,
  ApproveAuctionRequestSchema,
} from "@/gen/ecommerce/v1/auction_pb";
import { MoneySchema } from "@/gen/ecommerce/v1/common_pb";

const client = createServiceClient(AuctionService);

function money(amount: number, currency: string) {
  return create(MoneySchema, {
    amount: BigInt(amount),
    currency,
  });
}

export async function listAuctions(params: { status?: string }) {
  return client.listAuctions(
    create(ListAuctionsRequestSchema, {
      status: params.status || "",
    })
  );
}

export async function getAuction(params: { auctionId: string }) {
  return client.getAuction(
    create(GetAuctionRequestSchema, {
      auctionId: params.auctionId,
    })
  );
}

export async function createAuction(params: {
  skuId: string;
  title: string;
  description?: string;
  auctionType: string;
  status: string;
  startAt: Date;
  endAt: Date;
  startPriceAmount: number;
  reservePriceAmount?: number;
  buyoutPriceAmount?: number;
  bidIncrementAmount?: number;
  currency: string;
}) {
  if (!params.skuId.trim()) {
    throw new Error("sku_id is required.");
  }
  if (!params.title.trim()) {
    throw new Error("title is required.");
  }
  if (!Number.isFinite(params.startPriceAmount)) {
    throw new Error("start_price is required.");
  }
  if (params.startPriceAmount < 0) {
    throw new Error("start_price must be zero or positive.");
  }
  if (params.reservePriceAmount != null && params.reservePriceAmount < 0) {
    throw new Error("reserve_price must be zero or positive.");
  }
  if (params.buyoutPriceAmount != null && params.buyoutPriceAmount < 0) {
    throw new Error("buyout_price must be zero or positive.");
  }
  if (params.bidIncrementAmount != null && params.bidIncrementAmount < 0) {
    throw new Error("bid_increment must be zero or positive.");
  }
  if (!Number.isFinite(params.startAt.getTime()) || !Number.isFinite(params.endAt.getTime())) {
    throw new Error("start_at/end_at is required.");
  }

  return client.createAuction(
    create(CreateAuctionRequestSchema, {
      skuId: params.skuId,
      auctionType: params.auctionType,
      status: params.status,
      startAt: timestampFromDate(params.startAt),
      endAt: timestampFromDate(params.endAt),
      title: params.title.trim(),
      description: params.description?.trim() || "",
      startPrice: money(params.startPriceAmount, params.currency),
      reservePrice:
        typeof params.reservePriceAmount === "number"
          ? money(params.reservePriceAmount, params.currency)
          : undefined,
      buyoutPrice:
        typeof params.buyoutPriceAmount === "number"
          ? money(params.buyoutPriceAmount, params.currency)
          : undefined,
      bidIncrement:
        typeof params.bidIncrementAmount === "number"
          ? money(params.bidIncrementAmount, params.currency)
          : undefined,
    })
  );
}

export async function updateAuction(params: {
  auctionId: string;
  skuId: string;
  title: string;
  description?: string;
  auctionType: string;
  status: string;
  startAt: Date;
  endAt: Date;
  startPriceAmount: number;
  reservePriceAmount?: number;
  buyoutPriceAmount?: number;
  bidIncrementAmount?: number;
  currency: string;
}) {
  if (!params.auctionId.trim()) {
    throw new Error("auction_id is required.");
  }
  if (!params.skuId.trim()) {
    throw new Error("sku_id is required.");
  }
  if (!params.title.trim()) {
    throw new Error("title is required.");
  }
  if (!Number.isFinite(params.startPriceAmount)) {
    throw new Error("start_price is required.");
  }
  if (!Number.isFinite(params.startAt.getTime()) || !Number.isFinite(params.endAt.getTime())) {
    throw new Error("start_at/end_at is required.");
  }
  if (params.startPriceAmount < 0) {
    throw new Error("start_price must be zero or positive.");
  }
  if (params.reservePriceAmount != null && params.reservePriceAmount < 0) {
    throw new Error("reserve_price must be zero or positive.");
  }
  if (params.buyoutPriceAmount != null && params.buyoutPriceAmount < 0) {
    throw new Error("buyout_price must be zero or positive.");
  }
  if (params.bidIncrementAmount != null && params.bidIncrementAmount < 0) {
    throw new Error("bid_increment must be zero or positive.");
  }

  return client.updateAuction(
    create(UpdateAuctionRequestSchema, {
      auctionId: params.auctionId,
      skuId: params.skuId,
      auctionType: params.auctionType,
      status: params.status,
      startAt: timestampFromDate(params.startAt),
      endAt: timestampFromDate(params.endAt),
      title: params.title.trim(),
      description: params.description?.trim() || "",
      startPrice: money(params.startPriceAmount, params.currency),
      reservePrice:
        typeof params.reservePriceAmount === "number"
          ? money(params.reservePriceAmount, params.currency)
          : undefined,
      buyoutPrice:
        typeof params.buyoutPriceAmount === "number"
          ? money(params.buyoutPriceAmount, params.currency)
          : undefined,
      bidIncrement:
        typeof params.bidIncrementAmount === "number"
          ? money(params.bidIncrementAmount, params.currency)
          : undefined,
    })
  );
}

export async function listBids(params: { auctionId: string }) {
  return client.listBids(
    create(ListBidsRequestSchema, {
      auctionId: params.auctionId,
    })
  );
}

export async function listAutoBids(params: { auctionId: string }) {
  return client.listAutoBids(
    create(ListAutoBidsRequestSchema, {
      auctionId: params.auctionId,
    })
  );
}

export async function closeAuction(params: { auctionId: string }) {
  return client.closeAuction(
    create(CloseAuctionRequestSchema, {
      auctionId: params.auctionId,
    })
  );
}

export async function approveAuction(params: { auctionId: string }) {
  return client.approveAuction(
    create(ApproveAuctionRequestSchema, {
      auctionId: params.auctionId,
    })
  );
}
