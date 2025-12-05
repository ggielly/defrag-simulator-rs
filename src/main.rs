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
    Used,       // Bloc déjà défragmenté (vert)
    Unused,     // Bloc libre
    Pending,    // Bloc à défragmenter (blanc)
    Bad,        // Bloc défectueux
    Unmovable,  // Bloc système non déplaçable
    Reading,    // Bloc en cours de lecture
    Writing,    // Bloc en cours d'écriture
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
    // Menu state
    menu_open: bool,
    selected_menu: usize,
    selected_item: usize,
}

struct DefragStats {
    total_to_defrag: usize,  // Nombre total de clusters à défragmenter
    clusters_defragged: usize, // Nombre de clusters défragmentés
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
        let mut rng = rand::thread_rng();
        
        // Calculer le nombre de clusters à défragmenter (comme dans PHP)
        let num_pending = (total_clusters as f32 * fill_percent) as usize;
        let num_bad = (total_clusters as f32 * 0.02) as usize;
        
        // Créer le disque avec des secteurs Pending (à défragmenter)
        let mut clusters: Vec<ClusterState> = Vec::with_capacity(total_clusters);
        
        // Ajouter les secteurs Pending (fichiers fragmentés à déplacer)
        for _ in 0..(num_pending.saturating_sub(2)) {
            clusters.push(ClusterState::Pending);
        }
        
        // Ajouter les blocs Writing et Reading initiaux
        clusters.push(ClusterState::Writing);
        clusters.push(ClusterState::Reading);
        
        // Compléter avec des secteurs Unused (espace libre)
        while clusters.len() < total_clusters - num_bad {
            clusters.push(ClusterState::Unused);
        }
        
        // Mélanger tout le disque pour simuler la fragmentation
        clusters.shuffle(&mut rng);
        
        // Ajouter les blocs Bad à des positions aléatoires
        let mut bad_positions: Vec<usize> = (0..clusters.len()).collect();
        bad_positions.shuffle(&mut rng);
        for &pos in bad_positions.iter().take(num_bad) {
            clusters.insert(pos.min(clusters.len()), ClusterState::Bad);
        }
        
        // Tronquer si nécessaire
        clusters.truncate(total_clusters);
        
        // Mettre un bloc Unmovable au début (comme le boot sector)
        if !clusters.is_empty() {
            clusters[0] = ClusterState::Unmovable;
        }
        
        let total_to_defrag = clusters.iter().filter(|&&c| c == ClusterState::Pending).count() + 2; // +2 pour Reading/Writing initiaux
        
