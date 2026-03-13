use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

fn main() -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

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

            let title = Paragraph::new("EasyTier TUI - 按 q 退出")
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("标题"));
            f.render_widget(title, chunks[0]);

            let items = [
                ListItem::new("1. 网络管理"),
                ListItem::new("2. 节点查看"),
                ListItem::new("3. 服务控制"),
                ListItem::new("4. 查看日志"),
            ];
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("菜单"))
                .style(Style::default().fg(Color::White));
            f.render_widget(list, chunks[1]);

            let status = Paragraph::new("状态: 就绪")
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[2]);
        })?;

        if let Ok(event) = crossterm::event::read() {
            if let crossterm::event::Event::Key(key) = event {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
