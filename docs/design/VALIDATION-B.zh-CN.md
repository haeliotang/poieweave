# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[README](README.zh-CN.md)；代码 `apps/desktop/src-tauri/src/{auth,db,inference,http,lib,objects}.rs`、`crates/poie-{objects,plan,state}`、`scripts/`、CI。
- 快照：分支 `design/phase-b-closure`，HEAD **`ef96756`**；写本文件前工作树干净。本报告只对该快照负责。
- 方法：只读对照；运行 `cargo test -p poie-objects -p poie-plan --locked`（25 通过）、`scripts/check-schema-registry.py`（24 schemas，exit 0）、`check-docs.py`（exit 0）、`check-test-matrix.py`（380/396，exit 0）；用 `cargo tree` 检查依赖。未跑全量 `cargo test`，未调用真实 provider 或 Tauri 运行时，未改源码/设计/git；只覆盖写本文件。
- 结论：**方向正确，仍有 3 个 B 阻断项与 7 组 R 残留；不能开工。**

## 0. 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B1 六工具契约 | **BLOCKED** | 与 A 冻结的函数目录不是同一套，且无工具版本/上限 |
| B2 只读 SAP preview | PASS* | 只读约束已写；但 A/B 都声称拥有该 API，归属与命名需合一 |
| B3 审批表/状态机/原子消费 | **BLOCKED** | B 与 PLATFORM 的消费前置状态矛盾，且有重复执行窗口 |
| B4 registry/锁/RunState/取消 | PASS* | 核心已写；`status` 值域与取消落审计未冻结 |
| B5 先审批后执行/plan digest | PASS | `plan.propose` auto、`action.draft` confirm、subject 含 plan/action 已明确 |
| B6 `answer.emit` 协议 | PASS* | 协议/`citation_missing`/suggestions 已写；citation 示例不合 EvidenceRef |
| B7 审计链/迁移/重建 | **BLOCKED** | 日志无 `turnId/toolVersion`，索引却加 `turn_id`，重建不可等价 |
| R1 | REQUIRED | 错误码表漏 `audit_integrity_failed`；`CommandError` 仍无类型定义 |
| R2 | PASS | analyst/dispatcher 白名单明确，analyst 无 `action.draft` |
| R3 | REQUIRED | 预算 char/byte 与 32 KiB context 口径未定义；截断算法未定义 |
| R4 | REQUIRED | citation kind `evidence` 非法；示例无 digest/fingerprint；`evidenceRefs` 未入 RunState |
| R5 | REQUIRED | 45 例与“6×4×3”矩阵算术不符；无覆盖表/`check-agent-evals.py` |
| R6 | REQUIRED | 仍在用 `App.tsx:1021` 的 `inference_complete`；Settings 实际用 `inference_test`；无 UI 测试 |
| R7 | REQUIRED | checker 只扫 docs/design，未实现 SCHEMAS §检查器的规则 3/4 |
| R8 | REQUIRED | `SCHEMAS.md:4/62` 仍称 checker “当前尚未存在/A0 加入” |
| R9 | REQUIRED | capability 生命周期（寿命/会话绑定/撤销/测试）被删掉 |

## 1. B 阻断项

### B1（阻断）六工具不是 A 函数目录的投影

- A:119–126 冻结了六个函数（含 `evidence.get`/`sim.run`），并明确“B 的 Tool Catalog 是这些函数的投影（同名/同参数），不得另造一套工具语义”；A:122 的 `plan.preview_sap` 接收 host 持有的 **Plan object**，明确“不接受 planRef”。
- B:64–74 的六工具替换为 `action.draft`/`answer.emit`，移除 `evidence.get`/`sim.run`；B:69 的 `plan.preview_sap` 只接收 `{planFingerprint}`。这不是同名/同参数投影，两个冻结文档对“B1 六工具”给出不同集合。
- B:66 的 `catalog.describe` 输出 `OntologyCatalog`；A 的 `functions()`（`crates/poie-objects/src/lib.rs:387–405`）仍列出 `evidence.get`/`sim.run`，而 B:73 声称它们不向模型暴露。模型读取 Catalog 时会看到被 B 排除的工具；B 没有定义过滤或改写 A Catalog 的函数目录。
- B:74/95/129 要求 tool version，但六工具表没有版本号；B:123 只有全局预算，没有逐工具上限。所谓“封闭的版本化 Tool Catalog”仍不可由代码直接落地。

**最小修复**：二选一并写死：① 更新 A:119–126，使 B 的六工具成为正式的超集/映射（含 `action.draft`/`answer.emit`、preview 输入、`evidence.get`/`sim.run` 的隐藏规则）；或 ② 把 B 六工具改回 A 的四项投影 + host-only 扩展，并说明 `catalog.describe` 如何过滤。补 `toolVersion`/每工具上限。

