//! Windows 98 Disk Defragmenter Graphical Renderer
//! Full implementation with menu bar, status bar, dialogs, and proper pause/resume.

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

use super::sdl_backend::{colors, SdlBackend, SdlConfig, SdlEvent};
use super::win98_widgets::*;
use super::ResourceCache;
use crate::app::App;
use crate::helpers;
use crate::models::{ClusterState, DefragPhase};

const CLUSTER_SIZE: u32 = 8;
const CLUSTER_GAP: u32 = 1;

// ---------------------------------------------------------------------------
// Cluster color mapping
// ---------------------------------------------------------------------------

fn cluster_color(state: &ClusterState) -> Color {
    match state {
        ClusterState::Used => colors::DEFRAG_DONE,
        ClusterState::Pending => colors::DEFRAG_IDLE,
        ClusterState::Reading | ClusterState::Writing => colors::DEFRAG_PROGRESS,
        ClusterState::Unused => Color::RGB(0, 0, 80),
        ClusterState::Bad => Color::RGB(200, 100, 0),
        ClusterState::Unmovable => Color::RGB(200, 200, 200),
    }
}

// ---------------------------------------------------------------------------
// Settings dialog state
// ---------------------------------------------------------------------------

struct SettingsState {
    drive_index: usize,
    speed_index: usize,
    fill_value: u32,
    sound_enabled: bool,
}

impl SettingsState {
    fn from_app(app: &App) -> Self {
        let drive_index = app
            .drive_collection
            .get_all()
            .iter()
            .position(|d| d.letter() == app.current_drive.letter())
            .unwrap_or(0);
        Self {
            drive_index,
            speed_index: 1,
            fill_value: (app.progress_percent() + 0.5) as u32,
            sound_enabled: app.audio.as_ref().map_or(false, |a| a.is_enabled()),
        }
    }
}

fn progress_percent(app: &App) -> f32 {
    helpers::progress_percent(app)
}

// ---------------------------------------------------------------------------
// Main renderer
// ---------------------------------------------------------------------------

pub struct Win98GraphicalRenderer {
    backend: SdlBackend,
    _resource_cache: ResourceCache,

    window: Win98WindowWidget,
    menu_bar: MenuBar,
    status_bar: StatusBar,
    progress_bar: ProgressBar,
    disk_panel: SunkenPanel,

    start_pause_button: Button,
    stop_button: Button,
    settings_button: Button,

    // Dialogs
    about_dialog: Option<Dialog>,
    settings_dialog: Option<Dialog>,
    settings_state: Option<SettingsState>,

    // Mouse state
    mouse_x: i32,
    mouse_y: i32,

    // Timing
    tick_rate: Duration,
    last_tick: Instant,
}

