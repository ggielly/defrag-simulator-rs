//! Shared helper functions used by both terminal and graphical UIs.

use crate::app::App;
use crate::models::DefragPhase;

/// Progress percentage (0.0–100.0).
pub fn progress_percent(app: &App) -> f32 {
    if app.stats.total_to_defrag == 0 {
        return 100.0;
    }
    (app.stats.clusters_defragged as f32 / app.stats.total_to_defrag as f32) * 100.0
}

/// Progress ratio (0.0–1.0).
pub fn progress_ratio(app: &App) -> f64 {
    if app.stats.total_to_defrag == 0 {
        return if app.phase == DefragPhase::Finished {
            1.0
        } else {
            0.0
        };
    }
    app.stats.clusters_defragged as f64 / app.stats.total_to_defrag as f64
}

/// Elapsed time formatted as HH:MM:SS.
pub fn elapsed_str(app: &App) -> String {
    let secs = app.stats.start_time.elapsed().as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Estimated remaining time formatted as HH:MM:SS, or None.
pub fn eta_str(app: &App) -> Option<String> {
    let remaining = app.estimated_time_remaining()?;
    let secs = remaining.as_secs();
    Some(format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    ))
}

/// Status message describing the current phase.
pub fn phase_status(app: &App) -> &'static str {
    match app.phase {
        DefragPhase::Initializing => "Initializing...",
        DefragPhase::Analyzing => "Analyzing disk...",
        DefragPhase::Defragmenting => "Defragmenting...",
        DefragPhase::Finished => "Complete",
    }
}

/// Action line for the bottom bar (phase + demo/sound info).
pub fn action_text(app: &App) -> String {
    if app.paused {
        return "[ PAUSED ]".to_string();
    }
    match app.phase {
        DefragPhase::Initializing => "Initializing...".to_string(),
        DefragPhase::Analyzing => "Analyzing disk...".to_string(),
        DefragPhase::Defragmenting => match app.animation_step % 3 {
            0 => "Reading...".to_string(),
            1 => "Writing...".to_string(),
            _ => "Updating FAT...".to_string(),
        },
        DefragPhase::Finished => "Complete".to_string(),
    }
}

/// Truncate a filename to `max_len` chars, respecting UTF-8 boundaries.
pub fn truncate_str<'a>(s: &'a str, max_len: usize) -> &'a str {
    if s.len() <= max_len {
        return s;
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Drive letter display string for status messages.
pub fn drive_label(app: &App) -> String {
    format!("Drive {}", app.current_drive.letter())
}
