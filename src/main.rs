use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_CLI_PATH: &str = "easytier-cli";
const CONFIG_FILE: &str = ".easytier-tui.conf";
const EASYTIER_DIR: &str = ".easytier";
const NOTES_FILE: &str = "notes.md";

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct Config {
    cli_path: Option<String>,
    rpc_portal: Option<String>,
    install_path: Option<String>,
}

struct VersionInfo {
    version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PeerInfo {
    #[serde(rename = "ipv4")]
    ipv4: Option<String>,
    #[serde(rename = "hostname")]
    hostname: Option<String>,
    #[serde(rename = "cost")]
    cost: Option<String>,
    #[serde(rename = "lat_ms")]
    lat_ms: Option<String>,
    #[serde(rename = "rx_bytes")]
    rx_bytes: Option<String>,
    #[serde(rename = "tx_bytes")]
    tx_bytes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct NodeInfo {
    #[serde(rename = "instance_name")]
    instance_name: Option<String>,
    #[serde(rename = "ipv4")]
    ipv4: Option<String>,
    #[serde(rename = "ipv6")]
    ipv6: Option<String>,
    #[serde(rename = "hostname")]
    hostname: Option<String>,
    #[serde(rename = "core_version")]
    core_version: Option<String>,
    #[serde(rename = "listeners")]
    listeners: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RouteInfo {
    #[serde(rename = "dest")]
    dest: Option<String>,
    #[serde(rename = "gateway")]
    gateway: Option<String>,
    #[serde(rename = "interface")]
    interface: Option<String>,
    #[serde(rename = "metric")]
    metric: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct StatsInfo {
    #[serde(rename = "global")]
    global: Option<GlobalStats>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GlobalStats {
    #[serde(rename = "rx_bytes")]
    rx_bytes: Option<String>,
    #[serde(rename = "tx_bytes")]
    tx_bytes: Option<String>,
    #[serde(rename = "rx_packets")]
    rx_packets: Option<u64>,
    #[serde(rename = "tx_packets")]
    tx_packets: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuPage {
    Main,
    Install,
    Deploy,
    JoinNetwork,
    TokenJoin,
    ServiceManage,
    PeerList,
    NodeInfo,
    RouteList,
    BandwidthStats,
    VpnPortal,
    Diagnostics,
    ConfigFile,
    Notes,
    About,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    None,
    NetworkName,
    NetworkSecret,
    Token,
    Notes,
}

struct InputState {
    mode: InputMode,
    value: String,
    cursor_position: usize,
}

struct AppState {
    selected: usize,
    items: Vec<String>,
    current_page: MenuPage,
    input_mode: InputMode,
    input_value: String,
    peers: Vec<PeerInfo>,
    node_info: Option<NodeInfo>,
    routes: Vec<RouteInfo>,
    stats: Option<GlobalStats>,
    status_message: String,
    logs: Vec<String>,
    service_running: bool,
    current_network: String,
    network_list: Vec<String>,
    diagnostics_json: String,
    notes_content: String,
    config_content: String,
    install_path: String,
    is_installing: bool,
    install_progress: String,
}

impl AppState {
    fn new() -> Self {
        let install_path = dirs::home_dir()
            .map(|p| p.join(EASYTIER_DIR).to_string_lossy().to_string())
            .unwrap_or_else(|| format!("~/.{}", EASYTIER_DIR));

        Self {
            selected: 0,
            items: vec![
                "1. 安装/更新 EasyTier".to_string(),
                "2. 部署新网络 (服务器)".to_string(),
                "3. 加入现有网络".to_string(),
                "4. Token 加入网络".to_string(),
                "5. 服务管理".to_string(),
                "6. 节点列表".to_string(),
                "7. 路由信息".to_string(),
                "8. 节点信息".to_string(),
                "9. 带宽统计".to_string(),
                "10. VPN Portal".to_string(),
                "11. 网络诊断 (JSON)".to_string(),
                "12. 配置文件".to_string(),
                "13. 笔记/FAQ".to_string(),
                "14. 关于".to_string(),
            ],
            current_page: MenuPage::Main,
            input_mode: InputMode::None,
            input_value: String::new(),
            peers: Vec::new(),
            node_info: None,
            routes: Vec::new(),
            stats: None,
            status_message: "就绪".to_string(),
            logs: Vec::new(),
            service_running: false,
            current_network: "default".to_string(),
            network_list: vec!["default".to_string()],
            diagnostics_json: String::new(),
            notes_content: String::new(),
            config_content: String::new(),
            install_path,
            is_installing: false,
            install_progress: String::new(),
        }
    }

    fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn down(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }

    fn enter(&mut self) {
        match self.selected {
            0 => {
                self.current_page = MenuPage::Install;
                self.status_message = "安装/更新 EasyTier".to_string();
            }
            1 => {
                self.input_mode = InputMode::NetworkName;
                self.input_value = String::new();
                self.current_page = MenuPage::Input;
                self.status_message = "输入网络名称 (留空自动生成)".to_string();
            }
            2 => {
                self.input_mode = InputMode::NetworkName;
                self.input_value = String::new();
                self.current_page = MenuPage::Input;
                self.status_message = "输入网络名称".to_string();
            }
            3 => {
                self.input_mode = InputMode::Token;
                self.input_value = String::new();
                self.current_page = MenuPage::Input;
                self.status_message = "输入 Token (easytier://...)".to_string();
            }
            4 => {
                self.current_page = MenuPage::ServiceManage;
                self.check_service_status();
            }
            5 => {
                self.current_page = MenuPage::PeerList;
                self.load_peers();
            }
            6 => {
                self.current_page = MenuPage::RouteList;
                self.load_routes();
            }
            7 => {
                self.current_page = MenuPage::NodeInfo;
                self.load_node_info();
            }
            8 => {
                self.current_page = MenuPage::BandwidthStats;
                self.load_stats();
            }
            9 => {
                self.current_page = MenuPage::VpnPortal;
                self.load_vpn_portal();
            }
            10 => {
                self.current_page = MenuPage::Diagnostics;
                self.run_diagnostics();
            }
            11 => {
                self.current_page = MenuPage::ConfigFile;
                self.load_config_file();
            }
            12 => {
                self.current_page = MenuPage::Notes;
                self.load_notes();
            }
            13 => {
                self.current_page = MenuPage::About;
                self.status_message = format!("EasyTier TUI v{}", APP_VERSION);
            }
            _ => {}
        }
    }

    fn back(&mut self) {
        self.current_page = MenuPage::Main;
        self.input_mode = InputMode::None;
        self.input_value = String::new();
        self.status_message = "返回主菜单".to_string();
    }

    fn handle_input(&mut self, c: char) {
        self.input_value.push(c);
    }

    fn handle_backspace(&mut self) {
        self.input_value.pop();
    }

    fn submit_input(&mut self) {
        match self.input_mode {
            InputMode::NetworkName => {
                self.deploy_network();
            }
            InputMode::Token => {
                self.join_by_token();
            }
            _ => {}
        }
        self.input_mode = InputMode::None;
    }

    fn get_cli_path(&self) -> String {
        if let Ok(path) = std::env::var("EASYTIER_CLI") {
            return path;
        }

        if let Some(config) = self.load_config() {
            if let Some(cli_path) = config.cli_path {
                return cli_path;
            }
        }

        DEFAULT_CLI_PATH.to_string()
    }

    fn get_core_path(&self) -> String {
        format!("{}/easytier-core", self.install_path)
    }

    fn load_config(&self) -> Option<Config> {
        let config_path = dirs::home_dir()?.join(CONFIG_FILE);
        if config_path.exists() {
            std::fs::read_to_string(config_path)
                .ok()
                .and_then(|content| toml::from_str(&content).ok())
        } else {
            None
        }
    }

    fn check_service_status(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path).args(["service", "status"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            self.service_running = stdout.contains("running") || stdout.contains("started");
            self.status_message = if self.service_running {
                "服务正在运行".to_string()
            } else {
                "服务未运行".to_string()
            };
        } else {
            self.service_running = false;
            self.status_message = "无法检查服务状态".to_string();
        }
    }

    fn start_service(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path).args(["service", "start"]).output() {
            if output.status.success() {
                self.service_running = true;
                self.status_message = "服务已启动".to_string();
            } else {
                self.status_message = "启动失败".to_string();
            }
        } else {
            self.status_message = "启动失败".to_string();
        }
    }

    fn stop_service(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path).args(["service", "stop"]).output() {
            if output.status.success() {
                self.service_running = false;
                self.status_message = "服务已停止".to_string();
            } else {
                self.status_message = "停止失败".to_string();
            }
        } else {
            self.status_message = "停止失败".to_string();
        }
    }

    fn install_easytier(&mut self) {
        self.status_message = "正在获取最新版本...".to_string();

        // 检测平台
        let (platform, extension) = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                ("macos-aarch64", "zip")
            } else {
                ("macos-x86_64", "zip")
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "x86_64") {
                ("linux-x86_64", "zip")
            } else if cfg!(target_arch = "aarch64") {
                ("linux-aarch64", "zip")
            } else {
                ("linux-x86_64", "zip")
            }
        } else {
            ("windows-x86_64", "zip")
        };

        // 获取最新版本信息
        let version_info = match self.fetch_latest_version() {
            Ok(v) => v,
            Err(e) => {
                self.status_message = format!("获取版本失败: {}", e);
                return;
            }
        };

        self.status_message = format!("准备下载 v{} ...", version_info.version);

        // 下载文件
        let filename = format!(
            "easytier-{}-v{}.{}",
            platform, version_info.version, extension
        );
        let download_url = format!(
            "https://github.com/EasyTier/EasyTier/releases/download/v{}/{}",
            version_info.version, filename
        );

        match self.download_and_install(&download_url, &filename, &version_info.version, platform) {
            Ok(_) => {
                self.status_message = "安装成功！".to_string();
            }
            Err(e) => {
                self.status_message = format!("安装失败: {}", e);
            }
        }
    }

    fn fetch_latest_version(&self) -> Result<VersionInfo, String> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://api.github.com/repos/EasyTier/EasyTier/releases/latest")
            .header("User-Agent", "easytier-tui")
            .send()
            .map_err(|e| format!("请求失败: {}", e))?;

        let json: serde_json::Value =
            serde_json::from_str(&response.text().map_err(|e| e.to_string())?)
                .map_err(|e| format!("解析失败: {}", e))?;

        let version = json["tag_name"]
            .as_str()
            .unwrap_or("v2.4.5")
            .trim_start_matches('v')
            .to_string();

        Ok(VersionInfo { version })
    }

    fn download_and_install(
        &mut self,
        url: &str,
        filename: &str,
        version: &str,
        _platform: &str,
    ) -> Result<(), String> {
        // 创建安装目录
        let install_dir = dirs::home_dir()
            .ok_or("无法获取用户目录")?
            .join(EASYTIER_DIR);

        std::fs::create_dir_all(&install_dir).map_err(|e| format!("创建目录失败: {}", e))?;

        // 下载文件
        self.status_message = "正在下载...".to_string();

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "easytier-tui")
            .send()
            .map_err(|e| format!("下载失败: {}", e))?;

        let bytes = response.bytes().map_err(|e| format!("读取失败: {}", e))?;

        // 保存临时文件
        let temp_path = install_dir.join(filename);
        std::fs::write(&temp_path, &bytes).map_err(|e| format!("保存失败: {}", e))?;

        self.status_message = "正在解压...".to_string();

        // 解压
        let file = std::fs::File::open(&temp_path).map_err(|e| format!("打开文件失败: {}", e))?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解压失败: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("读取失败: {}", e))?;

