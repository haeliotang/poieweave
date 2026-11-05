# Phase C2b 设计门禁（独立复验）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`/`plan`、`poie-state::review`/`action`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)、[traceweave 输入清单](traceweave-adoption-inputs.zh-CN.md)。
- 快照：branch `design/c2b`，HEAD **`6771fa5`**；写本文件前 `git status --porcelain` 为空；只读校验，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条复核 2e714e6 的 5 个阻断与 6771fa5 的修订，对照设计 §5–§13 与 `sap.rs`/`plan.rs`/`review.rs`/`action.rs`/`whatif lib.rs`/`db.rs`/`governance lib.rs`/PLATFORM；运行 docs、schema 与三个 lib 测试。
- 结论：**CHANGES_REQUIRED**（4 个 C2b 阻断；未达到“零阻断 PASS”）。

## 前一轮 5 项状态

- 1（commit 持久化）：**已闭合**。§10:224–227 新增 `promotion_commits`，`commit_json`/`commit_sha256`/`inputs_sha256`/`token_id`/`committed_at`/`committed_by` 可落；重复 commit 可返回已存 commit。propose 的部分写仍见阻断 4。
- 2（公开 API/时间基）：**已闭合**。§5.2:102 与 §6.1:118 均为四参数；`origin_now_s = snapshot.nowS`，C1 `run_plan` 确以 `snapshot.now_s` 为 cursor（`whatif lib.rs:411`、`plan.rs:329`）。措辞残留：§6.1:118 的“不得由 callers 另传”与公开参数矛盾，§6.1:121 的 `now_ms` 应写 `proposal_now_ms`，不单独计阻断。
- 3（preview assumed）：**部分闭合**。§6.1:125 已把 ownership/reservations/predicates 设为 true/并集，但 TimeWindow 仍会自拒，见阻断 1。
- 4（digest）：**部分闭合**。plan/variant/inputs/commit 已换独立 domain 并补 `\n`；`actionSha256` 仍无公式，见阻断 2。
- 5（授权/no-live/CommandError）：**部分闭合**。window 已改为 `poie-view-plan`，no-live 断言已具体化；但 `command` 仍是 `whatif.promote`，见阻断 3。

## 阻断

1. **BLOCKER：`preview_actions` 仍会把正常多动作计划判成 `sap_rejected`（TimeWindow `not_yet_valid`）。** §6.1:125 固定 `now_ms=proposal_now_ms`；§5.2:104 把每个动作的 `valid_from` 重定为 `proposal_now_ms + (start_s-origin_now_s)*1000`。任何 `start_s > origin_now_s` 的动作（多动作计划的第 2 个起）都会触发 `review.rs:83-88` 的 `not_yet_valid`；`sap.rs:228-267` 把 TimeWindow 标为 `failed` 并写入 `rejections`，而 §6.1:126 只允许 `missing_authorization`，其余一律 `sap_rejected`。修正：把未来时间窗标为 assumed/预期（仅 `valid_until <= now` 的 expired 才算拒绝），或按各动作 `valid_from` 分别审查；补多动作计划正常通过 preview 的测试。

2. **BLOCKER：`actionSha256` 仍未定义，且单动作/`proposalSha256` 的覆盖集不闭合。** §5.2:96 与 §10:221 要求持久化 `actionSha256`/`action_sha256`，但全文无 domain/payload，DIGEST:30 只登记 `poie.proposed-action/0.1` 名称；§5.1:77 的 `actionsSha256` 是对 draft 数组的 digest，不能替代单动作 digest。另 §5.1:79 排除 `times` 却又显式包含 `nowMs`，`promotion_proposed_actions.action_json` 存 draft 还是含 `promotionEvidence` 的 `ProposedAction` 也未写明。修正：定义 `actionSha256` 的公式、排除字段与表内 JSON 类型（并与 draft/`actionsSha256` 对齐），解决 `times`/`nowMs` 的歧义，补固定 digest 向量。

3. **BLOCKER：`authorize` 的 `command` 仍不是实际命令，且 token 的 policy version 与 subject digest 不一致。** §7:168 写 `authorize(..., command="whatif.promote", ...)`；实际命令是 §7:156–162 的 `whatif_promote_*`，PLATFORM:90 也以 `whatif_*` 为 command，`whatif.promote` 仅是 token tool（§6.2:135）。`authorize` 只有一个 `command` 参数，没有独立 policy scope 参数；按字面实现会以不存在的命令被 `scope_denied`，或丢失逐命令粒度。另 §6.2:137 用 `POLICY_VERSION`（`governance lib.rs:17` 为 `poie-agent/0.1`）计算 `subject_digest`，§6.2:142 却把 token `policy_version` 设为 `poie-promote/0.1`；按前者重算会导致审批绑定不一致。修正：`command` 传实际 `whatif_promote_*`，另设 policy/tool scope `whatif.promote`；`subject_digest` 显式传 `poie-promote/0.1`，并写明审批时重算比对；no-live 还需补命令注册表与仅新增三张表的可执行断言。

4. **BLOCKER：propose 的 proposal+token 写入未声明同事务，重复/过期状态也无原子约束。** §6.2:134 在 propose 创建 token，但 §9:201 只覆盖 approve/reject/cancel，§9:202 只覆盖 commit；proposal 与 token 两次插入之间崩溃会留下无 token 的 pending proposal（或反向孤儿），与 §9:203 “pending/approved 保留”冲突。§6.2:148 的 duplicate 检查是 check-then-insert，§10:216 的 `promotion_proposals` 没有 `(result_id,status)` 的 partial unique 约束；并发 propose 可创建两个 pending。§9:203 的过期只写 token，proposal 的 `expired` 状态在运行期无更新路径。修正：propose 单事务插入 proposal+token，加 pending/approved 的 partial unique 索引，定义运行期过期时 token 与 proposal 同步置 `expired` 并审计，补崩溃/并发测试。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          38 schemas, exit 0
cargo test -p poie-plan --lib                     19 passed, 0 failed
cargo test -p poie-whatif --lib                   7 passed, 0 failed
cargo test -p poie-state --lib                    64 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，因此 UI、迁移与 no-live 验收仍属设计审查而非运行证据。

VERDICT: CHANGES_REQUIRED
