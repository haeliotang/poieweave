# Schema 注册表与兼容矩阵

- 状态：设计冻结候选（Phase A1 前）
- 实现检查器：`scripts/check-schema-registry.py`（A0 新增并加入 CI；当前尚未存在，本表是唯一来源）
- 所有者：`PLATFORM.zh-CN.md` §1

| schema | 所有者 | 当前版本 | 兼容规则 |
| --- | --- | --- | --- |
| `poie.ontology-catalog/0.1` | A | 0.1 | minor 可加可选字段；major 拒绝 |
| `poie.object-page/0.1` | A | 0.1 | 同上 |
| `poie.action-preview/0.1` | A | 0.1 | 同上 |
| `poie.agent-run/0.1` | B | 0.1 | 状态机字段只能加可选；major 迁移 |
| `poie.agent-eval/0.1` | B | 0.1 | fixture schema |
| `poie.whatif-snapshot/0.1` | C | 0.1 | 同上 |
| `poie.whatif-variant/0.1` | C | 0.1 | 同上 |
| `poie.whatif-result/0.1` | C | 0.1 | 同上 |
| `poie.site-bundle/0.1` | D | 0.1 | 同上 |
| `poie.delivery-state/0.1` | D | 0.1 | 同上 |
| `poie.diagnostics/0.1` | D | 0.1 | 同上 |

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
3. 同 domain 多个 major 时，必须同时有兼容/迁移说明；
4. 生成物/SDK 中的 schema 与 Rust 常量一致。

检查器本身在 A0 加入根 CI；本文件先作为注册表唯一来源。