### B3（阻断）审批消费的前置状态矛盾，且“执行后置 consumed”可重复执行

- B:100–112 的流程是：`agent_approve` 置 `approved` → `agent_resume` 校验后签发 capability → 执行一次 → 置 `consumed`；SQL 也是 `WHERE state='approved'`（B:103–104）。
- PLATFORM:161 却写“`UPDATE ... WHERE state='pending' AND expires_at>?` 原子消费”。按 PLATFORM 实现，`agent_approve` 之后 token 已不是 `pending`，消费永远匹配不到；按 B 实现，PLATFORM 的“pending 原子消费”又不成立。两份上级契约直接冲突。
- B:112 的“执行一次，置 consumed”没有说明消费发生在执行前还是执行后。若在执行后，两个并发 `agent_resume` 可同时通过 `approved` 检查并各执行一次；若在执行前，崩溃会丢失已批准的执行。B 没有 `executing`/`consuming` 中间态或 capability 的单一消费点。
- 该问题同时吞掉 R9：B 没有 capability 的寿命（旧文本要求 ≤300s）、`Session.expires_at` 取 min、登出/接管/降级撤销、随机来源与测试。

**最小修复**：冻结唯一顺序与 SQL：`pending → approved`（`WHERE state='pending' AND expires_at>?`），`approved → consuming`（原子 claim，`WHERE state='approved'`），capability 单次签发，执行后 `consuming → consumed`；失败可 `consuming → failed` 或回退需明确。补 approve/reject/expire 的原子 SQL、capability 寿命/会话绑定/撤销测试，并让 PLATFORM:161 与 B 一致。

### B7（阻断）审计记录与可重建索引不等价

- B:163–166 的审计记录没有 `toolVersion`，也没有 `turnId`；B:172 却要求 SQLite `audit` 表加 `turn_id`。B:173 要求 `audit_rebuild` 从日志重放且“结果与索引一致”——日志无法派生 `turn_id`，重建必然不等价（除非额外定义 run/turn 映射，但当前没有）。
- 记录只有 `"tool"`，没有版本；token 和 `RunState` 都有 `toolVersion`。审计无法区分同名工具的不同版本，违反上一轮 B7 的 `tool@version` 要求。
- B:164 的 `decision` 枚举没有 `cancelled`/`interrupted`；B:140 明确取消后写 `cancelled`。取消/重启事件落不进审计枚举。
- B:173 的 `audit_integrity_failed` 不在 B:212 的错误码表中，R1 的码表与 B7 的自述不一致。

**最小修复**：审计记录加入 `toolVersion` 与 `turnId`（或从记录中去掉 `turn_id` 并说明派生规则），加入 `cancelled`/`interrupted` decision，登记 `audit_integrity_failed`；重建测试逐字段断言，包括 legacy 段。

## 2. 通过项与残留

- **B2 PASS***：B:77–88 的只读约束、`assumed` 展示、禁止 `validate_plan_through_sap`、ledger/authorization 计数测试均已明确；`cargo tree -p poie-plan` 无 `poie-exec`。但 B:69/79 称“新增”，A:275 已声明 Phase A 新增同一 API；实际 `poie-plan/src/sap.rs` 只有会构造 `Authorization` 的 `validate_plan_through_sap`（sap.rs:77–82），没有 preview。必须明确归属（A 已承诺 vs B 新增）并统一 `SapPreview`/`SapPreviewDto` 命名；这是 R8/R4 的文档修复，不是 B2 的安全否决。
- **B4 PASS***：B:119–140 的 registry、锁外 provider、多 `tool_calls`、预算、`interrupted`、取消丢弃结果均已写；`RunState` 含 `argsJsonRedacted`/`impact`/`citations`。残留：`status` 没有值域（created/running/waiting_approval/completed/failed/cancelled/interrupted 需列出）；等待审批时的 cancel 路径未写；取消审计 decision 与 B7 冲突。
- **B5 PASS**：B:75/107–115 固定“审批 → capability → 执行”，`plan.propose` auto、`action.draft` confirm、subject 含 `planFingerprint + actionId + argsSha256`；操作员批的是具体 action，不是未来计划。该通过受 B3 的消费矛盾约束。
- **B6 PASS***：B:144–156 定义 claims/citations、`citation_missing`、suggestions 不参与事实校验、UI 只渲染文本；满足 B6 枚举。残留见 R4：示例 citation 无 digest/fingerprint，`state.query` 的 `object/evidence` 中 `evidence` 不是 PLATFORM §5.1 的合法 kind，RunState 只写 `citations[]` 却要求匹配 `evidenceRefs`。
- **B0/现有代码**：`auth.rs:22–26` 仍无 `session_id`，`db.rs:58–89` 仍启动 DROP、无 `user_version`，`lib.rs:112–128` 仍在 DB 锁内做 HTTP。这些是 B0 的实施对象；设计已点名，未计为设计阻断。
- **A1 实际缺口**：A 文档声明 `preview_plan_through_sap`，但 `crates/poie-plan` 无该符号；B 依赖它。实施前应先补 A1 或明确由 B 实现；否则 B2/B5 的 preview 无法调用。

