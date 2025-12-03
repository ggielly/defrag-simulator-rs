use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, Paragraph, Widget},
};
use rand::prelude::SliceRandom;
use std::{
    io::{stdout, Result},
    sync::mpsc,
    time::{Duration, Instant},
};
use clap::Parser;

// -- CLI Arguments ------------------------------------------------------------

#[derive(clap::Parser)]
#[command(name = "defrag", version = "0.1.0", about = "MS-DOS Defragmenter Simulation")]
struct Args {
    /// Animation speed: fast, normal, or slow
    #[arg(long, default_value = "normal")]
    speed: String,

    /// Grid size in format WxH (e.g., 85x20)
    #[arg(long, default_value = "78x16")]
    size: String,

    /// Initial disk fill percentage
    #[arg(long, default_value_t = 0.65)]
    fill: f32,
}

// -- Application State --------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum ClusterState {
    Used,
    Free,
    Bad,
    Reading,
    Writing,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum DefragPhase {
    Initializing,
    Analyzing,
    Defragmenting,
    Finished,
}

struct App {
    running: bool,
    tick_rate: Duration,
    width: usize,
    height: usize,
    clusters: Vec<ClusterState>,
    stats: DefragStats,
    phase: DefragPhase,
    // Animation state
    animation_step: u64,
    read_pos: Option<usize>,
    write_pos: Option<usize>,
}

struct DefragStats {
    used_clusters: usize,
    moved_clusters: usize,
    start_time: Instant,
}

// -- Main Application Logic ---------------------------------------------------

