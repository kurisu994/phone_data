# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于Rust 2024 edition实现的手机号归属地查询服务，使用二分查找算法在phone.dat数据文件中快速查找手机号信息。

### 技术栈
- **Rust Edition**: 2024 (需要Rust 1.85+)
- **Web框架**: actix-web 4.11.0
- **序列化**: serde 1.0.228
- **错误处理**: anyhow 1.0.100
- **静态初始化**: lazy_static 1.5.0

### 核心功能
- 手机号归属地查询（省份、城市、邮编、长途区号）
- 运营商类型识别（移动、联通、电信、广电及其虚拟运营商）
- HTTP API服务，支持GET和POST请求
- 高性能二分查找算法

### 数据文件格式

phone.dat文件结构：
```
| 4 bytes | 版本号
| 4 bytes | 第一个索引的偏移
| offset - 8 | 记录区（格式："<省份>|<城市>|<邮编>|<长途区号>\0"）
| 剩余部分 | 索引区（每条9字节："<手机号前七位><记录区偏移><卡类型>"）
```

## 常用命令

### 构建和运行
```bash
# 构建项目
cargo build

# 运行开发服务器
cargo run

# 生产构建（带优化）
cargo build --release

# Docker构建（多架构）
docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 -t phone_data .
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试（带输出）
cargo test --package phone_data --test demo_test -- --nocapture

# 运行库测试
cargo test --lib

# 运行统一测试套件（推荐，确保算法一致性）
cargo test --test unified_tests -- --nocapture

# 运行集成测试
cargo test --test integration_tests
```

### 性能基准测试
```bash
# 运行所有基准测试
cargo bench

# 运行算法对比基准测试
cargo bench --bench algorithm_comparison

# 运行查找性能基准测试
cargo bench --bench lookup_performance

# 生成HTML报告
cargo bench --bench algorithm_comparison -- --output-format html
```

### API测试
服务启动后默认在8080端口：

```bash
# 查询方式1: GET参数查询
curl 'http://127.0.0.1:8080/query?phone=18086834111'

# 查询方式2: 路径参数查询
curl 'http://127.0.0.1:8080/query2/18086834111'
```

## 代码架构

### 主要模块

#### `src/lib.rs` - 模块导出和默认实现
- 重新导出所有公共类型和算法实现
- 默认导出SIMD优化算法作为主要实现：`PhoneDataSimd as PhoneData`
- 提供统一的trait接口：`PhoneLookup`和`PhoneStats`

#### `src/common.rs` - 公共类型和接口
- `PhoneNoInfo`: 查询结果数据结构
- `ErrorKind`: 统一错误类型枚举
- `CardType`: 运营商类型枚举
- `PhoneLookup` trait: 所有查找算法必须实现的接口
- `PhoneStats` trait: 算法统计信息接口

#### `src/binary_search.rs` - 二分查找算法
- 经典二分查找实现，O(log n)时间复杂度
- 内存占用最小（6.2MB）
- 适合内存受限环境
- 查找时间：~175ns

#### `src/phone_hash.rs` - 哈希查找算法
- HashMap查找实现，O(1)平均时间复杂度
- 极致查询性能（~105ns）
- 内存占用较大（156MB）
- 适合高频查询场景

#### `src/phone_simd.rs` - SIMD优化算法
- 向量化指令优化的二分查找
- 批量查询性能优异
- 跨平台SIMD支持
- 适合数据分析和批量处理场景

#### `src/phone_bloom.rs` - 布隆过滤器算法
- 两阶段查找：布隆预过滤 + 二分查找
- 1%误报率设置
- 失败查询性能优异
- 适合输入质量不高的场景

#### `src/main.rs` - Web服务入口
- 基于actix-web框架的HTTP API服务
- 支持两种查询接口：
  - `/query?phone=<号码>` - GET参数查询
  - `/query2/<号码>` - 路径参数查询
- 全局状态管理：使用lazy_static管理PhoneData实例
- 统一的JSON响应格式：`Message<T>`结构体

### 关键特性