        Self {
            running: true,
            tick_rate: Duration::from_millis(80), // Légèrement plus rapide
            width,
            height,
            clusters,
            stats: DefragStats {
                total_to_defrag,
                clusters_defragged: 0,
                start_time: Instant::now(),
            },
            phase: DefragPhase::Initializing,
            animation_step: 0,
            read_pos: None,
            write_pos: None,
            // Menu state
            menu_open: false,
            selected_menu: 0,
            selected_item: 0,
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
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                if self.menu_open {
                                    self.menu_open = false;
                                } else {
                                    self.running = false;
                                }
                            }
                            KeyCode::F(10) | KeyCode::Tab => {
                                // Ouvrir/fermer le menu
                                self.menu_open = !self.menu_open;
                                if self.menu_open {
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Left => {
                                if self.menu_open {
                                    self.selected_menu = if self.selected_menu == 0 { 4 } else { self.selected_menu - 1 };
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Right => {
                                if self.menu_open {
                                    self.selected_menu = (self.selected_menu + 1) % 5;
                                    self.selected_item = 0;
                                }
                            }
                            KeyCode::Up => {
                                if self.menu_open {
                                    let max_items = self.get_menu_items(self.selected_menu).len();
                                    self.selected_item = if self.selected_item == 0 { max_items.saturating_sub(1) } else { self.selected_item - 1 };
                                }
                            }
                            KeyCode::Down => {
                                if self.menu_open {
                                    let max_items = self.get_menu_items(self.selected_menu).len();
                                    self.selected_item = (self.selected_item + 1) % max_items;
                                }
                            }
                            KeyCode::Enter => {
                                if self.menu_open {
                                    self.handle_menu_action();
                                    self.menu_open = false;
                                }
                            }
                            _ => {}
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
                // Logique fidèle à l'implémentation PHP
                let mut rng = rand::thread_rng();
                
                // Trouver et effacer le bloc Reading actuel (le mettre en Unused)
                if let Some(reading_idx) = self.clusters.iter().position(|&c| c == ClusterState::Reading) {
                    self.clusters[reading_idx] = ClusterState::Unused;
                }
                
                // Trouver et convertir le bloc Writing en Used (défragmenté)
                if let Some(writing_idx) = self.clusters.iter().position(|&c| c == ClusterState::Writing) {
                    self.clusters[writing_idx] = ClusterState::Used;
                    self.stats.clusters_defragged += 1;
                }
                
                // Chercher un bloc Pending (à défragmenter) - choix ALÉATOIRE comme dans PHP
                let pending_indices: Vec<usize> = self.clusters
                    .iter()
                    .enumerate()
                    .filter(|&(_, c)| *c == ClusterState::Pending)
                    .map(|(i, _)| i)
                    .collect();
                
                if let Some(&pending_idx) = pending_indices.choose(&mut rng) {
                    // Marquer ce bloc comme Reading
                    self.clusters[pending_idx] = ClusterState::Reading;
                    self.read_pos = Some(pending_idx);
                    
                    // Trouver le premier bloc Unused et le marquer comme Writing
                    if let Some(unused_idx) = self.clusters.iter().position(|&c| c == ClusterState::Unused) {
                        self.clusters[unused_idx] = ClusterState::Writing;
                        self.write_pos = Some(unused_idx);
                    }
                } else {
                    // Plus de blocs Pending, défragmentation terminée
                    self.read_pos = None;
                    self.write_pos = None;
                    self.phase = DefragPhase::Finished;
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

        // Rendre le menu déroulant par-dessus tout
        self.render_menu_dropdown(frame, frame.area());
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        // Construire le header avec les menus - style MS-DOS
        let menu_names = self.get_menu_names();
        let mut spans = Vec::new();
        
        spans.push(Span::raw(" "));
        
        for (i, name) in menu_names.iter().enumerate() {
            // Première lettre soulignée (style MS-DOS)
            let first_char = name.chars().next().unwrap_or(' ');
            let rest = &name[first_char.len_utf8()..];
            
            if self.menu_open && self.selected_menu == i {
                // Menu sélectionné - inversé
                spans.push(Span::styled(
                    format!(" {} ", name),
                    Style::new().black().on_cyan()
                ));
            } else {
                // Menu normal avec première lettre en rouge
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    first_char.to_string(),
                    Style::new().red().on_white()
                ));
                spans.push(Span::styled(
                    rest.to_string(),
                    Style::new().black().on_white()
                ));
            }
            spans.push(Span::styled("  ", Style::new().black().on_white()));
        }
        
        // Remplir le reste avec des espaces et ajouter F1=Help
        let current_len: usize = spans.iter().map(|s| s.content.len()).sum();
        let padding = area.width as usize - current_len - 9;
        spans.push(Span::styled(" ".repeat(padding), Style::new().black().on_white()));
        spans.push(Span::styled("F1=Help  ", Style::new().black().on_white()));
        
        let header = Paragraph::new(Line::from(spans));
        frame.render_widget(header, area);
    }

    fn render_menu_dropdown(&self, frame: &mut Frame, area: Rect) {
        if !self.menu_open {
            return;
        }
        
        let items = self.get_menu_items(self.selected_menu);
        if items.is_empty() {
            return;
        }
        
        // Calculer la position X du menu
        let menu_positions = [1, 12, 22, 29, 36];
        let menu_x = menu_positions.get(self.selected_menu).copied().unwrap_or(1) as u16;
        
        // Trouver la largeur maximale des items
        let max_width = items.iter().map(|s| s.len()).max().unwrap_or(10) + 4;
        let menu_height = items.len() as u16 + 2;
        
        // Zone du menu déroulant
        let menu_area = Rect::new(
            area.x + menu_x,
            area.y + 1, // Juste sous le header
            max_width as u16,
            menu_height,
        );
        
        // Dessiner le fond du menu
        let menu_block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::new().bg(Color::White).fg(Color::Black));
        
        frame.render_widget(menu_block.clone(), menu_area);
        
        // Dessiner les items
        let inner = menu_block.inner(menu_area);
        for (i, item) in items.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            
            let item_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
            
            if item.is_empty() {
                // Séparateur
                let sep = Paragraph::new("─".repeat(inner.width as usize))
                    .style(Style::new().fg(Color::DarkGray).bg(Color::White));
                frame.render_widget(sep, item_area);
            } else if i == self.selected_item {
                // Item sélectionné
                let selected = Paragraph::new(format!(" {:<width$}", item, width = inner.width as usize - 1))
                    .style(Style::new().fg(Color::White).bg(Color::Black));
                frame.render_widget(selected, item_area);
            } else {
                // Item normal
                let normal = Paragraph::new(format!(" {}", item))
                    .style(Style::new().fg(Color::Black).bg(Color::White));
                frame.render_widget(normal, item_area);
            }
        }
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

        // Line 2: Status (Cluster, Percent) + Legend (Passed/Used, Pending)
        let percent = if self.stats.total_to_defrag == 0 { 
            100.0 
        } else { 
            (self.stats.clusters_defragged as f32 / self.stats.total_to_defrag as f32) * 100.0 
        };
        let line2_spans = vec![
            Span::raw(format!("│ Cluster {:<6}                    {:>3}% │", self.stats.clusters_defragged, percent.min(100.0) as u8)),
            Span::raw("│ "),
            Span::styled("•", Style::new().fg(Color::Rgb(0, 200, 0))),
            Span::raw(" - Optimized    "),
            Span::styled("•", Style::new().white()),
            Span::raw(" - Fragmented    │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line2_spans)).style(Style::new().on_blue()), footer_layout[1]);

        // Line 3: Status (Progress Bar) + Legend (Reading, Writing)
        let progress_bar = self.create_progress_bar(percent);
        let line3_spans = vec![
            Span::raw(format!("│ {} │", progress_bar)),
            Span::raw("│ "),
            Span::styled("r", Style::new().fg(Color::Yellow).bg(Color::Blue)),
            Span::raw(" - Reading      "),
            Span::styled("W", Style::new().fg(Color::Green).bg(Color::Blue)),
            Span::raw(" - Writing         │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line3_spans)).style(Style::new().on_blue()), footer_layout[2]);

        // Line 4: Status (Elapsed Time) + Legend (Bad, Unmovable)
        let elapsed = self.stats.start_time.elapsed();
        let elapsed_str = format!(
            "Elapsed Time: {:02}:{:02}:{:02}",
            elapsed.as_secs() / 3600,
            (elapsed.as_secs() % 3600) / 60,
            elapsed.as_secs() % 60
        );
        let line4_spans = vec![
            Span::raw(format!("│ {:^38} │", elapsed_str)),
            Span::raw("│ "),
            Span::styled("B", Style::new().fg(Color::Red).bg(Color::Black)),
            Span::raw(" - Bad Block    "),
            Span::styled("X", Style::new().fg(Color::White).bg(Color::Blue)),
            Span::raw(" - Unmovable       │"),
        ];
        frame.render_widget(Paragraph::new(Line::from(line4_spans)).style(Style::new().on_blue()), footer_layout[3]);

        // Line 5: Status (Text) + Legend (Drive Info)
        let line5 = "│            Full Optimization           ││ Drive C: ░ = Unused Space             │";
        frame.render_widget(Paragraph::new(line5).style(Style::new().on_blue()), footer_layout[4]);

        // Line 6: Bottom Border
        let bottom_border = "└────────────────────────────────────────┘└────────────────────────────────────────┘";
        frame.render_widget(Paragraph::new(bottom_border).style(Style::new().on_blue()), footer_layout[5]);

        // --- Action Line ---
        // Messages d'action aléatoires comme dans l'implémentation PHP
        let action_text = match self.phase {
            DefragPhase::Initializing => "Initializing...",
            DefragPhase::Analyzing => "Analyzing disk...",
            DefragPhase::Defragmenting => {
                // Alterner entre les messages comme dans PHP
                match self.animation_step % 3 {
                    0 => "Reading...",
                    1 => "Writing...",
                    _ => "Updating FAT...",
                }
            },
            DefragPhase::Finished => "Complete",
        };
        // Calculer le padding pour justifier à droite comme dans le PHP original
        let version_text = "| MS-DOS Defrag ";
        let total_width = area.width as usize;
        let action_len = action_text.len() + 2; // "  " prefix
        let version_len = version_text.len();
        let padding = total_width.saturating_sub(action_len + version_len);
        
        let action_line = Paragraph::new(format!(
            "  {}{}{}",
            action_text,
            " ".repeat(padding),
            version_text
        )).style(Style::new().on_red().white().bold());
        frame.render_widget(action_line, footer_layout[6]);
    }

    fn create_progress_bar(&self, percent: f32) -> String {
        let bar_width: usize = 38;
        let clamped_percent = percent.min(100.0).max(0.0);
        let filled_width = ((clamped_percent / 100.0) * bar_width as f32) as usize;
        let empty_width = bar_width.saturating_sub(filled_width);
        format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width))
    }

    fn get_menu_items(&self, menu_idx: usize) -> Vec<&'static str> {
        match menu_idx {
            0 => vec!["Begin optimization", "Drive...", "Optimization Method...", "", "Exit"],  // Optimize
            1 => vec!["Analyze drive", "File fragmentation..."],  // Analyze
            2 => vec!["Print disk map", "Save disk map..."],  // File
            3 => vec!["Sort by name", "Sort by extension", "Sort by date", "Sort by size"],  // Sort
            4 => vec!["Contents", "About MS-DOS Defrag..."],  // Help
            _ => vec![],
        }
    }

    fn get_menu_names(&self) -> Vec<&'static str> {
        vec!["Optimize", "Analyze", "File", "Sort", "Help"]
    }

