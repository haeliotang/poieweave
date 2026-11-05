# Phase C2b 设计门禁（最终独立复验）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`/`plan`、`poie-state::review`/`action`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)。
- 快照：branch `design/c2b`，HEAD **`edcf5d6`**；写本文件前 `git status --porcelain` 为空；只读源码/设计，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条复核 9d86071 的 2 个阻断与 edcf5d6 的修订，对照设计 §5–§13、`sap.rs`/`review.rs`/`governance lib.rs`/`whatif.rs`/`db.rs`/DIGEST/SCHEMAS/PLATFORM；运行 docs、schema 与四个 lib 测试。
- 结论：**PASS（0 个 C2b 阻断）**。

## 三项复核

1. **actionSha256/action_json：已闭合。** §5.2:103 明确 `actionSha256 = sha256("poie.proposed-action/0.1\n" + canonical ProposedActionDraft)`，`action_json` 存 draft、`action_sha256` 同式、读取时重算；§5.1:77 的 `actionsSha256` 是 draft 数组 `poie.proposed-action-draft/0.1`，§10:225–226 的列与 §10:238 的校验一致；两个域均已登记（DIGEST:31/36）。
2. **proposal digest 覆盖集：已闭合。** §5.1:79 明确排除 `requestedAt/decidedAt/committedAt`，保留 `nowMs`、actions/actionsSha、scope、stateVersion、SAP preview；状态/审批时间变化不改 digest，token `args` 绑 `proposalSha256`，无 `times`/`nowMs` 冲突。
3. **subject_digest/policy/白名单/过期同步：已闭合。** §6.2:141 与 §6.2:146 显式使用 `poie-promote/0.1`（`subject_digest` 接受 policy 字符串，`governance lib.rs:364`；`POLICY_VERSION` 仍为 `poie-agent/0.1`，但 C2b 不再引用）；§7:172 的 no-live 允许列表含 `whatif_promote_*`/`promotion_*`；§9:207 把 propose 纳入过期同步。

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

## 非阻断备注

- `actionSha256` 用 `poie.proposed-action/0.1` 覆盖 draft，而 `actionsSha256` 用 draft 域；两域均已登记、公式可复算，属命名不对称，不构成歧义。
- §5.1:79 的排除项写 `id`（结构字段为 `proposalId`）；按 `proposalId` 处理即可，建议实现时统一命名。
- `no_live_write` 白名单按命令/表前缀列；`append_audit` 是 §12 必需的审计写，测试应将其作为允许的审计 sink（非 live 写），不阻断。

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，UI、迁移与 no-live 验收仍属设计审查而非运行证据。

VERDICT: PASS
