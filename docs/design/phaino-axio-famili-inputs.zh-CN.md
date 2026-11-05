# D2/C2b 设计输入清单：PhainoWeave / AxioWeave / FamiliWeave

- 状态：设计输入（不是门禁报告，不代表已实现）
- 日期：2026-11-06
- 目标：把另外三个项目的可复用机制映射到 Poieweave 的 D2（发布/交付）与 C2b（promote/治理）
- 基线：Poieweave `v0.3.1`（C2a、C2b 已实现；D2 尚未开始）
- 同系列参考：[traceweave 输入清单](traceweave-adoption-inputs.zh-CN.md)

## 0. 结论

| 项目 | 价值 | 主要吸收方向 | 优先级 |
| --- | --- | --- | --- |
| **PhainoWeave** | 最高，直接相关 | 验证优先、证据图、residual/abstention、run replay、内容寻址 | P0 |
| **AxioWeave** | 高，偏治理 | mandate/scope、职责分离、三路审计、动作注册表、case packet | P0/P1 |
| **FamiliWeave** | 中，偏发布工程 | 签名/公证诚实标注、SHA/attestation、updater 边界、平台 QA | P0 |

一句话：**Phaino 帮 Poieweave 把“结论”变成可验证的残余与弃权；Axio 帮把“审批”变成有范围、可回避的授权；Famili 帮把“发布”做成诚实可验证的桌面工程。**

## 1. 总表

| ID | 输入 | 来源 | 进 D2 | 进 C2b | 优先级 |
| --- | --- | --- | --- | --- | --- |
| P-1 | 验证优先：residuals + abstention + counterevidence | Phaino overview / inferencer contract | ✅ | ✅ | P0 |
| P-2 | 分层权限：LLM 只能提议，不能给 verdict | Phaino overview | ✅ | ✅ | P0 |
| P-3 | run/replay 比较（不是只比 digest） | Phaino overview | ✅ | ✅ | P1 |
| P-4 | 内容寻址 archive + 单写者 WAL + append-only | Phaino overview | ✅ | — | P1 |
| P-5 | bundle 的 license/sensitivity envelope | Phaino overview | ✅ | — | P1 |
| P-6 | typed contracts + conformance/version bump | Phaino architecture | ✅ | ✅ | P1 |
| P-7 | 一份结果对应一个 published bundle/replay key | Phaino overview | ✅ | — | P2 |
| A-1 | MandateRef / ScopeSpec（范围、期限、撤销） | Axio decision-kernel | ✅ | ✅ | P0 |
| A-2 | 职责分离/实例级回避/申诉 | Axio decision-kernel | ✅ | ✅（多用户） | P1 |
| A-3 | 三路审计 + 命令预登记 + 结果未知 | Axio batch1 | ✅ | ✅ | P0 |
| A-4 | 固定动作注册表 + 权限映射 | Axio ontology-actions | ✅ | ✅ | P1 |
| A-5 | Case packet + golden fixtures | Axio case-packet schema | ✅ | — | P1 |
| A-6 | 类型化 gate + 版本/来源冻结 | Axio decision-kernel | ✅ | ✅ | P1 |
| A-7 | 版本化依赖/影响清单 | Axio ontology-actions | ✅ | — | P2 |
| A-8 | 申诉/保护令/救济 | Axio governance | ✅ | 未来 | P2 |
| F-1 | 签名/公证状态诚实标注 | Famili release-signing | ✅ | — | P0 |
| F-2 | SHA256SUMS + attestation + 发布检查单 | Famili release-process | ✅ | — | P0 |
| F-3 | updater 关闭边界与独立 key | Famili updater-plan | ✅ | — | P0 |
| F-4 | macOS/Windows/Linux 平台 QA | Famili platform-qa | ✅ | — | P1 |
| F-5 | Tauri 版本/资产/smoke CI | Famili release-process | ✅ | — | P1 |
| F-6 | local-first/无遥测/隐私边界 | Famili README/docs | ✅ | — | P1 |

## 2. PhainoWeave 输入

### P-1 验证优先（P0，D2/C2b）

Phaino 的输出不是 argmax，而是：

- candidate state（分布/区域/窗口）
- supporting + conflicting evidence
- residuals（物理约束残余）
- calibration
- abstention（`insufficient_evidence` 是显式结果）

