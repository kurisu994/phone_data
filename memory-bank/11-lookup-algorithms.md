---
name: 11-lookup-algorithms
description: 四种查找算法的实现约定、重复代码同步点与新增算法流程
paths:
  - "src/binary_search.rs"
  - "src/phone_hash.rs"
  - "src/phone_simd.rs"
  - "src/phone_bloom.rs"
---

# 查找算法实现

四个文件是同一件事的四种做法，因此**改动一处就要横向检查另外三处**。共同骨架：

1. 结构体持 `version: String`；
2. `new()` 打开**相对路径** `phone.dat`（无配置项，cwd 决定数据来源）；
3. 解析头部 8 字节 → 记录区 → 索引区（或直接展开成 `HashMap`）；
4. `find` 先校验长度，取前 7 位转 `i32`，再查，最后经 `utils::build_phone_info`
   或 `utils::parse_record_data` + `CardType::from_u8` 组装 `PhoneNoInfo`。

不满足这些共同点的实现会让 `tests/` 的一致性断言失效，视为 bug。

## 数据加载约定

- 读文件、读头部、读记录区、9 字节循环读索引，四份实现目前是逐字复制的。
  改解析逻辑时**四处同改**——没有抽公共 loader，不要以为改 `common.rs` 就能生效。
- `phone_hash.rs` 与 `phone_bloom.rs` 把容量 / 期望条数**硬编码为 `517258`**
  （`HashMap::with_capacity(517258)`、`BloomFilter::new(517258, 0.01)`）。
  数据文件条数变化时这两个常量不会自动跟随，属于已知耦合点。
- 布隆过滤器误报率固定 1%，`BloomFilter` 的位图按 `u64` 分块，哈希函数用
  `DefaultHasher` 对 `(item, seed)` 做 `hash_count` 轮。

## 二分查找的三处副本

`binary_search.rs::PhoneData`、`phone_simd.rs::simd_binary_search`、
`phone_bloom.rs::binary_search` 是同一段二分逻辑的三份实现，写法一致：
`while left < right`、`mid = left + ((right - left) >> 1)`、`unsafe get_unchecked`。

`simd_binary_search` 额外暴露 `prefetch_binary_search` / `prefetch_index`，按
`target_arch` 分派 `_mm_prefetch`（x86/x86_64，`_MM_HINT_T0`）与 `prfm pldl1keep`
（aarch64，内联汇编）。**其他架构下 `prefetch_index` 是空函数**，这是有意的：
无预取时退化为普通二分，不要加 `cfg` 分支外的 fallback 报错。

## unsafe 的前提

`get_unchecked(mid)` 只在 `left <= mid < right <= index.len()` 时安全。改动循环边界、
提前 `return` 或把 `right` 初值换成 `len() - 1` 都会让这个前提失效。改二分时要么保持
不变量，要么换回 `self.index[mid]`。

## 重复的固有方法 vs trait 方法

`PhoneDataHash`、`PhoneDataSimd`、`PhoneDataBloom` 都**同时**有
`impl X { pub fn find(...) }` 和 `impl PhoneLookup for X { fn find(...) }`，两份实现体
基本相同（`phone_hash.rs` 是完全复制，`phone_bloom.rs` 的固有版多一次布隆预检）。

- 调用点解析到哪个取决于书写方式：`PhoneDataHash::find(&x, no)` 走固有方法，
  `PhoneLookup::find` 泛型/`impl Trait` 或 `use PhoneLookup` 后 `x.find(no)` 走 trait 方法。
- **改查找逻辑时必须两个都改**，否则测试（经 trait）和直接调用（经固有方法）会分叉。
  合并它们是待办，不是已完成的事实。
- `find_batch` 同理：`phone_simd.rs` / `phone_bloom.rs` 有优化版固有方法，
  其余场景落到 `PhoneLookup` 的默认逐条实现。

## 索引与内存统计

`PhoneStats::memory_usage_bytes` 的算法各不相同，是刻意的（体现内存权衡）：

- `binary_search` / `phone_simd`：`records.len() + index.len() * size_of::<Index>()`
- `phone_bloom`：上一行再加位图字节数
- `phone_hash`：`size_of::<Self>() + capacity * size_of::<(i32, PhoneRecord)>() + len * size_of::<PhoneRecord>()`

`tests/test_suite.rs` 断言「哈希 > 二分」「布隆 > 二分」，调整统计口径时必须保证这两个不等式仍成立。

## 新增一种算法

1. `src/` 下新建文件，**不要**重新定义 `Index` / `ParsedRecord`，从 `common` 引入；
2. 实现 `PhoneLookup` + `PhoneStats`；
3. 在 `lib.rs` 加 `pub mod` 与 `pub use`；
4. 在 `tests/test_suite.rs` 加入一致性断言与统计断言；
5. 需要跑分则加到 `benches/`（注意 `benches` 在 Docker 构建中被 stub 掉，见 `20-build-deploy.md`）。