fn main() -> Result<()> {
    let args = Args::parse();
    let (width, height) = parse_size(&args.size).unwrap_or((78, 16));

    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Setup Ctrl+C handler
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    // Create and run app
    let mut app = App::new(width, height, args.fill);
    app.run(&mut terminal, rx)?;

    // Restore terminal
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

impl App {
    fn new(width: usize, height: usize, fill_percent: f32) -> Self {
        let total_clusters = width * height;
        let mut clusters = vec![ClusterState::Free; total_clusters];
        let mut used_clusters = 0;

        // Generate fragmented disk
        let mut rng = rand::thread_rng();
        let num_used = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * 0.02) as usize;

        let mut positions: Vec<usize> = (0..total_clusters).collect();
        positions.shuffle(&mut rng);

        for &pos in positions.iter().take(num_bad) {
            clusters[pos] = ClusterState::Bad;
        }
        for &pos in positions.iter().skip(num_bad).take(num_used) {
            clusters[pos] = ClusterState::Used;
            used_clusters += 1;
        }

        Self {
            running: true,
            tick_rate: Duration::from_millis(100), // ~10 FPS
            width,
            height,
            clusters,
            stats: DefragStats {
                used_clusters,
                moved_clusters: 0,
                start_time: Instant::now(),
            },
            phase: DefragPhase::Initializing,
            animation_step: 0,
            read_pos: None,
            write_pos: None,
        }
    }

    fn run(&mut self, term: &mut Terminal<impl Backend>, rx: mpsc::Receiver<()>) -> Result<()> {
        let mut last_tick = Instant::now();
        while self.running {
            term.draw(|frame| self.render(frame))?;

            // Handle Ctrl+C
            if rx.try_recv().is_ok() {
                self.running = false;
            }

            // Handle other keyboard events
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            self.running = false;
                        }
                    }
                }
            }

            // Update app state on each tick
            if last_tick.elapsed() >= self.tick_rate {
                self.update();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn update(&mut self) {
        self.animation_step += 1;

        match self.phase {
            DefragPhase::Initializing => {
                // Wait for a bit before starting analysis
                if self.animation_step > 20 { // 2 seconds
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            DefragPhase::Analyzing => {
                // Simulate analysis by showing a sweeping "Reading" block
                let total_clusters = self.width * self.height;
                let scan_pos = (self.animation_step as usize * 5).min(total_clusters-1);
                self.read_pos = Some(scan_pos);

                if self.animation_step > (total_clusters as u64 / 5) + 10 {
                    self.read_pos = None;
                    self.phase = DefragPhase::Defragmenting;
                    self.animation_step = 0;
                }
            }
            DefragPhase::Defragmenting => {
                // On each tick, perform one step of the defrag animation
                if self.read_pos.is_some() && self.write_pos.is_some() {
                    // Step 3: We have shown R and W, now perform the swap
                    let read_idx = self.read_pos.unwrap();
                    let write_idx = self.write_pos.unwrap();
                    self.clusters[write_idx] = ClusterState::Used;
                    self.clusters[read_idx] = ClusterState::Free;
                    self.stats.moved_clusters += 1;
                    self.read_pos = None;
                    self.write_pos = None;
                } else if self.read_pos.is_some() {
                    // Step 2: We have shown R, now show W
                    self.clusters[self.read_pos.unwrap()] = ClusterState::Reading; // Keep it as R
                    let hole_idx = self.clusters.iter().position(|&c| c == ClusterState::Free).unwrap();
                    self.clusters[hole_idx] = ClusterState::Writing;
                    self.write_pos = Some(hole_idx);
                } else {
                    // Step 1: Find a block to move and mark it for reading
                    let first_hole = self.clusters.iter().position(|&c| c == ClusterState::Free);
                    let last_used = self.clusters.iter().rposition(|&c| c == ClusterState::Used);

                    if let (Some(hole_idx), Some(used_idx)) = (first_hole, last_used) {
                        if used_idx > hole_idx {
                            self.clusters[used_idx] = ClusterState::Reading;
                            self.read_pos = Some(used_idx);
                        } else {
                            self.phase = DefragPhase::Finished; // Defrag complete
                        }
                    } else {
                        self.phase = DefragPhase::Finished; // No more blocks to move
                    }
                }
            }
            DefragPhase::Finished => {
                // Wait for a bit then quit
                if self.animation_step > 50 { // 5 seconds
                    self.running = false;
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Clear the frame with a blue background
        frame.render_widget(Block::new().style(Style::new().on_blue()), frame.area());

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Main window (grid + borders)
                Constraint::Length(7), // Footer
            ])
            .split(frame.area());

        self.render_header(frame, main_layout[0]);

        // Create the main window with a double border
        let main_window_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::new().on_blue());
        
        // Get the inner area for the grid
        let grid_area = main_window_block.inner(main_layout[1]);
        
        // Render the block first, then the grid inside
        frame.render_widget(main_window_block, main_layout[1]);
        self.render_grid(frame, grid_area);

        self.render_footer(frame, main_layout[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_text = "  Optimize   Analyze   File   Sort                                           Help   ";
        let header = Paragraph::new(header_text)
            .style(Style::new().on_white().black());
        frame.render_widget(header, area);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect) {
        let grid_widget = DiskGridWidget {
            clusters: &self.clusters,
        };
        frame.render_widget(grid_widget, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        // This is the detailed footer implementation from before
        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top border
                Constraint::Length(1), // Row 1
                Constraint::Length(1), // Row 2
                Constraint::Length(1), // Row 3
                Constraint::Length(1), // Row 4
                Constraint::Length(1), // Bottom border
                Constraint::Length(1), // Action line
            ])
            .split(area);

        // Line 1: Top Border
        let top_border = "┌──────────────── Status ────────────────┐┌──────────────── Legend ────────────────┐";
        frame.render_widget(Paragraph::new(top_border).style(Style::new().on_blue()), footer_layout[0]);

        // Line 2: Status (Cluster, Percent) + Legend (Used, Free)
        let percent = if self.stats.used_clusters == 0 { 100.0 } else { (self.stats.moved_clusters as f32 / self.stats.used_clusters as f32) * 100.0 };
        let line2_spans = vec![
            Span::raw(format!("│ Cluster {:<6}                    {:>3}% │", self.stats.moved_clusters, percent as u8)),
            Span::raw("│ "),
            Span::styled("█", Style::new().white()),
            Span::raw(" - Used         "),
            Span::styled("░", Style::new().gray()),
            Span::raw(" - Free          │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line2_spans)).style(Style::new().on_blue()), footer_layout[1]);

        // Line 3: Status (Progress Bar) + Legend (Reading, Writing)
        let progress_bar = self.create_progress_bar(percent);
        let line3_spans = vec![
            Span::raw(format!("│ {} │", progress_bar)),
            Span::raw("│ "),
            Span::styled("R", Style::new().yellow()),
            Span::raw(" - Reading      "),
            Span::styled("W", Style::new().green()),
            Span::raw(" - Writing         │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line3_spans)).style(Style::new().on_blue()), footer_layout[2]);

        // Line 4: Status (Elapsed Time) + Legend (Bad, Unmovable)
        let elapsed = self.stats.start_time.elapsed();
        let elapsed_str = format!(
            "{:02}:{:02}:{:02}",
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60
        );
        let line4_spans = vec![
            Span::raw(format!("│ {:^38} │", elapsed_str)),
            Span::raw("│ "),
            Span::styled("B", Style::new().red()),
            Span::raw(" - Bad Block    "),
            Span::raw("X - Unmovable       │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line4_spans)).style(Style::new().on_blue()), footer_layout[3]);

        // Line 5: Status (Text) + Legend (Drive Info)
        let line5 = "│            Full Optimization           ││ Drive C: 1 block = 1 cluster          │";
        frame.render_widget(Paragraph::new(line5).style(Style::new().on_blue()), footer_layout[4]);

        // Line 6: Bottom Border
        let bottom_border = "└────────────────────────────────────────┘└────────────────────────────────────────┘";
        frame.render_widget(Paragraph::new(bottom_border).style(Style::new().on_blue()), footer_layout[5]);

        // --- Action Line ---
        let action_text = match self.phase {
            DefragPhase::Initializing => "Initializing...",
            DefragPhase::Analyzing => "Analyzing disk...",
            DefragPhase::Defragmenting => "Optimizing...",
            DefragPhase::Finished => "Complete",
        };
        let action_line = Paragraph::new(format!("  {}| Defrag Rust ", action_text))
            .style(Style::new().on_red().white());
        frame.render_widget(action_line, footer_layout[6]);
    }

    fn create_progress_bar(&self, percent: f32) -> String {
        let bar_width = 36;
        let filled_width = ((percent / 100.0) * bar_width as f32) as usize;
        let empty_width = bar_width - filled_width;
        format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width))
    }
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
                    ClusterState::Used => ("█", Style::new().white().on_blue()),
                    ClusterState::Free => ("░", Style::new().gray().on_blue()),
                    ClusterState::Bad => ("B", Style::new().red().on_blue()),
                    ClusterState::Reading => ("R", Style::new().yellow().on_blue()),
                    ClusterState::Writing => ("W", Style::new().green().on_blue()),
                };
                if let Some(cell) = buf.cell_mut((area.x + col, area.y + row)) {
                    cell.set_symbol(symbol)
                        .set_style(style);
                }
            }
        }
    }
}


// -- Utility Functions --------------------------------------------------------

fn parse_size(size_str: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Size must be in format WxH"));
    }
    let width: usize = parts[0].parse().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid width"))?;
    let height: usize = parts[1].parse().map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid height"))?;
    Ok((width, height))
}