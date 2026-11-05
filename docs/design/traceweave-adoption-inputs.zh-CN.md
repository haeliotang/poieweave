# D2/C2b 设计输入清单：从 traceweave 选择性吸收

- 状态：设计输入（不是门禁报告，不代表已实现）
- 日期：2026-11-06
- 来源项目：`/Users/hao/Documents/traceweave`（个人计算 substrate）
- 目标：把 traceweave 的治理/发布工程经验映射到 Poieweave D2（站点交付发布）与 C2b（timed promote/审批）
- 原则：只吸收语义与证据纪律；不复制 Android/Binder/Keystore/GGUF 实现

## 1. 结论

有选择地有帮助。Poieweave 已有审批 token、审计链、digest 与证据文化；traceweave 更成熟的是：

1. **签名材料保管 + 恢复演练**（不是“有备份”，而是演练通过）
2. **发布校验产物 + 回滚边界**（错误签名/版本回退必须失败）
3. **能力授权/身份绑定/撤销**（task-scoped grant，而不是单次弹窗）
4. **档案携带审计链头 + 删除墓碑 + 冲突审核**
5. **崩溃后 outcome_unknown，不自动重放**

## 2. 总表

| ID | 输入 | 主要来源 | 进 D2 | 进 C2b | 优先级 |
| --- | --- | --- | --- | --- | --- |
| D2-1 | 签名档案/口令/恢复卡三件套 + 定期恢复演练 | signing_custody_checklist | ✅ | — | P0 |
| D2-2 | 发布校验产物（实际签名、版本、摘要） | agent_os_release_process | ✅ | — | P0 |
| D2-3 | 发布通道/回滚兼容/禁止降级静默 | release_process、release_checklist | ✅ | — | P0 |
| D2-4 | bundle 携带 auditHead/auditCount，导入重算 | milestones 2026-10-31 | ✅ | — | P1 |
| D2-5 | 删除墓碑，旧 bundle 不复活已删数据 | auto_sync_design | ✅ | — | P1 |
| D2-6 | 外部审计锚（Keychain HMAC 链头） | milestones 2026-10-31 | ✅ | — | P1 |
| D2-7 | 档案格式版本/兼容迁移 | release_process、ArchiveCompat | ✅ | — | P1 |
| D2-8 | 发布证据包与变更日志纪律 | release_process、CHANGELOG | ✅ | — | P2 |
| C2b-1 | capability grant：scope/任务级授权，非单次调用 | agent_os_android_binder_mvp | — | ✅ | P0 |
| C2b-2 | grant 绑定身份/签名指纹/operator | 同上 | — | ✅ | P0 |
| C2b-3 | revoke/expiry/重启失效/outcome_unknown | 同上、milestones | — | ✅ | P0 |
| C2b-4 | parameterized SAP validation + Authorization.basis | Poieweave B/C 契约 | — | ✅ | P0 |
| C2b-5 | plan_to_proposed + promote 审批链 | Poieweave C2b | — | ✅ | P0 |
| C2b-6 | 审计归因（谁批准了什么 scope） | traceweave broker | — | ✅ | P1 |
| C2b-7 | capability inventory/最小权限 | system_capability_inventory | — | ✅ | P1 |
| C2b-8 | 授权/撤销 UI | traceweave approval UI | — | ✅ | P2 |
| X-1 | vector clock/冲突逐条审核 | auto_sync_design | 未来联邦 | 未来联邦 | 暂缓 |
| X-2 | 崩溃/重启不自动重放 | milestones reboot 记录 | 验收共用 | 验收共用 | P0 |
| X-3 | 证据收据/限制随记录保留 | release_process | 全阶段 | 全阶段 | P0 |

## 3. D2 设计输入

D2 仍按 [Phase D](phase-d-site-delivery.zh-CN.md) 的 channel/签名/安装器范围；以下只补“发布工程”缺口。

