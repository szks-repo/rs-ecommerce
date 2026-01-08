# Inventory Management (Draft)

## 目的
- SKU 単位で在庫を管理する（product_sku を最小単位とする）
- 在庫のトラッキング（いつ・誰が・どの理由で増減したか）を残す
- デジタル SKU は在庫管理を不要にする（在庫=無限として扱う）

## 前提/スコープ
- 対象はストア内の在庫。マルチストアは `store_id` で分離。
- 物理在庫は `location`（倉庫/店舗）単位で管理。
- 予約/引当は別途 `inventory_reservations` で扱う。

## 主要な指標
- **on_hand**: 実在庫
- **reserved**: 予約済み（カート引当）
- **available**: `on_hand - reserved`（計算値）

## データモデル（案）
### inventory_stocks
SKU × ロケーションの現在値（集計結果）
- `id` (uuid)
- `store_id` (uuid)
- `sku_id` (uuid)  ※ product_skus.id
- `location_id` (uuid)
- `on_hand` (int)
- `reserved` (int)
- `created_at`, `updated_at`

制約:
- `on_hand >= 0`
- `reserved >= 0`

推奨インデックス:
- `UNIQUE (store_id, sku_id, location_id)`
- `INDEX (store_id, sku_id)`
- `INDEX (store_id, location_id)`

### inventory_movements
在庫トラッキング（append-only）
- `id` (uuid)
- `store_id` (uuid)
- `sku_id` (uuid)
- `location_id` (uuid)
- `movement_type` (text)
  - `inbound` / `outbound` / `adjust` / `reserve` / `release` / `consume` / `transfer_out` / `transfer_in`
- `quantity` (int, 符号なし)
- `before_on_hand` / `after_on_hand` (int)
- `before_reserved` / `after_reserved` (int)
- `reason` (text, 任意)
- `source_type` (text, 任意: order/cart/manual/import)
- `source_id` (uuid, 任意)
- `actor_id` (uuid, 任意)
- `occurred_at` (timestamptz)

推奨インデックス:
- `INDEX (store_id, sku_id, occurred_at DESC)`
- `INDEX (store_id, location_id, occurred_at DESC)`
- `INDEX (store_id, source_type, source_id)`

### inventory_reservations
カート引当（既存想定）
- `id` (uuid)
- `store_id` (uuid)
- `sku_id` (uuid)
- `location_id` (uuid)
- `cart_id` / `cart_item_id`
- `quantity`
- `status` (active/consumed/expired/released)
- `expires_at`

推奨インデックス:
- `INDEX (store_id, sku_id, status, expires_at)`
- `INDEX (store_id, cart_id)`

## DBスキーマ（SQLイメージ）
※既存の `inventory_reservations` がある前提で、`stocks`/`movements` を確定させる。

```sql
CREATE TABLE IF NOT EXISTS inventory_stocks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    location_id uuid NOT NULL REFERENCES store_locations(id),
    on_hand integer NOT NULL DEFAULT 0,
    reserved integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS inventory_stocks_unique
    ON inventory_stocks (store_id, sku_id, location_id);
CREATE INDEX IF NOT EXISTS inventory_stocks_sku_idx
    ON inventory_stocks (store_id, sku_id);
CREATE INDEX IF NOT EXISTS inventory_stocks_location_idx
    ON inventory_stocks (store_id, location_id);

CREATE TABLE IF NOT EXISTS inventory_movements (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    store_id uuid NOT NULL REFERENCES stores(id),
    sku_id uuid NOT NULL REFERENCES product_skus(id),
    location_id uuid NOT NULL REFERENCES store_locations(id),
    movement_type text NOT NULL,
    quantity integer NOT NULL,
    before_on_hand integer NOT NULL,
    after_on_hand integer NOT NULL,
    before_reserved integer NOT NULL,
    after_reserved integer NOT NULL,
    reason text,
    source_type text,
    source_id uuid,
    actor_id uuid,
    occurred_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS inventory_movements_sku_idx
    ON inventory_movements (store_id, sku_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS inventory_movements_location_idx
    ON inventory_movements (store_id, location_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS inventory_movements_source_idx
    ON inventory_movements (store_id, source_type, source_id);
```

## ワークフロー（案）
### 入荷
1. `inventory_stocks` を更新（on_hand += qty）
2. 同一トランザクションで `inventory_movements` を記録（inbound）

### 出荷/販売確定（consume）
1. `inventory_reservations` をロック
2. `inventory_stocks` を更新（on_hand -= qty, reserved -= qty）
3. `inventory_movements` を記録（consume）

### 予約/引当（reserve）
1. `inventory_stocks` を更新（reserved += qty）
   - `on_hand - reserved >= qty` を満たす場合のみ
2. `inventory_reservations` を作成
3. `inventory_movements` を記録（reserve）

### 予約解放（release/expire）
1. `inventory_stocks` を更新（reserved -= qty）
2. `inventory_reservations` を更新（released/expired）
3. `inventory_movements` を記録（release）

### 調整（adjust）
1. `inventory_stocks` の on_hand を上書き or 差分更新
2. `inventory_movements` を記録（adjust）

## 重要な設計ポイント
- **在庫変更は必ず movement を記録**（トラッキングの担保）
- **movement は append-only**（監査用途、後で集計検証可能）
- **トランザクション境界**: `stocks` と `movements` は同一 Tx
- **デジタル SKU**: `inventory_stocks` を作らず、予約/引当も不要

## API（案）
- `ListInventoryStocks(store_id, sku_id?, location_id?)`
- `SetInventory(stock)` / `AdjustInventory(diff)`
- `ListInventoryMovements(store_id, sku_id?, location_id?, date_range?)`
- `ReserveInventory(cart_item)` / `ReleaseReservation` / `ConsumeReservation`

### ロケーション別在庫 API（補足）
- `ListInventoryLocations(store_id)`（既存の Location を流用）
- `GetInventoryStock(store_id, sku_id, location_id)`
- `ListInventoryStocks(store_id, sku_id?, location_id?, status?)`
  - `status`: `in_stock | low | out` などのビュー用フィルタ（APIは任意）
- `SetInventoryByLocation(store_id, sku_id, location_id, on_hand, reason?)`
- `AdjustInventoryByLocation(store_id, sku_id, location_id, delta, reason?)`
- `TransferInventory(store_id, sku_id, from_location_id, to_location_id, quantity, reason?)`
  - 1 Tx で `transfer_out` / `transfer_in` の2つの movement を記録

## UI（案）
- SKU 詳細に在庫セクション
- ロケーション別在庫の一覧
- 在庫履歴（movement）を時系列で確認
- 調整（入出庫/棚卸し）を記録できるフォーム

### ロケーション別 UI（補足）
- SKU 詳細: ロケーションごとの在庫テーブル
  - 列: Location / On-hand / Reserved / Available / 最終更新
  - 行アクション: 調整 / 移動
- ロケーション詳細: 該当ロケーションの SKU 在庫一覧
  - 絞り込み: 在庫あり/なし, SKU 検索
  - 大規模想定: ページング + サーバー検索
- 在庫移動 UI
  - from/to のロケーション選択
  - 数量と理由（必須にして監査）
  - 実行前に available を表示

## 連携/今後の拡張
- 複数ロケーション間の移動（transfer）
- 受発注連携（inbound/outbound を外部イベントから取り込み）
- 在庫同期（outbox）
