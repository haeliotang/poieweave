# Phase D 设计：站点交付与运行闭环

- 状态：设计草案（待独立校验后进入开发）
- 日期：2026-11-04
- 范围：Phase D / 交付清单、通道、证据/版本绑定、离线导入、诊断、回滚
- 上游：Phase A–C、现有发布流水线与证据链
- 下游：真实站点安装/运维（外部条件）

## 1. 背景与问题

Poieweave 已经能构建 Linux/Windows/macOS universal 产物，并在 tag 上自动发布 Release。但“能安装”不等于“能在站点交付和运维”：

- 没有 release channel 和可解释的升级边界；
- 没有把应用版本、模型 digest、合同版本、证据摘要、策略绑在一起的交付清单；
- 断网环境没有可校验的交付元数据；
- 没有本地诊断包/版本核对；
- 没有回滚/召回流程；
- 站点策略（Agent scope、远程推理、自动更新）没有统一入口。

Phase D 不做 OTA 平台，也不做安装器/模型热切换；它先做**可验证的交付清单**和**离线核对/回滚闭环**。

## 2. 目标

1. 定义 **Delivery Manifest**：app 版本/commit/target、channel、模型与合同 digest、证据摘要、策略、构建产物摘要。
2. 定义 **Site Bundle**：manifest + 可选诊断/证据摘要，可在断网环境导入/导出；**不包含应用二进制或模型文件**。
3. 实现 **Release Channel**：stable/canary/site 的本地订阅状态；显示在设置/状态栏；channel 是订阅，不是“目标版本”。
4. 实现 **Delivery Plan**：根据 channel、版本兼容、model/contracts/evidence digest、召回状态，解释可更新/不可更新原因。
5. 实现 **导入/回滚/召回**：导入只更新交付状态/策略/证据引用；回滚切换保留的上一清单；召回阻止继续使用。
6. 实现 **Diagnostics**：host/版本/channel、model/execution/federation fingerprint、审计摘要、最近错误（来自 audit）；不含密钥/prompt/文件内容。
7. 统一 `HostPolicy`：Agent scope、远程推理、自动更新默认关闭；D 的 SQLite 覆盖由同一策略加载器强制。

## 3. 非目标

- 不做自动安装/自动更新器；不替换 Tauri 包；不修改运行中的二进制。
- 不导入/切换模型文件；模型仍由构建期/本地 `ModelStore` 提供（Phase D 只记录 digest）。若未来要 model bundle，另立设计。
- 不做跨平台签名/公证的替代；SHA-256 只做完整性，未签名明确标记“未认证”。
- 不做中心化遥测/日志上传；默认零遥测。
- 不做 Kubernetes/Helm/多租户；单站点桌面交付。
- 不把 bundle 当 live 数据库备份/恢复工具；不承诺断网自动获取新版本。
- 不做 SBOM/attestation 的强制门禁（列为可选 O2）。

## 4. 用户与场景

| 场景 | 操作 |
| --- | --- |
| 现场基线 | 导出 manifest + diagnostics，记录版本/证据 digest |
| 隔离站点核对 | 携带 bundle 导入，校验版本/证据/策略，不替换 app |
| 回滚 | 保留上一交付清单，导入/切换并核对 |
| 召回 | 标记某个 release 不可用，阻止继续安装/切换 |
| 审计 | 查看 manifest/diagnostics，确认运行版本与证据 |
| 支持 | 导出脱敏 diagnostics 交给维护者 |

## 5. 数据模型

### 5.1 Delivery Manifest / Site Bundle

```toml/json
schema = "poie.site-bundle/0.1"
bundleId = "..."
createdAt = "..."
releaseId = "v0.1.1+<commit>"
app = { product = "Poieweave", version = "0.1.1", commit = "...", target = "universal-apple-darwin" }
channel = "stable" # stable | canary | site
site = { id = "...", name = "..." }
coreCrates = [ { name = "poie-ontology", version = "0.1.0" } ]
model = { id = "base-01", sha256 = "..." }
contracts = [
  { id = "physical-ontology", version = "0.3-draft", revision = "2026-11-03", sha256 = "..." },
  { id = "state-action-protocol", version = "0.2-draft", revision = "...", sha256 = "..." },
  { id = "viability-expansion", version = "0.2-draft", revision = "...", sha256 = "..." }
]
evidence = [ { id = "vec-base01", kind = "report", fingerprint = "...", sha256 = "..." } ]
policy = { agentScopes = ["read","propose"], remoteInference = false, autoUpdate = false }
rollbackTo = "v0.1.0+<commit>"  // 可选
artifacts = [ { name = "Poieweave_0.1.1_universal.app.zip", os = "macos", target = "universal", sha256 = "...", bytes = 1234 } ]
diagnostics = { schema = "poie.diagnostics/0.1", inline = false }
integrity = { algorithm = "sha256", manifestSha256 = "..." }
```

规则：

