# Phase A 设计门禁校验（A0 + A1，终验）

- 校验对象：[A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)；代码 `crates/poie-ontology`、`crates/poie-plan`、`apps/desktop/src-tauri`、`apps/desktop/src`、`scripts/`。
- 快照：分支 `design/phase-a-d`，HEAD **`907b246`**；工作树干净（本文件除外）。本报告只对 `907b246` 负责。
- 方法：只读对照上述文档与 `model.rs`/`plan.rs`/`sap.rs`/`sandbox.rs`/`lib.rs`/`execution.rs`/`domain.ts`/`App.tsx`/`i18n.ts`；`check-docs.py` 与 `check-test-matrix.py` 均 exit 0；未跑全量 `cargo test`；未改源码/设计/git，仅写本文件。
- 复核基线：`b7d0eb9` 报告的两项残留（R1 source 值域、R2 checker/README 时点）。

## 0. 判定

**PASS。** 两项残留均已实质闭合，未引入新的 A0/A1 契约矛盾；Phase A 零阻断项。B/C/D 门禁不参与本判定。

## 1. 两项残留复核

### R1 `PropertyDef.source` 值域（已闭合）

- A:76（枚举定义）、A:177（`PropertyDef.source`）、A:181（`PropertyValue.source`）三处均为 `model|declared|observed|evidence`；A:177 明示“与 PropertyValue 同一枚举”，SDK 生成器不会再见到两个不同的 source 联合。
- `status_evidence` 的分支语义未变：`Some -> evidence`、`None -> declared`，与 `model.rs:270-272`/`708-716` 一致；`PropertyDef` 与 `PropertyValue` 不再有三值残留。

### R2 checker 时点与 README 记录（已闭合）

- A:395 把 `scripts/check-schema-registry.py` + 根 CI 列入 **A0**；A:396 的 A1 步骤已删除该 checker；SCHEMAS:4/45 与 PLATFORM:9 同为 A0。四处时点一致，A0 验收由 SCHEMAS §检查器 的四条规则 + 根 CI 定义。
- README:103/125 已删除旧快照 `0527c7f`，PASS 指向本报告；README `当前状态` 与之一致，无旧 SHA 残留。SCHEMAS:3 的“Phase A1 前”是文档冻结状态（A0 早于 A1），不是 checker 时点，不构成矛盾。

## 2. A0/A1 就绪清单

- [x] A0 是第 0 步；`execution_gate_set` 先 `auth::require`；不生成 Authorization；`dispatch_allowed=false`、`authorized/dispatched=0`（A:40-46、395）。
- [x] `sandbox_read/write` 均 no-follow（read 用 `symlink_metadata`/`O_NOFOLLOW`，write 原子 rename）+“根内 symlink 指向根外”负例；删除 `sandbox_root`，统一 `host_info.sandboxRoot`（A:41；PLATFORM:79）。
- [x] 浏览器 fallback 与 `execution.rs` 断言改为 `synthetic-preview`/`blocked`/`false`、`review_status=="blocked"`、`rejections==actions.len()`、`authorized/dispatched=0`（A:43）。
- [x] ActionType 只含 process id 与模型字段；无 `label/description/sap.*`；`requires_capabilities` 与 `Process` 一致（A:98-109、211-217；model.rs:304-315）。
- [x] `ActionPreview/PlannedActionView` 归 host `objects.rs`，`SapPreview` 归 `poie-plan`；`EvidenceRef` 至少一个 digest；limit 默认 100/最大 500、catalog ≤1 MiB、opaque cursor（A:192、271、297、396；PLATFORM:121）。
- [x] 属性 `source` 四值一致；model/evidence/declared 三分支语义已由 A:76/177/181 定义。
- [x] checker 在 A0、根 CI；README 旧 SHA 已清除；`check-docs.py`/`check-test-matrix.py` exit 0。

## 3. 非阻塞说明

- 三分支 fixture（模型导出→`model`、`Some`→`evidence`、`None`→`declared`）尚未写入 A §11；语义已冻结，属 A1 测试落地项，不阻断设计门禁。
- 旧 `VALIDATION-FAST.zh-CN.md` 与 `VALIDATION.zh-CN.md`（`c4bc294`）是历史快照，其 Phase A 项已由后续 commit 闭合；不作为本判定依据，建议后续标注 superseded。
- A:43 写 `FALLBACK_EXECUTION`，实际符号为 `fallbackExecution`（domain.ts:237）；A0 按实际符号施工即可，不影响契约。
- 现有实现仍是 A0 前状态（sandbox.rs:26-35 `metadata`、lib.rs:224 `sandbox_root`、execution.rs:181-197 ready、domain.ts:237 ready、i18n.ts:78 复核通过）；这些是 A0 施工对象，不是设计门禁残留。

**结论：A0/A1 零设计阻断，可以开工；本报告仅对 `907b246` 负责。**

VERDICT: PASS
