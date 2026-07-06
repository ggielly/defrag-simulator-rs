//! Win98-style widgets for SDL2 rendering
//! Provides reusable UI components matching the Windows 98 look

use super::sdl_backend::{colors, SdlBackend};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

/// A rectangular area with position and size.
#[derive(Debug, Clone, Copy)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Area {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn to_sdl_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    pub fn inner(&self, margin: u32) -> Self {
        Self {
            x: self.x + margin as i32,
            y: self.y + margin as i32,
            width: self.width.saturating_sub(margin * 2),
            height: self.height.saturating_sub(margin * 2),
        }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.width as i32
            && py >= self.y
            && py < self.y + self.height as i32
    }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

pub struct Button {
    pub area: Area,
    pub text: String,
    pub state: ButtonState,
    pub is_default: bool,
}

impl Button {
    pub fn new(x: i32, y: i32, width: u32, height: u32, text: &str) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            text: text.to_string(),
            state: ButtonState::Normal,
            is_default: false,
        }
    }

    pub fn with_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        backend.fill_rect(
            self.area.x,
            self.area.y,
            self.area.width,
            self.area.height,
            colors::BUTTON_FACE,
        );
        match self.state {
            ButtonState::Pressed => draw_sunken_border_on(&self.area, backend),
            ButtonState::Disabled => draw_raised_border_on(&self.area, backend),
            _ => {
                draw_raised_border_on(&self.area, backend);
                if self.is_default {
                    backend.draw_rect(
                        self.area.x - 1,
                        self.area.y - 1,
                        self.area.width + 2,
                        self.area.height + 2,
                        colors::BLACK,
                    );
                }
            }
        }
        let color = if self.state == ButtonState::Disabled {
            colors::BUTTON_SHADOW
        } else {
            colors::TEXT
        };
        let _ = backend.draw_text_centered(
            &self.text,
            self.area.x,
            self.area.y + 4,
            self.area.width,
            13,
            color,
        );
    }

    pub fn update_hover(&mut self, mx: i32, my: i32) {
        if self.area.contains(mx, my) {
            if self.state != ButtonState::Pressed && self.state != ButtonState::Disabled {
                self.state = ButtonState::Hovered;
            }
        } else if self.state == ButtonState::Hovered {
            self.state = ButtonState::Normal;
        }
    }

    pub fn on_mouse_down(&mut self, mx: i32, my: i32) -> bool {
        if self.area.contains(mx, my) && self.state != ButtonState::Disabled {
            self.state = ButtonState::Pressed;
            return true;
        }
        false
    }

    pub fn on_mouse_up(&mut self, mx: i32, my: i32) -> bool {
        if self.state == ButtonState::Pressed {
            self.state = ButtonState::Normal;
            return self.area.contains(mx, my);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Window widget
// ---------------------------------------------------------------------------

pub struct Win98WindowWidget {
    pub area: Area,
    pub title: String,
    pub active: bool,
}

impl Win98WindowWidget {
    pub fn new(x: i32, y: i32, width: u32, height: u32, title: &str) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            title: title.to_string(),
            active: true,
        }
    }

    pub fn title_bar_area(&self) -> Area {
        Area::new(self.area.x + 3, self.area.y + 3, self.area.width - 6, 18)
    }

    pub fn client_area(&self) -> Area {
        Area::new(
            self.area.x + 4,
            self.area.y + 25,
            self.area.width - 8,
            self.area.height - 29,
        )
    }

    pub fn close_button_area(&self) -> Area {
        let ta = self.title_bar_area();
        let sz = 14;
        Area::new(ta.x + ta.width as i32 - sz - 2, ta.y + 2, sz as u32, sz as u32)
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        let a = &self.area;
        backend.fill_rect(a.x, a.y, a.width, a.height, colors::SURFACE);
        draw_window_border_on(a, backend);

        let ta = self.title_bar_area();
        let color = if self.active { colors::DIALOG_BLUE } else { colors::DIALOG_GRAY };
        backend.fill_rect(ta.x, ta.y, ta.width, ta.height, color);
        let _ = backend.draw_text(&self.title, ta.x + 4, ta.y + 2, 14, colors::WHITE);

        // Title bar buttons
        let btn_size: i32 = 14;
        let btn_y = ta.y + 2;
        let mut bx = ta.x + ta.width as i32 - btn_size - 2;
        draw_title_button(backend, bx, btn_y, btn_size as u32, colors::BLACK);
        let _ = backend.draw_text_centered("X", bx, btn_y + 1, btn_size as u32, 9, colors::BLACK);
        bx -= btn_size + 2;
        draw_title_button(backend, bx, btn_y, btn_size as u32, colors::BLACK);
        let _ = backend.draw_text_centered("□", bx, btn_y + 1, btn_size as u32, 9, colors::BLACK);
        bx -= btn_size + 2;
        draw_title_button(backend, bx, btn_y, btn_size as u32, colors::BLACK);
        let _ = backend.draw_text_centered("_", bx, btn_y + 1, btn_size as u32, 9, colors::BLACK);
    }
}

