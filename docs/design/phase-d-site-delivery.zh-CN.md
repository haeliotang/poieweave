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
3. `DeliveryState` 只保存当前/上一份摘要；完整 manifest 存在 `delivery_manifests`，rollback 只换 state。
4. `delivery_diagnostics_export` 生成固定、可复现、脱敏的 JSON 文件。
5. 本地、离线、无网络；显式错误码；审计事件 `delivery.*`。
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

- `manifestSha256 = sha256("poie.manifest/0.1\n" + canonical_json(manifest_without_manifestSha256))`（digest domain 使用 DIGEST 的 `poie.manifest/0.1`；wire schema 仍是 `poie.site-manifest/0.1`）。
- 必须先按原始字节解析为 JSON 并拒绝未知字段，再做 canonicalization；D1 的 manifest **不接受**未知 minor 字段（PLATFORM §2 的 general tolerance 不适用于自哈希 payload）。
- 上限：bundle ≤256 KiB、manifest ≤128 KiB、单字符串 ≤1 KiB、数组 ≤64、嵌套 ≤8；超限 `invalid_parameter`。
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
  schemaVersion: "0.1",
  current: ManifestSummary|null,
  previous: ManifestSummary|null,
  recalled: bool,
  updatedAt: i64
}
ManifestSummary {
  siteId: string, siteName: string, target: string, channel: string, releasedAt: string,
  manifestSha256: string, importedAt: i64, importedBy: string, source: "offline"
}
```

- 完整 manifest 存在 `delivery_manifests(manifest_sha256 PK, manifest_json, imported_at)`；state **只存摘要**，不重复存完整 manifest。
- `current/previous` 都是 `Option<ManifestSummary>`；rollback 执行 **swap**：`(current, previous) = (previous, current)`，因此可连续回滚一次退回再回滚；没有 previous → `rollback_unavailable`。
- `delivery_clear`（登录、写）把 current/previous 都置 null；用于 `delivery_conflict` 后换站点。D1 无 un-recall，clear 后重新 import。
- `recalled = current.channel == "recall"`；导入 recall manifest 只设置 current/recalled，不改变 policy/HostPolicy；rollback 恢复 previous 后按 previous.channel 重算 recalled。D1 不提供 un-recall。
- `DeliveryState.schemaVersion` 必填；`ManifestSummary` 由 §4.1 摘要字段组成本身，不含完整 manifest。

## 5. 命令

| 命令 | 输入 | 输出 | 权限 |
| --- | --- | --- | --- |
| `delivery_manifest_inspect` | path | SiteManifest + computed sha | 只读；登录 |
| `delivery_import` | path, expectedManifestSha256 | DeliveryState | 写；登录；无审批 |
| `delivery_state` | — | DeliveryState | 只读；登录 |
| `delivery_rollback` | — | DeliveryState | 写；登录；无审批 |
| `delivery_clear` | — | DeliveryState | 写；登录；无审批 |
| `delivery_diagnostics_export` | outputDir, overwrite? | DiagnosticsSummary | 写；登录 |

- 命令通过 `auth::require` + 统一 `{code,message,details}` 错误包络；D1 的 import/rollback 是 **登录会话内的本地元数据写**，不需要 ApprovalToken/HostPolicy/Window（这与 PLATFORM D1 行一致；D2 若改为远程/策略变更再引入审批）。
- 错误码：`invalid_parameter`、`unsupported_schema`、`stale_manifest`、`manifest_invalid`、`delivery_conflict`、`rollback_unavailable`、`unauthorized`、`scope_denied`、`io_error`。
- `import` 的 `expectedManifestSha256` 必填；空/不匹配拒绝。
- `rollback` 无 previous → `rollback_unavailable`。
- 审计事件：`delivery.inspect`、`delivery.import`、`delivery.rollback`、`delivery.diagnostics`，写入现有 audit（不含 manifest 原文/路径）。

## 6. 持久化与原子性

SQLite `user_version=4`：

```text
delivery_manifests(manifest_sha256 PK, schema_version, manifest_json, imported_at)
delivery_state(id PK CHECK(id=1), state_json, updated_at)
```

导入事务：

1. `BEGIN IMMEDIATE`；
2. 重算/校验 manifest；
3. 读 state；若 `current.manifestSha256 == 新 sha` → 直接返回**未修改**的 state（幂等 no-op）；
4. `INSERT OR IGNORE` manifest（同 sha 已存在则复用；同 site/target 不同 sha 允许，作为历史）；
5. `previous=current`；`current=新摘要`；`recalled = 新 channel == "recall"`；
6. 写 state；`COMMIT`。
- 失败 `ROLLBACK`；不改变 state。
- 重复导入同一 sha：若已是 current，**不改变 previous/state**；若只是历史存在，则提升为 current、previous=旧 current。
- `delivery_conflict`：state 已存在且新 manifest 的 siteId 与 current 不同（D1 单站点；先 `delivery_clear` 再 import，D2 再设计多站点）。
- 不同 target 但同 site：允许作为历史/current，UI 显示 target；不自动删除。
- `manifest_json` 原样保存，用于回滚/诊断；不重新解析后重写。
- `user_version` 从 3→4；创建表并设 4；幂等；不修改 whatif/agent 表。

## 7. 诊断

`delivery_diagnostics_export(outputDir)` 写 `delivery-diagnostics.json`（固定名，覆盖需确认）：

```text
DiagnosticsSummary {
  schema: "poie.delivery-diagnostics/0.1",
  truncated: boolean,
  generatedAt: i64,               // host 注入 now_ms；测试可注入固定时钟
  appVersion, os, arch,
  delivery: {
    schemaVersion,
    current: { siteHashPrefix, target, channel, releasedAt, manifestSha256, importedAt } | null,
    previous: 同上 | null,
    recalled
  },
  checks: [{ id, status:"ok|failed|unknown", detailCode: string }],
  errorCodes: string[],           // 固定白名单
  logTail: string[]               // 最多 50 行，先脱敏再截断
}
```

- 不包含 secrets、API key、prompt、模型输出、绝对路径、用户名、UUID、站点 note 原文、importedBy。
- `siteId` 只输出 `sha256(siteId)` 前 12 位；`siteName` 不输出。
- `checks.detailCode` 只允许固定枚举；`logTail` 先用固定正则脱敏（key/token/绝对路径/用户名/UUID），再按 128 KiB 总上限截断并 `truncated=true`。
- `path`/`outputDir` 都是 **sandbox 相对路径**（拒绝绝对路径、`..`、symlink、非 UTF-8）；复用 A0 `sandbox::resolve` 的 no-follow 规则；文件 ≤256 KiB。
- `detailCode` 枚举：`manifest_ok`、`digest_mismatch`、`state_absent`、`state_recalled`、`manifest_missing`、`io_denied`。
- 目标 `delivery-diagnostics.json` 已存在且 `overwrite=false` → `delivery_conflict`；`overwrite=true` 才覆盖。
- 不发送网络/遥测。

## 8. UI

- Settings “站点交付”区：
  - 输入 sandbox 相对 JSON 路径（不使用系统文件选择器，避免 capability/绝对路径）；显示 inspect 的 site/target/channel/sha；
  - Import 需再次确认 sha；成功后显示 current/previous；
  - Rollback 按钮；
  - 输入 sandbox 相对 outputDir 导出诊断。
- 所有状态只来自 host；浏览器预览显示 host-only 占位。
- Recall 状态用文字 + 颜色，不能只靠颜色。
- 不显示“部署成功/在线”文案；只显示“元数据已导入”。

## 9. 依赖与过程门禁

- 新 crate `poie-delivery`（纯 Rust）：manifest/schema/digest/state/compare，无 Tauri/网络/fs。
- host `delivery.rs`：文件读取（UTF-8/大小）、SQLite 事务、诊断。
- 依赖：poie-digest、serde/serde_json；不依赖 poie-exec。
- 根 members、桌面 path dep、两处 Cargo.lock、test-matrix/status-snapshot、desktop workflow **仅加 `crates/**` 路径过滤**（不发布/不改安装器）、ADR、cargo-deny。
- D1 不改 release workflow 的发布产物/安装器；D2 另立设计。
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

- release channel/签名/信任阈值、安装器/portable 发布、policy tighten-only、审批 token 通用化、updater/telemetry、多 OS artifact、raw exe/tauri targets 修正。
- D2 必须重新校验 release workflow/tauri targets，不与 D1 混做。

## 12. 参考

- [PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