            let outpath = install_dir.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p).map_err(|e| format!("创建目录失败: {}", e))?;
                }

                let mut outfile =
                    std::fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入失败: {}", e))?;
            }
        }

        // 设置可执行权限
        let core_path = install_dir.join("easytier-core");
        let cli_path = install_dir.join("easytier-cli");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if core_path.exists() {
                let mut perms = std::fs::metadata(&core_path)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&core_path, perms).map_err(|e| e.to_string())?;
            }
            if cli_path.exists() {
                let mut perms = std::fs::metadata(&cli_path)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&cli_path, perms).map_err(|e| e.to_string())?;
            }
        }

        // 清理临时文件
        let _ = std::fs::remove_file(&temp_path);

        // 更新 CLI 路径
        self.install_path = install_dir.to_string_lossy().to_string();

        Ok(())
    }

    fn deploy_network(&mut self) {
        let network_name = if self.input_value.is_empty() {
            // 自动生成
            format!("net-{}", rand_string(8))
        } else {
            self.input_value.clone()
        };

        self.status_message = format!("部署网络: {}", network_name);

        // TODO: 实际部署逻辑
        // 1. 生成配置
        // 2. 启动 easytier-core
        self.input_value = String::new();
    }

    fn join_by_token(&mut self) {
        let token = self.input_value.clone();
        self.status_message = format!("通过 Token 加入网络: {}", &token[..token.len().min(30)]);

        // TODO: 解析 Token 并加入
        self.input_value = String::new();
    }

    fn load_peers(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path)
            .args(["-o", "json", "peer", "list"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(peers) = serde_json::from_str::<Vec<PeerInfo>>(&stdout) {
                    self.peers = peers;
                    self.status_message = format!("已加载 {} 个节点", self.peers.len());
                    return;
                }
            }
        }
        self.status_message = "easytier-core 未运行".to_string();
        self.peers.clear();
    }

    fn load_node_info(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path)
            .args(["-o", "json", "node"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(info) = serde_json::from_str::<NodeInfo>(&stdout) {
                    self.node_info = Some(info);
                    self.status_message = "节点信息已加载".to_string();
                    return;
                }
            }
        }
        self.status_message = "easytier-core 未运行".to_string();
        self.node_info = None;
    }

    fn load_routes(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path)
            .args(["-o", "json", "route"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(routes) = serde_json::from_str::<Vec<RouteInfo>>(&stdout) {
                    self.routes = routes;
                    self.status_message = format!("已加载 {} 条路由", self.routes.len());
                    return;
                }
            }
        }
        self.status_message = "easytier-core 未运行".to_string();
        self.routes.clear();
    }

    fn load_stats(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path)
            .args(["-o", "json", "stats", "show"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(stats) = serde_json::from_str::<StatsInfo>(&stdout) {
                    self.stats = stats.global;
                    self.status_message = "带宽统计已加载".to_string();
                    return;
                }
            }
        }
        self.status_message = "easytier-core 未运行".to_string();
        self.stats = None;
    }

    fn load_vpn_portal(&mut self) {
        let cli_path = self.get_cli_path();

        if let Ok(output) = Command::new(&cli_path)
            .args(["-o", "json", "vpn-portal"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                self.config_content = stdout.to_string();
                self.status_message = "VPN Portal 配置已加载".to_string();
                return;
            }
        }
        self.status_message = "easytier-core 未运行".to_string();
    }

    fn load_config_file(&mut self) {
        let config_path = dirs::home_dir()
            .map(|p| p.join(EASYTIER_DIR).join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(format!("~/.{}/config.toml", EASYTIER_DIR)));

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                self.config_content = content;
                self.status_message = "配置文件已加载".to_string();
                return;
            }
        }
        self.config_content = "配置文件不存在".to_string();
        self.status_message = "无配置文件".to_string();
    }

    fn load_notes(&mut self) {
        let notes_path = dirs::home_dir()
            .map(|p| p.join(EASYTIER_DIR).join(NOTES_FILE))
            .unwrap_or_else(|| PathBuf::from(format!("~/.{}/{}", EASYTIER_DIR, NOTES_FILE)));

        if notes_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&notes_path) {
                self.notes_content = content;
                self.status_message = "笔记已加载".to_string();
                return;
            }
        }
        self.notes_content = "# 笔记/FAQ\n\n在此记录您的笔记...".to_string();
    }

    fn run_diagnostics(&mut self) {
        self.load_node_info();
        self.load_peers();
        self.load_routes();
        self.load_stats();

        let diagnostics = serde_json::json!({
            "node_info": self.node_info,
            "peers": self.peers,
            "routes": self.routes,
            "stats": self.stats,
            "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&diagnostics) {
            self.diagnostics_json = json;
            self.status_message = "诊断完成".to_string();
        }
    }
}

