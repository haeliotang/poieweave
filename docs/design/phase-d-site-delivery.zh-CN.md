# Phase D 设计：站点交付（D1 元数据交付）

- 状态：设计草案（Phase D 门禁重跑后方可开发）
- 日期：2026-11-05
- 范围：D1 仅 JSON 交付元数据：manifest 检验、原子导入、状态、回滚、诊断导出；D2 才做 release channel/签名/安装器
- 上游：A/B/C1；D1 不依赖审批链
- 下游：无二进制/模型导入

## 1. 背景

Poieweave 需要把“站点交付”变成可验证、可回滚的元数据操作，而不是把安装器、更新器或二进制塞进应用。D1 只处理站点 manifest/state/diagnostics 的 JSON 导入、状态与回滚；不导入 app/model 二进制，不做联网更新/遥测，不修改 HostPolicy。

## 2. 目标

1. `SiteManifest` JSON 可检验、可哈希、可拒绝未知字段。
2. 原子导入：先写事务，再切 state；失败不留下半状态。
3. `DeliveryState` 保存当前/上一份完整 manifest 与摘要；rollback 只换 state。
4. `delivery_diagnostics_export` 生成固定、可复现、脱敏的 JSON 文件。
5. 本地、离线、无网络；显式错误码；审计。
6. D1 不做 channel 选择/签名/安装器/策略更新；D2 另立设计并重跑 D 门禁。

## 3. 非目标

- 不导入/执行 app、model、脚本、二进制、HTML、zip；只接受 UTF-8 JSON。
- 不做联网下载、updater、telemetry、自动更新、安装器、DMG/MSI/portable exe。
- 不做 release channel 自动选择、签名验证、阈值信任、策略更新。
- 不做审批 token/run/call（D2 若需要再设计）。
- 不声称“部署成功”；只声称“元数据已导入/回滚”。

## 4. 数据模型

### 4.1 SiteManifest

```text
SiteManifest {
  schema: "poie.site-manifest/0.1",
  schemaVersion, siteId, siteName,
  target: string,                 // 单目标标识；D1 不做多 OS artifact
  channel: "stable"|"canary"|"recall",
  releasedAt: string,             // ISO-8601 UTC
  modelSha256: string|null,       // 仅引用，不导入
  evidenceSha256: string|null,
  policy: { profile: string },    // D1 只读 metadata，不应用
  notes: string,
  manifestSha256: string          // computed
}
```

- `manifestSha256 = sha256("poie.site-manifest/0.1\n" + canonical_json(manifest_without_manifestSha256))`；拒绝未知字段，避免“宽容解析导致 self-hash 不一致”。
- `target` 单值；D1 不支持多 OS artifact。
- `policy` 仅展示，不写入 HostPolicy、不改变 channel。
- 文件上限 256 KiB；字符串长度/数组上限固定。

### 4.2 Bundle envelope

```text
SiteBundle {
  schema: "poie.site-bundle/0.1",
  manifest: SiteManifest
}
```

- 只接受 bundle envelope；`manifest` 为唯一 payload。
- 导入前重新计算 `manifestSha256`，与请求 `expectedManifestSha256` 比较；不一致 → `stale_manifest`（inspect→import TOCTOU 防护）。
- 不接受额外字段/尾随内容/非 UTF-8/超限。

### 4.3 DeliveryState

```text
DeliveryState {
  schema: "poie.delivery-state/0.1",
  current: ManifestSummary|null,
  previous: ManifestSummary|null,
  recalled: bool,
  updatedAt: i64
}
ManifestSummary {
  siteId, siteName, target, channel, releasedAt,
  manifestSha256, importedAt, importedBy, source:"offline"
}
```

- 完整 manifest 存在 `delivery_manifests(manifest_sha256 PK, manifest_json, imported_at)`；state 只存摘要。
- `current/previous` 是摘要；rollback 把 previous 设为 current，current 清空/设回 previous 的前一份；保留 manifest 行。
- `recalled=true`：UI/读取显示 recall 且禁止把该 manifest 当 current 使用；仍允许 rollback 到 previous。D1 不提供“解除 recall”。

## 5. 命令

| 命令 | 输入 | 输出 | 权限 |
| --- | --- | --- | --- |
| `delivery_manifest_inspect` | path | SiteManifest + computed sha | 只读；登录 |
| `delivery_import` | path, expectedManifestSha256 | DeliveryState | 写；登录；无审批 |
| `delivery_state` | — | DeliveryState | 只读；登录 |
| `delivery_rollback` | — | DeliveryState | 写；登录；无审批 |
| `delivery_diagnostics_export` | outputDir | DiagnosticsSummary | 写；登录 |

