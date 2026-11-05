# Schema 注册表与兼容矩阵

- 状态：设计冻结候选（Phase A1 前）
- 实现检查器：`scripts/check-schema-registry.py`（已加入根 CI；扫描 product-layer 文档）
- 所有者：`PLATFORM.zh-CN.md` §1

| schema | 所有者 | 当前版本 | 兼容规则 |
| --- | --- | --- | --- |
| `poie.ontology-catalog/0.1` | A | 0.1 | minor 可加可选字段；major 拒绝 |
| `poie.object-page/0.1` | A | 0.1 | 同上 |
| `poie.object-detail/0.1` | A | 0.1 | 同上 |
| `poie.action-preview/0.1` | A | 0.1 | 同上 |
| `poie.sap-preview/0.1` | A | 0.1 | 同上 |
| `poie.agent-run/0.1` | B | 0.1 | 状态机字段只能加可选；major 迁移 |
| `poie.agent-eval/0.1` | B | 0.1 | fixture schema |
| `poie.agent-audit/0.1` | B | 0.1 | JSONL audit record |
| `poie.agent-approval/0.1` | B | 0.1 | persisted approval token |
| `poie.whatif-snapshot/0.1` | C | 0.1 | 同上 |
| `poie.whatif-variant/0.1` | C | 0.1 | 同上 |
| `poie.whatif-result/0.1` | C | 0.1 | 同上 |
| `poie.whatif-compare/0.1` | C | 0.1 | 同上 |
| `poie.timed-scenario/0.2` | C2a | 0.2 | C2a |
| `poie.timed-snapshot/0.1` | C2a | 0.1 | C2a |
| `poie.timed-variant/0.1` | C2a | 0.1 | C2a |
| `poie.timed-result/0.1` | C2a | 0.1 | C2a |
| `poie.timed-compare/0.1` | C2a | 0.1 | C2a |
| `poie.promotion-commit/0.1` | C2b | 0.1 | C2b |
| `poie.proposed-action-draft/0.1` | C2b | 0.1 | C2b |
| `poie.promotion-inputs/0.1` | C2b | 0.1 | C2b |
| `poie.promotion/0.1` | C2b | 0.1 | C2b |
| `poie.proposed-action/0.1` | C2b | 0.1 | C2b |
| `poie.site-manifest/0.1` | D1 | 0.1 | site manifest（自哈希，拒绝未知字段） |
| `poie.site-bundle/0.1` | D1 | 0.1 | 同上 |
| `poie.delivery-state/0.1` | D1 | 0.1 | 同上 |
| `poie.delivery-diagnostics/0.1` | D1 | 0.1 | diagnostics export |


## Digest domains（不是 wire schema）

| domain | 用途 |
| --- | --- |
| `poie.model/0.1` | model digest |
| `poie.catalog/0.1` | catalog digest |
| `poie.plan/0.1` | plan sha256 wrapper |
| `poie.simulation/0.1` | simulation sha256 wrapper |
| `poie.snapshot/0.1` | What-if snapshot digest |
| `poie.variant/0.1` | What-if result |
| `poie.evidence/0.1` | evidence binding |
| `poie.contracts/0.1` | contract registry |
| `poie.approval/0.1` | approval digest |
| `poie.manifest/0.1` | site manifest sha256 |
| `poie.agent-audit/0.1` | agent audit record hash |
| `poie.timed-scenario/0.2` | C2a |
| `poie.timed-compare/0.1` | C2a |
| `poie.timed-result/0.1` | C2a |
| `poie.timed-snapshot/0.1` | C2a |

## 解析与兼容函数

```rust
pub struct SchemaVersion { pub domain: String, pub major: u16, pub minor: u16 }
pub fn parse(schema: &str) -> Result<SchemaVersion, SchemaError>;
pub fn compatible(ours: SchemaVersion, theirs: SchemaVersion) -> Compatibility;
// - 同 domain、theirs.major == ours.major、theirs.minor <= ours.minor => Compatible
// - 同 domain、theirs.major == ours.major、theirs.minor > ours.minor => CompatibleWithWarnings
// - major 不同或 domain 不同 => Reject
```

`schema` 字段统一写成 `"poie.<domain>/<major>.<minor>"`；持久化 DTO 另带 `schemaVersion`（同值），避免只靠字符串。

## 检查器

`scripts/check-schema-registry.py` 扫描仓库中的 `schema = "poie.*"` 与 `schemaVersion`，要求：

1. 每个 schema 在表中登记；
2. 未知 domain 失败；
3. 同 domain 多个 major 时失败（MVP 无多 major；未来需迁移说明）；
4. 生成物/SDK 与 Rust Catalog 一致：由 `cargo run -p poie-sdkgen -- --check` 在 CI 执行。

检查器已加入根 CI，扫描 product-layer 文档与 `poie-objects`/`poie-sdkgen`/SDK 源码；历史实验 schema 不在本检查范围。本文件是 schema 注册表唯一来源（DIGEST 域为附加表）。
