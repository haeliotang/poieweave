# Phase D（D1）设计门禁复验（独立）

- 校验对象：[D1](phase-d-site-delivery.zh-CN.md)（D）、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)；代码 `apps/desktop/src-tauri`（auth/db/sandbox/lib）、`crates/poie-digest`。
- 快照：branch `design/phase-d-d1`，HEAD **`9f1c21b`**；写本文件前 `git status --porcelain` 干净；只读校验，只覆盖本文件。
- 方法：核对 `684cbaf..9f1c21b` 全部改动（4 文件：PLATFORM/README/phase-d/VALIDATION-D）；`DIGEST`/`SCHEMAS`/Phase A 未动。逐条复验 5 组声称修复；核对 host auth/db/sandbox 与命令返回类型；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。D1 未实现，不以未实现为失败。
- 结论：**CHANGES_REQUIRED**。已闭合：state 摘要化/`schemaVersion`/typed Summary/swap/`delivery_clear` 出口；README D 行/P2/退出条件/R17/状态；PLATFORM `poie-delivery` deps；diagnostics 相对路径/detailCode/overwrite/redaction；D1 授权例外主体。未闭合 4 组如下。

## 声称修复核对

| 声称 | 结果 |
| --- | --- |
| PLATFORM `delivery_*` D1 例外 auth+audit；README P2/exit/R17/status；deps digest/serde | 主体成立；§4 窗口列、`scope_denied`、错误码/包络、clear 审计仍有残差 |
| DIGEST D1 payload=SiteManifest 无扁平 sha、无 integrity | **不成立**：DIGEST/Phase A 未改，仍写 `integrity.manifest_sha256` |
| DeliveryState 只摘要 + `schemaVersion` + clear + conflict 出口 + swap | 主体成立；clear 无审计/测试/权限行 |
| path sandbox 相对、detailCode 枚举、overwrite 冲突、诊断脱敏 | 主体成立；`sandbox::resolve` 引用错、path 错误码/`errorCodes` 未列 |
| SCHEMAS 旧 diagnostics 删除 | **不成立**：SCHEMAS:26 仍在 |

## 阻断

1. **[阻断] DIGEST/Phase A 未改，D1 manifest 自哈希仍无唯一语义。** D:35–46 的 SiteManifest 只有扁平 `manifestSha256`（D:45），无 `integrity` 对象；D:49 排除 `manifest_without_manifestSha256`。DIGEST:26/91 仍写 `canonical_json(manifest_without_integrity)`，DIGEST:94 仍称“`integrity.manifest_sha256` 是唯一排除字段；其他字段（包括 artifacts/evidence/contracts）全部纳入”；Phase A:153 同。`9f1c21b` 只改 4 文件，DIGEST/Phase A 未动；按 DIGEST 实现会把待算的扁平字段纳入自哈希（或排除空集），所得 sha 与 D:49 不同，`expectedManifestSha256`/`stale_manifest` 无唯一实现。DIGEST §6.5 的固定 fixture 向量也未补。另 D:50 拒绝未知 minor，PLATFORM:19 允许未知 minor 并忽略字段；D 单方面称 PLATFORM §2 不适用，但 PLATFORM 未登记该例外。

2. **[阻断] `delivery_clear` 的写路径没有审计/测试/权限闭合。** D:88/100 新增 clear（写；登录），但 D:107 审计事件仍只有 inspect/import/rollback/diagnostics，没有 `delivery.clear`；PLATFORM:89 要求 `delivery_*`“会话 + 审计”，README P2 要求配置写有审计。D:187 测试与 D:191 验收均无 clear，README:83 却把“回滚/clear 可测”列为 D1 退出条件。D:101 的 diagnostics 行只写“写；登录”，未像 import/rollback/clear 标注“无审批”，与 D:103 的 D1 无审批不一致。

3. **[阻断] D1 授权例外仍留 scope/Window 矛盾，错误契约未落到 host。** PLATFORM:89 的允许窗口仍写 `main/settings`，PLATFORM:94 与 D:103 又说 D1 不引入 Window；D:104 仍列 `scope_denied`（PLATFORM:50 定义为窗口/命令 scope 不匹配），D1 无 scope 机制，该码不可达。D:104 的 `manifest_invalid`/`stale_manifest`/`delivery_conflict`/`rollback_unavailable`/`io_error` 也未登记到 PLATFORM §3 的稳定错误码表。host 只有 `auth.rs:156 require -> Result<_, String>`，`lib.rs` 命令仍返回 `Result<_, String>`；D:103 的“统一 `{code,message,details}`”没有 `unauthorized`/`scope_denied`/`io_error` 的映射规则。需删除/改写 PLATFORM:89 窗口列，消解 `scope_denied`，登记 D1 错误码，并定义 `auth::require` 错误到 `CommandError` 的映射。

4. **[阻断] SCHEMAS 旧 diagnostics 未删；路径事实与 README 范围仍有残差。** `9f1c21b` 未改 SCHEMAS；SCHEMAS:26 仍登记 `poie.diagnostics/0.1`（owner D），与 SCHEMAS:25 `poie.delivery-diagnostics/0.1` 重复；“SCHEMAS stale diagnostics removed”不成立。D:158 称复用 `sandbox::resolve` 的 no-follow，但 `sandbox.rs:11–24` 的 resolve 只拒绝非 relative；symlink no-follow 在 `sandbox.rs:26–44` 的 `reject_symlinks`/`read`/`write`，按引用实现会漏 symlink 检查。D:158 也未给 path 违规的错误码（`invalid_parameter`? `io_error`?），D:150 `errorCodes` 的“固定白名单”未列值。README:94–95 仍要求 Phase D 先定义 release manifest/产物清单，而 D1 已把 release 推给 D2；README:133 的“D1 设计门禁 PASS 后进入实现”在门禁未过时不宜作为当前状态。

## 已执行检查

```text
git rev-parse --short HEAD                         9f1c21b
git diff 684cbaf..9f1c21b --stat                   4 files: PLATFORM/README/phase-d/VALIDATION-D
git status --porcelain（写前）                      clean
python3 scripts/check-docs.py                       OK, exit 0
python3 scripts/check-schema-registry.py            28 schemas, exit 0
cargo metadata --no-deps --offline                  28 members, poie-delivery absent
host                                               auth.rs:156 require; db.rs:135 user_version=3; sandbox.rs:11 resolve, :26 reject_symlinks
```

边界：只读校验；除本文件外未改任何文件；未跑全量 cargo/Tauri/Playwright。D1 尚未实现，未以未实现计为失败。

VERDICT: CHANGES_REQUIRED