1. **多算法统一接口**：所有算法实现`PhoneLookup`和`PhoneStats` traits，确保接口一致性
2. **phone.dat数据文件解析**：统一的二进制文件格式解析，支持版本号、记录区和索引区
3. **内存优化策略**：不同算法提供不同的内存-性能权衡，从6.2MB到156MB
4. **错误处理体系**：使用anyhow crate进行现代化错误管理，自定义ErrorKind枚举
5. **性能基准测试**：完整的Criterion基准测试套件，支持算法性能对比

### 数据结构

#### 索引结构 (Index)
```rust
struct Index {
    phone_no_prefix: i32,    // 手机号前七位
    records_offset: i32,     // 记录区偏移
    card_type: u8,           // 运营商类型
}
```

#### 查询结果 (PhoneNoInfo)
```rust
pub struct PhoneNoInfo {
    province: String,        // 省份
    city: String,           // 城市
    zip_code: String,       // 邮政编码
    area_code: String,      // 长途区号
    card_type: String,      // 运营商类型
}
```

## 高级架构设计

### 多算法架构
项目实现了四种不同的查找算法，通过统一的`PhoneLookup`和`PhoneStats` traits提供一致接口：
- **二分查找** (`lib.rs`): O(log n)时间，O(1)空间，内存受限环境首选
- **哈希查找** (`phone_hash.rs`): O(1)平均时间，O(n)空间，极致性能首选
- **SIMD优化** (`phone_simd.rs`): 向量化指令，批量查询优化，跨平台支持
- **布隆过滤器** (`phone_bloom.rs`): 两阶段查找，失败查询优化，1%误报率

### 统一接口设计
所有算法实现都遵循相同的trait接口：
```rust
pub trait PhoneLookup {
    fn find(&self, no: &str) -> Result<PhoneNoInfo>;
    fn find_batch(&self, phones: &[&str]) -> Vec<Result<PhoneNoInfo>>;
    fn validate_phone_no(&self, no: &str) -> Result<i32>;
}

pub trait PhoneStats {
    fn total_entries(&self) -> usize;
    fn version(&self) -> &str;
    fn memory_usage_bytes(&self) -> usize;
}
```

### 性能基准测试架构
项目包含完整的Criterion基准测试套件 (`benches/`):
- **算法对比测试** (`algorithm_comparison.rs`): 单次查询、批量查询、初始化时间对比
- **查找性能测试** (`lookup_performance.rs`): 不同算法的详细性能分析
- **跨平台测试**: 支持x86_64、aarch64等多架构
- **HTML报告生成**: 支持生成详细的性能分析报告

### Docker多架构部署
- **多阶段构建**: builder + runtime优化镜像大小
- **多架构支持**: linux/amd64、linux/arm64、linux/arm/v7
- **CI/CD流程**: GitHub Actions自动化构建和发布
- **交叉编译**: 使用musl工具链进行静态链接优化

## 开发注意事项

- **数据文件要求**：phone.dat文件必须存在于项目根目录，文件大小约4.5MB
- **服务配置**：默认配置为200个worker线程，监听8080端口
- **数据规模**：数据文件最后更新于2025年02月，包含517,258条手机号段记录
- **号码格式**：支持7-11位手机号码，查询基于手机号前七位进行匹配
- **Rust版本**：项目使用Rust 2024 edition，需要Rust 1.85+
- **默认实现**：默认导出SIMD优化算法作为主要实现，提供平衡的性能和内存使用
- **测试策略**：使用统一测试套件确保所有算法的一致性，推荐运行`cargo test --test unified_tests`
- **性能优化**：生产构建使用LTO、代码单元合并等优化技术，显著减小二进制文件大小

### 库使用模式

```rust
// 使用默认SIMD优化实现
use phone_data::{PhoneData, PhoneLookup};

let phone_data = PhoneData::new()?;
let result = phone_data.find("18086834111")?;

// 使用其他算法实现
use phone_data::{PhoneDataHash, PhoneDataBloom, PhoneDataSimd};

// 哈希查找 - 最快性能
let hash_data = PhoneDataHash::new()?;

// 布隆过滤器 - 适合大量失败查询
let bloom_data = PhoneDataBloom::new()?;
```