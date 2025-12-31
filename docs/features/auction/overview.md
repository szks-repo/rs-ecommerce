# Feature: Auction

## Purpose
- ストア単位のオークション販売（通常入札/封印入札）を提供し、落札は管理者承認で確定する

## Scope
- Included:
  - 通常入札（English）
  - 封印入札（sealed bid）
  - 管理者承認による落札確定
  - ストア単位で入札ルールと手数料を設定
- Excluded:
  - モール横断/テナント間連携
  - オークション自動延長（現時点では未導入）
  - フロント（ストアフロント）UI

## Domain Model (draft)
- Entities:
  - Auction（オークション）
  - AuctionBid（入札）
  - AuctionSettings（ストア設定）
- Value Objects:
  - Money
  - AuctionType: open | sealed
  - AuctionStatus: draft | scheduled | running | ended | awaiting_approval | approved | rejected | canceled
- Invariants:
  - running 状態以外は入札不可
  - 封印入札は終了/承認まで価格や最高額を公開しない
  - 落札は管理者承認が必須
  - 入札価格は start_price 以上かつ bid_increment 規定を満たす（open のみ）

## APIs
- AuctionService.CreateAuction
- AuctionService.ListAuctions
- AuctionService.GetAuction
- AuctionService.PlaceBid
- AuctionService.ListBids
- AuctionService.CloseAuction
- AuctionService.ApproveAuction
- AuctionService.GetAuctionSettings / UpdateAuctionSettings

## Data Model
- Tables:
  - auctions
  - auction_bids
  - store_auction_settings

## Flows
- オークション作成:
  1. 管理者が商品/SKUを選択してオークション作成
  2. 開始時刻により status を scheduled or running に設定
  3. 監査ログに auction.create
- 入札:
  1. running のオークションに入札
  2. open は最低入札条件をチェック
  3. bid を保存し、open は current_price を更新
  4. 監査ログに auction.bid
- 終了:
  1. CloseAuction で終了処理
  2. 最高入札を算出して awaiting_approval に遷移
  3. 監査ログに auction.end
- 承認:
  1. 管理者が ApproveAuction を実行
  2. approved へ遷移、承認者・承認時刻を保存
  3. 監査ログに auction.approve

## Audit
- Actions:
  - auction.create
  - auction.bid
  - auction.end
  - auction.approve

## Open Questions
- 予約価格（reserve）の扱い: 未達成時は未成立/再オークション？
- 手数料の課金タイミング
