# Phase D（D1）设计门禁终验（独立）

- 校验对象：[D1](phase-d-site-delivery.zh-CN.md)（下称 D）、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)；代码 `apps/desktop/src-tauri`、`crates/poie-digest`、`tauri.conf.json`、`.github/workflows`、`scripts/`。
- 快照：branch `design/phase-d-d1`，HEAD **`028701c`**；写本文件前 `git status --porcelain` 干净；只读校验，只写本文件。
- 方法：核对 `028701c` 的 diff；逐条对照 D1 与 PLATFORM/SCHEMAS/DIGEST/README；核对 host 的 auth/db/sandbox/命令注册、Tauri bundle 目标、desktop workflow 的路径与 release job；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。未跑全量 cargo/Tauri/Playwright（D1 尚未实现，设计门禁不以未实现为失败）。
- 结论：**CHANGES_REQUIRED**。rescope 移除了 D2 的 channel 选择/签名/安装器设计，旧阻断 2（DeliveryPlan 无候选来源）随之闭合；但旧阻断 1/3/4/5/6 没有逐条闭合，且 D1 新增了状态/幂等/诊断矛盾。以下 6 组是 D1 阻断。

## 判定

1. **[阻断] 授权/审批/审计契约与平台相反，旧阻断 3 未闭合。** D:92/94 把 import/rollback 定为“写；登录；无审批”，D:97 明确“不引入 Window/HostPolicy/审批”；PLATFORM:89 仍规定 `delivery_*` 写需“会话 + 审批 + policy”，PLATFORM:94 要求每个命令 Window + `authorize(...)`，PLATFORM:97 要求 HostPolicy 与统一 `policy.check`，PLATFORM:166 要求 D 的 import 用同一 ApprovalToken；README:67 的 P2 也要求交付状态写入受 ApprovalToken/HostPolicy 约束并审计。实际 host 只有 `auth::require`（auth.rs:156），没有 `authorize`、`CommandError` 或 delivery 命令（lib.rs:260）；D:19 声称审计，但 §5/§6 没有审计事件、`CommandError` 映射或读写分权。D1 等于新开一条比平台契约更弱的写路径，且未修改 PLATFORM/README。需二选一：恢复审批/Window/policy/审计，或在 PLATFORM/README 显式登记 D1 例外并定义 import/rollback/state/diagnostics 的授权与审计。
2. **[阻断] manifest digest 与 DIGEST/SCHEMAS 不一致，TOCTOU 无法稳定实现。** D:36/49 用 `poie.site-manifest/0.1` 作 SHA domain，并排除扁平 `manifestSha256`；DIGEST:26/91 规定 `poie.manifest/0.1`，DIGEST:94 只排除 `integrity.manifest_sha256`；SCHEMAS:22 把 `poie.site-manifest/0.1` 登记为 wire schema，SCHEMAS:42 才把 `poie.manifest/0.1` 登记为 digest domain。同一 manifest 在两套契约下会得到不同 sha，`expectedManifestSha256`/`stale_manifest` 无法稳定；D 也未说明先对原始 JSON 验 digest 再反序列化，无法防重复键/尾随内容导致的 typed-reserialize 差异。另 D:49 拒绝所有未知字段，与 PLATFORM:19“未知 minor 可接受、忽略未知字段”冲突；D:52 声称“字符串长度/数组上限固定”却未给任何数值，bundle 也无独立大小上限。需统一一个 domain/排除字段、登记清楚，并给固定测试向量与具体上限。
3. **[阻断] DeliveryState 自相矛盾，rollback/recall 状态机未定义。** D:17 说 state 保存“当前/上一份完整 manifest 与摘要”，D:83 说“state 只存摘要”，D:84 又把 rollback 写成“previous 设为 current，current 清空/设回 previous 的前一份”，但结构只有 current/previous 两槽（D:72–73），不存在“previous 的前一份”；D:169 又说“交换 current/previous”。三种语义（清空、回退一层、交换）互斥，连续两次 rollback、current 为空、previous 为空均无规则。D:70–76 的 DeliveryState 只有 `schema`，缺 PLATFORM:20 强制要求的 `schemaVersion`；D:77–80 的 ManifestSummary 的 `importedAt/importedBy` 类型与来源未定义。D:74/85 的 `recalled` 是 state 级 bool，但 §6 导入步骤从不设置它，manifest 的 `channel:"recall"`（D:39）也没有映射规则；rollback 到 previous 后是否清除 recalled 未定义，故“禁止把该 manifest 当 current 使用”（D:85）没有执行点。需给出唯一的状态转移表（含二次 rollback、recall 进入/清除、current 空）和 schemaVersion。
4. **[阻断] “重复导入幂等”与事务步骤直接矛盾，冲突规则缺失。** D:116 第 4 步无条件 `previous=current; current=新摘要`，D:119 却要求同一 sha 重复导入“返回相同 state”。若同 sha 已是 current，第二次导入会把 previous 改成同一 sha（state 变化）；若同 sha 已在 previous，则 current/previous 互换。按 D:113–117 的实现不可能幂等。D:115 只允许“同 site/target 不同 sha”作为历史，但对不同 siteId/target、current 为 null、导入旧于当前 manifest 均无规则；D:98 的 `delivery_conflict` 没有任何触发条件。需在事务内先判 `sha == current.manifestSha256` 直接 no-op，明确 site/target 冲突与历史保留规则，并给重复导入/跨 site/旧 manifest 的测试。
5. **[阻断] 诊断契约自相矛盾，导出行为不可实现。** D:132 把完整 `DeliveryState` 放进诊断，而 DeliveryState 的 ManifestSummary 含 `siteId/siteName/importedBy`（D:78–79）；D:139–140 又说不得含用户名和站点原文、siteId/siteName 只留长度/哈希前缀。二者不能同时成立，且没有 redaction 算法/负例。D:141 要求超限写 `truncated=true`，但 D:128–136 的 DiagnosticsSummary 没有 `truncated` 字段。D:125 要求同名文件“覆盖需确认”，但命令签名只有 `outputDir`（D:95），没有 overwrite 参数，host 无法在“确认后覆盖”与“拒绝”之间表达。D:133 的 `checks.detail` 和 D:135 的 `logTail` 是自由文本，D:139 只给断言未给过滤规则；`generatedAt` 注释“caller-supplied now?”也未收口（D:130 vs D:142）。另外 inspect/import 的 `path` 与 diagnostics 的 `outputDir` 没有绑定现有 sandbox 的 relative/no-follow/1 MiB 限制（sandbox.rs:9/26/46/58），D:159 只说 host 读文件。需固定 DiagnosticsSummary（含 truncated）、redaction/错误码白名单、overwrite 语义和路径策略。
6. **[阻断] D1 的 deferral 没有落到路线图/发布契约，旧阻断 6 只是改名。** README:12 仍把 D 写成“通道…发布流水线”，README:94–95 要求 Phase D 先定义 release manifest 并让产物清单与 workflow 一致，README:119（R17）仍要求 D 增加 inspect/plan/recall/channel 并去掉 DMG/portable 误声明；PLATFORM:195 仍写 D1=manifest/model store/bundle、D2=channel/plan/recall/UI（D1 现在却含 state/rollback/diagnostics/UI 与 recall）。`git show --stat 028701c` 只改了 phase-d 与 SCHEMAS，未改 README/PLATFORM/DIGEST/status；`desktop-build.yml:146–147` 仍上传裸 `Poieweave`/`Poieweave.exe`，`:203/:206` 发布所有 `*.exe`，`tauri.conf.json:31` 仍是 `targets:"all"`，与 D:162“D1 不改 release workflow/安装器”并存；D:161 又要求改 desktop workflow 的 `crates/**` 路径，未说明这是否算改 release workflow，也没有 D2 的登记/门禁。SCHEMAS 还留着旧 `poie.diagnostics/0.1`（SCHEMAS:26）且 `site-bundle`/`delivery-state` 的 owner 仍是 D。需把 D1/D2 拆分写进 README/PLATFORM/status（含 D2 门禁），并明确 D1 是否触碰含 release job 的 `desktop-build.yml`；否则 Phase D 的既有验收项无人承接。

## 已执行检查

```text
git status --porcelain（写本文件前）                  clean
python3 scripts/check-docs.py                        OK, exit 0
python3 scripts/check-schema-registry.py             28 schemas, exit 0（未校验 digest domain 与 wire schema 的区别）
cargo metadata --no-deps --offline                   28 members, poie-delivery absent
git show --stat 028701c                              phase-d + SCHEMAS + VALIDATION-D only
```

边界：只读校验；未改源码/设计/git，只写本文件；未跑全量 cargo/Tauri/Playwright，也未把“D1 未实现”计为失败。上列 6 组均为设计契约/安全边界/状态语义/发布事实问题，不是施工细节。

VERDICT: CHANGES_REQUIRED
