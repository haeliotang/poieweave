# Phase D（站点交付）设计门禁终验（独立）

- 校验对象：[Phase D](phase-d-site-delivery.zh-CN.md)（下称 D）、[DIGEST](DIGEST.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)、[Phase C](phase-c-what-if.zh-CN.md)；代码 `apps/desktop/src-tauri`、`apps/desktop/src`、`crates/poie-governance`、`crates/poie-digest`、CI 与 `scripts/`。
- 快照：`main` HEAD **`cb93e34`**；写本文件前工作树干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：核对 D 与 PLATFORM/SCHEMAS/DIGEST/README/C；核对 host 的 auth/db/sandbox/agent/inference、`tauri.conf.json`、`desktop-build.yml`/`ci.yml`、workspace/test-matrix；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。未跑全量 cargo/Tauri/Playwright（设计门禁不要求 D 已实现）。
- 结论：**CHANGES_REQUIRED**。D 的边界（仅元数据 JSON、不导入 app/模型、无 updater/遥测、SHA-256 明示非签名、默认保守）成立；但以下 6 组是 D 契约阻塞，不能带进 D1 开发。

## 判定

1. **[阻断] DeliveryState/回滚没有持久化模型，原子性与幂等未定义。** D:96–103 的 `DeliveryState` 只保存 `ManifestSummary`，回滚（D:123）却要恢复完整 manifest；文档没有 manifest 表/行、`previous` 保留策略、`user_version` 目标或迁移步骤（D:254 只列测试，`db.rs:135` 仍固定 `user_version=3`），也没有导入的单事务/失败边界（D:188 的“失败不改状态”只是测试口号）。重复导入同一 bundle、连续两次回滚、导入旧于 installed 的 bundle、`bundleId`/`createdAt` 的生成与比较均无规则；D:123 还把“数据库迁移失败”与清单回滚混在一起，而 D 不替换二进制，迁移不应由导入触发。最小收口：给出 `delivery_manifests`/`delivery_state` DDL、v3→v4 迁移与备份、单事务写序、重复导入 no-op、`previous` 单槽交换/保留语义。

2. **[阻断] DeliveryPlan 没有候选来源，且未登记 schema。** 离线场景下 `delivery_plan(desiredChannel?)`（D:167）没有任何路径或存储提供 `candidateManifest`：`delivery_bundle_inspect` 不落盘，导入又直接安装，没有“候选清单”表或 inbox；因此 D:27 的“解释为什么没有更新”无法实现。`DeliveryPlan` 无 `schemaVersion`，`poie.delivery-plan/0.1` 未在 SCHEMAS 登记；`ManifestSummary` 从未定义；D:121 说字段用 `adoptManifest`，结构里只有 `actions`；`constraints/actions` 的 id/kind 枚举与确定性排序未定。最小收口：定义 plan 输入（bundle path 或 inspect 返回的 candidate 句柄）、`ManifestSummary`、plan schema 与稳定排序。

3. **[阻断] 审批与命令 scope 的安全契约未定义。** D:177 只写“写命令要求登录 + ApprovalToken”；但 PLATFORM:143–160 的 token 需要 `run_id/call_id/tool`，D 没有 run/call，也没有为 import/channel/policy/recall/rollback 创建、批准 pending token 的命令；现有 `agent_approve`（`agent.rs:558–571`）批准后立即执行，不能直接复用。导入的 subject digest（bundle sha256、channel、policy、旧 installed）未定义，inspect→import 存在 TOCTOU；token 的 session 绑定也未写入 D。另 `CommandError` 与 `authorize(window,command,session,policy)` 在仓库中不存在（PLATFORM:94、§3），D 用 `DeliveryError` 且命令表没有 Window；D 也未说明 `delivery_status/plan/inspect/diagnostics` 是否要求登录，而 PLATFORM:89 把全部 `delivery_*` 当写入。最小收口：定义 delivery approval subject/创建/消费流程、`CommandError` 映射、每个命令的 Window+`authorize`，并把读命令单列。

4. **[阻断] Manifest/Site Bundle/schema/digest 自相矛盾。** D:58 把 `poie.site-bundle/0.1` 当作 manifest 的 schema，但 D:25 的 bundle 是“manifest + 可选诊断”；没有 bundle envelope，`ManifestSummary` 未定义；D:76 的 `diagnostics.inline` 没有 inline=true 的形状，也未说明诊断是否随 bundle 导入。D:62 的 `app.target` 是单值，D:75 的 artifacts 可多 OS，二者无法同时成立；示例只列 macOS 产物。D:85 允许忽略未知 minor 字段，但 D:84/DIGEST:91 的自哈希覆盖这些字段；`poie-digest::sha256_of` 对类型化值序列化会丢未知字段，必须明确“先对原始 JSON 验 digest，再反序列化”。`poie.manifest/0.1` 只是 digest domain，不是 wire schema；`DeliveryPlan`/`ManifestSummary`/`HostPolicy` 未登记；`contracts.toml` 只有一句，无 schema/生成器/黄金样例；`release.json`/`RELEASE.json` 大小写不一致（D:87/200/275）；D:35 的 `ModelStore` 与 PLATFORM:110 的“不存在 ModelStore”冲突。最小收口：定义 bundle/manifest 两个 DTO 与 schema、目标/产物关系、raw-JSON 验 digest 顺序、contracts/release 文件 schema 与命名，删除 ModelStore。