Phaino 规则：`status=abstained` + `abstain_reason`，禁止“空候选 + ok”。

**对 Poieweave：**
- What-if/Plan 结果增加：
  - `residuals`：计划对截止/容量/验证等级的残余
  - `abstained` / `insufficient_evidence`
  - `counterevidence`
- C2b proposed 计划不能只带 fingerprint，要带“验证残余 + 未满足项”
- UI 不把 `candidate` 渲染成 `feasible`；保留 caveat

**验收：**
- 至少一个“证据不足 → abstained”的 fixture
- 至少一个 residual 超阈值但结构 candidate 的 case，UI 不得显示通过
- 同输入同 residual/digest

### P-2 分层权限（P0，D2/C2b）

Phaino：

- Inferencer 不能写
- Verifier 不能提假设
- LLM 最多是 Inferencer，不能产生 verdict
- Surface 只读投影

**对 Poieweave：**
- Agent 只能 proposal/draft，不能 commit/authorize
- C2b promotion 的 `proposed` 只能由 host + 人工审批产生
- D2 channel/import 不能由 Agent 直接执行
- 审计记录“谁提议、谁验证、谁批准”

### P-3 run/replay（P1，D2/C2b）

Phaino replay = 比较，不重执行；差异归因到环境/插件版本/输出。

**对 Poieweave：**
- 增加 replay report：输入、action set、residual、outcome 的差异
- 不只比较 `resultSha`；对 nondeterministic 输入明确标记
- C2b commit 后可生成 replay key，供 D2 bundle 引用

### P-4/P-5 archive/bundle（P1，D2）

- 单写者 WAL + append-only + content-addressed blobs
- published bundle 带 license/sensitivity envelope
- 外部数据集 snapshot id，不 in-place 更新

**对 Poieweave D2：**
- site bundle 用内容寻址；manifest 带输入/证据 snapshot
- 诊断/导出不泄露敏感 payload
- license/sensitivity 字段先做可选，未来站点数据用得上

### P-6/P-7（P1/P2）

- 合同版本化 + conformance test；破坏性变更 major bump
- 一个结果对应一个 published bundle/replay key

## 3. AxioWeave 输入

### A-1 MandateRef / ScopeSpec（P0，D2/C2b）

Axio 的授权不是布尔值，而是：

- 授权者、受托主体、权能
- 对象/动作/边界/资源上限
- 有效时间、失效条件、撤销
- 生效时重新核验

**对 Poieweave：**
- C2b promotion token 增加：
  - `granted_by` / `granted_to`
  - `scope`（已有）
  - `valid_until`（已有）
  - `revocation` / `supersedes`
- D2 channel/policy 写必须绑定 scope + mandate
- 单操作员 MVP 可先不做多主体，但字段/审计要预留

### A-2 职责分离/回避/申诉（P1，D2/C2b 多用户）

- 申请、批准、执行、复核不混同
- 实例级 recusal，不是全局角色
- 申诉/保护令/救济独立状态

**对 Poieweave：**
- 当前单操作员不强制；D2 多用户前必须设计
- C2b commit 失败/争议需可复核，不允许同人“申请+批准+执行”无声通过

### A-3 三路审计（P0，D2/C2b）

Axio：

- 业务版本 + 核验回执 + APPLIED 审计同事务
- 命令预登记 + 结果核对
- “连接断开不证明回滚”；结果未知先核对

**对 Poieweave：**
- C2b commit 已是单事务；补 `result_unknown` 语义
- D2 import/rollback 外部效果未知时，不自动重试
- 审计记录 attempt/generation

### A-4 固定动作注册表（P1）

- action_type、required_powers、effect_class、handler、separation_of_duties
- 未登记写路径拒绝
- 不允许 AI 上传动作

**对 Poieweave：**
- D2 channel/import/rollback 与 C2b promote 注册为固定动作
- 权限映射与命令表绑定；测试断言无未登记写路径

### A-5 Case packet / golden fixtures（P1，D2）

- 可交换卷宗带 provenance、scope、version
- JSON Schema + golden fixtures + 独立 checker
- blocked case 也可合法保存/交换

**对 Poieweave D2：**
- site bundle 增加 `casePacket` 可选字段：provenance、scope、evidence refs
- golden fixtures 用于 import/rollback/negative cases

### A-6/A-7/A-8（P1/P2）

