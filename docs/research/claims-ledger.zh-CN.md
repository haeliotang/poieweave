# 研究声明与证据对应表

日期：2026-11-15
范围：模型级/软件级证据；不含物理验收

每条声明都标明证据与边界。没有证据的声明不应出现在对外材料中。

| ID | 声明 | 证据 | 边界 |
| --- | --- | --- | --- |
| C1 | 无备件时车床维修构成自指循环 | `experiments/s1-tool-cycle`、S1 证书 `tool_cycle` | Base-08 修改场景 |
| C2 | 1 个 spare 可解锁 2 h 修复 | `experiments/s1-tool-cycle`、S1 证书 | 单资产 |
| C3 | 替代 restorer 可旁路循环（4 h） | `experiments/s1-alt-restorer`、S1 证书 | 合成替代过程 |
| C4 | 两条路径都存在时选更便宜的 2 h | `experiments/s1-alt-choice`、S1 证书 `path_choice` | 单资产 |
| C5 | Base-05 故障间隔 10/15/20/30 天 → 40/60/80/≥90 天 | `experiments/s2-survival-horizon` | 模型内 |
| C6 | Base-06 独立覆盖 25/56、联合 24/56 | `experiments/s2-joint-coverage` | 双故障、约定假设 |
| C7 | 再生能力使 horizon 40→60 天 | `experiments/s2-insert-regeneration` | 模型内 |
| C8 | 负载 3/4 kW 时能量成为 binding constraint | `experiments/s2-load-threshold` | 模型内 |
| C9 | 冗余泵 60→90 天 | `experiments/s2-redundancy-load` | 单冗余布局 |
| C10 | 补货间隔 10/20/30/60 天 → 60/50/40/30 天 | `experiments/s2-resupply-frequency` | 单一 consumable |
| C11 | 补货不能修复能力缺口（filter-1 仍 0 天） | `experiments/s2-adversarial-resupply` | 31 子集 |
| C12 | 时间语义改变 grease/insert 边界与 deadline | `experiments/s2-time-boundary`、`s2-time-insert`、`s2-time-horizon`、`s2-time-repair-sweep`、`certificate-s2-time` | 长修复/密集故障 |
| C13 | 时间语义在能量/sparse/标称再生下不变 | `experiments/s2-time-energy*`、`s2-regen-event` | 指定负载/修复时长 |
| C14 | 冗余效应大于时间语义效应 | `experiments/s2-redundancy-time`、S2 指标证书 | 单一布局 |
| C15 | 审批/租约/撤销/拒绝/并发 fail closed | `experiments/s3-recovery-loop`、`s3-approval-matrix`、`s3-lease-boundary`、`s3-revocation-inflight`、`s3-concurrency-deadline`、S3 证书 | 模型内执行 |
| C16 | 独立模型 8 项 26 案例一致 | `certificate-s4-independent`、`s4-*` | 窄模型 |
| C17 | bundle 可在 clean-room 中验证 | `certificate-clean-room` | 同一 verifier 代码 |
| C18 | 随机对抗中 0 天 collapse 是瞬时语义假象 | `experiments/s2-adversarial-random-time`、time 证书 | 200 子集、单一长修复 |
| C19 | 延迟故障只减少、不消除瞬时 0 天质量 | `experiments/s2-adversarial-random-window`、time 证书 | day0–10 窗口 |
| C20 | 显式替代资源组可按计划并发调度 | `experiments/s3-scheduler`、scheduler 测试 | 贪心、无抢占；`occupies` 为 AND |
| C21 | 生存循环应用被调度计划并按 makespan 推进 | `experiments/s5-scheduled-survival`、survival 测试 | 批次级库存耦合 |
| C22 | 最小 FMI 3.0 描述与容器通过检查 | `experiments/fmi-conformance` | 不执行 FMU |
| C23 | 另一进程的 crash tail 可在重开时截断 | `experiments/s5-multiprocess-crash` | 单写者、本地文件 |
| C24 | 签名路径可使用操作员提供的私钥 | `experiments/certificate-attestation`（`--key`） | 非信任锚 |
| C25 | 三个预登记场景中 advanced horizon 不低于 instant | `experiments/s5-semantics-parity` | 观察，不是普遍不变量 |
| C26 | `occupies` 的所有资源必须同时可用 | `experiments/s3-scheduler`、scheduler 测试 | 严格模式 |
| C27 | 调度与执行使用同一计划，且资源区间不重叠 | `experiments/s5-scheduled-survival`、scheduler 测试 | 批次级库存 |
| C28 | 替代策略不会删除未标记为可替代的必需资源 | `experiments/s3-scheduler`、scheduler 测试 | 显式 replaceable 集合 |
| C29 | 库存按规划快照结算，状态原子提交 | `experiments/s5-scheduled-survival`、survival 测试 | 批次级、无回滚日志 |
| C30 | 初始维修不覆盖已结算的能源/负载 | `experiments/s5-scheduled-survival`、survival 测试 | 模型级 |
| C31 | horizon 处未完成维修不被报告为完成 | `experiments/s5-scheduled-survival`、`SurvivalTermination` 测试 | 批次级 |
| C32 | horizon 时刻的阻塞不会被归类为完成 | `experiments/s5-scheduled-survival`、survival 测试 | 结构化判定 |
| C33 | PV 修复完成前不产生该 PV 电力 | `experiments/s5-scheduled-survival`、survival 测试 | 最小合成 PV |
| C34 | canonical kernel 产生类型化终止与逐事件 trace | `experiments/s5-canonical-engine`、engine 测试 | 稀疏场景 |
| C35 | 稀疏直接修复场景 canonical 与 legacy 一致 | `experiments/s5-canonical-engine` | 单场景 |
| C36 | 单机器人竞争 canonical 5 h、legacy 3 h | `experiments/s5-migration-batch2` | 批次 2a |
| C37 | 密集长修复在维修期间故障时第 2 天中断 | `experiments/s5-migration-batch2` | 新的因果时序语义 |
| C38 | 规划后缺失的资源键按零扣除 | `experiments/s5-migration-batch2`、survival/engine 测试 | 单计划与多计划 |
| C39 | 同时刻补货与故障各执行一次 | `experiments/s5-migration-batch2`、engine 测试 | 事件队列 |
| C40 | 维修期间 PV 故障从事件时刻停止发电 | `experiments/s5-migration-batch2`、engine 测试 | 单 PV |
| C41 | 维修恰好完成于 horizon 记为 Completed | `experiments/s5-migration-batch2`、engine 测试 | 边界规则 |
| C42 | 实际选中的机器人失效会中断维修 | `experiments/s5-migration-batch2`、engine 测试 | 替代池 |
| C43 | 未选中的备用机器人失效不中断当前维修 | `experiments/s5-migration-batch2`、engine 测试 | 执行依赖绑定 |
| C44 | 必需工具失效会中断维修 | `experiments/s5-migration-batch2`、engine 测试 | fail-closed |
| C45 | 所选能力提供者的上游依赖失效会中断 | `experiments/s5-migration-batch2`、engine 测试 | 派生 witness |
| C46 | 未选能力提供者失效不中断当前维修 | `experiments/s5-migration-batch2`、engine 测试 | 派生 witness |
| C47 | `repair_completed` trace 含 capability witness | `experiments/s5-migration-batch2`、engine 测试 | 当前派生实现 |
| C48 | 初始能源超过电池容量被拒绝 | `experiments/s5-migration-batch3`、engine 测试 | 输入校验 |
| C49 | 离散补货能源不超过电池容量 | `experiments/s5-migration-batch3`、engine 测试 | 截断策略 |
| C50 | 负载耗尽时刻按真实时间积分 | `experiments/s5-migration-batch3`、engine 测试 | 单负载 |
| C51 | 维修期间负载持续扣除 | `experiments/s5-migration-batch3`、engine 测试 | 无 PV |
| C52 | 补货到达前不可追溯使用 | `experiments/s5-migration-batch3`、engine 测试 | 单一 consumable |
| C53 | Batch 3 三场景无未解释分歧 | `experiments/s5-migration-batch3` | 不含 3D 联合 |
| C54 | 联合能源账本逐项守恒（6+7.5+2-3-0.5-2.5=9.5） | `experiments/s5-migration-batch3`、engine 测试 | 当前结算语义 |
| C55 | horizon 同刻补货不进入本次运行 | `experiments/s5-migration-batch3`、engine 测试 | 边界规则 |
| C56 | canonical C5 四区间 horizon/energy/失败时刻一致 | `experiments/s2-canonical-c5` | trace 事件数为记录粒度差异 |
| C57 | canonical C6 独立 25/56、联合 24/56 一致 | `experiments/s2-canonical-c6` | 覆盖枚举为外部适配器 |
| C58 | canonical C7 六行 40/60、50/60 一致 | `experiments/s2-canonical-c7` | 模型内 |
| C59 | canonical C8 11 行负载制度与能源一致 | `experiments/s2-canonical-c8` | 恒定负载 |
| C60 | canonical C9 冗余泵 60→90、17 天一致 | `experiments/s2-canonical-c9` | 单一冗余布局 |
| C61 | canonical C10 补货间隔 60/50/40/30 一致 | `experiments/s2-canonical-c10` | 单一 consumable |

## 不应出现的声明

- “真实设备可连续运行 X 天”
- “已通过工业安全验收”
- “已支持 FMI 3.0 协同仿真”
- “发布签名已被第三方信任”
- “多节点分区安全已验证”

这些均无当前证据，必须在对外材料中避免。
