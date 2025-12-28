# Identity Architecture (Draft)

## 目的
認証・スタッフ・ロール/権限を「Identity境界」として整理し、  
ストア運営の**ログイン/権限/スタッフ管理**を一つのドメインとして統合する。

## 現状の実装配置
- **RPCファサード**: `src/rpc/identity.rs`
  - 既存のAuth/StoreStaff/Permissionsをまとめて公開する窓口
- **認証**: `src/auth/service.rs`
  - パスワード検証 + JWT発行
- **スタッフ**: `src/store_staff/service.rs`
  - staff作成・ログインID/メール/電話の扱い
- **権限/ロール**: `src/permissions/service.rs`
  - role/permissionの作成・割当

## 統合方針
1. **IdentityServiceを正規API**にする  
   - クライアント/UIはIdentityServiceだけを利用
2. **ドメインロジックを `src/identity` に集約**  
   - 認証/スタッフ/権限のユースケースをここに定義
3. **内部依存は「infra + repositories」で吸収**  
   - DB/暗号/JWTはinfrastructure層に切り出して薄く保つ

## API設計の基本
IdentityServiceは「ストア運営者/スタッフのID管理」に限定し、  
ストア設定や商品など他コンテキストへは直接踏み込まない。

## RPC層の責務
- ConnectRPCのリクエスト/レスポンス整形
- 業務ロジック（IdentityService）へのパス
- それ以外の判断はIdentity側へ寄せる

## 入口の統一
- 認証/スタッフ/ロール/権限は **IdentityServiceのみ** を公開する
- AuthService / StoreStaffService / RoleService は削除済み

### 主要RPC
- `SignIn`
- `CreateStaff`
- `CreateRole`
- `AssignRoleToStaff`
- `ListRoles`

※ 現状のprotobufは `proto/ecommerce/v1/identity.proto`

## ドメイン境界
Identityが担う責務:
- 認証 (credential検証 + JWT発行)
- スタッフのID/ログイン情報の管理
- ロール/権限の定義と付与
- Audit用のactor解決/出力補助

Identityが担わない責務:
- ストア設定
- 商品/在庫/受注
- テナント作成や初期設定

## データモデルの考え方
- **store_staff** は「メールが無い現場スタッフ」も想定  
  - `login_id` / `phone` などで柔軟に運用
- Role/Permissionはストア単位でスコープ

## 実装ロードマップ (整理・統合)
1. `src/identity/service.rs` を新設し、  
   `SignIn/CreateStaff/CreateRole/AssignRole` をユースケース化
2. `src/rpc/identity.rs` は `IdentityService` のみを呼び出す構造へ
3. 既存 `auth/store_staff/permissions` は内部モジュールとして段階的に縮小

## 依存ルール
- UI → IdentityService (唯一の入口)
- Identity → Infra (DB/JWT/Hash)
- 他ドメイン → Identity (必要時に参照)

## 補足
- `src/identity/mod.rs` は現時点で空だが、統合の起点として予約済み。
