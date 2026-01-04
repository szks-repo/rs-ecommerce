/// 監査ログの記録・検索に関する境界。
///
/// ドメイン操作からの監査記録は service 経由で行い、永続化は
/// `src/infrastructure/audit.rs` に委譲する。

pub mod service;
