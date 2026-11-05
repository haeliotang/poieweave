# 真实环境验收记录（2026-11-05，macOS）

## 1. 范围

针对 A–D MVP 的桌面真实环境验收：构建、启动、真实 app-data/SQLite、宿主文件/模型路径、完整门禁。  
不包含：真实外部 provider 的模型能力评测、物理设备、发布签名/公证。

## 2. 已完成证据

### 2.1 真实 UI 流程（2026-11-05）

已在真实 macOS app 中完成：

- 创建 operator `ADA ACCEPTANCE` 并登录（真实 PBKDF2/SQLite 会话）
- 配置本机 OpenAI-compatible mock provider（`http://127.0.0.1:8765/v1`），保存配置并写入审计
- Agent 面板发起问题：真实 host `agent_run` 完成，工具轨迹 `catalog.describe → answer.emit`，响应含 citation；RunState 写入 `agent_runs`
- Plan → What-if：创建快照、添加变体、运行 `structure`，结果显示 `candidate` + “relative to this static snapshot / not a physical feasibility proof” caveat
- 真实 app DB 验证：`whatif_snapshots=1、whatif_variants=1、whatif_results=1`，audit 出现 `whatif.snapshot/variant/run`

证据：

- [acceptance-agent.png](assets/acceptance-agent.png)
- [acceptance-whatif.png](assets/acceptance-whatif.png)

注意：provider 是**本机 mock**，只验证工具循环、引用与宿主持久化，不验证模型能力。

### 2.2 构建/后端证据

| 项 | 结果 | 证据 |
| --- | --- | --- |
| macOS app 构建 | 通过 | `pnpm tauri build --bundles app`，产物 `apps/desktop/src-tauri/target/release/bundle/macos/Poieweave.app`（7.13 MiB） |
| 启动 | 通过 | 实际启动 app，截图 [assets/poieweave-launch.png](assets/poieweave-launch.png)（3024×1964） |
| 真实 app data | 通过 | `~/Library/Application Support/com.poieweave.desktop/poieweave.sqlite3` |
| DB 迁移 | 通过 | `PRAGMA user_version = 4`；表：`agent_runs`、`approval_tokens`、`whatif_*`、`delivery_*`、`audit`、`operators` 等 |
| 宿主真实文件/SQLite/model 测试 | 16 通过 | v4 迁移 + `agent_runs`、D1 导入/回滚/诊断、C1 snapshot/variant/run/persist |
| 真实 UI：认证/provider/Agent/What-if | 通过 | 见 §2.1 与截图 |
| 完整门禁 | 通过 | `scripts/verify-all.sh` → `ALL GATES PASSED` |
| Rust 列表测试 | 417 | 库 401 + 其他 16 |
| UI smoke | 7 | Playwright（vite preview，非 Tauri host） |

## 3. 未完成/受阻

### 3.1 UI 自动化权限

System Events 仍返回 `-10004`，但已改用 CoreGraphics 事件 + 截图驱动真实 UI；认证/provider/Agent/What-if 已实际执行。以下流程因窗口切换/坐标不稳定尚未完成自动验收：

- Agent 动作草稿审批（approve/reject）
- What-if 的 plan/compare 分支
- Settings 站点交付：inspect/import/rollback/clear/diagnostics

需要二选一：

1. 给运行验收的终端/DSH 授予 macOS **辅助功能（Accessibility）** 权限，再继续自动驱动；或
2. 由人工按 §4 步骤执行并回报结果。

### 3.2 真实 provider

当前环境没有真实外部 provider key；已用本机 mock provider 验证 host 工具循环。真实模型能力验收需要：

- OpenAI-compatible endpoint + model + API key；或
- 明确同意用本机 mock provider（只能验证工具循环/审批/引用，不验证模型能力）。

### 3.3 其他边界

- 未做真实物理设备/多节点/签名公证。
- Playwright 只覆盖浏览器预览，不覆盖 Tauri webview。
- 真实 UI 验收发现并修复 What-if/Delivery 文案缺失（`whatif.*`/`delivery.*` i18n keys）；修复后重建 app 复核。

## 4. 人工验收步骤（macOS）

1. 启动 `Poieweave.app`。
2. Settings → 创建 operator（≥2 字符名，≥10 字符口令）→ 登录。
3. Settings → 配置 inference（HTTPS；本机 `http://127.0.0.1:PORT/v1` 允许）→ 测试调用成功。
4. Agent 面板：发送只读问题，确认出现工具轨迹与 citations；发送含动作草稿的请求，确认进入 `waiting_approval`，批准/拒绝后状态正确。
5. Plan 视图 → What-if：创建快照、添加变体、运行 structure/plan、比较；确认显示 caveat 且无 live/SAP 写入。
6. Settings → 站点交付：对 sandbox 下 `delivery/site.json` inspect → import（确认 sha）→ refresh/rollback/clear → export diagnostics；确认无“部署成功”文案、诊断无密钥。
7. 检查 `agent-audit.log` 的 prevHash/hash 链与 diagnostics 文件。

## 5. 结论

**真实环境验收大部分通过**：构建/启动/真实 DB/宿主文件/认证/provider 配置/Agent tool loop/What-if structure 已由实际 UI 运行和 16 个宿主测试验证。  
**尚未验收**：Agent 审批执行、What-if plan/compare、D1 交付 UI；真实外部 provider 能力未验证（仅 mock）。
