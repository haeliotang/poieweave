# Phase D2 设计：发布、签名、通道与交付包

- 状态：设计草案（待 D2 门禁）
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
- `SHA256SUMS` 覆盖全部资产（不含自身）；`release-verification.json` 记录校验结果。

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

- 导入时重算 audit chain head/auditCount；不一致 → `audit_mismatch`。
- 删除墓碑优先于导入内容：旧 bundle 不得复活已删数据。
- `auditHead` 在 bundle 内，能改行的人也能改头；外部锚才提高保障。
- `casePacket` 可选，未知字段拒绝；D2 不实现多节点同步。

## 7. 审计锚

- 本地 Keychain 不可导出 HMAC key，对 `idx|hash|at` 做 HMAC-SHA256。
- 导出 bundle 时写锚定摘要；导入时校验。
- UI 显示“链完整 + 锚定 N 条”；不宣称防篡改。
- 截断尾部仍是较短合法链；记录该边界。

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
