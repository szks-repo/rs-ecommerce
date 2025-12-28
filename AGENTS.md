# rs-ecommerce — Agent Guide

## 目的・ゴール
- Rust製のECommerce Plattformを実現する（本店型が主流、モール対応も可能）
- 日本国内をメインターゲット

## 非機能要件（想定）
- 注文数: 100,000 Tx/日
- 管理者数: 最大 500 名
- モール: 最大 2,000 テナント

## プロダクト方針
- 本店型の独自EC構築を主軸に、モール機能はオプションとして提供
- まずは銀行振込・代引きのみを対象にする

## コンポーネント
- StoreBackend
  - 高速で柔軟なオペレーション
  - 外部API / Webhook連携
- StoreFront
  - 高速検索とUX最適化

## アーキテクチャ原則
- Context境界を明確に分ける（DDD志向）
- Contextの複雑度に応じて DomainModel / TransactionScript を使い分け
- ConnectRPC JSON（pbjson）を採用
- DBはPostgres、検索はMeilisearch
- 監査ログは全設定系更新に差し込む

## 開発環境
- まずは Docker Compose を採用
- 詳細は `docs/dev/setup.md`

## 主要ドキュメント
- ドキュメント索引: `docs/README.md`
- アーキテクチャ概要: `docs/architecture/overview.md`
- DBスキーマ: `docs/architecture/db_schema.md`
- ConnectRPC: `docs/api/connectrpc.md`
- 監査ログ: `docs/operations/audit_log.md`
- Store Settings: `docs/features/store_settings/overview.md`

## 競合
- Shopify

## 実装上の注意
- 検索インデックス更新は商品登録/更新時に必ず反映
- 監査ログは actor / request_id / ip / user_agent を記録
- APIはJSON-onlyで進める（現状）