- typed gate + 版本/来源冻结
- 版本化依赖/影响清单
- 申诉/保护令/救济：未来多用户治理

## 4. FamiliWeave 输入

### F-1 签名/公证诚实标注（P0，D2）

Famili 明确：

- macOS ad-hoc 未公证
- Windows unsigned NSIS
- Linux unsigned AppImage/DEB
- Gatekeeper/SmartScreen 会警告
- 文案不得写“已签名/已公证”

**对 Poieweave D2：**
- 发布说明与 UI 显示真实签名状态
- 未签名/未公证不得静默
- 每次发布重新核验，不用旧资产

### F-2 发布检查单（P0，D2）

- 版本一致
- `release:check`：单测/构建/budget/E2E/宿主测试
- SHA256SUMS
- `gh attestation verify`
- 发布后校验/已知问题/回滚版本

**对 Poieweave D2：**
- 采用 `release:check` 门禁
- 资产清单 + SHA + attestation
- 发布后从 Release 下载复验

### F-3 updater 边界（P0，D2）

Famili 未开自动更新，原因：

- 需要稳定 endpoint
- 独立 updater key
- 签名清单 + CI
- 更新包签名与 Apple/Windows 证书不是同一套

**对 Poieweave D2：**
- 默认关闭 updater
- 先做“检查更新”只读；不自动下载/安装
- 任何更新必须签名+版本+平台+回滚测试

### F-4/F-5 平台 QA（P1）

- macOS 交通灯/拖拽区/app data 迁移/图标
- Windows NSIS/WebView2/SmartScreen
- Linux AppImage/DEB/窗口装饰
- CI 启动 smoke；不把 CI smoke 当人工 QA

### F-6 local-first/隐私（P1）

- 默认本地；无遥测
- 明确数据边界
- 诊断不泄露

**对 Poieweave：** D2 诊断/导出继续本地、redacted；发布说明写清无遥测。

## 5. 明确不吸收

- Phaino 的地理定位/物理约束科学、Python 插件运行时
- Axio 的民主投票/伦理仲裁/完整案件/申诉/保护令系统、PostgreSQL/axum 运行时
- Famili 的 BCI/信号层/LSM/身体地图/心理声明
- 三者的领域模型都不要搬进 Poieweave

## 6. 落地顺序

1. **D2 P0**：Famili 发布/签名/updater + Axio mandate/三路审计
2. **C2b v0.2 P0**：Phaino residual/abstention + Axio scope/revocation
3. **D2 P1**：Phaino bundle/archive + Axio case packet/golden fixtures
4. **未来**：Axio 多主体回避/申诉、Phaino replay 网络

## 7. 与现有设计的接口

| 输入 | Poieweave 落点 |
| --- | --- |
| P-1/P-2 | `poie-whatif` outcome、`poie-plan` residual、C2b `PromotionProposal` |
| P-3 | result/replay registry、D2 bundle replay key |
| P-4/P-5 | `delivery.rs` bundle、DB manifest、D2 archive |
| A-1/A-3 | `approval_tokens`、`promotion_proposals`、D2 policy/channel |
| A-4 | host command registry、D2/C2b action table |
| A-5 | `SiteManifest` / `SiteBundle` 可选 casePacket |
| F-1..F-5 | D2 release workflow、`release:check`、Release notes |

## 8. 参考

- Phaino: `/Users/hao/Documents/phainoweave/docs/architecture/overview.md`
- Phaino: `/Users/hao/Documents/phainoweave/docs/specs/inferencer-contract.md`
- Phaino: `/Users/hao/Documents/phainoweave/docs/roadmap/ROADMAP.md`
- Axio: `/Users/hao/Documents/axioweave/docs/architecture/decision-kernel.md`
- Axio: `/Users/hao/Documents/axioweave/docs/architecture/ontology-actions.md`
- Axio: `/Users/hao/Documents/axioweave/docs/architecture/batch1-rc1-spec.md`
- Axio: `/Users/hao/Documents/axioweave/specs/case-packet.schema.json`
- Famili: `/Users/hao/Documents/familiweave/docs/release-signing.md`
- Famili: `/Users/hao/Documents/familiweave/docs/release-process.md`
- Famili: `/Users/hao/Documents/familiweave/docs/updater-plan.md`
- Famili: `/Users/hao/Documents/familiweave/docs/platform-qa.md`
