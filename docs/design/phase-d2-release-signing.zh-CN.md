# Phase D2 设计：发布、签名、通道与交付包

- 状态：设计草案（D2 P0 release verification 已实现；D2 P1 bundle v0.2 结构已实现，审计链验证待接）
- 日期：2026-11-06
- 范围：D2 发布工程、签名状态、updater 边界、平台 QA、bundle 审计头/墓碑/锚
- 上游：D1 manifest/import/state/rollback、Famili 发布经验、Axio mandate/三路审计、Phaino bundle
- 非目标：D2 不实现远程服务、自动更新安装、多节点同步

## 1. 背景

D1 只做本地 JSON 元数据交付；D2 要让发布本身可验证、可回滚、诚实标注签名状态。当前 `v0.3.1` 已有跨平台构建与 `SHA256SUMS`，但缺：

- 发布 receipt（签名/版本/摘要/通道）
- 签名材料保管与恢复演练
- updater 关闭边界
- bundle auditHead/墓碑/锚
- 平台 QA 清单

## 2. 发布契约

### 2.1 版本与产物

- `tag = v<semver>`；`apps/desktop/package.json`、`Cargo.toml`、`tauri.conf.json`（若有）版本必须一致。
- 每个平台产物名包含版本：
  - macOS `Poieweave_<version>_universal.app.zip`
  - Windows `Poieweave_<version>_x64-setup.exe`
  - Linux `Poieweave_<version>_amd64.deb`
- `SHA256SUMS` 覆盖全部资产（不含自身），条目使用**扁平资产名**（不是 CI 内部目录路径）；`release-verification.json` 记录校验结果。
- Windows 资产统一重命名为 `Poieweave_<version>_x86_64.exe`；release job 在上传前完成重命名。
- `scripts/verify_release.py` 兼容 basename 归一化；v0.3.1 现有资产的 receipt 已验证 `ok=true`（仅旧 `Poieweave.exe` 名称 warning）。

### 2.2 发布 receipt

`poie.release-verification/0.1`：

```text
{ schema, tag, expectedVersion, artifacts[], sha256Sums,
  macosSignature:{present,status:"signed|ad-hoc|unsigned|unknown|not_checked",detail[]},
  failures[], warnings[], ok }
```

- `ok=false` 不得发布。
- `macosSignature` 只报告事实，不推断公证/Gatekeeper 信任。
- 当前 `scripts/verify_release.py` 实现；CI release job 生成并上传 receipt。

### 2.3 签名状态

- 明确三档：`developer-id-notarized`、`ad-hoc`、`unsigned`。
- 未签名/未公证的产物必须在 Release notes 与 UI 中标注，不得写“已签名/已公证”。
- Apple/Windows 代码签名证书与 Tauri updater key **不是同一套**。
- 私钥/口令不进仓库、日志、截图、聊天；丢失恢复流程见 §3。
- `scripts/signing_custody_card.py` 生成不含机密的恢复卡；卡存在不等于可恢复，只有演练通过才算。

## 3. 签名材料保管与恢复

### 3.1 三件套

1. 加密签名档案（keystore/证书 + 元数据）
2. 口令（Keychain/离线副本）
3. 恢复说明卡（不含机密）

### 3.2 恢复演练

- `scripts/recover_signing_archive`（D2 实现）：
  - 解密档案 → 解析证书 SHA-256 → 签测试产物 → 验签 → 清理
  - 输出 `verified:true` receipt
- 只有演练通过才算“可恢复”；档案存在不算。
- 触发重做：换机/重装、口令位置变化、密钥轮换、分发渠道变化。
- 不做：自动生成新密钥、把口令写入仓库、跳过演练。

### 3.3 决策点（需用户确认）

- 继续用现有开发密钥，还是新密钥？
- 内部分发/侧载是否接受 `ad-hoc` 标注？
- 签名档案和口令由谁保管、放哪里？
- 是否现在就做 Keychain HMAC 审计锚？

## 4. 通道与回滚

- channel：`stable` / `beta` / `recall`。
- D2 只支持**人工选择**通道；不自动 channel 切换。
- recall：发布 recall manifest；不自动卸载/降级。
- 回滚：签名一致、版本递增的修复版本；不通过卸载绕过。
- 旧 bundle 导入先做格式兼容检查；不静默恢复已删数据（§6）。
- 降级策略：默认拒绝；需要显式 `--allow-downgrade` 且仅限安全回滚演练。

## 5. Updater 边界

