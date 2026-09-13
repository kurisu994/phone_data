# 项目定位

Rust 编写的手机号归属地查询服务：读取 `phone.dat` 二进制库，按手机号前 7 位查出省份、城市、
邮编、长途区号与运营商；同时以 actix-web 暴露 HTTP API。项目规模很小（`src/` 约 1.3k 行），
复杂度集中在「同一份数据用四种查找算法实现，并保证结果完全一致」。

## 领域语言

| 术语 | 含义 |
|---|---|
| `phone.dat` | 归属地二进制库，约 4.5MB，517,258 条号段，最后更新 2025-02 |
| 号段 / prefix | 手机号**前 7 位**，是唯一查询键，以 `i32` 存储与比较 |
| 记录区 offset | 索引里存的偏移；读记录时实际起点是 `offset - 8`（头部 8 字节的补偿） |
| 卡类型 | `CardType` 枚举，库中存 `u8`，合法值 1–8 |
| 版本号 | 头部前 4 字节，形如 `1701` 表示 17 年 1 月，按 UTF-8 字符串读出 |

## 技术栈（来自 Cargo.toml，勿凭记忆写版本）

- Rust **2024 edition**（需 1.85+）
- `actix-web` ^4.11.0、`serde`/`serde_derive` ^1.0.228、`anyhow` ^1.0.100、`lazy_static` ^1.5.0
- dev: `criterion` 0.5（`html_reports`）

## 四种算法与选择依据

统一实现 `PhoneLookup` + `PhoneStats`，结果必须逐字段一致（`tests/test_suite.rs` 有断言）。
选择只看内存 / 性能权衡的定性结论，**不要往记忆里抄具体 ns 数字**（README 的表已自相矛盾且会过期）：

- `binary_search.rs` — 原始 `records` + `index` 两个 `Vec`，内存最小，通用默认
- `phone_simd.rs` — 与二分同结构，多了 aarch64/x86 预取，批量查询友好
- `phone_bloom.rs` — 多一个布隆位图（约 0.3MB），失败查询快，初始化慢
- `phone_hash.rs` — 展开成 `HashMap`，单次查询最快，内存代价最高（百 MB 级），初始化最慢

**默认导出是 SIMD，不是二分**：`lib.rs` 里 `pub use phone_simd::PhoneDataSimd as PhoneData`。
凡是用 `phone_data::PhoneData` 的地方（`main.rs`、`benches/`、`tests/test_suite.rs`）跑的
都是 SIMD 实现；测试里把 `PhoneData` 变量叫成 `binary_data` 属于历史命名，不要据此判断算法。

## 已知文档与代码的偏差

写文档或排查时以代码为准。分两部分：还没修的，和已经修好、提交前别回退的。

### 仍未修正：`README.md` 的性能数字

`README.md` 里散布着多套互相矛盾的数字，**一律不要引用**，也不要据此判断算法优劣：

- 单查延迟同时存在 `~175ns` / `~105ns` / `~150ns` / `~209ns` / `~212ns` / `~231ns` / `~233ns` /
  `~235ns` / `~260ns` 等说法，出现在特性列表、方案对比表、选型建议、性能总结四处
- 内存数字为 `6.2MB` / `6.5MB` / `156MB`，另有「单查询 106ns」「QPS 可达 970 万」等孤立说法
- 保留的定性结论（哈希最省时间但最耗内存、二分与 SIMD 最小、布隆失败查询快）与代码一致，
  但不要引用任何具体数字

数字不可信的成因之一见 `13-tests-benches.md`：改动前 `benches/algorithm_comparison.rs` 里
名为「二分」的 bench 实际加载的是 SIMD 实现，跑出来的数本身就名不对实。

### 已修正（见 `b952330`、`07e0708`，不要改回去）

- 「二分查找在 `src/lib.rs`」→ `src/binary_search.rs`（`README.md` 与当时的 `CLAUDE.md`）
- 「二分查找是默认实现」→ 「默认 SIMD 优化实现」
- `benches/algorithm_comparison.rs` 的二分项 → 真正的 `binary_search::PhoneData`
- `CLAUDE.md` 已整体删除（见 `e4b56f8`），协作指引改由 `AGENTS.md` + `memory-bank/` 承载；
  排查时不必再找 `CLAUDE.md`，`.dockerignore` 里的残留条目也已清掉

`AGENTS.md` 末尾那段「本文档与 README.md 中若干描述已与代码不符」在性能数字清理完之前
仍然成立，不是过期描述。

## 常用命令

```bash
cargo build --release                              # 生产构建（LTO + strip）
cargo test --test unified_tests -- --nocapture     # 统一测试套件（推荐入口）
cargo bench --bench algorithm_comparison           # 算法对比
cargo run                                          # 起服务，监听 8080
```

所有测试与 bench 都依赖**当前工作目录下存在 `phone.dat`**，没有数据文件时全部失败。

## 分领域规范

| 文件 | 约束范围 |
|---|---|
| `10-data-core.md` | 二进制格式解析、公共类型与 trait |
| `11-lookup-algorithms.md` | 四种查找算法实现 |
| `12-web-api.md` | HTTP 服务与响应格式 |
| `13-tests-benches.md` | 测试与基准测试 |
| `20-build-deploy.md` | 构建配置、Docker、CI |