## 3. R 残留

- **R1**：B:212 已列 16 个码，但漏 `audit_integrity_failed`（B:173）与 `preview_unavailable`/`policy_blocked`（PLATFORM:55/62 及 A 的 PreviewError 映射）；`CommandError` 只有 JSON 形状，没有 Rust 类型/`details` 约束。补全码表与类型。
- **R2 PASS**：B:178–184 白名单正确；analyst 无 `action.draft`，dispatcher 不摄入外部文档；`policyVersion` 固定。
- **R3**：B:120/123 同时有 `max_prompt_chars=4000`、`max_tool_result_chars=20000`、`max_context_chars=32 KiB`，但没有定义 char 还是 byte、context 是否含 tool results/system prompt、超限时如何截断且保留 citation。现有 `inference.rs:146–151` 用 `.len()`（byte）；CJK 下 20000 chars 可超过 32 KiB。补计量与截断规则。
- **R4**：B:149 的 citation 只有 `kind/id/source`，不满足 B:144 与 PLATFORM:122 的“至少一个 fingerprint|sha256”；B:67 的 `evidence` kind 不在 PLATFORM:122 的值域；B:153 的 `evidenceRefs` 在 B:131/135 的 RunState 中不存在。统一结构并给合法示例。
- **R5**：B:206 的“6 工具 × 4 类参数 × 3 类语义”若按笛卡尔积至少 72 例，与“45 例”矛盾；没有逐用例覆盖表，`scripts/check-agent-evals.py` 尚不存在且未定义计数/矩阵校验。补表并明确 45 或 72。
- **R6**：B:216 说 `inference_complete` 只留 Settings，但 `App.tsx:1021` 仍由 Agent composer 调用，Settings 测试实际用 `inference_test`（App.tsx:1299）；`inference_complete` 没有 Settings 消费者。应删除该命令或改写 Settings，避免保留自由聊天回退；B 没有 Playwright/UI 验收用例。
- **R7**：`scripts/check-schema-registry.py` 只扫 `docs/design/*.md`，明确跳过 PLATFORM/SCHEMAS/DIGEST，未扫描 Rust/SDK；SCHEMAS:55–60 要求扫描仓库、`schemaVersion`、多 major 兼容说明、生成物与 Rust 常量一致，实际只做了登记检查（4 条规则中的 1–2 条）。CI 步骤已加（`.github/workflows/ci.yml:40`），但 checker 名不副实；`check-agent-evals.py` 仍未建。
- **R8**：`SCHEMAS.md:4` “当前尚未存在”与 `:62` “A0 加入根 CI”均与事实不符（checker 由本 HEAD 的 B 提交新增）；`PLATFORM:9` “A0 新增”同样时点错误。A:275 与 B:79 对 preview 的归属重复；A:126 与 B:64–74 的工具集冲突（见 B1）。
- **R9**：见 B3；capability 的 ≤300s、session 绑定、登出/接管/降级撤销、128-bit 随机来源、单次消费测试均未在 B 中保留。

## 4. 已执行的检查

```text
cargo test -p poie-objects -p poie-plan --locked   25 passed
python3 scripts/check-schema-registry.py            OK (24 schemas)
python3 scripts/check-docs.py                       OK
python3 scripts/check-test-matrix.py                OK (380/396)
cargo tree -p poie-plan --edges normal              no poie-exec
Cargo.lock / desktop Cargo.lock                     no jsonschema evaluator
```

`schemars` 只出现在 Tauri 依赖闭包中，是 schema 生成器，不是运行时 JSON Schema 求值器；B1 的“无求值器”方向成立。

## 5. 边界

本报告只读对照 `ef96756`；未验证真实 provider、Tauri 运行时、SQLite 并发与桌面 Playwright。B0 的 session/audit/DB 改动尚未实现，故只按设计契约审查；不要求重写架构，B1/B3/B7 与 R 项都可在现有分层内定点修订。

VERDICT: CHANGES_REQUIRED
