use crate::app::App;
use crate::models::{ClusterState, DefragPhase};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

// -- UI Components ------------------------------------------------------------

pub struct TuiWrapper {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TuiWrapper {
    pub fn new() -> Result<Self, std::io::Error> {
        use crossterm::{
            terminal::{enable_raw_mode, EnterAlternateScreen},
            ExecutableCommand,
        };

        std::io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> Result<(), std::io::Error> {
        self.terminal.draw(f).map(|_| ())
    }

    pub fn cleanup(&mut self) -> Result<(), std::io::Error> {
        use crossterm::{
            terminal::{disable_raw_mode, LeaveAlternateScreen},
            ExecutableCommand,
        };

        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}

pub fn render_app(app: &App, frame: &mut Frame) {
    frame.render_widget(Block::new().style(Style::new().on_blue()), frame.area());

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(7),
        ])
        .split(frame.area());

    render_header(app, frame, main_layout[0]);

    let main_window_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .style(Style::new().on_blue());

    let grid_area = main_window_block.inner(main_layout[1]);

    frame.render_widget(main_window_block, main_layout[1]);
    render_grid(app, frame, grid_area);

    render_footer(app, frame, main_layout[2]);
    render_menu_dropdown(app, frame, frame.area());
    render_about_box(app, frame);
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let menu_names = get_menu_names();
    let mut spans = Vec::new();

    spans.push(Span::raw(" "));

    for (i, name) in menu_names.iter().enumerate() {
        let first_char = name.chars().next().unwrap_or(' ');
        let rest = &name[first_char.len_utf8()..];

        if app.menu_open && app.selected_menu == i {
            spans.push(Span::styled(
                format!(" {} ", name),
                Style::new().black().on_cyan(),
            ));
        } else {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                first_char.to_string(),
                Style::new().red().on_white(),
            ));
            spans.push(Span::styled(
                rest.to_string(),
                Style::new().black().on_white(),
            ));
        }
        spans.push(Span::styled("  ", Style::new().black().on_white()));
    }

    let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let padding = area.width as usize - current_len - 9;
    spans.push(Span::styled(
        " ".repeat(padding),
        Style::new().black().on_white(),
    ));
    spans.push(Span::styled("Esc=Quit", Style::new().black().on_white()));

    let header = Paragraph::new(Line::from(spans));
    frame.render_widget(header, area);
}

fn render_menu_dropdown(app: &App, frame: &mut Frame, area: Rect) {
    if !app.menu_open {
        return;
    }

    let items = get_menu_items(app.selected_menu);
    if items.is_empty() {
        return;
    }

    let menu_positions = [1, 12, 22, 29, 36];
    let menu_x = menu_positions.get(app.selected_menu).copied().unwrap_or(1) as u16;

    let max_width = items.iter().map(|s| s.len()).max().unwrap_or(10) + 4;
    let menu_height = items.len() as u16 + 2;

    let menu_area = Rect::new(area.x + menu_x, area.y + 1, max_width as u16, menu_height);

    let menu_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::new().bg(Color::White).fg(Color::Black));

    frame.render_widget(menu_block.clone(), menu_area);

    let inner = menu_block.inner(menu_area);
    for (i, item) in items.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }

        let item_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);

        if item.is_empty() {
            let sep = Paragraph::new("─".repeat(inner.width as usize))
                .style(Style::new().fg(Color::DarkGray).bg(Color::White));
            frame.render_widget(sep, item_area);
        } else if i == app.selected_item {
            let selected = Paragraph::new(format!(
                " {:<width$}",
                item,
                width = inner.width as usize - 1
            ))
            .style(Style::new().fg(Color::White).bg(Color::Black));
            frame.render_widget(selected, item_area);
        } else {
            let normal = Paragraph::new(format!(" {}", item))
                .style(Style::new().fg(Color::Black).bg(Color::White));
            frame.render_widget(normal, item_area);
        }
    }
}

