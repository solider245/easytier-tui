# easytier-tui

一个基于 Rust + Ratatui 的 EasyTier 终端管理界面 (TUI)。

## 功能规划

- [ ] 网络管理 (创建/加入/离开)
- [ ] 节点查看 (peers, routes)
- [ ] 服务控制 (启动/停止/重启)
- [ ] 日志查看
- [ ] 带宽统计

## 技术栈

- Rust
- [Ratatui](https://ratatui.rs/) - TUI 框架
- [Crossterm](https://github.com/crossterm-rs/crossterm) - 终端操作

## 开发计划

### Phase 1: 基础框架
- 项目初始化
- 基础 TUI 布局
- 菜单导航

### Phase 2: 核心功能
- 调用 easytier-cli 获取信息
- 网络列表展示
- 节点状态展示

### Phase 3: 高级功能
- 服务管理
- 日志查看
- 配置编辑

## 快速开始

```bash
# 编译
cargo build --release

# 运行
cargo run
```

## 依赖

- easytier-core
- easytier-cli

确保 easytier-cli 已安装并在 PATH 中。

## License

MIT
