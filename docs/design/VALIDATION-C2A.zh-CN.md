# Phase C2a 设计门禁（独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)、[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)/[PLATFORM](PLATFORM.zh-CN.md)、C1；代码 `poie-sim::timed`、`poie-ontology::time`、host `sandbox.rs`/`domain.rs`/`db.rs`。
- 快照：branch `design/phase-c2a`，HEAD **`e0cc495`**；写本文件前工作树 clean；只读校验，只写本文件。
- 方法：逐条复核 P1–P4；用 canonical JSON 重算 §4.3/§5.1/§5.3 三个向量；对照 `sandbox::resolve`、`Model::from_doc`、host `state.model`、DB v4；运行 docs/schema 检查与 whatif/timed lib 测试。
- 结论：**CHANGES_REQUIRED（3 个 C2a 阻断残留）**。P1 已闭合；P2/P3/P4 仍有残留。

## 已核对（非阻断）

- **P1 路径已闭合**：`scenarioPath` 明确为 sandbox 相对**文件**（C2a:45），sandbox 布局与 repo→app sandbox 复制已写（C2a:43/50）；`..` 仅在 `resolve_inside` 归一化后仍在 root 内时允许（C2a:46），§8 的“同目录 `../base-01` 合法”不再矛盾（C2a:164）。`base-01/model.toml` 的 pump-p1 本身已是 `degraded`，host 的同一 patch 为 no-op，`state.model` digest 相等成立。
- **scenario/result 向量正确**：`sha256("poie.timed-scenario/0.2\n"+raw)` 对 2925-byte scenario.toml 实测 `5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3`；result payload 实测 `e7e6e69a44e8b31bd01af60f754b85173aa2fe64b9b27c148c7cd010ce29d423`，与 C2a:58/113 一致；`snapshotSha256` 拼写已改。
- **其余 P4 项已核对**：compare 的 row-baseline 符号、baseline 不在 rows 时 warning、固定 verdict 顺序（C2a:127）；scenario list DTO（C2a:134）；TimedCompareResult `schemaVersion`（C2a:120）；`whatif_timed_snapshots.schema_version`（C2a:152）；C1 `poie.variant/0.2`/`poie.snapshot/0.2` 已从 SCHEMAS 删除；DIGEST §1 已列 timed 域。
- 检查：`check-docs.py` OK；`check-schema-registry.py` OK（33 schemas）；`cargo test -p poie-whatif --lib` 5/5；`cargo test -p poie-sim --lib timed` 6/6。

## 阻断（C2a BLOCKER）

1. **B3 残留 — snapshot 固定向量与自哈希规则仍冲突（seed）。** C2a:78 的规则排除 `seed`，但 C2a:80 的固定 payload 含 `"seed":7`；实测含 seed 才得 `34dd4a21a49004d208e630fab6ed174b49cf3f1ee33fef8e99e182983c8201ef`，按规则去 seed 得 `b4a1459860896c689d26c4ffd752cfb2d071891d63f4f3df07860a31505e9835`。C1 的 `snapshot_digest_excludes_identity_fields`（`crates/poie-whatif/src/lib.rs`:672/679）也明确断言 seed 不参与 digest。按 §5.1 实现的测试必失败；需删除 `seed` 排除项或重算向量。
2. **B2 残留 — TimedCase 的 now/start 与 variant start 冲突，deadline 规则重复。** C2a:94 允许 `beginning` start=0、`at` startS∈[0,horizonS]；C2a:36 却要求 runner `start_s∈[now_s,horizon_s]`。snapshot 允许 nowS>0（C2a:77），若 host 以 snapshot.nowS 填 `TimedCase.now_s`，则 beginning 或 startS<nowS 的合法变体在 run 被拒；`TimedCase.now_s` 的来源未写。另 C2a:96 用 `startS` 比较 deadline（beginning 可为 null），C2a:97 又改用 `start`，两条规则并存。
3. **B4 残留 — compare delta 的 null 语义未定义，schema 注册表仍漂移。** C2a:127 的 `delta.completionDeltaS = row.completionS - baseline.completionS` 未覆盖 completionS 为 null 的情况；`TimedOutcome` 的 blocked/unknown/search_limit/capacity_conflict 分支 completion_s=None（`crates/poie-sim/src/timed.rs`:427/437/447/465），C2a:127 的排序也承认 completionS null 最后，故 row/baseline 任一 null 时 delta 未定义。另 SCHEMAS:36–49 的 digest domain 表仍无 timed 域（DIGEST §1 在 ad722e9 已列，e0cc495 未改该表），且 SCHEMAS:22 的 `poie.timed-scenario/0.1` 行与实际 0.2 TOML/§4.3 的 0.2 raw 域并存但无设计使用。

## 边界

只读设计/源码；除本文件外未改任何文件。未运行 Tauri/Playwright，未验证真实 app sandbox/UI；`cargo test` 仅覆盖现有 lib。`sandbox::resolve_inside` 尚未出现在 `sandbox.rs`（现只有拒绝 `..` 的 `resolve`），实现前需新增；这不改变本门禁结论。C2b 延后声明不受影响。

VERDICT: CHANGES_REQUIRED
