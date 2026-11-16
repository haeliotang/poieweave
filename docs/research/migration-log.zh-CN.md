# Canonical Migration Log

日期：2026-11-16
状态：Batch 1–3D 内核级迁移探针完成；正式 S2 尚未迁移

## 批次表

| 批次 | 场景 | 状态 | 结论 |
| --- | --- | --- | --- |
| 1 | 稀疏直接修复（Base-08） | 接受 | 30 天一致 |
| 2a | 单机器人竞争 | 接受 | canonical 5 h vs legacy 3 h，资源占用语义修复 |
| 2b | 双机器人竞争 | 接受 | 3 h 一致 |
| 2c | 密集长修复 | 接受（语义变更） | 维修期间故障在事件时刻生效；canonical 2 天中断，legacy 15.6 天为延迟处理 |
| 3a | resupply 到货前不可用 | 接受 | 双方 t=0 阻塞 |
| 3b | load 时间积分 | 接受 | 5h、4.5kWh 一致 |
| 3c | 电池初始超容量 | 接受 | 双方拒绝 |
| 3d | 联合能源账本 | 内核账本测试通过 | 独立迁移对照未单列 |
| 4 | S2 指标/证书 | 未开始 | — |
| B-C5 | Base-05 故障间隔 | 接受 | canonical 40/60/80/90，能源与失败时刻一致；trace 事件数为完成记录粒度 |
| B-C6 | Base-06 联合覆盖 | 接受 | 首轮 6/56 暴露非服务故障误排队；修复后 25/56、24/56 一致 |
| B-C7 | insert 再生 | 接受 | 6/6 行一致；40/50→60 |
| B-C8 | 负载制度 | 接受 | 11/11 行一致；2.6 kW 切换、期末能源一致 |
| B-C9 | 冗余泵负载 | 接受 | 首轮 70/16 暴露 requires_any 过度维修；修复后 90/17、6/6 一致 |
| B-C10 | 补货频率 | 接受 | 10/20/30/60 → 60/50/40/30，4/4 一致 |
| B-C11 | 对抗补货子集 | 接受（语义修正） | t=0 事件延迟到初始维修后；最小集合 filter-1；0→0.125 d；day-10 legacy 行无效 |
| B-C12 | grease 时间边界 | 取代（语义修正） | canonical 第 2 天中断；旧 10/16/21 天为延迟故障模型 |
| B-C13 | 能量边界不变性 | 接受（语义修正） | 16/3/1 天不变；精确耗尽最多移 1 h |
| B-C14 | 冗余与密集故障 | 接受（语义修正） | 冗余泵 90 天一致；baseline 31→2 天 |
| B-C18 | 随机对抗（时间） | 接受（语义修正） | 零天 0/200 不变；均值 9.9→5.4、p90 15.7→6.0 |
| B-C19 | 随机延迟窗口 | 接受（语义修正） | 零天 0/200 不变；均值 14.0→2.2、p90 15.6→2.0 |
| B-C69 | load 证书行 | 接受 | Base-05 0–4 kW 5/5 一致 |
| B-C70 | regeneration 证书行 | 接受 | Base-05/08 8/8 一致 |
| D | Certificate Impact Matrix | 初版 | 6 证书：3 historical、1 superseded、2 blocked、0 reissued |
| D | metrics 证书拆分 | 更新 | 13 组：7 migrated、3 semantic、2 partial、1 superseded、0 not migrated；聚合证书仍 BLOCKED |
| B-C71 | metrics 四组 | 接受（含定义修正） | regenerative/yield/recharge 一致；coverage 5/10→9/10 服务影响 |
| B-C72 | order 证书 | 接受 | 24/25、40/44，order_sensitive=0 |
| B-C73 | C68 Monte Carlo | 接受（语义修正） | 中位 72→68、均值 64.8→64.0、survived 622→611 |
| B-C74 | C12 剩余分量 | 接受（语义修正） | horizon/insert/repair>=2d → 2 天；repair<=1d 6 天一致 |
| B-C75 | energy interventions | 接受（语义修正） | 12 行日级一致；16→16.25 d |
| B-C76 | redundancy restorers | 接受（语义修正） | 7/8 一致；共享机器人 baseline 30→20 |
| E-C77 | aux batch1 | 抽样通过 | energy-boundary 18 + heldout 2，20/20 一致 |
| E-C78 | aux batch2 | 抽样通过 | resupply-time 4 + delivery 3 + regen-cross 12，19/19 一致 |
| E-C79 | aux batch3 | 抽样通过 | joint-boundary 6 + joint-consumables 8，14/14 一致 |
| E-C80 | aux batch4 | 抽样通过 | regen-cost 9 + energy-recharge 3 + recharge-bound 1，13/13 一致 |
| E-C81 | aux batch5 | 定义修正 | coverage 5→9、11→33；threshold 60/61 一致 |
| E-C82 | aux batch6 | 抽样通过 | redundancy 2 + long-horizon 1 + recharge-rule 6，9/9 一致 |
| E-C83 | aux batch7 | 抽样通过 | threshold battery 4 + initial 5 + regen-event 8，17/17 一致 |
| E-C84 | aux batch8 | 语义修正 | regen-long 8 行；joint double-failure 10→10.25 |
| E-C85 | aux batch9 | 抽样通过 | insert-bound 6 + joint-load 4，10/10 一致 |
| D | canonical 证书重签 | 完成 5 个 | joint/load/regeneration/order/metrics-v2；历史证书保留 |

| A | 状态页数值归一化 | 完成 | 12/12 映射；C68 Monte Carlo 为 legacy-only |
| A | 84 实验分类 | 完成 | 25 claimed + 48 aux + 2 deterministic-sweep + 6 superseded + 2 exploratory + 1 definitional；0 unclassified、0 missing seed |

## 差异记录规则

每条差异必须记录：

- 旧/新入口与版本
- 第一条差异事件
- 事件前后库存、时间、资产状态
- 分类：语义修复 / 预期模型差异 / 未解释异常
- 受影响声明与证书

禁止直接修改 expected 输出让 CI 变绿。

## 批次 2 记录

详见 [s5-migration-batch2](../../experiments/s5-migration-batch2/REPORT.md)。

- 竞争场景的差异来自共享机器人序列化，属于语义修复。
- 密集长修复先暴露了耗尽键漏扣缺陷；修复后，事件队列又显示维修期间故障
  必须在事件时刻生效。canonical 在第 2 天中断，legacy 15.6 天是延迟处理，
  因此 batch 2c 接受的是事件时序语义，不是历史数值保持。

| E | S5-T Migration Report | 初版 | 明确 NOT ESTABLISHED；列出 9 精确 + 8 语义修正 + 剩余边界 |