- 命令通过 `auth::require` + 统一 `{code,message,details}` 错误包络；D1 不引入 Window/HostPolicy/审批。
- 错误码：`invalid_parameter`、`unsupported_schema`、`stale_manifest`、`manifest_invalid`、`delivery_conflict`、`rollback_unavailable`、`unauthorized`、`scope_denied`、`io_error`。
- `import` 的 `expectedManifestSha256` 必填；空/不匹配拒绝。
- `rollback` 无 previous → `rollback_unavailable`。

## 6. 持久化与原子性

SQLite `user_version=4`：

```text
delivery_manifests(manifest_sha256 PK, schema_version, manifest_json, imported_at)
delivery_state(id PK CHECK(id=1), state_json, updated_at)
```

导入事务：

1. `BEGIN IMMEDIATE`；
2. 重算/校验 manifest；
3. `INSERT OR IGNORE` manifest（同 sha 已存在则复用；同 site/target 不同 sha 允许，作为历史）；
4. 读 state；`previous=current`；`current=新摘要`；
5. 写 state；`COMMIT`。
- 失败 `ROLLBACK`；不改变 state。
- 重复导入同一 sha：幂等，不重复追加历史；返回相同 state。
- `manifest_json` 原样保存，用于回滚/诊断；不重新解析后重写。
- `user_version` 从 3→4；创建表并设 4；幂等；不修改 whatif/agent 表。

## 7. 诊断

`delivery_diagnostics_export(outputDir)` 写 `delivery-diagnostics.json`（固定名，覆盖需确认）：

```text
DiagnosticsSummary {
  schema: "poie.delivery-diagnostics/0.1",
  generatedAt: i64,               // caller-supplied now? 固定由 host 注入
  appVersion, os, arch,
  delivery: DeliveryState,
  checks: [{ id, status:"ok|failed|unknown", detail }],
  errorCodes: string[],           // 固定白名单，不含自由文本
  logTail: string[]               // 最多 50 行，redacted
}
```

- 不包含 secrets、API key、prompt、模型输出、绝对路径、用户名、UUID、站点 note 原文。
- `siteId`/`siteName` 只保留长度/哈希前缀，默认不导出原文。
- 大小上限 128 KiB；超出截断并 `truncated=true`。
- `generatedAt` 由 host 传入 `now_ms`；同一 state/输入在测试注入时钟下可复现；不承诺跨时刻字节相同。
- 不发送网络/遥测。

## 8. UI

- Settings “站点交付”区：
  - 选择本地 JSON（文件选择器）；显示 inspect 的 site/target/channel/sha；
  - Import 需再次确认 sha；成功后显示 current/previous；
  - Rollback 按钮；
  - Export diagnostics 到用户选择目录。
- 所有状态只来自 host；浏览器预览显示 host-only 占位。
- Recall 状态用文字 + 颜色，不能只靠颜色。
- 不显示“部署成功/在线”文案；只显示“元数据已导入”。

## 9. 依赖与过程门禁

- 新 crate `poie-delivery`（纯 Rust）：manifest/schema/digest/state/compare，无 Tauri/网络/fs。
- host `delivery.rs`：文件读取（UTF-8/大小）、SQLite 事务、诊断。
- 依赖：poie-digest、serde/serde_json；不依赖 poie-exec。
- 根 members、桌面 path dep、两处 Cargo.lock、test-matrix/status-snapshot、desktop workflow `crates/**` 路径、ADR、cargo-deny。
- D1 不改 release workflow/安装器；D2 另立设计。
- 实施顺序：D0 manifest schema/digest → crate → DB v4 → host commands → UI → tests/docs。

## 10. 测试与验收

- manifest 自哈希稳定；未知字段/超大/非 UTF-8/尾随内容拒绝。
- inspect→import sha 变化 → `stale_manifest`；导入不改变 HostPolicy/live。
- 导入事务失败回滚；重复导入幂等；rollback 交换 current/previous；无 previous 拒绝。
- recall 显示且不当作 current；D1 不提供 un-recall。
- 诊断无 secret/prompt/绝对路径/UUID；大小上限；固定错误码。
- UI host-only 占位、导入确认、rollback；无“部署成功”文案。
- 验收：离线导入一份 JSON、回滚、导出诊断；不联网、不导入二进制、不改策略。

## 11. D2（延后，需重跑门禁）

- release channel/签名/信任阈值、安装器/portable 发布、policy tighten-only、审批 token 通用化、updater/telemetry 明确禁止或设计、多 OS artifact。
- D2 必须重新校验 release workflow/tauri targets，不与 D1 混做。

## 12. 参考

- [PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