// ---------------------------------------------------------------------------
// Progress bar
// ---------------------------------------------------------------------------

pub struct ProgressBar {
    pub area: Area,
    pub progress: f64,
    pub fill_color: Color,
}

impl ProgressBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            progress: 0.0,
            fill_color: colors::DEFRAG_IDLE,
        }
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.max(0.0).min(1.0);
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        backend.fill_rect(self.area.x, self.area.y, self.area.width, self.area.height, colors::WHITE);
        draw_sunken_border_on(&self.area, backend);
        let inner = self.area.inner(2);
        let fill_w = (inner.width as f64 * self.progress) as u32;
        if fill_w > 0 {
            backend.fill_rect(inner.x, inner.y, fill_w, inner.height, self.fill_color);
        }
    }
}

// ---------------------------------------------------------------------------
// Sunken panel
// ---------------------------------------------------------------------------

pub struct SunkenPanel {
    pub area: Area,
    pub bg_color: Color,
}

impl SunkenPanel {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            area: Area::new(x, y, width, height),
            bg_color: colors::BLACK,
        }
    }

    pub fn inner_area(&self) -> Area {
        self.area.inner(2)
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        backend.fill_rect(self.area.x, self.area.y, self.area.width, self.area.height, self.bg_color);
        draw_sunken_border_on(&self.area, backend);
    }
}

// ---------------------------------------------------------------------------
// Menu bar
// ---------------------------------------------------------------------------

pub struct MenuBar {
    pub menus: Vec<MenuDef>,
    pub open_menu: Option<usize>,
    pub hovered_menu: Option<usize>,
    pub selected_item: Option<usize>,
    pub area: Area,
}

pub struct MenuDef {
    pub label: &'static str,
    pub items: Vec<MenuItem>,
}

