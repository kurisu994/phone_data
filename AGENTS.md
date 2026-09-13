# Repository Guidelines

## 项目结构与模块组织

本项目使用 Rust 2024 edition，提供手机号归属地查询库和 Actix Web 服务。

- `src/main.rs`：HTTP 路由、服务启动与 JSON 响应；`src/lib.rs`：公共模块与类型导出。
- `src/common.rs`：公共类型、解析工具及 `PhoneLookup`、`PhoneStats` 接口。
- `src/binary_search.rs`、`src/phone_{hash,simd,bloom}.rs`：四种查询实现。根导出的 `PhoneData` 是 SIMD 别名；真正的二分实现是 `phone_data::binary_search::PhoneData`。
- `tests/`：集成测试与统一测试套件；`benches/`：Criterion 性能基准。
- `phone.dat`：运行时数据文件；`memory-bank/`：项目约定；`Dockerfile` 和 `.github/workflows/`：镜像构建与发布。

## 构建、测试与本地开发

使用支持 Rust 2024 edition 的工具链。在仓库根目录执行命令，并保留 `phone.dat`；查询实现按当前工作目录读取该文件。

| 命令 | 用途 |
| --- | --- |
| `cargo check --all-targets` | 优先检查库、服务、测试与基准目标能否编译 |
| `cargo build --release` | 生成启用 LTO、符号剥离等优化的生产构建 |
| `cargo run` | 启动服务，默认监听 `0.0.0.0:8080`，使用 200 个 worker |
| `cargo test --test unified_tests -- --nocapture` | 执行统一测试套件并显示输出 |
| `cargo test --test integration_tests` | 检查兼容性、批量查询和失败路径 |
| `cargo test` | 按需执行完整测试 |
| `cargo bench --bench algorithm_comparison` | 对比四种算法；`lookup_performance` 目标检查默认实现 |

启动后可用 `curl 'http://127.0.0.1:8080/query?phone=18086834111'` 验证查询接口。

## 编码风格与命名

使用四空格缩进；模块、函数与变量使用 `snake_case`，类型和 trait 使用 `PascalCase`，常量使用 `SCREAMING_SNAKE_CASE`。公共类型和函数添加中文文档注释，复杂逻辑说明关键约束。沿用 `anyhow::Result` 与 `ErrorKind`，复用公共解析逻辑。

仓库没有专用 rustfmt 或 Clippy 配置；可用 `cargo fmt --all -- --check` 和 `cargo clippy --all-targets` 辅助检查，保持修改聚焦，避免顺带格式化无关文件。

## 测试规范

使用 Rust 内置 `#[test]`，测试函数遵循 `test_<behavior>` 命名。优先扩展 `tests/test_suite.rs` 或集成测试，并通过 `unified_tests` 运行共享套件；单独运行 `test_suite` 不执行其中的用例。

算法变更应覆盖有效的 7–11 位号码、无效长度、不存在号段、批量查询与结果一致性。测试真正的二分实现时显式导入其模块，勿依赖历史变量名 `binary_data`。沿用记忆文档的断言约定，避免写死随数据更新变化的省市。当前没有覆盖率百分比门槛；性能变更按需运行 Criterion，并在评审中说明环境与结果。

## 提交与 Pull Request

Git 历史采用可选表情加类型、范围及中文主题，例如 `🐛 fix(bench): 修正二分算法基准入口`。提交格式为 `[可选表情] 类型(可选范围): 动词开头的主题`，主题不超过 50 字；不添加自动生成或共同作者署名。

PR 应说明问题、行为变化、影响范围及验证命令与结果，关联已有 issue；接口变更附请求与响应示例，性能变更附可复现对比。提交前优先编译检查，再根据风险运行针对性测试，并注明未验证事项。

## 项目记忆（memory-bank）

开工先读 [`memory-bank/00-project.md`](memory-bank/00-project.md)，协作约定与索引见
[`memory-bank/README.md`](memory-bank/README.md)。分领域规范按代码路径自动注入，
四层结构（共识层 / `active/` / `journal/` / `archive/`）与写入门槛见该 README，
**没有有效增量时不修改任何记忆文件**。

注意：本文档与 `README.md` 中若干描述已与代码不符（例如二分查找的实际位置、默认实现、
性能数字），`memory-bank/00-project.md` 的「已知文档与代码的偏差」一节列了差异，以代码为准。