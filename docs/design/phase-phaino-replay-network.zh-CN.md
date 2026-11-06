# Phaino replay 网络协议（packet v0.1）

状态：packet/比较已实现；**网络传输未实现**。

## 1. 目标

让两个节点在不共享数据库、不互相执行代码的前提下，比较同一 promotion 的 replay 结果。

## 2. Packet

```text
poie.replay-packet/0.1
{
  schema, producer, proposalId, createdAt,
  resultSha256, snapshotSha256, variantSha256, inputsSha256,
  planSha256, actionsSha256
}
```

- 只包含 digest 与身份，不包含模型、状态、证据内容。
- 所有 digest 必须是 64 hex。
- `producer` 只用于标识，不参与比较。

## 3. 比较

```text
poie.replay-report/0.1
{ schema, ok, mismatches[] }
```

比较字段：

- resultSha256
- snapshotSha256
- variantSha256
- inputsSha256
- planSha256
- actionsSha256

任一不同 → `ok=false`，列出字段。

## 4. 命令

- `whatif_promote_packet(proposalId)`：导出本地 packet。
- `whatif_promote_compare_packet(proposalId, packetJson)`：与远端 packet 比较，不写库。

## 5. 边界

- **没有网络传输**：packet JSON 需人工/外部通道交换。
- 没有签名/身份认证；producer 字段不可信。
- 没有跨节点状态同步、冲突合并或在线发现。
- 相同 digest 只证明这些 digest 字段相同，不证明两边运行时完全相同。

## 6. 未来

- 传输层（HTTPS/对象存储/邮件附件）
- 签名 packet
- 多节点比较矩阵
- 与审计链锚绑定