pub enum MenuItem {
    Entry(&'static str),
    Separator,
    EntryEnabled(&'static str, bool),
}

impl MenuBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            menus: Vec::new(),
            open_menu: None,
            hovered_menu: None,
            selected_item: None,
            area: Area::new(x, y, width, height),
        }
    }

    pub fn set_menus(&mut self, menus: Vec<MenuDef>) {
        self.menus = menus;
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        backend.fill_rect(self.area.x, self.area.y, self.area.width, self.area.height, colors::SURFACE);
        let mut x = self.area.x + 2;
        for (i, menu) in self.menus.iter().enumerate() {
            let label_w = measure_menu_label(backend, menu.label) as u32 + 8;
            let is_open = self.open_menu == Some(i);
            let is_hovered = self.hovered_menu == Some(i) && !is_open;
            if is_open || is_hovered {
                backend.fill_rect(x, self.area.y, label_w, self.area.height, colors::BLACK);
                let _ = backend.draw_text(menu.label, x + 4, self.area.y + 2, 12, colors::WHITE);
            } else {
                let _ = backend.draw_text(menu.label, x + 4, self.area.y + 2, 12, colors::TEXT);
            }
            x += label_w as i32 + 2;
        }
    }

    pub fn draw_open_menu(&self, backend: &mut SdlBackend) -> Option<(usize, usize)> {
        let idx = self.open_menu?;
        let menu = &self.menus[idx];
        let mut x = self.area.x + 2;
        for i in 0..idx {
            x += measure_menu_label(backend, self.menus[i].label) as i32 + 10;
        }
        let mw = self.max_menu_width(backend, idx) as u32 + 16;
        let mh = self.menu_height(backend, idx) as u32;
        let y = self.area.y + self.area.height;

        backend.fill_rect(x, y, mw, mh, colors::SURFACE);
        draw_raised_border_on(&Area::new(x, y, mw, mh), backend);

        let mut iy = y + 2;
        for (ii, item) in menu.items.iter().enumerate() {
            match item {
                MenuItem::Separator => {
                    let inner_x = x + 2;
                    let sep_w = mw - 4;
                    backend.fill_rect(inner_x, iy + 1, sep_w, 1, colors::BUTTON_SHADOW);
                    iy += 5;
                }
                MenuItem::Entry(label) | MenuItem::EntryEnabled(label, _) => {
                    let selected = self.selected_item == Some(ii);
                    let inner_x = x + 2;
                    let iw = mw - 4;
                    if selected {
                        backend.fill_rect(inner_x, iy, iw, 16, colors::BLACK);
                        let _ = backend.draw_text(label, inner_x + 4, iy + 2, 12, colors::WHITE);
                    } else {
                        let _ = backend.draw_text(label, inner_x + 4, iy + 2, 12, colors::TEXT);
                    }
                    iy += 16;
                }
            }
        }
        Some((idx, menu.items.len()))
    }

    pub fn menu_item_at(&self, backend: &mut SdlBackend, mx: i32, my: i32) -> Option<(usize, usize)> {
        let menu_idx = self.open_menu?;
        let menu = &self.menus[menu_idx];
        let mut x = self.area.x + 2;
        for i in 0..menu_idx {
            x += measure_menu_label(backend, self.menus[i].label) as i32 + 10;
        }
        let mw = self.max_menu_width(backend, menu_idx) as u32 + 16;
        let y = self.area.y + self.area.height;
        let mh = self.menu_height(backend, menu_idx) as u32;
        if mx < x || mx > x + mw as i32 || my < y || my > y + mh as i32 {
            return None;
        }
        let mut iy = y + 2;
        for (ii, item) in menu.items.iter().enumerate() {
            match item {
                MenuItem::Separator => iy += 5,
                MenuItem::Entry(_) | MenuItem::EntryEnabled(_, _) => {
                    if my >= iy && my < iy + 16 {
                        return Some((menu_idx, ii));
                    }
                    iy += 16;
                }
            }
        }
        Some((menu_idx, menu.items.len()))
    }

    pub fn menu_label_at(&self, backend: &mut SdlBackend, mx: i32, my: i32) -> Option<usize> {
        if !self.area.contains(mx, my) && self.open_menu.is_none() {
            return None;
        }
        let mut x = self.area.x + 2;
        for (i, menu) in self.menus.iter().enumerate() {
            let w = measure_menu_label(backend, menu.label) as i32 + 10;
            if mx >= x && mx < x + w && my >= self.area.y && my < self.area.y + self.area.height {
                return Some(i);
            }
            x += w;
        }
        None
    }

    fn max_menu_width(&self, backend: &mut SdlBackend, idx: usize) -> usize {
        self.menus[idx]
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Separator => 0,
                MenuItem::Entry(l) | MenuItem::EntryEnabled(l, _) => measure_menu_label(backend, l),
            })
            .max()
            .unwrap_or(60)
    }

    fn menu_height(&self, backend: &mut SdlBackend, idx: usize) -> usize {
        let mut h = 4;
        for item in &self.menus[idx].items {
            match item {
                MenuItem::Separator => h += 5,
                _ => h += 16,
            }
        }
        h
    }
}

