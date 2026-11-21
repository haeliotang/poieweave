# S5-V N18 — Alternative-Path Causal Integrity

基线：`e51689060b1d7b88aaa7ea3c3396233c66d04e2d`。

```text
N18-0 HISTORICAL ORACLE CORRECTION       IN PROGRESS
N18-A OR + SPLIT ROOT CAUSE              VERIFIED
N18-B CANONICAL CAUSAL-PATH INTEGRITY    ESTABLISHED (SCOPED, v10)
N18-C MECHANISM-SPECIFIC ORACLE          NOT ESTABLISHED
N18-D FALSIFICATION                      NOT ESTABLISHED
N18-E CERTIFICATE GOVERNANCE             IN PROGRESS
N18-F REPRODUCIBILITY                    NOT ESTABLISHED
```

## 已建立

- 冻结 N17 OR+split 失败见证。
- 根因：`task_dependencies` 从静态 `process.occupies` 推导；带
  `robot_alternatives` 时该集合为空，实际选择的上游依赖未登记。
- 修复：从 `task_resources` 的实际 selected resources 计算每任务上游闭包。
- 修复后：Task A 选择 robot-m2，完成 @7200 时 bus-2 退出闭包；
  bus-2@7199 中断、bus-2@7201 不中断、bus-1@7201 中断；
  A/B 共享 robot-m2 时 bus-2 不在 A 完成时提前退出。
- v9 冻结，41 个 v10 correction successors 签发。

## 未完成

- N17 relation 语义重分类（已完成初版）、严格 metamorphic、真实负例、生成式
  attempt accounting。
- N18 机制特定 expected-effect oracle。
- 16 类对抗控制、OR mutant、reducer。
- N18 external-validation certificate、CI、clean check。
