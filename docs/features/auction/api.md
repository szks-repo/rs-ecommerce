# Auction APIs (Draft)

## AuctionService

### CreateAuction
- 入力: store/tenant, product_id, variant_id, auction_type, start_at, end_at, bid_increment, start_price, reserve_price?, buyout_price?, actor
- 出力: Auction

### ListAuctions
- 入力: store/tenant, status?, page?
- 出力: Auction[] + page

### GetAuction
- 入力: store/tenant, auction_id
- 出力: Auction

### PlaceBid
- 入力: store/tenant, auction_id, customer_id?, amount, actor?
- 出力: Auction + Bid

### ListBids
- 入力: store/tenant, auction_id
- 出力: Bid[]

### CloseAuction
- 入力: store/tenant, auction_id, actor
- 出力: Auction（awaiting_approval へ）

### ApproveAuction
- 入力: store/tenant, auction_id, actor
- 出力: Auction（approved）

### GetAuctionSettings
- 入力: store/tenant
- 出力: AuctionSettings

### UpdateAuctionSettings
- 入力: store/tenant, bid_increment, fee_rate?, actor
- 出力: AuctionSettings