// ---------------------------------------------------------------------------
// Status bar (bottom of window)
// ---------------------------------------------------------------------------

pub struct StatusBar {
    pub area: Area,
}

impl StatusBar {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { area: Area::new(x, y, width, height) }
    }

    pub fn draw(&self, backend: &mut SdlBackend, left: &str, center: &str, right: &str) {
        draw_sunken_border_on(&self.area, backend);
        let inner = self.area.inner(2);
        let third = inner.width / 3;

        let _ = backend.draw_text(left, inner.x + 2, inner.y + 2, 11, colors::TEXT);
        let _ = backend.draw_text_centered(center, inner.x + third, inner.y + 2, third, 11, colors::TEXT);
        let _ = backend.draw_text(right, inner.x + third * 2 + 2, inner.y + 2, third, 11, colors::TEXT);
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

pub struct Checkbox {
    pub area: Area,
    pub label: &'static str,
    pub checked: bool,
}

impl Checkbox {
    pub fn new(x: i32, y: i32, label: &'static str, checked: bool) -> Self {
        Self {
            area: Area::new(x, y, 16, 16),
            label,
            checked,
        }
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        draw_sunken_border_on(&self.area, backend);
        if self.checked {
            let inner = self.area.inner(2);
            backend.fill_rect(inner.x, inner.y, inner.width, inner.height, colors::BLACK);
        }
        let _ = backend.draw_text(self.label, self.area.x + 20, self.area.y + 1, 12, colors::TEXT);
    }

    pub fn contains(&self, mx: i32, my: i32) -> bool {
        self.area.contains(mx, my)
            || (mx >= self.area.x
                && mx < self.area.x + self.area.width as i32 + measure_label_width(self.label) as i32
                && my >= self.area.y
                && my < self.area.y + self.area.height as i32)
    }
}

// ---------------------------------------------------------------------------
// Simple label
// ---------------------------------------------------------------------------

pub struct Label {
    pub area: Area,
    pub text: String,
    pub size: u16,
    pub color: Color,
}

impl Label {
    pub fn new(x: i32, y: i32, text: &str) -> Self {
        Self {
            area: Area::new(x, y, 0, 16),
            text: text.to_string(),
            size: 12,
            color: colors::TEXT,
        }
    }

    pub fn draw(&self, backend: &mut SdlBackend) {
        let _ = backend.draw_text(&self.text, self.area.x, self.area.y, self.size, self.color);
    }
}

// ---------------------------------------------------------------------------
// Dialog overlay (used for settings / about)
// ---------------------------------------------------------------------------

pub struct Dialog {
    pub area: Area,
    pub title: String,
    pub active: bool,
}

impl Dialog {
    pub fn new(center_x: i32, center_y: i32, width: u32, height: u32, title: &str) -> Self {
        Self {
            area: Area::new(center_x - width as i32 / 2, center_y - height as i32 / 2, width, height),
            title: title.to_string(),
            active: true,
        }
    }

    pub fn draw(&self, backend: &mut SdlBackend, content_fn: impl FnOnce(&mut SdlBackend, Area)) {
        // Dimmed overlay
        let screen = backend.get_size();
        backend.fill_rect(0, 0, screen.0, screen.1, Color::RGBA(0, 0, 0, 80));

        // Dialog box
        backend.fill_rect(self.area.x, self.area.y, self.area.width, self.area.height, colors::SURFACE);
        draw_raised_border_on(&self.area, backend);

        // Title bar
        let ta = Area::new(self.area.x + 3, self.area.y + 3, self.area.width - 6, 18);
        backend.fill_rect(ta.x, ta.y, ta.width, ta.height, colors::DIALOG_BLUE);
        let _ = backend.draw_text(&self.title, ta.x + 4, ta.y + 2, 14, colors::WHITE);

        // Content
        let client = Area::new(self.area.x + 8, self.area.y + 25, self.area.width - 16, self.area.height - 33);
        content_fn(backend, client);
    }

    pub fn ok_button_area(&self) -> Area {
        let bw = 80;
        let bh = 23;
        let bx = self.area.x + (self.area.width - bw) / 2;
        let by = self.area.y + self.area.height - bh - 8;
        Area::new(bx as i32, by as i32, bw, bh)
    }
}

// ---------------------------------------------------------------------------
// Border drawing helpers (shared by all widgets)
// ---------------------------------------------------------------------------

fn draw_raised_border_on(area: &Area, backend: &mut SdlBackend) {
    let x = area.x;
    let y = area.y;
    let w = area.width as i32;
    let h = area.height as i32;
    backend.draw_hline(x, x + w - 1, y, colors::BUTTON_HIGHLIGHT);
    backend.draw_vline(x, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
    backend.draw_hline(x + 1, x + w - 2, y + 1, colors::BUTTON_FACE);
    backend.draw_vline(x + 1, y + 1, y + h - 2, colors::BUTTON_FACE);
    backend.draw_hline(x, x + w - 1, y + h - 1, colors::WINDOW_FRAME);
    backend.draw_vline(x + w - 1, y, y + h - 1, colors::WINDOW_FRAME);
    backend.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_SHADOW);
    backend.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_SHADOW);
}

