//! Identity domain boundary (planned).
//!
//! 現時点では `AuthService` / `StoreStaff` / `Permissions` に分散している
//! 認証・スタッフ・ロール/権限のロジックを、将来的にこのモジュールへ統合する。
//! いまは `src/rpc/identity.rs` が既存サービスへのファサードになっているため、
//! ここは空のままにしている。
//!
//! 統合方針は `docs/architecture/identity.md` を参照。

pub mod context;
pub mod error;
pub mod repository;
pub mod service;
pub mod status;