    fn handle_menu_action(&mut self) {
        match (self.selected_menu, self.selected_item) {
            (0, 0) => {
                // Begin optimization - restart defrag
                if self.phase == DefragPhase::Finished {
                    self.phase = DefragPhase::Defragmenting;
                    self.animation_step = 0;
                }
            }
            (0, 4) => {
                // Exit
                self.running = false;
            }
            (1, 0) => {
                // Analyze drive
                if self.phase != DefragPhase::Analyzing {
                    self.phase = DefragPhase::Analyzing;
                    self.animation_step = 0;
                }
            }
            _ => {}
        }
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
                // Couleurs fidèles au defrag MS-DOS original
                let (symbol, style) = match cluster {
                    // Bloc défragmenté/optimisé (vert clair avec point)
                    ClusterState::Used => ("•", Style::new().fg(Color::Rgb(0, 200, 0)).bg(Color::Rgb(0, 100, 0))),
                    // Espace libre (gris sur bleu)
                    ClusterState::Unused => ("░", Style::new().fg(Color::Gray).bg(Color::Blue)),
                    // Bloc fragmenté à défragmenter (blanc/gris clair)
                    ClusterState::Pending => ("•", Style::new().fg(Color::Black).bg(Color::White)),
                    // Bloc défectueux (rouge sur noir)
                    ClusterState::Bad => ("B", Style::new().fg(Color::Red).bg(Color::Black)),
                    // Bloc système non déplaçable
                    ClusterState::Unmovable => ("X", Style::new().fg(Color::White).bg(Color::Blue)),
                    // Bloc en lecture (r minuscule, jaune sur bleu foncé)
                    ClusterState::Reading => ("r", Style::new().fg(Color::Yellow).bg(Color::Rgb(0, 0, 139))),
                    // Bloc en écriture (W majuscule, vert sur bleu foncé)
                    ClusterState::Writing => ("W", Style::new().fg(Color::Green).bg(Color::Rgb(0, 0, 139))),
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