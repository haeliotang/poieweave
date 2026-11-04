# Phase B 设计门禁终验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md) §5–§15、[PLATFORM](PLATFORM.zh-CN.md) §1/§3/§4/§6、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md) §6.4/§7.2.1、preview 实现、CI、`scripts/verify-all.sh`。
- 快照：`design/phase-b-closure`，HEAD **`053f529`**；写本文件前工作树干净。本报告只对该快照负责；只读校验，未改源码/设计/git，只覆盖本文件。
- 方法：审阅 `5ed076e..053f529` 全量 diff；逐项核对 §5–§15 与 PLATFORM/SCHEMAS/A；检查 `crates/poie-plan` preview 实现与 `poie-objects::functions()`；运行下面列出的 checker/test；确认 `evals/agent`、`scripts/check-agent-evals.py` 及 CI/verify-all 的 eval 引用已移除。
- 结论：**PASS。设计门禁可放行 B0；不要求 B0 代码、fixture 或 runner 已存在。**

## 五项判定

1. **B1–B7 阻断已闭合。** B1：§6 六工具均有版本/args/output/citation/approval/每轮上限/入口，且与 A §6.4、`poie-objects::functions()`、SDK 的六个函数一致（`sdkgen --check` exit 0）。B2：§6.1 与 A §7.2.1 统一为函数内 throwaway ledger；实现只用局部 `ActionLedger`，不建 `Authorization`，返回 `authorized_actions=0`/`authorization_created=false`，测试覆盖 3 个 assumed。B3/B5：§7 固定“审批→capability→执行”，含 `pending|approved|rejected|expired|executing|interrupted|consumed|cancelled`、原子条件更新、`started_at` 租约恢复、`(runId,callId)` 幂等和双 resume 竞争；PLATFORM §6 的 `ApprovalToken` 已含同状态与 `started_at_ms`。B4：§8 的 `RunState` 持久化 args/结果摘要/citations/`truncated`/`pendingApproval`，registry + `spawn_blocking` + 锁边界 + 取消标志明确。B6：§9 的 `answer.emit` claims/citations、`citation_missing`、suggestions 不参与事实校验已冻结。B7：§10 的 `poie.agent-audit/0.1` 记录、`prevHash`/`hash`、genesis、anchor、尾部残行、`user_version` 迁移与重建均已写入；DIGEST/SCHEMAS 已登记 audit domain。
2. **评测声明已诚实化，不再有假产物/假 CI 绿灯。** 仓库当前无 `evals/agent`、无 `scripts/check-agent-evals.py`，CI 与 `verify-all.sh` 也没有 agent eval 步骤；§12 明确“设计门禁只冻结 schema 与覆盖目标，不提交占位 fixture，不声称 CI eval 已绿”，并把 72 fixtures、runner、checker 列为 B0 交付物。§12 的 TOML schema（`schema/id/preregistered_at/prompt/profile/[script]/[approval]/[expect]`）字段具体；覆盖目标为六工具 × 12 场景，审批只允许 `action.draft`。这足以让 B0 开工，不再以静态文件数量冒充通过。
3. **PLATFORM/SCHEMAS/preview/function schema 无新矛盾。** `SapPreview` 字段与 §6/§6.1 的 camelCase 投影一致；六工具 input schema 的必填项与 §6 表一致；错误码表覆盖 §13 所需 B 码（含 `citation_missing`/`provider_response_too_large`/`approval_*`/`run_*`/`audit_integrity_failed`）；`poie.agent-run/0.1`、`poie.agent-eval/0.1`、`poie.agent-audit/0.1` 已登记；`check-schema-registry.py` 通过（24 schemas，rule 4 由 `sdkgen --check` 承担）。未发现新的跨文档反向依赖或第二套工具语义。
4. **B0 前置已枚举。** §5 给出 session id（`auth::Session.session_id`，128-bit hex）、`approval_tokens` 持久化、`PRAGMA user_version` 审计迁移并删除启动 DROP、`agent_*` 的 `CommandError` 包络；§7 给出表结构与原子状态机；§12 给出 runner/fixtures/checker；§14 给出 `poie-governance` 依赖、根 members、两份 Cargo.lock、test-matrix/status-snapshot、desktop workflow 路径与 ADR。四项前置（session、审批表、审计迁移、runner/fixtures）均可直接转成 B0 任务。
5. **非阻断的 B0 施工注意事项。** §12 的“12 个场景”目前是覆盖目标而非逐场景矩阵，B0 在 fixtures/runner PR 中应把每个工具的 12 个场景名与断言写死；`RunState.status` 与 `toolCalls[].decision` 的枚举值宜在 B0 首次迁移时固定；“`check-schema-registry.py` 已存在且在 CI”与 §14:236 的“补”措辞、§6:73 的“B 首 PR 改 A functions()”与实际 A1 已六函数存在文档漂移；`poie.agent-approval/0.1` 注册行尚未被 §7/PLATFORM 引用。以上是实施期收口项，不影响设计契约或 B0 开工。

## 已执行检查

```text
cargo run --locked -q -p poie-sdkgen -- --check          exit 0
python3 scripts/check-schema-registry.py                 24 schemas, exit 0
python3 scripts/check-docs.py                            OK, exit 0
python3 scripts/check-test-matrix.py                     381/397, exit 0
cargo test --locked -q -p poie-plan -p poie-objects      4 passed
git diff --name-only 5ed076e..053f529                    77 files：72 fixtures + checker 移除，§12/§15/CI 收窄
```

边界：未跑全量 `cargo test`、Tauri/provider/Playwright；`poie-governance` 与 B0 代码尚未实现；真实 72 fixtures、runner、checker 属 B0 实现。本报告是设计门禁，不把“B0 未实现”计为失败。

VERDICT: PASS