fn render_grid(app: &App, frame: &mut Frame, area: Rect) {
    let grid_widget = DiskGridWidget {
        clusters: &app.clusters,
    };
    frame.render_widget(grid_widget, area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let footer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let top_border =
        "┌──────────────── Status ────────────────┐┌──────────────── Legend ────────────────┐";
    frame.render_widget(
        Paragraph::new(top_border).style(Style::new().on_blue()),
        footer_layout[0],
    );

    let percent = if app.stats.total_to_defrag == 0 {
        100.0
    } else {
        (app.stats.clusters_defragged as f32 / app.stats.total_to_defrag as f32) * 100.0
    };
    let line2_spans = vec![
        Span::raw(format!(
            "│ Cluster {:<6}                    {:>3}% │",
            app.stats.clusters_defragged,
            percent.min(100.0) as u8
        )),
        Span::raw("│ "),
        Span::styled("•", Style::new().fg(Color::Rgb(0, 200, 0))),
        Span::raw(" - Optimized    "),
        Span::styled("•", Style::new().white()),
        Span::raw(" - Fragmented        │"),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(line2_spans)).style(Style::new().on_blue()),
        footer_layout[1],
    );

    let progress_bar = create_progress_bar(percent);
    let line3_spans = vec![
        Span::raw(format!("│ {} │", progress_bar)),
        Span::raw("│ "),
        Span::styled("r", Style::new().fg(Color::Yellow).bg(Color::Blue)),
        Span::raw(" - Reading      "),
        Span::styled("W", Style::new().fg(Color::Green).bg(Color::Blue)),
        Span::raw(" - Writing           │"),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(line3_spans)).style(Style::new().on_blue()),
        footer_layout[2],
    );

    let elapsed = app.stats.start_time.elapsed();
    let elapsed_str = format!(
        "{:02}:{:02}:{:02}",
        elapsed.as_secs() / 3600,
        (elapsed.as_secs() % 3600) / 60,
        elapsed.as_secs() % 60
    );
    let remaining_str = if let Some(remaining) = app.estimated_time_remaining() {
        format!(
            " ETA {:02}:{:02}:{:02}",
            remaining.as_secs() / 3600,
            (remaining.as_secs() % 3600) / 60,
            remaining.as_secs() % 60
        )
    } else {
        String::new()
    };
    let time_display = format!("Time: {}{}", elapsed_str, remaining_str);
    let line4_spans = vec![
        Span::raw(format!("│ {:^38} │", time_display)),
        Span::raw("│ "),
        Span::styled("B", Style::new().fg(Color::Red).bg(Color::Black)),
        Span::raw(" - Bad block    "),
        Span::styled("X", Style::new().fg(Color::White).bg(Color::Blue)),
        Span::raw(" - Unmovable         │"),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(line4_spans)).style(Style::new().on_blue()),
        footer_layout[3],
    );

    let status_text = if let Some(filename) = &app.current_filename {
        let max_len = 38;
        let display_name = if filename.len() > max_len {
            &filename[..max_len]
        } else {
            filename
        };
        format!("File: {}", display_name)
    } else {
        "Full optimization".to_string()
    };

    let line5_content = format!("{:^38}", status_text);
    let line5 = format!(
        "│{}  ││ Drive C: ░ = Unused space              │",
        line5_content
    );

    frame.render_widget(
        Paragraph::new(line5).style(Style::new().on_blue()),
        footer_layout[4],
    );

    let bottom_border =
        "└────────────────────────────────────────┘└────────────────────────────────────────┘";
    frame.render_widget(
        Paragraph::new(bottom_border).style(Style::new().on_blue()),
        footer_layout[5],
    );

    let action_text = if app.paused {
        "[ PAUSED ]"
    } else {
        match app.phase {
            DefragPhase::Initializing => "Initializing...",
            DefragPhase::Analyzing => "Analyzing disk...",
            DefragPhase::Defragmenting => match app.animation_step % 3 {
                0 => "Reading...",
                1 => "Writing...",
                _ => "Updating FAT...",
            },
            DefragPhase::Finished => "Complete",
        }
    };

    let demo_indicator = if app.demo_mode { "[DEMO] " } else { "" };

    let sound_indicator = match &app.audio {
        Some(audio) if audio.is_enabled() => " [♪ ON] ",
        Some(_) => " [♪ OFF]",
        None => " [S=Sound]",
    };

    let version_text = "| MS-DOS defrag ";
    let total_width = area.width as usize;
    let action_len = action_text.len() + demo_indicator.len() + 2;
    let sound_len = sound_indicator.len();
    let version_len = version_text.len();
    let padding = total_width.saturating_sub(action_len + sound_len + version_len);

    let action_line = Paragraph::new(format!(
        "  {}{}{}{}{}",
        demo_indicator,
        action_text,
        " ".repeat(padding),
        sound_indicator,
        version_text
    ))
    .style(Style::new().on_red().white().bold());
    frame.render_widget(action_line, footer_layout[6]);
}

fn create_progress_bar(percent: f32) -> String {
    let bar_width: usize = 38;
    let clamped_percent = percent.min(100.0).max(0.0);
    let filled_width = ((clamped_percent / 100.0) * bar_width as f32) as usize;
    let empty_width = bar_width.saturating_sub(filled_width);
    format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width))
}

pub fn get_menu_items(menu_idx: usize) -> Vec<&'static str> {
    match menu_idx {
        0 => vec![
            "Begin optimization",
            "Drive...",
            "Optimization method...",
            "",
            "Exit",
        ],
        1 => vec!["Analyze drive", "File fragmentation..."],
        2 => vec!["Print disk map", "Save disk map..."],
        3 => vec![
            "Sort by name",
            "Sort by extension",
            "Sort by date",
            "Sort by size",
        ],
        4 => vec!["Contents", "About MS-DOS Defrag..."],
        _ => vec![],
    }
}

