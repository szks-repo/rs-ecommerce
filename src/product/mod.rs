/// 商品・SKU・在庫・メディアを扱う境界。
///
/// 商品マスタ/バリアント（SKU）/デジタル配信/画像アップロードの
/// 主要ユースケースを service に集約する。

pub mod digital;
pub mod domain;
pub mod media;
pub mod service;