- manifest 只包含元数据；`artifacts` 是对已发布产物的引用，不随 bundle 复制二进制。
- 不包含 `files[]`/模型文件；没有路径遍历面。
- `manifestSha256 = sha256("poie.manifest/0.1\n" + canonical_json(manifest without integrity))`，见 [DIGEST](DIGEST.md)。
- 未知 schema major 拒绝；minor 忽略未知字段。
- SHA-256 不等于签名；未签名显示“未认证”。
- artifact/target 来自构建期 `release.json`，不由运行时猜测。
- `contracts` 由机器可读 `contracts.toml`（新增）生成，不再解析契约正文。

### 5.2 Channel / DeliveryState

通道是**订阅**，不是固定目标版本：

```text
DeliveryChannel = stable | canary | site
DeliveryState {
  schemaVersion, channel, siteId?,
  installed: ManifestSummary?, previous: ManifestSummary?,
  lastImportedAt, lastDiagnosticsAt,
  policy: HostPolicy,
  recalledReleaseIds: string[]
}
```

优先级：**manifest 的 channel/site 字段只描述该清单来源；本地 `DeliveryState.channel` 决定订阅**；冲突时以本地订阅为准，UI 显示差异并要求显式切换。`site` 通道表示站点固定，不随 stable/canary 自动变化。

默认：channel=`stable`；remoteInference=false；autoUpdate=false；agentScopes=[read,propose]。

### 5.3 DeliveryPlan / Recall

```text
DeliveryPlan {
  desiredChannel, installedManifest?, candidateManifest?,
  constraints[{ id, ok, detail }],
  blockedReasons[],
  actions[{ kind, label, requiresApproval }]
}
```

- 约束：schema/version 兼容、model digest、contracts digest、evidence 完整、回滚目标存在、未召回。
- `delivery_plan` 必须能解释“为什么没有更新/为什么被挡住”；不能只返回成功/失败。
- `recall(releaseId)`：加入本地召回列表并写审计；已安装则显示召回+允许回滚；不删除证据/不自动切换。
- 回滚：切换到 `previous` 清单；不覆盖 SQLite，不替换二进制；数据库迁移失败则拒绝并保留原状态。

### 5.4 Diagnostics

```text
Diagnostics {
  schemaVersion, generatedAt, app, channel, siteId?,
  host { version, os, dataDir basename, sandbox basename },
  modelSha256, executionFingerprint, federationFingerprint,
  auditSummary { count, lastEventAt, eventCounts },
  recentErrors[] { at, code, event }, // 仅来自 audit，不新建错误表
  capabilities { remoteInferenceConfigured, agentEnabled },
  redactions[]
}
```

禁止：API key、Authorization、完整 prompt、SQLite 内容、文件内容、口令、绝对路径细节。`redactions` 由固定规则扫描：删除包含 `key/token/secret/password/authorization` 的字段或替换为 `<redacted>`，并有负例测试。

## 6. 架构

```text
crates/poie-delivery       NEW: manifest/DeliveryState/DeliveryPlan/Diagnostics; pure Rust
  ├── poie-digest          canonical JSON/SHA-256
  ├── poie-ontology        model digest
  └── poie-viability       evidence metadata (optional)
        │
        ▼
apps/desktop/src-tauri     delivery.rs: state/commands/policy/audit
apps/desktop/src           Delivery settings/status UI
.github/workflows         release.json + manifest + artifact upload
```

### 6.1 `poie-delivery`

- `Manifest`, `ManifestSummary`, `DeliveryState`, `DeliveryPlan`, `Diagnostics` DTO；
- `Manifest::verify()`：schema/version/digest/contracts/evidence/artifact refs；
- `manifest_sha256` canonical 计算；
- 不读文件树、不写数据库、不网络。

### 6.2 Host 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `delivery_status` | 无 | DeliveryState |
| `delivery_plan` | desiredChannel? | DeliveryPlan |
| `delivery_bundle_export` | 无 | bundle JSON（写入 sandbox） |
| `delivery_bundle_inspect` | bundle JSON path | Manifest + validation |
| `delivery_bundle_import` | bundle JSON path, approvalTokenId | DeliveryState（只更新清单/策略/证据引用） |
| `delivery_bundle_rollback` | approvalTokenId | DeliveryState |
| `delivery_recall` | releaseId, approvalTokenId | DeliveryState |
| `delivery_channel_set` | channel, approvalTokenId | DeliveryState |
| `delivery_policy_set` | HostPolicy, approvalTokenId | DeliveryState |
| `delivery_diagnostics_export` | 无 | Diagnostics JSON |

所有写命令要求登录 + `ApprovalToken`（见 PLATFORM §6）+ 审计；路径只接受 sandbox 内相对名，不跟随符号链接。

### 6.3 模型/证据/合同绑定