fn draw_sunken_border_on(area: &Area, backend: &mut SdlBackend) {
    let x = area.x;
    let y = area.y;
    let w = area.width as i32;
    let h = area.height as i32;
    backend.draw_hline(x, x + w - 1, y, colors::BUTTON_SHADOW);
    backend.draw_vline(x, y, y + h - 1, colors::BUTTON_SHADOW);
    backend.draw_hline(x + 1, x + w - 2, y + 1, colors::WINDOW_FRAME);
    backend.draw_vline(x + 1, y + 1, y + h - 2, colors::WINDOW_FRAME);
    backend.draw_hline(x, x + w - 1, y + h - 1, colors::BUTTON_HIGHLIGHT);
    backend.draw_vline(x + w - 1, y, y + h - 1, colors::BUTTON_HIGHLIGHT);
    backend.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_FACE);
    backend.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_FACE);
}

fn draw_window_border_on(area: &Area, backend: &mut SdlBackend) {
    let x = area.x;
    let y = area.y;
    let w = area.width as i32;
    let h = area.height as i32;
    backend.draw_hline(x, x + w - 1, y, colors::BUTTON_FACE);
    backend.draw_vline(x, y, y + h - 1, colors::BUTTON_FACE);
    backend.draw_hline(x, x + w - 1, y + h - 1, colors::WINDOW_FRAME);
    backend.draw_vline(x + w - 1, y, y + h - 1, colors::WINDOW_FRAME);
    backend.draw_hline(x + 1, x + w - 2, y + 1, colors::BUTTON_HIGHLIGHT);
    backend.draw_vline(x + 1, y + 1, y + h - 2, colors::BUTTON_HIGHLIGHT);
    backend.draw_hline(x + 1, x + w - 2, y + h - 2, colors::BUTTON_SHADOW);
    backend.draw_vline(x + w - 2, y + 1, y + h - 2, colors::BUTTON_SHADOW);
}

fn draw_title_button(backend: &mut SdlBackend, x: i32, y: i32, size: u32, _color: Color) {
    let area = Area::new(x, y, size, size);
    backend.fill_rect(x, y, size, size, colors::BUTTON_FACE);
    draw_raised_border_on(&area, backend);
}

fn measure_menu_label(backend: &mut SdlBackend, label: &str) -> usize {
    backend.get_text_width(label, 12).unwrap_or(label.len() as u32 * 7) as usize
}

fn measure_label_width(label: &str) -> u32 {
    label.len() as u32 * 7
}