impl Win98GraphicalRenderer {
    pub fn new() -> Result<Self, String> {
        let config = SdlConfig {
            width: 640,
            height: 480,
            title: "Disk Defragmenter".to_string(),
            scale: 1,
        };
        let backend = SdlBackend::new(config)?;

        let ww = 560u32;
        let wh = 420u32;
        let wx = ((640 - ww) / 2) as i32;
        let wy = ((480 - wh) / 2) as i32;

        let window = Win98WindowWidget::new(wx, wy, ww, wh, "Disk Defragmenter");

        let client = window.client_area();

        let menu_bar = {
            let mut mb = MenuBar::new(client.x, client.y - 1, client.width, 20);
            mb.set_menus(vec![
                MenuDef {
                    label: "File",
                    items: vec![
                        MenuItem::Entry("Restart"),
                        MenuItem::Separator,
                        MenuItem::Entry("Exit"),
                    ],
                },
                MenuDef {
                    label: "Analyze",
                    items: vec![MenuItem::Entry("Analyze drive")],
                },
                MenuDef {
                    label: "View",
                    items: vec![
                        MenuItem::Entry("Show Legend"),
                        MenuItem::Entry("Show Status Bar"),
                    ],
                },
                MenuDef {
                    label: "Help",
                    items: vec![MenuItem::Entry("About Disk Defragmenter...")],
                },
            ]);
            mb
        };

        let disk_panel = SunkenPanel::new(
            client.x + 8,
            client.y + 24,
            client.width - 16,
            client.height - 140,
        );

        let legend_y = disk_panel.area.y + disk_panel.area.height as i32 + 10;
        let progress_bar = ProgressBar::new(client.x + 8, legend_y + 16, client.width - 16, 16);

        let button_y = progress_bar.area.y + progress_bar.area.height as i32 + 28;

        let settings_button = Button::new(client.x + 8, button_y, 85, 23, "Settings...");
        let start_pause_button = Button::new(
            client.x + client.width as i32 - 176,
            button_y,
            75,
            23,
            "Start",
        );
        let stop_button = Button::new(
            client.x + client.width as i32 - 90,
            button_y,
            75,
            23,
            "Stop",
        );

        let status_bar = StatusBar::new(
            client.x,
            client.y + client.height as i32 - 22,
            client.width,
            22,
        );

        let mut resource_cache = ResourceCache::new();
        let _ = resource_cache.load_image_from_file("cluster_sprites", "static/imgs/cluster_sprites.png");
        let _ = resource_cache.load_image_from_file("border_left", "static/imgs/border_left.png");
        let _ = resource_cache.load_image_from_file("right_scroll_bar", "static/imgs/right_scroll_bar.png");
        let _ = resource_cache.load_image_from_file("title_left", "static/imgs/title_left.png");
        let _ = resource_cache.load_image_from_file("title_right", "static/imgs/title_right.png");
        let _ = resource_cache.load_image_from_file("title_bg", "static/imgs/title_bg.png");
        let _ = resource_cache.load_image_from_file("top_right_scroll", "static/imgs/top_right_scroll.png");

        Ok(Self {
            backend,
            _resource_cache: resource_cache,
            window,
            menu_bar,
            status_bar,
            progress_bar,
            disk_panel,
            start_pause_button,
            stop_button,
            settings_button,
            about_dialog: None,
            settings_dialog: None,
            settings_state: None,
            mouse_x: 0,
            mouse_y: 0,
            tick_rate: Duration::from_millis(80),
            last_tick: Instant::now(),
        })
    }

    // -- Main loop ------------------------------------------------------------