### D2-1 签名保管与恢复演练（P0）

必须定义：

- 三件套：加密签名档案、口令、恢复说明卡
- 口令与档案分开存放；仓库/聊天/日志/截图不出现机密
- 恢复演练脚本 + 收据：认证 → 解密 → 指纹 → 签测试产物 → 验签 → 清理
- 只有演练 `verified=true` 才算“可恢复”
- 触发重做：换机、重装、口令位置变化、密钥轮换、分发渠道变化

验收证据：恢复演练收据（不含机密），错误口令/缺失档案必须失败。

### D2-2 发布校验产物（P0）

候选发布必须生成机器可读 verification：

- 实际签名证书 SHA-256、包名/版本递增、产物摘要
- 与期望 signer/上一版摘要比对；错误签名/版本回退失败
- 缺少 verification 产物不得视为发布通过
- 测试密钥只能临时生成，不进入仓库/Release

验收证据：错误 signer、错误版本、输入变更均失败；正确候选生成 verification。

### D2-3 通道/回滚边界（P0）

- 通道（stable/beta）与召回/回滚策略
- 不允许静默降级；旧数据格式兼容性先评估
- bundle 版本与 app 版本的关系
- 回滚仅限签名一致、版本递增的修复版本；不卸载绕过

验收证据：回滚演练、旧 bundle 兼容、拒绝降级路径。

### D2-4 bundle 审计链头（P1）

- site bundle 写 `auditHead/auditCount`
- 导入时重算并比对；不一致拒绝
- UI 显示链头前 16 位，提示用户外部留存
- 明确边界：链头在 bundle 内，能改行的人也能改链头；外部锚才提高保障

验收证据：篡改一行后导入拒绝；链头显示；截断尾部行为明确。

### D2-5 删除墓碑（P1）

- bundle 删除用 tombstone/删除标记表达
- 旧 bundle 导入不复活已删除数据
- 删除传播与导入顺序有测试
- 不做自动多节点同步（未来）

验收证据：旧 bundle 导回后已删数据仍删除；删除标记可审计。

### D2-6 外部审计锚（P1）

- 本地 Keychain 不可导出密钥 HMAC `idx|hash|time`
- 导出 bundle 时携带锚定信息或锚定摘要
- 校验页显示“链完整 + 锚定数量”
- 不宣称防篡改；截断尾部仍是较短合法链

验收证据：锚定缺失/不匹配可检测；迁移旧行可惰性补锚。

### D2-7 档案兼容迁移（P1）

- bundle schema 版本化；旧版本导入路径
- 兼容层只接受已知旧格式；未知版本拒绝
- 迁移不继承旧授权/URI/凭据
- 迁移夹具 + 真实旧 bundle 测试

验收证据：旧 bundle → 新版本恢复；错误版本拒绝；不继承授权。

### D2-8 发布证据/变更日志（P2）

- CHANGELOG、release notes、候选/设备/回滚收据
- 修正历史证据时保留原件，另出带日期新测量

## 4. C2b 设计输入

C2b 的核心不是“再弹一次审批”，而是把**一次性工具审批**升级为**有范围、可撤销、可归因的授权**。

### C2b-1 capability grant（P0）

- grant 绑定 `scope`（site/plan/scenario/action family）
- 明确允许/拒绝的工具与效果
- grant 不是永久权限；有 expiry 与 revoke
- promote 使用 grant + plan digest，不直接复用 agent run token

### C2b-2 身份/签名绑定（P0）

- grant 记录 operator、session、身份/签名指纹
- 身份变化使 grant 失效
- UI 显示“谁在授权什么范围”

### C2b-3 撤销/过期/重启（P0）

- revoke 立即生效；撤销后调用拒绝
- 过期 token/grant 不可用
- 重启后授权失效，不自动重放
- 执行中断 → `outcome_unknown`，不自动重试

### C2b-4 参数化 SAP validation（P0）

