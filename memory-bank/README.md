# Memory Bank

本项目的长期记忆：约定怎么写代码、当前在做什么、以及为什么当初那么决定。
**开局先读 `00-project.md`**，它给出项目定位、领域语言和已知的文档偏差。

## 索引

| 文件 | 内容 | 何时读 |
|---|---|---|
| `00-project.md` | 项目定位、领域语言、算法矩阵、文档与代码的偏差 | 每次开工 |
| `10-data-core.md` | phone.dat 二进制格式、公共类型层、trait 契约 | 动数据解析或公共类型前 |
| `11-lookup-algorithms.md` | 四种查找算法的实现约定与同步点 | 动任一算法实现前 |
| `12-web-api.md` | actix-web 服务、路由与 JSON 契约 | 动 HTTP 层前 |
| `13-tests-benches.md` | 测试入口、一致性断言、Criterion | 写测试 / 跑分前 |
| `20-build-deploy.md` | release 配置、Docker 多架构、CI | 改构建或发布前 |

## 路径映射表

下表由各文件 frontmatter 的 `paths` 实际抽取生成，**不要手写维护**。碰下表左列路径时，
对应规范会自动注入（Claude Code via `.claude/hooks/inject-memory-bank.py`）。

| 文件 | 说明 | 自动注入路径 |
|---|---|---|
| `10-data-core.md` | phone.dat 二进制格式、公共类型层与统一 trait 契约 | `src/common.rs` `src/lib.rs` |
| `11-lookup-algorithms.md` | 四种查找算法的实现约定、重复代码同步点与新增算法流程 | `src/binary_search.rs` `src/phone_hash.rs` `src/phone_simd.rs` `src/phone_bloom.rs` |
| `12-web-api.md` | actix-web 服务入口、路由与统一 JSON 响应契约 | `src/main.rs` |
| `13-tests-benches.md` | 测试入口、算法一致性断言与 Criterion 基准测试约定 | `tests/**` `benches/**` |
| `20-build-deploy.md` | release 构建配置、Docker 多架构镜像与 CI 发布流程 | `Cargo.toml` `Dockerfile` `.dockerignore` `.github/workflows/**` |

`00-project.md` 不带 `paths`（开局必读，按需手动打开）。

**Pi / Grok / Cursor 等平台不支持路径触发注入**，在这些环境里请按上表人工查阅；
不要为了「补上」而从别的平台拷 hook 文件过来。

## 写入门槛

- 仅记录有助于后续工作的实质进展、重要决策、可复用结论和未解决阻塞；判断依据是
  是否产生值得跨会话保留的新信息。
- 纯查询、例行检查、无变化的状态确认，以及没有产生新结论的重复验证，**不写入**。
  例如：检查远端后确认无更新、查看工作区状态、重复确认已有配置、跑一遍已有测试确认还绿。
- 未达到门槛时，不修改任何记忆文件；不得为了完成会话收尾而追加流水。
- 达到门槛时，按内容归属更新必要层，避免重复抄写已记录的内容。

## 四层的写入约定

| 层 | 内容 | 频率 | 归属 | 写法 | git 行为 |
|---|---|---|---|---|---|
| `00-project.md`、`1X-*.md`、`20-*.md` | 项目共识、编码约定 | 极低 | 团队 | **改写**，走 PR review | 冲突要人工解决，这是对的 |
| `active/<branch>.md` | 本次做什么、验收标准 | 任务内高频 | 单分支 | 随便改 | 一分支一人动，天然不冲突 |
| `journal/<dev>.md` | 有效进展、踩过的坑 | 有有效增量时 | 个人 | **只追加，禁改历史**，每段结尾留空行 | `merge=union` 自动合并 |
| `archive/YYYY-MM/<slug>.md` | 已完成任务的决策记录 | 一次性 | 团队 | 写完不再动 | 新增文件，不冲突 |

### 三条红线

1. **journal 只能 append**，且每段结尾留一个空行。`merge=union` 是逐行取并集，
   不留空行会让两人的条目黏成一坨；改历史条目直接破坏 union 的前提。
2. **不要放自动生成的汇总表**（「当前活跃任务一览」这类）。派生数据必冲突且冲突无信息量，
   源数据在 `active/` 和 `journal/` 里，随时能重建。
3. **共识层改动走 PR**，冲突了不要 `--ours` 硬合——那说明两人对约定的理解分叉了。

### 体量预算

- 单份规范 ≤ 9,000 字符（硬上限 9,400，超出会被 hook 截断，后半部分永远注入不到）
- 单次注入总量 ≤ 9,500 字符（多出的降级为索引行）
- 用 `wc -m` 数字符，不是行数

超限就按领域拆分，并给每份独立的、更精准的 `paths`。

## 维护

```bash
# 体量核对
wc -m memory-bank/*.md

# 模拟 hook 输入，验证 paths 命中
R=$(pwd); printf '{"tool_name":"Read","tool_input":{"file_path":"%s/src/main.rs"},"cwd":"%s","session_id":"t","hook_event_name":"PostToolUse"}' "$R" "$R" \
  | python3 .claude/hooks/inject-memory-bank.py
```

新增规范文件：写 `paths` frontmatter、补上面映射表、`wc -m` 核对、模拟注入实测即可，
hook 会自动发现新文件，无需改动 `.claude/settings.json`。
