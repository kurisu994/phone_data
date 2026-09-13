---
name: 13-tests-benches
description: 测试入口、算法一致性断言与 Criterion 基准测试约定
paths:
  - "tests/**"
  - "benches/**"
---

# 测试与基准

## 前置条件

所有测试、bench 都在运行时打开**当前工作目录**下的 `phone.dat`。
没有数据文件（CI 里常见）时不是「跳过」而是 panic，`cargo test` 会整体失败。
因此这两个目录的测试不能在无数据环境跑，也不要用它做纯逻辑验证。

## 测试入口与组织结构

| 文件 | 角色 |
|---|---|
| `tests/unified_tests.rs` | 唯一测试函数 `run_unified_test_suite()`，内部 `mod test_suite;` + 调 `run_all_tests()` |
| `tests/test_suite.rs` | 真正的测试主体，6 组 `pub fn`，由上面那个测试串起来 |
| `tests/integration_tests.rs` | 独立的 `#[cfg(test)] mod`，覆盖兼容性与失败路径一致性 |
| `tests/demo_test.rs` | `include!("../src/lib.rs")` 的最小冒烟测试 |

`test_suite.rs` 用的是 `pub fn` 而非 `#[test]`，所以 `cargo test --test test_suite` 本身
不跑用例，**必须从 `unified_tests` 进入**：

```bash
cargo test --test unified_tests -- --nocapture
```

`demo_test.rs` 通过 `include!` 把 `lib.rs` 拉进测试 crate，等于再编译一份库代码，
容易出现「lib 改了但 include 的副本没同步」的错觉。任何情况下优先用
`use phone_data::...` 的集成测试，不要扩展这个 include 写法。

## 一致性断言（改算法的护栏）

`test_algorithm_consistency` 对同一手机号跑四种实现，逐项断言 `province` / `city` /
`card_type` 相等；`test_statistics` 断言 `total_entries` 与 `version` 四者一致，
且内存统计满足「哈希 > 二分」「布隆 > 二分」。

新增或修改算法后这两组断言是唯一能发现语义漂移的地方，不要为了让它们通过而放宽断言。

⚠️ 命名陷阱：`test_suite.rs` 与 `integration_tests.rs` 里的 `PhoneData::new()` 被命名为
`binary_data`，但 `PhoneData` 是 **SIMD** 的别名（见 `10-data-core.md`）。所谓「二分 vs 哈希」
的对比实际是「SIMD vs 哈希」。想真正测 `binary_search.rs` 必须显式
`use phone_data::binary_search::PhoneData`。

## 用例约定

- 固定命中号：`18086834111`（四川成都，电信）、`13800138000`（北京，移动）；
  其它常用样本见各文件的 `test_phones` 列表。
- 期望值只断言「`card_type` 包含 移动/电信」这类子串，**不写死省市**——数据文件会更新，
  写死会产生与代码无关的失败。
- 无效输入集合：超长（12 位）、过短（1 位、6 位、空串）、不存在号段（`99999999999` 等）。
- 7 位号码合法且应能命中（取的就是前 7 位），不要在测试里把它当非法输入。
- 断言失败信息保留中文描述，便于定位。

## 基准测试

两个 bench 都声明 `harness = false` 并用 `criterion_group!` / `criterion_main!`，
新增 bench 目标必须在 `Cargo.toml` 里补 `[[bench]]` 段，否则不生效。

- `benches/algorithm_comparison.rs` — 四种算法的单次 / 批量 / 初始化对比
- `benches/lookup_performance.rs` — 固定号码的查找与号码前缀解析开销；有意只跑默认实现（SIMD）

⚠️ 别名陷阱（与上面 `binary_data` 那个同源）：`phone_data::PhoneData` 是 **SIMD**，给「二分」
跑分必须显式 `use phone_data::binary_search::PhoneData as BinaryPhoneData`。
`algorithm_comparison.rs` 原先的 `load_binary_search_data() -> PhoneData` 实际加载的是 SIMD
（已在 `b952330` 修掉，改为真正的二分实现）；改回来就会让「二分」与「SIMD」两组数字
同名不同实，无法解释结果差异。

**不要**把跑出的数字写进 README 或记忆文件：README 里现存的那几张性能表已经互相矛盾，
再抄一份只会继续腐烂。

`criterion` 属 `[dev-dependencies]`，feature 为 `html_reports`；HTML 报告用
`-- --output-format html` 生成。