    pub fn run(&mut self, app: &mut App) -> Result<(), String> {
        let target_fps = 60;
        let frame_duration = Duration::from_micros(1_000_000 / target_fps);

        while self.backend.is_running() && app.running {
            let frame_start = Instant::now();

            self.handle_events(app);

            // Only update game logic at tick rate, and only when not paused
            if self.last_tick.elapsed() >= self.tick_rate && !app.paused {
                app.update();
                self.last_tick = Instant::now();
            }

            self.update_ui_state(app);
            self.render(app);

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        Ok(())
    }

    // -- Event handling -------------------------------------------------------

    fn handle_events(&mut self, app: &mut App) {
        let events = self.backend.poll_events();
        for event in events {
            match event {
                SdlEvent::Quit => app.running = false,
                SdlEvent::KeyDown(keycode) => self.handle_keydown(app, keycode),
                SdlEvent::MouseMove { x, y } => {
                    self.mouse_x = x;
                    self.mouse_y = y;
                    self.update_hover();
                }
                SdlEvent::MouseDown { x, y, .. } => self.handle_mouse_down(app, x, y),
                SdlEvent::MouseUp { x, y, .. } => self.handle_mouse_up(app, x, y),
                _ => {}
            }
        }
    }

    fn handle_keydown(&mut self, app: &mut App, keycode: Keycode) {
        // If a dialog is open, Escape closes it
        if self.about_dialog.is_some() {
            if keycode == Keycode::Escape || keycode == Keycode::Return {
                self.about_dialog = None;
            }
            return;
        }
        if self.settings_dialog.is_some() {
            if keycode == Keycode::Escape {
                self.settings_dialog = None;
                self.settings_state = None;
            }
            return;
        }

        match keycode {
            Keycode::Escape | Keycode::Q => app.running = false,
            Keycode::Space => self.toggle_defrag(app),
            Keycode::Return => {
                if self.menu_bar.open_menu.is_some() {
                    self.execute_menu_action(app);
                    self.menu_bar.open_menu = None;
                    self.menu_bar.selected_item = None;
                } else {
                    self.toggle_defrag(app);
                }
            }
            Keycode::R => {
                if !self.menu_bar.open_menu.is_some() {
                    app.restart();
                }
            }
            Keycode::D => {
                if !self.menu_bar.open_menu.is_some() {
                    app.toggle_demo_mode();
                }
            }
            Keycode::S => {
                if let Some(ref mut audio) = app.audio {
                    audio.toggle();
                }
            }
            Keycode::F1 => {
                self.about_dialog = Some(Dialog::new(320, 240, 400, 260, "About Disk Defragmenter"));
            }
            Keycode::Left => {
                if self.menu_bar.open_menu.is_some() {
                    self.menu_bar.open_menu = self
                        .menu_bar
                        .open_menu
                        .map(|i| if i == 0 { self.menu_bar.menus.len() - 1 } else { i - 1 });
                    self.menu_bar.selected_item = Some(0);
                }
            }
            Keycode::Right => {
                if self.menu_bar.open_menu.is_some() {
                    self.menu_bar.open_menu = self
                        .menu_bar
                        .open_menu
                        .map(|i| (i + 1) % self.menu_bar.menus.len());
                    self.menu_bar.selected_item = Some(0);
                }
            }
            Keycode::Up => {
                if let Some(ref mut si) = self.menu_bar.selected_item {
                    *si = si.saturating_sub(1);
                }
            }
            Keycode::Down => {
                if let Some(idx) = self.menu_bar.open_menu {
                    let n = self.menu_bar.menus[idx].items.len();
                    if n > 0 {
                        self.menu_bar.selected_item =
                            Some(self.menu_bar.selected_item.map_or(0, |s| (s + 1) % n));
                    }
                }
            }
            _ => {}
        }
    }

    fn update_hover(&mut self) {
        self.start_pause_button.update_hover(self.mouse_x, self.mouse_y);
        self.stop_button.update_hover(self.mouse_x, self.mouse_y);
        self.settings_button.update_hover(self.mouse_x, self.mouse_y);

        // Menu bar hover
        if let Some(idx) = self.menu_bar.menu_label_at(&mut self.backend, self.mouse_x, self.mouse_y) {
            self.menu_bar.hovered_menu = Some(idx);
            if self.menu_bar.open_menu.is_some() {
                self.menu_bar.open_menu = Some(idx);
                self.menu_bar.selected_item = Some(0);
            }
        } else {
            self.menu_bar.hovered_menu = None;
        }

        // Menu item hover
        if self.menu_bar.open_menu.is_some() {
            if let Some((_, item_idx)) =
                self.menu_bar.menu_item_at(&mut self.backend, self.mouse_x, self.mouse_y)
            {
                self.menu_bar.selected_item = Some(item_idx);
            }
        }
    }

    fn handle_mouse_down(&mut self, app: &mut App, x: i32, y: i32) {
        if let Some(ref audio) = app.audio {
            audio.play_mouse_down();
        }

        // Dialog ok button
        if let Some(ref dlg) = self.about_dialog {
            if dlg.ok_button_area().contains(x, y) {
                self.about_dialog = None;
                return;
            }
        }
        if let Some(ref dlg) = self.settings_dialog {
            if dlg.ok_button_area().contains(x, y) {
                self.apply_settings(app);
                self.settings_dialog = None;
                self.settings_state = None;
                return;
            }
        }

        // Menu bar click
        if let Some(idx) = self.menu_bar.menu_label_at(&mut self.backend, x, y) {
            if self.menu_bar.open_menu == Some(idx) {
                self.menu_bar.open_menu = None;
                self.menu_bar.selected_item = None;
            } else {
                self.menu_bar.open_menu = Some(idx);
                self.menu_bar.selected_item = Some(0);
            }
            return;
        }

        // Menu item click
        if self.menu_bar.open_menu.is_some() {
            if let Some(_) = self.menu_bar.menu_item_at(&mut self.backend, x, y) {
                self.execute_menu_action(app);
                self.menu_bar.open_menu = None;
                self.menu_bar.selected_item = None;
                return;
            }
            // Click outside menu closes it
            self.menu_bar.open_menu = None;
            self.menu_bar.selected_item = None;
        }

        // Button clicks
        if self.start_pause_button.on_mouse_down(x, y) {
            return;
        }
        if self.stop_button.on_mouse_down(x, y) {
            return;
        }
        if self.settings_button.on_mouse_down(x, y) {
            return;
        }
    }

    fn handle_mouse_up(&mut self, app: &mut App, x: i32, y: i32) {
        if let Some(ref audio) = app.audio {
            audio.play_mouse_up();
        }

        if self.start_pause_button.on_mouse_up(x, y) {
            self.toggle_defrag(app);
        }
        if self.stop_button.on_mouse_up(x, y) {
            app.phase = DefragPhase::Finished;
        }
        if self.settings_button.on_mouse_up(x, y) {
            self.settings_dialog = Some(Dialog::new(320, 240, 340, 250, "Settings"));
            self.settings_state = Some(SettingsState::from_app(app));
        }
    }

    // -- Actions --------------------------------------------------------------

    fn toggle_defrag(&mut self, app: &mut App) {
        match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => {
                app.phase = DefragPhase::Analyzing;
                app.animation_step = 0;
            }
            DefragPhase::Analyzing | DefragPhase::Defragmenting => {
                app.toggle_pause();
            }
        }
    }

