# S5-V N10 — Boundary, Identity & Alternative-Resource Causal Challenge

基线：`492e20f838c33076dc6b36cc13d298390c8e68a7`。

## 状态

```text
N10 TASK-LEVEL CONTRACT HARDENING
  VERIFIED

N9 REPORT CONSISTENCY
  VERIFIED

N10 RESOURCE-CAUSAL EVENT BOUNDARIES
  ESTABLISHED (SCOPED: N10-00..N10-06)

N10 NAMED UPSTREAM DEPENDENCY VALIDITY
  NOT ESTABLISHED

N10 ALTERNATIVE-RESOURCE CAUSAL VALIDITY
  NOT ESTABLISHED

N10 BOUNDARY-AWARE RESOURCE-CAUSAL TRAJECTORY
  NOT ESTABLISHED

N10 EVIDENCE AND CERTIFICATE INTEGRITY
  NOT ESTABLISHED
```

N9 accepted claims remain scoped.

## 边界结果

```text
N10-00 no failure, completion 7200                ACCEPTED
N10-01 robot failure 7199, interruption 7199      ACCEPTED
N10-02 failure == completion 7200                 ACCEPTED (completion wins)
N10-03 failure 7201 after completion              ACCEPTED
N10-04 failure == group completion 10800          ACCEPTED (completion wins)
N10-05 failure 12000 after commit                 ACCEPTED (no interruption)
N10-06 failure == horizon 7200                    ACCEPTED (not applied)
```

- 7199/7201 使用 1 s 真实 step 与 exact event times，不做最近邻或取整。
- 同刻 completion/failure 顺序由 canonical native event_sequence 校验。
- 外部仅具 endpoint final row；轨迹深度标记 `ENDPOINT_ONLY`。
- canonical native resource release 仍为 `NOT_OBSERVED`。
- 外部 `release_s` 只是 FMU group bookkeeping，不作为原生 release 证据。

边界负例：7199 事件位移、horizon 伪应用、提交后故障中断、同刻顺序反转、故障未应用，全部检出。

## 未建立

```text
N10 NAMED UPSTREAM DEPENDENCY VALIDITY — NOT ESTABLISHED
N10 ALTERNATIVE-RESOURCE CAUSAL VALIDITY — NOT ESTABLISHED
N10 BOUNDARY-AWARE RESOURCE-CAUSAL TRAJECTORY — NOT ESTABLISHED
N10 EVIDENCE AND CERTIFICATE INTEGRITY — IN PROGRESS
```
