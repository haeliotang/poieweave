# S5-V N16 — Dependency-Closure Integrity

基线：`a2cb3f03dfbdb26a9ca0407898d0805ea79d667d`。

```text
N16-0 HISTORICAL VALIDATOR CORRECTION      VERIFIED
N16-A NAMED-UPSTREAM DEPENDENCY MODEL       ESTABLISHED (SCOPED)
N16-B DEPENDENCY-CLOSURE LIFECYCLE          ESTABLISHED (SCOPED)
N16-C RELEASE-CAUSAL CROSS-MODEL            ESTABLISHED (SCOPED: 13 CASES)
N16-D ADVERSARIAL/PROPERTY                  IN PROGRESS
N16-E EVIDENCE/CERTIFICATE                  IN PROGRESS
N16-F REPRODUCIBILITY                       IN PROGRESS
```

## 已建立

- N15 时序所有权 bug 回归：`N15-D2-SERIAL-NOFAIL`。
- 独立任务调度契约：duration、robot mask/OR、tool AND、contention、makespan。
- N16 FMU 自行计算命名上游 dependency closure，并产生
  `DependencyDeactivated` 原生事件及 dep_before/dep_after。
- 13 个固定双模型案例 ACCEPTED，包括：
  - bus-1@7199 中断、bus-1@7201 不因已退出依赖中断；
  - bus-2@7201 中断；
  - 共享 bus-1 不提前退出，bus-1@7201 仍中断；
  - 共享 tool-t1 生命周期；
  - 同刻任务完成/故障、组提交时刻故障、双上游故障、无关 sensor-x；
  - 1/900/1000 通信步长由后续 importer 时间不变性覆盖。
- executable mutant 提前 deactivate 共享 bus-1 被验证器拒绝。
- reducer 将 4 故障缩减为 1 个 bus-1@7201，保留双任务与
  `SHARED_DEPENDENCY_LOSS` 签名。

## 未完成

- OR 策略依赖差异案例。
- 生成式/属性测试。
- 完整 adversarial mutation 集。
- N16 external-validation certificate。
- N16 专属 CI 与 detached clean check。