fn rand_string(len: usize) -> String {
    use std::iter;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    iter::repeat_with(|| CHARSET[rand_index(CHARSET.len())] as char)
        .take(len)
        .collect()
}

fn rand_index(max: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;
    nanos % max
}

fn main() -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut state = AppState::new();

    terminal.clear()?;

    let _ = enable_raw_mode();
    let _ = terminal.backend_mut().execute(EnableMouseCapture);

    let result = run_app(&mut terminal, &mut state);

    let _ = disable_raw_mode();
    let _ = terminal.backend_mut().execute(DisableMouseCapture);
    let _ = terminal.backend_mut().execute(LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn run_app<W: Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    state: &mut AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let title = Paragraph::new(format!(
                " EasyTier TUI v{} | 方向键选择, Enter确认, Esc返回, q退出 ",
                APP_VERSION
            ))
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" EasyTier 管理器 "),
            );
            f.render_widget(title, chunks[0]);

            match state.current_page {
                MenuPage::Main => {
                    let items: Vec<ListItem> = state
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            let style = if i == state.selected {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(ratatui::style::Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };
                            ListItem::new(item.as_str()).style(style)
                        })
                        .collect();
                    let list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title(" 主菜单 "))
                        .highlight_style(Style::default().fg(Color::Yellow))
                        .highlight_symbol(">> ");
                    f.render_widget(list, chunks[1]);
                }
                MenuPage::Install => {
                    let content = format!(
                        "【安装/更新 EasyTier】\n\n\
                        安装路径: {}\n\n\
                        操作说明:\n\
                          i - 开始安装/更新\n\
                          b - 返回\n\n\
                        提示: 将从 GitHub 下载最新版本",
                        state.install_path
                    );
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" 安装 EasyTier "),
                        );
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Deploy => {
                    let info = Paragraph::new("【部署新网络】")
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 部署网络 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::JoinNetwork => {
                    let info = Paragraph::new("【加入现有网络】")
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 加入网络 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::TokenJoin => {
                    let info = Paragraph::new("【Token 加入网络】")
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" Token 加入 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::ServiceManage => {
                    let service_status = if state.service_running {
                        "[运行中] 🟢"
                    } else {
                        "[未运行] 🔴"
                    };
                    let content = format!(
                        "【服务管理】\n\n\
                        服务状态: {}\n\n\
                        操作说明:\n\
                          s - 启动服务\n\
                          x - 停止服务\n\
                          r - 刷新状态\n\
                          b - 返回\n",
                        service_status
                    );
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 服务管理 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::PeerList => {
                    if state.peers.is_empty() {
                        let msg = Paragraph::new(
                            "无节点信息 (请确保 easytier-core 已运行)\n\n提示: r 刷新, b 返回",
                        )
                        .style(Style::default().fg(Color::Yellow))
                        .block(Block::default().borders(Borders::ALL).title(" 节点列表 "));
                        f.render_widget(msg, chunks[1]);
                    } else {
                        let rows: Vec<Row> = state
                            .peers
                            .iter()
                            .map(|p| {
                                Row::new(vec![
                                    p.ipv4.clone().unwrap_or_default(),
                                    p.hostname.clone().unwrap_or_default(),
                                    p.cost.clone().unwrap_or_default(),
                                    p.lat_ms.clone().unwrap_or_default(),
                                    p.rx_bytes.clone().unwrap_or_default(),
                                    p.tx_bytes.clone().unwrap_or_default(),
                                ])
                            })
                            .collect();

                        let table = Table::new(
                            rows,
                            [
                                Constraint::Length(18),
                                Constraint::Length(15),
                                Constraint::Length(10),
                                Constraint::Length(10),
                                Constraint::Length(12),
                                Constraint::Length(12),
                            ],
                        )
                        .header(
                            Row::new(vec!["IPv4", "主机名", "状态", "延迟", "接收", "发送"])
                                .style(Style::default().fg(Color::Cyan)),
                        )
                        .block(Block::default().borders(Borders::ALL).title(" 节点列表 "))
                        .style(Style::default().fg(Color::White));
                        f.render_widget(table, chunks[1]);
                    }
                }
                MenuPage::NodeInfo => {
                    let content =
                        if let Some(ref node) = state.node_info {
                            format!(
                            "实例名: {}\nIPv4: {}\nIPv6: {}\n主机名: {}\n版本: {}\n监听端口: {}",
                            node.instance_name.as_deref().unwrap_or("N/A"),
                            node.ipv4.as_deref().unwrap_or("N/A"),
                            node.ipv6.as_deref().unwrap_or("N/A"),
                            node.hostname.as_deref().unwrap_or("N/A"),
                            node.core_version.as_deref().unwrap_or("N/A"),
                            node.listeners.as_ref().map(|l| l.join(", ")).unwrap_or_default(),
                        )
                        } else {
                            "无节点信息\n\n提示: r 刷新, b 返回".to_string()
                        };
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 节点信息 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::RouteList => {
                    if state.routes.is_empty() {
                        let msg = Paragraph::new("无路由信息\n\n提示: r 刷新, b 返回")
                            .style(Style::default().fg(Color::Yellow))
                            .block(Block::default().borders(Borders::ALL).title(" 路由列表 "));
                        f.render_widget(msg, chunks[1]);
                    } else {
                        let rows: Vec<Row> = state
                            .routes
                            .iter()
                            .map(|r| {
                                Row::new(vec![
                                    r.dest.clone().unwrap_or_default(),
                                    r.gateway.clone().unwrap_or_default(),
                                    r.interface.clone().unwrap_or_default(),
                                    r.metric.clone().unwrap_or_default(),
                                ])
                            })
                            .collect();

                        let table = Table::new(
                            rows,
                            [
                                Constraint::Length(20),
                                Constraint::Length(18),
                                Constraint::Length(15),
                                Constraint::Length(10),
                            ],
                        )
                        .header(
                            Row::new(vec!["目标", "网关", "接口", "Metric"])
                                .style(Style::default().fg(Color::Cyan)),
                        )
                        .block(Block::default().borders(Borders::ALL).title(" 路由列表 "))
                        .style(Style::default().fg(Color::White));
                        f.render_widget(table, chunks[1]);
                    }
                }
                MenuPage::BandwidthStats => {
                    let content = if let Some(ref stats) = state.stats {
                        format!(
                            "接收: {}\n发送: {}\n接收包: {}\n发送包: {}\n\n提示: r 刷新, b 返回",
                            stats.rx_bytes.as_deref().unwrap_or("N/A"),
                            stats.tx_bytes.as_deref().unwrap_or("N/A"),
                            stats.rx_packets.map(|n| n.to_string()).unwrap_or_default(),
                            stats.tx_packets.map(|n| n.to_string()).unwrap_or_default(),
                        )
                    } else {
                        "无统计信息\n\n提示: r 刷新, b 返回".to_string()
                    };
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 带宽统计 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::VpnPortal => {
                    let info = Paragraph::new(state.config_content.clone())
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" VPN Portal "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Diagnostics => {
                    let info = Paragraph::new(state.diagnostics_json.clone())
                        .style(Style::default().fg(Color::White))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" 网络诊断 (JSON) "),
                        );
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::ConfigFile => {
                    let info = Paragraph::new(state.config_content.clone())
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 配置文件 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Notes => {
                    let info = Paragraph::new(state.notes_content.clone())
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 笔记/FAQ "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Diagnostics => {
                    let info = Paragraph::new(state.diagnostics_json.clone())
                        .style(Style::default().fg(Color::White))
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(" 网络诊断 (JSON) "),
                        );
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::ConfigFile => {
                    let info = Paragraph::new(state.config_content.clone())
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 配置文件 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Notes => {
                    let info = Paragraph::new(state.notes_content.clone())
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 笔记/FAQ "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::About => {
                    let content = format!(
                        " EasyTier TUI v{}\n\n\
                        基于 Rust + Ratatui 开发\n\
                        用于管理 EasyTier 网络\n\n\
                        功能:\n\
                          - 安装/更新 EasyTier\n\
                          - 部署/加入网络\n\
                          - 服务管理\n\
                          - 节点/路由监控\n\
                          - 网络诊断\n\
                          - 配置文件管理\n\
                          - 笔记功能\n\n\
                        按 b 返回",
                        APP_VERSION
                    );
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 关于 "));
                    f.render_widget(info, chunks[1]);
                }
                MenuPage::Input => {
                    let prompt = match state.input_mode {
                        InputMode::NetworkName => "输入网络名称 (留空自动生成)",
                        InputMode::NetworkSecret => "输入网络密钥",
                        InputMode::Token => "输入 Token (easytier://...)",
                        _ => "输入",
                    };
                    let content = format!(
                        "{}\n\n当前输入: {}\n\nEnter 确认, Esc 取消",
                        prompt, state.input_value
                    );
                    let info = Paragraph::new(content)
                        .style(Style::default().fg(Color::White))
                        .block(Block::default().borders(Borders::ALL).title(" 输入 "));
                    f.render_widget(info, chunks[1]);
                }
            }

            let status = Paragraph::new(state.status_message.clone())
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[2]);
        })?;

        if let Ok(event) = event::read() {
            match event {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Esc => state.back(),
                            KeyCode::Char('b') => state.back(),
                            KeyCode::Up => state.up(),
                            KeyCode::Down => state.down(),
                            KeyCode::Enter => {
                                if state.current_page == MenuPage::Input {
                                    state.submit_input();
                                    state.back();
                                } else {
                                    state.enter();
                                }
                            }
                            KeyCode::Char('r') => match state.current_page {
                                MenuPage::PeerList => state.load_peers(),
                                MenuPage::NodeInfo => state.load_node_info(),
                                MenuPage::RouteList => state.load_routes(),
                                MenuPage::ServiceManage => state.check_service_status(),
                                MenuPage::BandwidthStats => state.load_stats(),
                                MenuPage::VpnPortal => state.load_vpn_portal(),
                                MenuPage::Diagnostics => state.run_diagnostics(),
                                MenuPage::ConfigFile => state.load_config_file(),
                                MenuPage::Notes => state.load_notes(),
                                _ => {}
                            },
                            KeyCode::Char('i') => {
                                if state.current_page == MenuPage::Install {
                                    state.install_easytier();
                                }
                            }
                            KeyCode::Char('s') => {
                                if state.current_page == MenuPage::ServiceManage {
                                    state.start_service();
                                }
                            }
                            KeyCode::Char('x') => {
                                if state.current_page == MenuPage::ServiceManage {
                                    state.stop_service();
                                }
                            }
                            KeyCode::Char(c) => {
                                if state.current_page == MenuPage::Input {
                                    state.handle_input(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if state.current_page == MenuPage::Input {
                                    state.handle_backspace();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