    fn execute_menu_action(&mut self, app: &mut App) {
        if let (Some(menu_idx), Some(item_idx)) = (self.menu_bar.open_menu, self.menu_bar.selected_item) {
            match (menu_idx, item_idx) {
                (0, 0) => app.restart(),     // File > Restart
                (0, 2) => app.running = false, // File > Exit
                (1, 0) => {
                    // Analyze > Analyze drive
                    if app.phase != DefragPhase::Analyzing {
                        app.phase = DefragPhase::Analyzing;
                        app.animation_step = 0;
                    }
                }
                (3, 0) => {
                    self.about_dialog =
                        Some(Dialog::new(320, 240, 400, 260, "About Disk Defragmenter"));
                }
                _ => {}
            }
        }
    }

    fn apply_settings(&mut self, app: &mut App) {
        if let Some(ref s) = self.settings_state {
            // Drive selection
            if let Some(drive) = app.drive_collection.get_by_index(s.drive_index) {
                app.current_drive = drive.clone();
                if let Some(ref mut audio) = app.audio {
                    audio.set_iops(drive.iops());
                }
            }
            // Sound toggle
            if s.sound_enabled && app.audio.is_none() {
                app.audio = crate::audio::AudioEngine::new();
            } else if !s.sound_enabled {
                app.audio = None;
            }
        }
    }

    // -- UI state sync --------------------------------------------------------

    fn update_ui_state(&mut self, app: &App) {
        // Window title
        self.window.title = match app.phase {
            DefragPhase::Defragmenting | DefragPhase::Analyzing => {
                format!(
                    "Defragmenting Drive {}{}",
                    app.current_drive.letter(),
                    if app.paused { " (Paused)" } else { "" }
                )
            }
            _ => "Disk Defragmenter".to_string(),
        };

        // Start/Pause button
        self.start_pause_button.text = match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => "Start".to_string(),
            DefragPhase::Analyzing | DefragPhase::Defragmenting => {
                if app.paused {
                    "Resume".to_string()
                } else {
                    "Pause".to_string()
                }
            }
        };

