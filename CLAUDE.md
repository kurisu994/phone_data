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

# 生产构建
cargo build --release
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试（带输出）
cargo test --package phone_data --test demo_test phone_test -- --nocapture

# 运行库测试
cargo test --lib
```

### API测试
服务启动后默认在8080端口：

```bash
# 查询方式1: GET请求
curl 'http://127.0.0.1:8080/query?phone=18086834111'

# 查询方式2: 路径参数
curl 'http://127.0.0.1:8080/query2/18086834111'
```

## 代码架构

### 主要模块

#### `src/lib.rs` - 核心库
- `PhoneData`: 主要数据结构，封装phone.dat文件解析和查询逻辑
- `PhoneNoInfo`: 查询结果的数据结构
- `CardType`: 运营商类型枚举，支持三大运营商和虚拟运营商
- 错误处理：使用anyhow crate进行现代化错误管理，自定义ErrorKind枚举实现Display和Error trait

#### `src/main.rs` - Web服务
- 使用actix-web框架提供HTTP API服务
- 支持两种查询接口：
  - `/query?phone=<号码>` - GET参数查询
  - `/query2/<号码>` - 路径参数查询
- 全局状态管理：使用lazy_static管理PhoneData实例
- 统一的响应格式：`Message<T>`结构体

### 关键特性

1. **二分查找算法**：在lib.rs:124-158的`find`方法中实现，O(log n)时间复杂度
2. **数据文件解析**：PhoneData::new()方法负责解析phone.dat二进制格式
3. **内存加载**：整个数据文件加载到内存中，提供快速查询性能
4. **错误处理**：自定义ErrorKind枚举处理各种异常情况

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
- **算法对比测试**: 单次查询、批量查询、初始化时间对比
- **性能分析**: 不同内存访问模式下的表现
- **跨平台测试**: 支持x86_64、aarch64等多架构

### Docker多架构部署
- **多阶段构建**: builder + runtime优化镜像大小
- **多架构支持**: linux/amd64、linux/arm64、linux/arm/v7
- **CI/CD流程**: GitHub Actions自动化构建和发布

## 开发注意事项

- phone.dat文件必须存在于项目根目录
- 服务默认配置为200个worker线程
- 数据文件更新时间：2025年02月，包含517258条手机号段记录
- 支持的手机号码长度：7-11位
- 查询基于手机号前七位进行匹配
- 项目使用Rust 2024 edition，需要Rust 1.85+
- 默认导出SIMD优化算法作为主要实现