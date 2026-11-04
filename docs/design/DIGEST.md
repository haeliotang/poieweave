# Digest 规范（Phase A/C/D 共用）

- 状态：设计冻结候选（Phase A1 前必须通过校验）
- 日期：2026-11-04
- 实现：`crates/poie-digest`（新增，workspace）；不依赖 Tauri/网络
- 使用方：`poie-objects`（model/catalog digest）、`poie-whatif`（variant/result digest）、`poie-delivery`（manifest digest）

## 1. 算法

统一使用 **SHA-256**，输入为：

```text
domain/version "\n" canonical_json(payload)
```

`domain` 取值：

| domain | payload | 用途 |
| --- | --- | --- |
| `poie.model/0.1` | ModelCanonical | 模型 digest |
| `poie.catalog/0.1` | Catalog（不含 digest 字段） | Catalog digest |
| `poie.plan/0.1` | 计划摘要 + actions | 需要 SHA-256 时使用；不替代现有 `Plan::digest()` 字符串 |
| `poie.simulation/0.1` | 仿真摘要 + 关键状态 | 需要 SHA-256 时使用；不替代现有 `Simulation::digest()` |
| `poie.variant/0.1` | snapshotSha256 + canonical patch/goal/verification/deadline + mode + settings | C1 What-if 结果 |
| `poie.manifest/0.1` | manifest（不含 integrity 字段） | site bundle |

输出统一为小写 64 位十六进制，不带 `sha256:` 前缀；对外字段命名为 `*Digest`，类型为字符串。

## 1.1 Fingerprint vs SHA-256

两类摘要必须分开命名，不能互相冒充：

| 类型 | 形式 | 用途 | 示例 |
| --- | --- | --- | --- |
| **fingerprint** | 现有 `Plan::digest()` / `Simulation::digest()` 生成的稳定文本 | 人类可读、复现实验、报告 | `plan-base01-...` |
| **sha256** | `sha256(domain + "\n" + canonical_json(payload))` | 证据提交、审批、manifest、跨进程比较 | 64 位 hex |

- `Plan::digest()`/`Simulation::digest()` **不是** SHA-256，不得写入 `*Sha256` 字段；
- 需要绑定时，把 fingerprint 作为 payload 字段再 hash，并同时保留 fingerprint 供复现；
- 任何 API 字段命名必须能区分：`planFingerprint` vs `planSha256`、`catalogDigest`（sha256）等。

## 1.2 扩展 domains

| domain | payload | 用途 |
| --- | --- | --- |
| `poie.approval/0.1` | tool/action + subject + session + policy version + expiry | B/C/D 审批绑定 |
| `poie.evidence/0.1` | evidence id + kind + fingerprint + source digest | D 证据绑定 |
| `poie.contracts/0.1` | contract id/version/revision | D manifests |
| `poie.agent-audit/0.1` | audit record without hash | B audit chain |

审批 digest 的字段顺序由各工具显式定义；不得把未提供字段拼成空字符串而不声明（用 `null` 并纳入规范）。

## 2. Canonical JSON

1. UTF-8，无 BOM；
2. 对象键按 Unicode 码点/字节字典序升序；嵌套对象同样；
3. 数组保持业务语义顺序；需要集合语义时按稳定 ID 排序；
4. 不含多余空白；字符串按 JSON 标准转义；
5. 数字：只允许有限 f64/i64/u64；拒绝 NaN/Infinity；整数不写成浮点；浮点使用 `serde_json` 稳定序列化，调用方不得自行格式化；`-0.0` 归一化为 `0.0`；浮点必须保留 IEEE-754 双精度语义，不使用十进制字符串近似；
6. 禁止 `null` 与缺省值混用：规范结构必须显式列出字段；可选字段缺省为 `null`，不能时而省略时而写 null；
7. 不包含时间戳、机器路径、随机 ID（除非它就是被测输入的一部分）。

## 3. 模型 canonical

`ModelCanonical` 只从已验证的 `Model` 字段构建：

```text
name, resources[], assets[], lots[], processes[], services[]
```

- 资源：id, unit symbol, unit dim/scale, mass_per_unit_kg, quality
- 资产：id, kind, status, status_evidence, requires[], requires_any[], provides[],
  capability_verification, degraded（不包含 `description`，它不在验证后的 Asset 结构中）
- 批次：id, resource, si, quality
- 工艺：id, inputs[], outputs[], occupies[], requires_capabilities[], restores[],
  duration, verification, asset_mass_change, mass_tolerance, yield
- 服务：id, requires[], requires_any[]

所有引用数组按 id 排序；`&'static Unit` 输出 symbol/dim/scale，不输出指针。`location/evidence` 等未进入 validated `Lot` 的原始文档字段不进入 digest，除非 PO 升版把它们提升为正式字段。

## 4. 计划/仿真 digest 的关系

- 现有 `Plan::digest()` 与 `Simulation::digest()` 是**人类可读、格式化的稳定字符串**，继续保留；
- 需要哈希时，把该字符串作为 payload 的一个字段再 canonical/hash；
- 任何 UI/manifest 同时展示时，必须区分 `digestString` 与 `sha256`，不得声称两者相等。

## 5. Manifest

```text
manifest_sha256 = sha256("poie.manifest/0.1\n" + canonical_json(manifest_without_integrity))
```

`integrity.manifest_sha256` 是唯一排除字段；其他字段（包括 artifacts/evidence/contracts）全部纳入。文件内容哈希独立计算，不从 manifest 哈希推导。

## 6. 测试向量与验收

1. 同一输入两次生成相等；
2. 修改任意一个字段（包括数组顺序/单位）digest 必变；
3. 键顺序不同的同一对象 digest 相等；
4. 非有限数字/未知 schema 返回稳定错误；
5. 固定 fixture 的 digest 写入测试快照，CI 重放；
6. `poie-digest` 有属性测试：随机 JSON 键序不变性；canonical JSON 可被标准 JSON 解析器解析；固定测试向量覆盖 `-0.0`、整数值浮点、字符串数组顺序、嵌套键序。
7. `poie-digest` 进入根 `members`/`Cargo.lock`/测试矩阵；`check-test-matrix` 通过。

## 7. 非目标

- 不做签名（Phase D 仅完整性；签名另立 ADR）；
- 不哈希 Tauri 配置、构建时间、主机路径；
- 不把 digest 当作安全性证明；它只证明同一 canonical 输入。