        // Stop button state
        self.stop_button.state = match app.phase {
            DefragPhase::Initializing | DefragPhase::Finished => ButtonState::Disabled,
            _ => {
                if self.stop_button.area.contains(self.mouse_x, self.mouse_y) {
                    ButtonState::Hovered
                } else {
                    ButtonState::Normal
                }
            }
        };

        // Progress bar
        self.progress_bar.set_progress(helpers::progress_ratio(app));
    }

    // -- Rendering ------------------------------------------------------------

    fn render(&mut self, app: &App) {
        self.backend.clear();

        // Menu bar (drawn on top of window frame)
        self.menu_bar.draw(&mut self.backend);

        // Window
        self.window.draw(&mut self.backend);

        // Client area background
        let client = self.window.client_area();
        self.backend
            .fill_rect(client.x, client.y, client.width, client.height, colors::SURFACE);

        // Disk panel
        self.disk_panel.draw(&mut self.backend);
        self.draw_disk_grid(app);

        // Legend
        self.draw_legend();

        // Progress bar
        self.progress_bar.draw(&mut self.backend);

        // Progress text
        self.draw_progress_text(app);

        // Buttons
        self.settings_button.draw(&mut self.backend);
        self.start_pause_button.draw(&mut self.backend);
        self.stop_button.draw(&mut self.backend);

        // Status bar
        let phase_str = helpers::phase_status(app);
        let elapsed = helpers::elapsed_str(app);
        let eta = helpers::eta_str(app).map_or(String::new(), |e| format!("ETA {}", e));
        self.status_bar
            .draw(&mut self.backend, phase_str, &elapsed, &eta);

        // Open menu dropdown
        self.menu_bar.draw_open_menu(&mut self.backend);

        // Dialogs
        if let Some(ref dlg) = self.about_dialog {
            let ok_area = dlg.ok_button_area();
            dlg.draw(&mut self.backend, |backend, client| {
                let _ = backend.draw_text("MS-DOS Disk Defragmenter", client.x + 8, client.y + 4, 14, colors::TEXT);
                let _ = backend.draw_text("Simulator v0.1.0", client.x + 8, client.y + 24, 12, colors::TEXT);
                let _ = backend.draw_text("Author: Guillaume 'GuY' Gielly", client.x + 8, client.y + 48, 12, colors::TEXT);
                let _ = backend.draw_text("License: GPL-v3", client.x + 8, client.y + 68, 12, colors::TEXT);
                let _ = backend.draw_text("github.com/ggielly/defrag-rs", client.x + 8, client.y + 88, 12, Color::RGB(0, 0, 200));
                let _ = backend.draw_text("A relaxing defrag simulation.", client.x + 8, client.y + 118, 12, colors::TEXT);
                let _ = backend.draw_text("Press Space to pause/resume.", client.x + 8, client.y + 138, 12, colors::TEXT);
                let _ = backend.draw_text("Press R to restart.", client.x + 8, client.y + 158, 12, colors::TEXT);
                let _ = backend.draw_text("Press S to toggle sound.", client.x + 8, client.y + 178, 12, colors::TEXT);
                let ok_btn = Button::new(ok_area.x, ok_area.y, ok_area.width, ok_area.height, "OK");
                ok_btn.draw(backend);
            });
        }

        if let Some(ref dlg) = self.settings_dialog {
            let ok_area = dlg.ok_button_area();
            let settings_clone = self.settings_state.clone();
            dlg.draw(&mut self.backend, |backend, client| {
                if let Some(ref settings) = settings_clone {
                    let _ = backend.draw_text("Drive:", client.x + 8, client.y + 8, 12, colors::TEXT);
                    let drives = ["C: HDD 2GB", "D: HDD 1GB", "E: Floppy", "F: SSHD 2GB"];
                    for (i, label) in drives.iter().enumerate() {
                        let ry = client.y + 28 + i as i32 * 18;
                        let selected = settings.drive_index == i;
                        let color = if selected { colors::WHITE } else { colors::TEXT };
                        let bg = if selected { colors::DIALOG_BLUE } else { colors::SURFACE };
                        backend.fill_rect(client.x + 8, ry, client.width - 16, 16, bg);
                        let _ = backend.draw_text(&format!("  {}", label), client.x + 8, ry + 1, 12, color);
                    }
                    let _ = backend.draw_text("Sound:", client.x + 8, client.y + 112, 12, colors::TEXT);
                    let chk = Checkbox::new(client.x + 60, client.y + 108, "Enabled", settings.sound_enabled);
                    chk.draw(backend);
                }
                let ok_btn = Button::new(ok_area.x, ok_area.y, ok_area.width, ok_area.height, "OK");
                ok_btn.draw(backend);
            });
        }

        self.backend.present();
    }

    fn draw_disk_grid(&mut self, app: &App) {
        let inner = self.disk_panel.inner_area();
        let cols = (inner.width / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;
        let rows = (inner.height / (CLUSTER_SIZE + CLUSTER_GAP)) as usize;

        for (i, cluster) in app.clusters.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            if row >= rows {
                break;
            }
            let x = inner.x + (col as u32 * (CLUSTER_SIZE + CLUSTER_GAP)) as i32;
            let y = inner.y + (row as u32 * (CLUSTER_SIZE + CLUSTER_GAP)) as i32;
            self.backend
                .fill_rect(x, y, CLUSTER_SIZE, CLUSTER_SIZE, cluster_color(cluster));
        }
    }

    fn draw_legend(&mut self) {
        let ly = self.disk_panel.area.y + self.disk_panel.area.height as i32 + 6;
        let client = self.window.client_area();
        let iw = (client.width / 3) as i32;

        let items: [(Color, &str); 3] = [
            (colors::DEFRAG_IDLE, "Not defragmented"),
            (colors::DEFRAG_PROGRESS, "In progress"),
            (colors::DEFRAG_DONE, "Defragmented"),
        ];
        for (i, (color, label)) in items.iter().enumerate() {
            let x = client.x + 16 + iw * i as i32;
            self.backend.fill_rect(x, ly, 12, 12, *color);
            let _ = self.backend.draw_text(label, x + 16, ly, 11, colors::TEXT);
        }
    }

    fn draw_progress_text(&mut self, app: &App) {
        let pct = progress_percent(app) as u32;
        let y = self.progress_bar.area.y - 16;
        let client = self.window.client_area();

        let status = if let Some(filename) = &app.current_filename {
            format!("Defragmenting: {}", helpers::truncate_str(filename, 50))
        } else {
            match app.phase {
                DefragPhase::Analyzing => "Analyzing drive...".to_string(),
                DefragPhase::Finished => "Defragmentation complete.".to_string(),
                _ => String::new(),
            }
        };
        let _ = self.backend.draw_text(&status, client.x + 8, y, 13, colors::TEXT);

        let pct_text = format!("{}%", pct);
        if let Ok(tw) = self.backend.get_text_width(&pct_text, 13) {
            let x = self.progress_bar.area.x + self.progress_bar.area.width as i32 - tw as i32;
            let _ = self.backend.draw_text(&pct_text, x, y, 13, colors::TEXT);
        }
    }
}

// -- Public entry point ------------------------------------------------------

pub fn run_win98_graphical(app: &mut App) -> Result<(), String> {
    let mut renderer = Win98GraphicalRenderer::new()?;
    renderer.run(app)
}
