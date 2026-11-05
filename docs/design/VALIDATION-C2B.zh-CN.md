# Phase C2b 设计门禁（独立复验）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`、`poie-state::review`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)。
- 快照：branch `design/c2b`，HEAD **`9d86071`**；写本文件前 `git status --porcelain` 为空；只读源码/设计，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条复核 6771fa5 的 4 个阻断与 9d86071 的修订，对照设计 §5–§13、`sap.rs`/`review.rs`/`governance lib.rs`/`whatif.rs`/`db.rs`/PLATFORM；运行 docs、schema 与四个 lib 测试。

## 四项复核

1. **preview 未来 TimeWindow：已闭合。** §6.1:126 明确 `missing_authorization` 与 `time_window=not_yet_valid` 是预期项，仅 `time_window=expired` 为真实拒绝；§6.1:127 要求多动作测试。`review.rs:83–95` 已提供稳定 `reason` 区分，`preview_actions` 可据此过滤；正常多动作计划不再必然落入 `sap_rejected`。
2. **actionSha256/action_json：阻断 1，未闭合。** §5.2:96 与 §10:225 仍要求 `actionSha256`/`action_sha256`，但全文无 domain/payload/排除字段；DIGEST:36 只登记 `poie.proposed-action/0.1` 名称；§5.1:77 的 `actionsSha256` 是 draft 数组 digest，不能替代单动作；§10:225 的 `action_json` 仍无“存 draft 还是 ProposedAction”的定义；§5.1:79 的 `proposalSha256` 仍排除 `times` 又包含 `nowMs`。本轮 HEAD 未新增所称公式。
3. **authorize/command/policy：阻断 2，部分闭合。** `command` 已改为实际命令 `whatif_promote_propose`（§7:159、§7:171），但 §6.2:140 仍用 `POLICY_VERSION` 计算 `subject_digest`，而 §6.2:145 存 `policy_version="poie-promote/0.1"`；`governance lib.rs:17` 的 `POLICY_VERSION` 是 `poie-agent/0.1`，按 §6.2 创建/审批重算会与 §7:171“显式用 poie-promote”冲突，导致审批绑定不一致。另 §7:171 的 no-live 白名单写 `whatif_timed_*`，不匹配新命令 `whatif_promote_*`，按字面测试会漏掉或拒绝 C2b 写命令。
4. **propose 事务/唯一/过期：已闭合。** §6.2:135 单事务写 proposal+token+audit；§6.2:136 加 pending/approved 的 partial unique；§9:206 在读/approve/commit 同步过期；§9:208 补崩溃、并发、过期测试。非阻断残留：propose 未列入过期同步点，超时 pending 行需先经读/approve/commit 才变 expired，否则可能被唯一索引判 `duplicate_proposal`。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          38 schemas, exit 0
cargo test -p poie-plan --lib                     19 passed, 0 failed
cargo test -p poie-whatif --lib                   7 passed, 0 failed
cargo test -p poie-state --lib                    64 passed, 0 failed
cargo test -p poie-governance --lib               9 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，UI、迁移与 no-live 验收仍属设计审查而非运行证据。

VERDICT: CHANGES_REQUIRED
