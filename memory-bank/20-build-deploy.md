---
name: 20-build-deploy
description: release 构建配置、Docker 多架构镜像与 CI 发布流程
paths:
  - "Cargo.toml"
  - "Dockerfile"
  - ".dockerignore"
  - ".github/workflows/**"
---

# 构建与部署

## release profile（Cargo.toml）

以**体积**而非速度为目标，四项互相配合，不要单独改动：

```toml
opt-level = 'z'      # 优化体积
lto = true
codegen-units = 1
panic = 'abort'      # 无 unwind，配合下面的 Docker 静态链接
strip = true
```

`panic = 'abort'` 与 `main.rs` 里 `PhoneData::new().unwrap()` 的组合意味着初始化失败
直接杀进程（无栈展开、无兜底），这是有意的。

`[[bench]]` 段显式声明 `harness = false`，对应 `benches/` 下两个 Criterion 入口。

## Dockerfile

两阶段：

- builder `rust:1.91-slim`：装 `musl-tools` + `binutils`，按 `TARGETARCH`/`TARGETVARIANT`
  选目标三元组→ `x86_64-unknown-linux-musl` / `aarch64-unknown-linux-musl` /
  `armv7-unknown-linux-musleabihf`，`rustup target add` 后 `cargo build --release --target`，
  再 `strip` 并拷回 `target/release/phone_data`（供最终阶段 COPY 统一路径）。
  不支持的架构组合直接 `exit 1`，不静默退化。
- runtime `alpine:3.22`：`TZ=Asia/Shanghai`，二进制与 `phone.dat` 都放进
  `/usr/local/bin` 并把 `WORKDIR` 设在那里——**因为代码用相对路径打开 `phone.dat`，
  WORKDIR 就是数据文件的定位方式，改 WORKDIR 会直接启动失败**。
  以 `USER 65532` 非 root 运行，`EXPOSE 8080`，`CMD ["./phone_data"]`。

builder 里那段 `mkdir -p benches && echo 'fn main() {}' > ...` 是**必需的**：
`.dockerignore` 排除了 `benches/`，而 `Cargo.toml` 声明了 `[[bench]]`，没有这两个 stub
文件 cargo 会因找不到 bench 目标而构建失败。改 `.dockerignore` 或 `Cargo.toml` 时要连带看这里。

## .dockerignore

刻意排除：`target/` `.git/` `.idea/` `*.md`、`tests/`、`benches/`、`data/`、
`Cargo.lock`、`Dockerfile`、`.dockerignore`、编辑器临时文件。

两个后果要记住：Docker 构建里没有测试和 bench（所以需要上面的 stub）；
`Cargo.lock` 被排除意味着**镜像内依赖版本由构建时的 registry 决定，可能与本仓库锁定的不一致**。
若要求可复现构建，需要去掉这一行的排除。

`phone.dat` **不在** ignore 列表里，会被 `COPY . .` 带进构建上下文。

## CI（.github/workflows/docker-publish.yml）

- 触发：`workflow_dispatch` 手动，或 push tag `v*`（普通分支 push 不发布）。
- 权限：`contents: read` + `id-token: write`（后者供 cosign keyless 用）。
- 流程：checkout → 装 cosign v2.4.0 → `setup-qemu-action` + `setup-buildx-action`
  （多架构必需）→ 登录 Docker Hub（secrets `DOCKERHUB_USERNAME` / `DOCKERHUB_PASSWORD`）
  → `docker/metadata-action` 生成 tag（branch / pr / semver）→
  `build-push-action` 构建 `linux/amd64,linux/arm64,linux/arm/v7` 并推送
  （PR 时 `push: false`），开启 gha 缓存、`provenance: true`、`sbom: true` →
  `cosign sign --yes` 对 digest 签名。
- 镜像名固定 `kurisu003/phone-data`，改仓库名要同步这里。

本地构建多架构镜像的等价命令见项目 README；单架构直接 `docker build .` 即可。
