## 2026-09-13 记忆银行对齐 CLAUDE.md 删除

- `e4b56f8` 删除了 `CLAUDE.md`，协作指引与「文档/代码偏差」说明迁到 `AGENTS.md` + `memory-bank/`。
- 同步修掉 `00-project.md` 里的两处悬空引用：偏差说明的载体由 `CLAUDE.md` 改为 `AGENTS.md`，
  「已修正」清单补记删除一事，避免以后按旧名去 grep。
- 遗留：`.dockerignore` 第 8 行仍列着 `CLAUDE.md`，无实际影响，本次未动。

## 2026-09-13 清理 .dockerignore 的 CLAUDE.md 条目

- 上一条遗留的 `.dockerignore:8` `CLAUDE.md` 已删除，构建上下文无变化（文件本就不存在）。
- 顺带确认：同文件第 7 行 `*.md` 已覆盖所有 Markdown，`README.md` 那行同样是冗余项，
  但本次未动；以后有人想往这里加 md 文件名时直接省掉即可。