5. **[阻断] 未签名 bundle 可改写策略，召回与诊断未收口。** D:25/73/170 的 bundle 携带 policy，导入会更新 `DeliveryState.policy`；D:36 只把 SHA-256 标为“未认证”，D:283 仍把“导入策略覆盖 Agent scope”留作开放问题，没有“只能收紧、不能放宽”的下限（尤其 remoteInference/agentScopes）。`HostPolicy` 只有 PLATFORM:97 一行 JSON，没有类型/校验/schema/版本，且 `channel` 同时出现在 HostPolicy 与 DeliveryState（D:97–105）形成双源。D:28/122 的召回一处说“阻止继续使用”，一处只说“显示召回+允许回滚”，没有执行点，也没有解除召回的命令。D:175 的 `delivery_diagnostics_export` 返回内联 JSON，D:218 却叫“导出”，没有写文件/大小上限；D:133 的 recentErrors 只列 `at/code/event`，但未定义 `code` 的固定枚举，也未禁止把 audit 的自由文本拼进 event；D:139 只按字段名扫描 key/token/secret，无法防止值内泄漏；`generatedAt` 使诊断不可复现且无 digest。最小收口：规定 policy 只可收紧、召回在 import/plan/rollback 的拒绝点与解除流程、诊断写 sandbox + ≤256 KiB + 固定错误码白名单（不含 audit.detail）+ 负例。

6. **[阻断] 发布流水线与 D 的事实/门禁不一致。** `desktop-build.yml:143–148` 上传 `target/release/Poieweave.exe`，`:203–207` 把所有 `*.exe` 发布，因此当前 tag 会发布便携 exe；D:201 却写“不生成 DMG/便携 exe”，manifest 产物清单也会漏项。`tauri.conf.json:31` 的 `bundle.targets = "all"` 使手动构建默认产出 DMG，和 D:201 的“不生成 DMG”不符。`release` job 只有 `needs: build`（:185），没有根 verify；`ci.yml` 无 tag/`workflow_call`；D:202 要求的 `needs: [build, verify]` 尚未落地。桌面 workflow 的 paths（:7–29）不含 `crates/**`，D 新增 `poie-delivery` 后只改 crate 不会触发桌面构建；`tests/**`/`playwright.config.ts` 重复两遍，D:203 只提“去重/补齐”。`check-release-manifest.py`、版本一致性检查、`poie-delivery` 的 members/Cargo.lock/test-matrix/ADR 仍是待办；D:269 的步骤 0 还把已存在的 `poie-digest` 当新 crate。最小收口：先冻结发布产物集合（含/不含 raw exe、DMG），再让 manifest 清单、workflow、`check-release-manifest.py` 与 `tauri.conf.json` 四者一致；tag 先跑根 verify；补 `crates/**` 路径与版本/矩阵校验。

## 非阻断（首个 D PR 收口）

- `write_atomic`（D:186）与现有 `sandbox::write` 重叠；后者未 fsync 父目录、未拒硬链接，D 应复用并补齐，而不是复制两份。
- `sandbox::MAX_FILE_BYTES = 1 MiB` 与 D 的单文件 256 KiB/bundle 1 MiB 不冲突，但 diagnostics 的导出上限仍缺。
- `DeliveryState.schemaVersion` 的字面值、`Diagnostics.schema` 字段、`poie.delivery-state/0.1` 的持久化格式未写；建议直接给出固定 JSON fixture。
- D:196 要求导入时与“当前”contracts/evidence 比较，但 host 只有 model digest，没有 contracts/evidence 的运行时注册表或编译期嵌入方式；需明确，否则只能做结构校验。
- Playwright 只测 host-only/禁用与现有 `isTauri()` 模式一致（D:218、D:258），保留即可。

## 已执行检查

```text
git rev-parse --short HEAD                         cb93e34
git status --porcelain（写本文件前）                clean
python3 scripts/check-docs.py                      OK, exit 0
python3 scripts/check-schema-registry.py           26 schemas, exit 0
cargo metadata --no-deps --offline                 poie-delivery 不存在（符合设计前状态）
```

边界：只读校验；未跑全量 cargo/Tauri/Playwright，未把“D 未实现”计为失败。上表 6 组均是设计契约/安全边界/发布事实问题，不是施工细节。

VERDICT: CHANGES_REQUIRED
