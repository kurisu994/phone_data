---
name: 10-data-core
description: phone.dat 二进制格式、公共类型层与统一 trait 契约
paths:
  - "src/common.rs"
  - "src/lib.rs"
---

# 数据格式与公共层

`src/common.rs` 是**唯一的公共类型层**。任何算法都不许在本地重新定义 `Index`、`ParsedRecord`
或解析函数——历史上 `phone_bloom.rs` / `phone_simd.rs` 各有一份重复的 `Index`，已在 `4c5a3ee`
统一到 `common.rs`；新增算法时不要把这个坑挖回来。

## phone.dat 二进制布局

```
| 4 bytes        | 版本号，按 UTF-8 读出（如 "1701" = 17 年 1 月）
| 4 bytes        | 第一个索引的偏移 index_offset（小端 i32）
| index_offset-8 | 记录区，每条 "<省>|<市>|<邮编>|<区号>\0"
| 剩余           | 索引区，每条 9 字节
```

索引条目 9 字节的切法：`[..4]` 号段前 7 位，`[4..8]` 记录区偏移，`[8]` 卡类型。
**索引必须按号段升序**——二分查找与布隆过滤的精确阶段都依赖这个前提，`phone.dat` 天然满足，
若将来换数据源必须先校验排序。

## 解析约定

- `utils::four_u8_to_i32` 固定小端；长度不足 4 字节时返回 `0` 而非报错（调用方需自行保证长度）。
- `utils::parse_record_data(records, offset)` 从 `offset - 8` 开始找 `\0`，把中间的串按 `|`
  切成 4 段；段数不等于 4 或非 UTF-8 都返回 `ErrorKind::InvalidPhoneDatabase`。
  **传 offset 时不要再自己减 8。**
- `utils::build_phone_info(record, card_type)` 负责把 `u8` 卡类型翻成中文描述，
  卡类型非法时抛出 `ErrorKind::InvalidOpNo`。

读文件统一用 `BufReader` + `read_exact`；索引区用 `loop` + 匹配
`io::ErrorKind::UnexpectedEof` 结束，这是「读到文件尾」的既有写法，不要在中间改成按长度计算。

## 公共类型

- `PhoneNoInfo { province, city, zip_code, area_code, card_type }` — 查询结果，也是 API 的
  JSON 形状，字段名直接用（`zip_code` / `area_code` 不会被改写成驼峰），改动即破坏 API 契约。
- `ErrorKind` — `InvalidPhoneDatabase` / `InvalidLength` / `NotFound` / `InvalidOpNo`，
  已实现 `Display` + `std::error::Error`，配合 `anyhow::Result` 使用。
- `Index { phone_no_prefix: i32, records_offset: i32, card_type: u8 }` — 二分/布隆/SIMD 共用。
- `CardType` — 库中 `u8` ↔ 枚举的映射固定为：

  | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
  |---|---|---|---|---|---|---|---|
  | 移动 | 联通 | 电信 | 电信虚拟 | 联通虚拟 | 移动虚拟 | 广电 | 广电虚拟 |

  取值经 `from_u8` 校验，描述经 `get_description()` 取中文，`get_code()` 取回 `u8`。
  新增运营商只能**追加** 9 及以后，不能改已有编号（库里存的是数字）。

## trait 契约

```rust
pub trait PhoneLookup {
    fn find(&self, no: &str) -> Result<PhoneNoInfo>;
    fn find_batch(&self, phones: &[&str]) -> Vec<Result<PhoneNoInfo>> { /* 默认逐条 find */ }
    fn validate_phone_no(&self, no: &str) -> Result<i32> { /* 默认实现 */ }
}
pub trait PhoneStats {
    fn total_entries(&self) -> usize;
    fn version(&self) -> &str;
    fn memory_usage_bytes(&self) -> usize;
}
```

`validate_phone_no` 的默认实现是**唯一**的长度与前缀解析规则：长度必须 7–11，7 位直接用，
更长取 `no[..7]`，解析失败算 `InvalidLength`。

⚠️ 现实偏差：四种算法都**没有调用** `validate_phone_no`，而是在各自的 `find` 里重复了一遍
同样的 6 行逻辑（`phone_hash.rs` 甚至写了两遍）。改校验规则时四处都要改，不能只改 trait。
统一它们属于待办，不是已完成的事实。

## lib.rs 只做导出

`lib.rs` 不承载任何算法逻辑，只声明模块、重导出 `common` 的公共类型，并把
`PhoneDataSimd as PhoneData` 作为默认实现导出。**新增算法后必须在这里加 `pub use`**，
否则 `tests/test_suite.rs`、`benches/` 无法通过短路径引用。