fn get_menu_names() -> Vec<&'static str> {
    vec!["Optimize", "Analyze", "File", "Sort", "Help"]
}

fn render_about_box(app: &App, frame: &mut Frame) {
    if !app.show_about_box {
        return;
    }

    let area = frame.area();

    let box_width = 52;
    let box_height = 18;
    let box_x = (area.width.saturating_sub(box_width)) / 2;
    let box_y = (area.height.saturating_sub(box_height)) / 2;

    let about_area = Rect::new(box_x, box_y, box_width, box_height);

    let shadow_area = Rect::new(box_x + 2, box_y + 1, box_width, box_height);
    frame.render_widget(
        Block::new().style(Style::new().bg(Color::Black)),
        shadow_area,
    );

    let about_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" About MS-DOS Defrag ")
        .title_alignment(Alignment::Center)
        .style(Style::new().bg(Color::Gray).fg(Color::Black));

    frame.render_widget(about_block.clone(), about_area);

    let inner = about_block.inner(about_area);

    let about_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            r"   ____  _____ _____ ____      _    ____",
            Style::new().fg(Color::Blue).bold(),
        )]),
        Line::from(vec![Span::styled(
            r"  |  _ \| ____|  ___|  _ \    / \  / ___|",
            Style::new().fg(Color::Blue).bold(),
        )]),
        Line::from(vec![Span::styled(
            r"  | | | |  _| | |_  | |_) |  / _ \| |  _",
            Style::new().fg(Color::Blue).bold(),
        )]),
        Line::from(vec![Span::styled(
            r"  | |_| | |___|  _| |  _ <  / ___ \ |_| |",
            Style::new().fg(Color::Cyan).bold(),
        )]),
        Line::from(vec![Span::styled(
            r"  |____/|_____|_|   |_| \_\/_/   \_\____|",
            Style::new().fg(Color::Cyan).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  MS-DOS Defrag Simulator v0.1.0",
            Style::new().fg(Color::Black).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Author: ", Style::new().fg(Color::DarkGray)),
            Span::styled(
                "Guillaume 'GuY' Gielly",
                Style::new().fg(Color::Black).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  License: ", Style::new().fg(Color::DarkGray)),
            Span::styled("GPL-v3", Style::new().fg(Color::Black)),
        ]),
        Line::from(vec![
            Span::styled("  Github: ", Style::new().fg(Color::DarkGray)),
            Span::styled(
                "github.com/ggielly/defrag-rs",
                Style::new().fg(Color::Blue).underlined(),
            ),
        ]),
        Line::from(""),
    ];

    let about_paragraph = Paragraph::new(about_text).style(Style::new().bg(Color::Gray));
    frame.render_widget(about_paragraph, inner);

    let button_width = 10;
    let button_x = inner.x + (inner.width.saturating_sub(button_width)) / 2;
    let button_y = inner.y + inner.height - 2;
    let button_area = Rect::new(button_x, button_y, button_width, 1);

    let ok_button = Paragraph::new("[   OK   ]")
        .style(Style::new().fg(Color::White).bg(Color::DarkGray).bold())
        .alignment(Alignment::Center);
    frame.render_widget(ok_button, button_area);
}

// -- Custom Grid Widget -------------------------------------------------------

struct DiskGridWidget<'a> {
    clusters: &'a [ClusterState],
}

impl Widget for DiskGridWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display_width = area.width as usize;
        if display_width == 0 {
            return;
        }

        for (y, row_chunks) in self.clusters.chunks(display_width).enumerate() {
            let row = y as u16;
            if row >= area.height {
                break;
            }
            for (x, cluster) in row_chunks.iter().enumerate() {
                let col = x as u16;
                if col >= area.width {
                    break;
                }
                let (symbol, style) = match cluster {
                    ClusterState::Used => (
                        "•",
                        Style::new()
                            .fg(Color::Rgb(0, 200, 0))
                            .bg(Color::Rgb(0, 100, 0)),
                    ),
                    ClusterState::Unused => ("░", Style::new().fg(Color::Gray).bg(Color::Blue)),
                    ClusterState::Pending => ("•", Style::new().fg(Color::Black).bg(Color::White)),
                    ClusterState::Bad => ("B", Style::new().fg(Color::Red).bg(Color::Black)),
                    ClusterState::Unmovable => ("X", Style::new().fg(Color::White).bg(Color::Blue)),
                    ClusterState::Reading => (
                        "r",
                        Style::new().fg(Color::Yellow).bg(Color::Rgb(0, 0, 139)),
                    ),
                    ClusterState::Writing => {
                        ("W", Style::new().fg(Color::Green).bg(Color::Rgb(0, 0, 139)))
                    }
                };
                if let Some(cell) = buf.cell_mut((area.x + col, area.y + row)) {
                    cell.set_symbol(symbol).set_style(style);
                }
            }
        }
    }
}