- promote 前用显式计划/参数验证 SAP，而不是固定 basis
- `Authorization.basis` 绑定 plan fingerprint、scope、operator、时间
- 验证失败不得 promote

### C2b-5 promote 审批链（P0）

- `plan_to_proposed` → 审批 → proposed plan
- 审批对象是计划/动作范围，不是单次工具调用
- 状态机：draft → waiting_approval → approved → proposed / rejected / expired
- 审计记录每一步

### C2b-6 审计归因（P1）

- 记录 grant id、scope、operator、plan digest、approve/reject 原因
- 审计可回答“谁批准了什么、范围多大、何时失效”

### C2b-7 capability inventory/最小权限（P1）

- 列出可用能力/工具/效果
- 默认最小权限；dispatcher/analyst 边界保持
- 高风险动作需显式 grant

### C2b-8 授权 UI（P2）

- 授权范围、期限、撤销入口
- 不能只显示“批准/拒绝”而不显示范围

## 5. 跨阶段输入

### X-1 多节点/冲突（暂缓）

vector clock、逐条冲突审核、tombstone 优先级属于未来 federation；D2 只做单站点 bundle 的删除墓碑，不做自动同步。

### X-2 崩溃/重启不自动重放（P0）

D2 安装/回滚和 C2b promote 都必须测试：

- 重启前 `executing` → `outcome_unknown`
- 不自动重放
- 授权/approval 重启后失效
- 文件/DB 状态可解释

### X-3 证据纪律（P0）

每个门禁/发布保留：命令、日期、环境、限制、失败路径；不把自动通过推断为真机/物理验收。

## 6. 明确不吸收

- Android APK/Binder/AIDL/Keystore/Pixel 设备代码
- 端侧 GGUF 模型运行时
- 个人记忆库、语音/视觉/Overlay
- traceweave 的“daemon 禁用/实验协议”未完成状态
- 任何把 Poieweave 变成个人 OS 的范围扩张

## 7. 依赖与顺序

1. **C2b 设计门禁**：C2b-1..7 必须进设计；C2b-8 可后置
2. **D2 设计门禁**：D2-1..7 必须进设计；D2-8 可后置
3. C2b 先行的理由：D2 的远程/策略写若需要审批，可复用 C2b grant 模型
4. D2 可与 C2b 并行设计，但实现前必须先定签名/回滚边界

## 8. Poieweave 落点

| 输入 | 落点 |
| --- | --- |
| D2-1/2/3 | `docs/design/phase-d-site-delivery.zh-CN.md` D2 节 + 新 release 设计 |
| D2-4/5/6/7 | `apps/desktop/src-tauri/src/delivery.rs`、`db.rs`、bundle schema |
| C2b-1..7 | `agent.rs` 审批 token、`db.rs`、新 `whatif` promote 命令、`poie-plan`/`poie-exec` |
| X-2 | `agent_approve/resume/reject`、execution 状态机测试 |
| 审计 | `db.append_audit` + 新锚定表/导出字段 |

## 9. 参考

- `/Users/hao/Documents/traceweave/docs/signing_custody_checklist.md`
- `/Users/hao/Documents/traceweave/docs/agent_os_release_process.md`
- `/Users/hao/Documents/traceweave/docs/production_signing_decision.md`
- `/Users/hao/Documents/traceweave/docs/release_checklist.md`
- `/Users/hao/Documents/traceweave/docs/agent_os_android_binder_mvp.md`
- `/Users/hao/Documents/traceweave/docs/agent_os_milestones_2026-10-30.md`
- `/Users/hao/Documents/traceweave/docs/auto_sync_design.md`
- `/Users/hao/Documents/traceweave/docs/local_device_archives.md`
- `/Users/hao/Documents/traceweave/docs/crash_reporting_2026-10-31.md`
- [Phase D](phase-d-site-delivery.zh-CN.md)、[Phase C2a](phase-c2-timed-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)
