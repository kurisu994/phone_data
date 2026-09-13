---
name: 12-web-api
description: actix-web 服务入口、路由与统一 JSON 响应契约
paths:
  - "src/main.rs"
---

# HTTP 服务

单文件 actix-web 应用。**它引用的是默认实现 `phone_data::PhoneData`，即 SIMD 版本**，
换默认算法会静默改变线上服务行为。

## 启动与状态

- `lazy_static!` 里的 `STATE: AppState` 持有全局单例 `PhoneData`，在**首次访问时**才初始化；
  `PhoneData::new().unwrap()` 失败会直接 panic（数据文件缺失 / 损坏即启动即崩，不降级）。
- `HttpServer::workers(200)` + `bind(("0.0.0.0", 8080))`。worker 数远大于 CPU 核数是既有设定，
  进程内存会随之放大（SIMD 版每 worker 一份数据），调整前先算内存账。
- 端口与绑定地址是硬编码的，没有环境变量或配置文件入口。

## 路由

| 方法 | 路径 | 处理函数 | 说明 |
|---|---|---|---|
| GET | `/` | `hello` | 返回固定文本 `Hello world!`，无内容协商 |
| GET | `/query` | `query_phone` | query 参数 `phone`，用 `web::Query<IParams>` |
| GET | `/query2/{phone}` | `query_phone2` | 路径参数，用 `web::Path<String>` |
| POST | `/echo` | `echo` | 原样回显请求体 |
| POST | `/hey` | `manual_hello` | 接收 JSON `{ "province": ... }` 并 `println!`，固定返回 `Hey there!` |

`/echo` 与 `/hey` 是脚手架遗留的演示端点，不参与业务，改动时不必为它们补业务校验。

## 响应契约

所有查询响应统一走 `Message<T> { code, data, success, result }`：

- 成功：`Message::ok(info)` → `code: 0`、`result: "ok"`、`success: true`、`data` 为 `PhoneNoInfo`
- 失败：`Message::err("查询失败")` → `code: -1`、`result: "查询失败"`、`success: false`、`data: null`

要点：

- **失败原因不外泄**：`find` 的 `Err` 被整体丢弃（`Err(_)`），`InvalidLength` / `NotFound` /
  `InvalidOpNo` 对外都是同一句「查询失败」。
- **HTTP 状态码恒为 200**，错误只体现在 body 的 `code` / `success`，不要改成 4xx/5xx，
  否则破坏调用方。
- `Message<T>` 的 `T` 只需 `Serialize`；`PhoneNoInfo` 字段名即 JSON 键
  （`zip_code`、`area_code`、`card_type`），改字段名等于改 API。

## 依赖与风格

- `use serde_derive::{Serialize, Deserialize}`——项目统一用 `serde_derive` 而非
  `serde` 的 derive feature，新增结构体沿用这个 import。
- 入参结构体（`IParams`、`DemoPa`）就地下定义在 `main.rs`，反序列化失败由 actix 返回 400。
