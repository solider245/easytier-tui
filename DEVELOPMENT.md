# EasyTier TUI 开发计划

## 项目概述
基于 Rust + Ratatui 的 EasyTier 终端管理界面 (TUI)

## 技术栈
- Rust
- Ratatui 0.26 (兼容 rustc 1.87)
- Crossterm 0.27

## 已完成功能
- [x] 节点列表查看 (peer list)
- [x] 节点信息查看 (node)
- [x] 路由信息查看 (route)
- [x] 服务控制 (启动/停止)
- [x] 带宽统计 (stats show)
- [x] 网络诊断 (JSON 输出，AI 友好)
- [x] 多网络切换
- [x] 关于/帮助页面
- [x] 配置文件支持 (~/.easytier-tui.conf)
- [x] 环境变量支持 (EASYTIER_CLI)

## 快捷键
- ↑↓ - 上下选择
- Enter - 确认
- Esc - 返回
- r - 刷新数据
- q - 退出
- s - 启动服务 (服务控制页)
- x - 停止服务 (服务控制页)

## 配置文件
位置: `~/.easytier-tui.conf`

```toml
cli_path = "/usr/local/bin/easytier-cli"
rpc_portal = "127.0.0.1:15888"
```

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
- easytier-cli (需安装 EasyTier)
