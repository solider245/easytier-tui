# EasyTier TUI 开发记录

## 2026-03-13

### 项目初始化

**背景**: 
用户希望为 EasyTier 创建 TUI 版本，结合 AI 调用能力

**技术选型**:
- 语言: Rust
- TUI 框架: Ratatui (参考 ztui 项目)
- 版本: ratatui 0.26 (兼容 rustc 1.87)

**遇到的问题**:
1. 初始使用 ratatui 0.29，需要 rustc 1.88，降级到 0.26
2. Frame API 变化: `f.area()` → `f.size()`
3. cargo build 输出缓冲问题导致菜单显示异常 (onekeyeasytier.sh)

**仓库地址**:
- onekeyeasytier: https://github.com/solider245/onekeyeasytier
- easytier-tui: https://github.com/solider245/easytier-tui

**下一步**:
- 完善菜单交互 (上下键选择)
- 调用 easytier-cli 获取网络信息

---

ADD: 初始化项目，创建基础 TUI 框架
