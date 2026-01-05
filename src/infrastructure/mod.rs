/// 外部依存（DB/メール/検索/ストレージ等）のアダプタ集。
///
/// アプリケーション層はこのモジュール経由でインフラにアクセスし、
/// ドメインロジックから具体実装を分離する。
pub mod audit;
pub mod db;
pub mod email;
pub mod metafields;
pub mod outbox;
pub mod search;
pub mod storage;
