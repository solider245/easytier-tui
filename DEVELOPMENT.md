# EasyTier TUI 开发计划

## 项目概述
基于 Rust + Ratatui 的 EasyTier 终端管理界面 (TUI)

## 技术栈
- Rust
- Ratatui 0.26 (兼容 rustc 1.87)
- Crossterm 0.27

## 开发阶段

### Phase 1: 基础框架
- [x] 项目初始化
- [x] 基础 TUI 布局
- [ ] 菜单导航 (上下键选择)
- [ ] 输入处理

### Phase 2: 核心功能
- [ ] 调用 easytier-cli 获取信息
- [ ] 网络列表展示
- [ ] 节点状态展示

### Phase 3: 服务管理
- [ ] 服务启动/停止
- [ ] 日志查看
- [ ] 带宽统计

### Phase 4: 高级功能
- [ ] AI 接口 (JSON 输出)
- [ ] 多网络支持

## 关键命令

```bash
# 编译
cargo build --release

# 运行
cargo run

# 清理
cargo clean
```

## 依赖
- easytier-core
- easytier-cli