- 默认**不启用** Tauri updater。
- 第一版只做“检查更新”（HTTPS 只读），不自动下载/安装。
- 启用前必须：
  - 稳定 HTTPS endpoint + `latest.json`
  - 独立 updater key pair（minisign）
  - 每个平台 `.sig` + 版本清单
  - 签名错误/网络中断/回滚测试
- 失败保留旧版本，不阻塞本地使用。
- 更新包签名与 Apple/Windows 证书分开；key 泄露立即轮换并撤销旧通道。

## 6. Bundle v0.2（D2）

D1 site bundle 仍是 JSON；D2 增加：

```text
SiteBundle {
  schema:"poie.site-bundle/0.2",
  manifest: SiteManifest,
  auditHead: string,       // audit chain head at export
  auditCount: u64,
  deletions: [{kind,id,reason,at}],  // tombstones
  casePacket?: CasePacket  // 可选，Axio 风格 provenance/scope
}
```

- 结构支持已实现：v0.2 bundle 可携带 auditHead/auditCount/deletions/casePacket；导入持久化到 `delivery_bundle_meta`（v7）。
- 本地 audit chain 已实现：`append_audit` 写 `prev_hash/hash`；`audit_chain`/`verify_audit_head`/`verify_audit_chain` 可用；导入 bundle 时 `auditHead/auditCount` 不匹配 → `audit_mismatch`。
- 本地锚已实现：随机 32 字节 HMAC key 存平台 SecretStore（macOS Keychain），对 `count|head` 做 HMAC-SHA256；`audit_anchor`/`audit_anchor_verify` 命令可用。它是**设备本地锚**，不是第三方时间戳。
- 墓碑已落本地 ledger：导入 bundle 的 `deletions` 写入 `delivery_tombstones`（PK kind,id，重复导入不覆盖首个墓碑）；`delivery_tombstones` 命令可查。
- 仍未实现：墓碑对 live 对象库的实际删除/拒绝复活（当前 Poieweave 没有对象库写路径）、跨节点链合并。
- 删除墓碑优先于导入内容：旧 bundle 不得复活已删数据。
- `auditHead` 在 bundle 内，能改行的人也能改头；外部锚才提高保障。
- `casePacket` 可选，未知字段拒绝；D2 不实现多节点同步。

## 7. 审计锚

- 本地 Keychain 不可导出 HMAC key，对 `idx|hash|at` 做 HMAC-SHA256。
- 导出 bundle 时写锚定摘要；导入时校验。
- UI 显示“链完整 + 锚定 N 条”；不宣称防篡改。
- 截断尾部仍是较短合法链；记录该边界。

## 7.1 本地锚实现

- `audit_anchor`：取当前 audit chain head/count，生成/读取 SecretStore 中的 HMAC key，写入 `audit_anchors`。
- `audit_anchor_verify`：校验最新锚的 HMAC，并重算整条链；任一失败返回 false。
- 截断尾部仍是较短合法链；锚只覆盖已锚定长度。
- 不宣称“防篡改”；只宣称“锚定后可检测修改/重算”。

## 8. 平台 QA

| 平台 | 必须人工验证 |
| --- | --- |
| macOS | 安装/首次启动、Gatekeeper 提示、Dock/图标、app data 迁移 |
| Windows | NSIS 安装/卸载、WebView2、SmartScreen 说明、路径/SQLite |
| Linux | AppImage FUSE/执行权限、DEB 依赖、窗口装饰、最小发行版 |
| 全部 | 版本一致、SHA256SUMS、Release notes 签名状态、无遥测 |

CI smoke 不能替代人工 QA。

## 9. 发布前门禁

- [ ] `cargo test` / `pnpm test:ui` / Playwright
- [ ] `verify_release.py` 通过
- [ ] 版本一致
- [ ] 签名状态诚实
- [ ] Release notes/CHANGELOG
- [ ] 平台 QA 收据
- [ ] updater 未启用（除非满足 §5）
- [ ] 旧 bundle/回滚演练
- [ ] 无 secrets/路径泄露

## 10. 参考

- [D1](phase-d-site-delivery.zh-CN.md)
- [traceweave 输入](traceweave-adoption-inputs.zh-CN.md)
- [Phaino/Axio/Famili 输入](phaino-axio-famili-inputs.zh-CN.md)
- `scripts/verify_release.py`、`.github/workflows/desktop-build.yml`
- Famili: `/Users/hao/Documents/familiweave/docs/release-signing.md`
- Famili: `/Users/hao/Documents/familiweave/docs/release-process.md`
- Famili: `/Users/hao/Documents/familiweave/docs/updater-plan.md`
