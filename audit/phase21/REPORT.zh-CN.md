# S5-V N21 — Reference-Oracle Qualification & Bounded Replanning Readiness

基线：`503f72074cc66d58b6b6af9b5086f97990045e66`。

```text
N21-0 HISTORICAL CORRECTION       VERIFIED
N21-A ORACLE QUALIFICATION        VERIFIED (BOUNDED)
N21-B REPLANNING REPRESENTABILITY BLOCKED
N21-C ONE-SHOT REPLANNING         NOT ESTABLISHED
N21-D CROSS-MODEL REPLANNING      NOT ESTABLISHED
N21-E ADVERSARIAL INTEGRITY       NOT ESTABLISHED
N21-F CERTIFICATE/REPRODUCIBILITY IN PROGRESS
```

## N21-0

- N20 证据冻结于 `historical-revalidation/n20-freeze.json`。
- N20 REPORT 的 N20-F IN PROGRESS 与 CLAIMS VERIFIED 冲突，已由
  `N20_REVALIDATION.md` 解释。
- 复现参考 oracle 共享工具 False PASS：不同机器人共享 tool-t1 时，v1
  错误允许 0..7200/0..10800 并行候选。
- v2 参考实现排除共享工具并行，保留合法串行 A→B / B→A。
- 用 v2 重新验证 N20 已接受 64 案例：64 accepted / 0 rejected。
- N20 held-out 合成中的重复去重修改 baseline 问题已记录为 N21 限制。

## N21-B BLOCKED

canonical 中断只返回 `Blocked` 与 trace/final_inventory，缺少资产状态、
已完成任务、未完成任务、选中资源、活跃依赖、applied failure IDs 和组状态。
无法从原生状态构建 checkpoint；R3 拼接新 episode 不能冒充连续性。
因此未实施重规划，未生成 checkpoint/authorization/resume 事件。

## 后续

如需继续 N21-C/D，必须实现 v11 opt-in checkpoint/resume 语义扩展，
并完成证书迁移。本轮未创建 v11。
