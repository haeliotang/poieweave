# H2b 开发集结果（自动生成）

本文件由 `poie-h2-joint-planning --dev --write <dir>` 生成；留出集未运行。预登记见 [PREREG.md](PREREG.md)。

| 方法 | 成功 | 场景数 | 成功率 | 平均完成 (h) | 平均动作数 | SAP 通过 | 最低期末储能 (kWh) | 决策数 | 非预知 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A_joint | 0 | 16 | 0.000 | 8.000 | 1.6 | 16/16 | 0.300 | 36 | 通过 |
| B1_task_priority | 0 | 16 | 0.000 | 8.000 | 2.2 | 16/16 | 0.300 | 36 | 通过 |
| B2_safety_stock | 0 | 16 | 0.000 | 8.000 | 2.2 | 16/16 | 0.300 | 36 | 通过 |
| B3_short_term | 0 | 16 | 0.000 | 8.000 | 2.2 | 16/16 | 0.300 | 36 | 通过 |
| abl_no_replanning | 0 | 16 | 0.000 | 8.000 | 1.0 | 16/16 | 0.500 | 16 | 通过 |
| abl_short_horizon | 0 | 16 | 0.000 | 8.000 | 1.6 | 16/16 | 0.300 | 36 | 通过 |
| abl_no_deadline | 0 | 16 | 0.000 | 8.000 | 1.6 | 16/16 | 0.300 | 36 | 通过 |
| abl_capability_blind | 0 | 16 | 0.000 | 8.000 | 2.2 | 16/16 | 0.300 | 36 | 通过 |

## 每场景结果

| 场景 | 方法 | 成功 | 完成 (h) | 动作 | 未达成 |
| --- | --- | --- | --- | --- | --- |
| h2b-00 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-00 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-00 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-00 | B3_short_term | false | 8.00 | 3 |  |
| h2b-00 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-00 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-00 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-00 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-01 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-01 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-01 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-01 | B3_short_term | false | 8.00 | 3 |  |
| h2b-01 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-01 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-01 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-01 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-02 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-02 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-02 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-02 | B3_short_term | false | 8.00 | 3 |  |
| h2b-02 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-02 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-02 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-02 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-03 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-03 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-03 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-03 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-03 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-03 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-03 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-03 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-04 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-04 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-04 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-05 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-05 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-05 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-05 | B3_short_term | false | 8.00 | 3 |  |
| h2b-05 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-05 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-05 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-05 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-06 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-06 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-06 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-06 | B3_short_term | false | 8.00 | 3 |  |
| h2b-06 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-06 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-06 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-06 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-07 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-07 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-07 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-07 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-07 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-07 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-07 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-07 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-08 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-08 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-08 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-09 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-09 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-09 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-10 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-10 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-10 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-10 | B3_short_term | false | 8.00 | 3 |  |
| h2b-10 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-10 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-10 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-10 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-11 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-11 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-11 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-11 | B3_short_term | false | 8.00 | 3 |  |
| h2b-11 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-11 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-11 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-11 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-12 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-12 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-12 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-12 | B3_short_term | false | 8.00 | 3 |  |
| h2b-12 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-12 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-12 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-12 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-13 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-13 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-13 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-13 | B3_short_term | false | 8.00 | 3 |  |
| h2b-13 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-13 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-13 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-13 | abl_capability_blind | false | 8.00 | 3 |  |
| h2b-14 | A_joint | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-14 | B1_task_priority | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-14 | B2_safety_stock | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-14 | B3_short_term | false | 8.00 | 1 | potable-water:not planned |
| h2b-14 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-14 | abl_short_horizon | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-14 | abl_no_deadline | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-14 | abl_capability_blind | false | 8.00 | 1 | potable-water:blocked: missing_resource electric-energy |
| h2b-15 | A_joint | false | 8.00 | 2 | potable-water:deadline missed;restore:bus-1:deadline missed |
| h2b-15 | B1_task_priority | false | 8.00 | 3 |  |
| h2b-15 | B2_safety_stock | false | 8.00 | 3 |  |
| h2b-15 | B3_short_term | false | 8.00 | 3 |  |
| h2b-15 | abl_no_replanning | false | 8.00 | 1 |  |
| h2b-15 | abl_short_horizon | false | 8.00 | 2 | potable-water:horizon exceeded |
| h2b-15 | abl_no_deadline | false | 8.00 | 2 | restore:bus-1:deadline missed |
| h2b-15 | abl_capability_blind | false | 8.00 | 3 |  |

## 读法与局限

- 子集为 **开发集（16 场景）**；留出集（8 场景）未运行、未查看。
- 成功 = 供水目标在 4 h 预测失效前完成、车床目标达成且无未达成目标；所有方法使用相同扰动轨迹、预算与 SAP 校验路径。
- 规划器是确定性贪心（ADR-0006）；合成场景、未校准；样本小，只作方向性比较。
- 2026-11-01 起（VEC §2.1 修订，issue #110）同时报告终端状态与非预知检查：期末储能 = 场景初始储能 − 已执行动作的用电；每次决策只使用不晚于决策时刻的事件（`max_lookahead_s` ≤ 0）；同一方法的同一实例（相同参数）在全部场景上运行。PREREG 没有终端约束，期末储能只作报告，不参与成功判定。
