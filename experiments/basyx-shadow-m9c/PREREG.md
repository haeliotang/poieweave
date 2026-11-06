# M9c BaSyx 影子/过期/漂移修正预演登记（PREREG）

- 实验 ID：`basyx-shadow-m9c`
- 日期：2026-11-06
- 状态：**已登记，尚未运行**
- 性质：AI 预演 · 本机单容器 · 非现场设备；不计入 M9.2 退出条件。
- 依据：M9b 首次运行 [REPORT](../basyx-shadow-nonzero/REPORT.md)

## 1. 从 M9b 学到的

M9b 首次运行结果：

- N1 通过：观测 `failed` 时 shadow plan **1 个动作**，未派发。
- N2 失败：`expired_evidence=6`、`pump-p1=Unknown`，但 `healthy=true`；M9b 期望 `healthy=false` 是错的——运行时 `healthy` 表示适配器**连接**状态，不是证据新鲜度。
- N3 失败：`degraded` 恢复后投影正确，但 shadow plan 仍有 1 个修复动作；M9b 期望 0 动作是错的。
- M9b 的旧 C1–C7 阶段未运行被误报为失败；M9b runner 已增加 profile 区分。

## 2. 修正后的可证伪问题

在相同 BaSyx 容器上：

1. `failed` 观测产生 **非零** shadow plan 且不派发；
2. `ObservedAt=now-120s` 时投影 `Unknown` 且 `expired_evidence>=1`；
3. 注入新鲜 `degraded` 后投影回到 `Degraded`，且仍不派发。

`healthy` 与 shadow 动作数不再作为 N2/N3 的硬性判据；原始值仍报告。

## 3. 判据

| 编号 | 期望 |
| --- | --- |
| N1 | `pump-p1=Failed`；计划动作 >0；`dispatched=false` |
| N2 | `pump-p1=Unknown`；`expired_evidence>=1` |
| N3 | `pump-p1=Degraded`；`dispatched=false` |

## 4. 方法

- 固定镜像、单容器、回环、无认证、InMemory
- 六个资产 Submodel：pump 注入状态，bat/bus/filter/robot/charger 注入 available，确保规划依赖完整
- 证据窗口 60 s
- 三阶段：failed fresh → failed stale → degraded fresh
- 运行 `run_m9c.sh`，写 `REPORT.md` + `raw/`

## 5. 局限

同 M9b；不替代 PLC/真实电表或现场验收。