- host 保存 `SourceModel`（Base-01 + degraded）与 `modelSha256`；D 只记录/校验，不导入模型。
- `contracts.toml` 机器可读注册表：id/version/revision/sha256；构建时生成，D 校验。
- 证据摘要来自现有实验产物（`docs/status/test-matrix`/报告），不重新哈希整个仓库；只记录声明 + 文件 sha256。
- 导入时若 model/contracts/evidence 与当前不匹配，拒绝并给出差异；不尝试热切换。

### 6.4 发布流水线与版本源

- 单一版本源：`apps/desktop/package.json` 的 `version`；`src-tauri/Cargo.toml` 必须一致；`RELEASE.json` 构建期注入 commit/target/channel。
- `.github/workflows/desktop-build.yml` 产出 Linux `.deb`、Windows NSIS、macOS universal `.app.zip`；**不生成 DMG/便携 exe**。manifest 示例只列实际产物；DMG 若手动发布，不写入自动 manifest。
- tag release 依赖根 `verify-all`/证据重放；`check-release-manifest.py` 校验版本/产物/digest。
- SBOM/attestation 可选（O2），不阻塞。

### 6.5 Policy 强制

- `HostPolicy` 由 PLATFORM 定义；D 的 SQLite 覆盖默认值。
- `inference_*`、`agent_*`、`execution_gate_set`、`sandbox_*` 必须读取同一 policy；`remoteInference=false` 时 `inference_configure/test/complete` 拒绝。
- `execution_gate_set`/`sandbox_*` 的 auth 缺口按 Phase A A0 修复；D 不重复实现。

## 7. UI

### 7.1 设置 → 交付

- 当前 channel/version/commit/target、manifest sha256；
- 当前 model/contracts/evidence digest；
- 按钮：导出 bundle、导入（先 inspect）、回滚、召回、导出诊断；
- 导入前展示：schema/版本/digest 差异、未认证标记、约束失败；
- 显示 `DeliveryPlan` 的“为什么没有更新/为什么被挡住”；
- 浏览器预览禁用 host-only 操作。

### 7.2 状态栏 / 关于

- 状态栏显示 channel + manifest 简写；
- 关于页显示完整 manifest 摘要与“未认证”标记；
- 无隐式遥测。

## 8. 安全与失败模式

| 风险 | 控制 |
| --- | --- |
| 路径穿越/符号链接 | 不接受文件树；bundle 是 JSON；路径只允许 sandbox 内相对文件 |
| bundle 篡改 | manifestSha256 + 字段校验；失败不改状态 |
| 过期/不兼容 | schema/version/model/contracts/evidence 检查 |
| 未签名误信 | 明确完整性≠签名；UI 标记未认证 |
| 密钥泄漏 | diagnostics 固定白名单 + redaction 扫描 + 负例 |
| 自动更新风险 | 默认关闭；MVP 无自动安装 |
| 回滚丢失状态 | 保留 previous 清单；DB 迁移先备份；失败拒绝 |
| 策略绕过 | 单一 HostPolicy loader；所有命令检查 |
| 召回失效 | 本地召回列表 + 审计；阻止使用 |

## 9. 测试与验收

### 单元/集成

- manifest canonical hash 稳定；篡改任一字段失败；
- unknown schema/版本/合同/证据/model 不匹配拒绝；
- inspect/import/rollback/recall/channel/policy 状态机；
- 导入失败不改 DeliveryState；
- diagnostics 不含禁字段，redaction 负例；
- `HostPolicy` 强制：remoteInference=false 拒绝推理，agent scope 不匹配拒绝；
- DB `user_version` 迁移/备份/回滚失败。

### UI

- Playwright：交付面板、plan 原因、导入错误、浏览器预览禁用；
- 截图更新。

### 验收标准

1. 断网可导入/检查一个 manifest，并更新 channel/policy/证据引用；不替换 app/模型；
2. 任一篡改/不匹配导致导入失败且状态不变；
3. 回滚/召回可测且审计；
4. 诊断包脱敏、无密钥/prompt；
5. 默认关闭远程推理/自动更新/遥测；策略由单一 loader 强制。

## 10. 实施顺序

1. `poie-delivery` manifest/state/plan/digest/validation + tests；
2. host `delivery.rs` state+migrations+commands+policy loader；
3. UI 交付面板/状态栏；
4. `contracts.toml` + `release.json` + `check-release-manifest.py`；
5. 发布流水线 manifest/artifact 对齐；
6. Playwright/文档/测试矩阵/状态快照。

## 11. 开放问题

- 是否将 `recall` 同步到远端（MVP 本地列表，不联网）。
- 是否把 SBOM/attestation 纳入门禁（O2，先不加）。
- 是否允许导入策略覆盖 Agent scope（MVP 允许，需审批/审计）。
- 站点多角色/双人审批：MVP 单操作员，不做。

## 12. 参考

- Palantir Apollo release channels/plans/recall：<https://www.palantir.com/docs/apollo/core/introduction/>
- [DIGEST](DIGEST.md)、[PLATFORM](PLATFORM.zh-CN.md)
- `docs/status/test-matrix.zh-CN.md`、`docs/status/status-snapshot.zh-CN.md`
